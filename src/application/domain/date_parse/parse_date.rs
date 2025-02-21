use time;
use crate::application::domain::date_parse::parser::{ParsedDate, ParsedDateAst};
use crate::application::domain::date_parse::error::DateParseError;
use crate::application::ports::time::TodayProvider;
use chumsky::Parser;

pub struct DateParser<'a, T: TodayProvider> {
    pub today_provider: &'a T,
}

impl<T: TodayProvider> DateParser<'_, T> {
    /// Parse a date.
    ///
    /// The following formats are accepted:
    /// - Date in the format of `yyyy-mm-dd` or `mm-dd` or `dd`
    ///   + 0 padding is accepted for years, months, days
    /// - "today", "tomorrow"
    /// - "m" "tu", "w", "th", "f", "sa", "su" for the next occurrence of that weekday (excluding today).
    pub(crate) fn parse(&self, text: &str) -> Result<time::Date, DateParseError> {
            ParsedDateAst::parser()
                .parse(text)
                .map_err(DateParseError::from)
                .and_then(|parsed|
                    self.eval(parsed))
                        .map_err(DateParseError::from)
    }

    pub fn eval(&self, parsed_date: ParsedDateAst) -> Result<time::Date, DateParseError> {
        match parsed_date {
            ParsedDateAst::Today => Ok(self.today_provider.today()),
            ParsedDateAst::Tomorrow => Ok(self.today_provider.today().next_day().unwrap()),
            ParsedDateAst::RelativeWeekDay(weekday) => Ok(self.today_provider.today().next_occurrence(weekday)),
            ParsedDateAst::ParsedDate(parsed_date) => self.try_from_parsed_date(parsed_date)
        }
    }

    fn try_from_parsed_date(&self, parsed_date: ParsedDate) -> Result<time::Date, DateParseError> {
        let ParsedDate { year, month, day } = parsed_date;

        time::Date::from_calendar_date(
            year.unwrap_or(self.today_provider.today().year()),
            month.unwrap_or(self.today_provider.today().month()),
            day).map_err(DateParseError::from)
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use time::macros::date;
    use crate::adapters::time_providers::fake::FakeTodayProvider;
    use crate::application::domain::date_parse::parse_date::DateParser;

    #[test]
    fn test_parse_date() {
        // date!(2025-02-22) // saturday
        let fake_pr = FakeTodayProvider::default();
        let date_parser = DateParser {
            today_provider: &fake_pr,
        };

        let test_table_success = [
            ("2026-02-03", date!(2026-02-03)),
            ("2025-03-17", date!(2025-03-17)),
            ("02-04", date!(2025-02-04)),
            ("2-04", date!(2025-02-04)),
            ("2-4", date!(2025-02-04)),
            ("4", date!(2025-02-04)),
            ("today", date!(2025-02-22)),
            ("tomorrow", date!(2025-02-23)),
            ("m", date!(2025-02-24)),
            ("tu", date!(2025-02-25)),
            ("wedn", date!(2025-02-26)),
            ("th", date!(2025-02-27)),
            ("f", date!(2025-02-28)),
            ("Saturda", date!(2025-03-01)),
            ("SU", date!(2025-02-23)),
        ];

        for (input, expected_output) in test_table_success {
            let result = date_parser.parse(input);
            let_assert!(Ok(result) = result, "input: {}, expected output: {}", input, expected_output);
            check!(result == expected_output);
        }

        let test_table_failure = [
            "unparsable","-", "--", "tomorroww", "2034-",
            "2024-23-01", "2025-02-29", "-2-02-02",
            "t", "-2034", "", "2025-02-29-12", "41", "02025-02-11"];

        for input in test_table_failure {
            let result = date_parser.parse(input);
            check!(result.is_err(), "input was:{}", input);
        }
    }

}
