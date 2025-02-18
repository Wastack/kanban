use time;
use crate::application::domain::date_parse;
use crate::application::domain::date_parse::chumsky::ParsedDate;
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
        let maybe_relative_date = self.parse_as_relative_day(text)
                    .or(self.parse_as_relative_weekday(text));

        let date = match maybe_relative_date {
            None => self.parse_date(text)?,
            Some(relative_date) => relative_date,
        };

        Ok(date)
    }

    fn parse_as_relative_day(&self, text: &str) -> Option<time::Date> {
        match text.to_lowercase().as_str() {
            "today" | "now" => {
                Some(self.today_provider.today())
            },
            "tomorrow" => {
                Some(self.today_provider.today().next_day().unwrap())
            },
            _ => None,
        }
    }
    fn parse_as_relative_weekday(&self, text: &str) -> Option<time::Date> {
        let week_day = parse_to_weekday(text)?;

        Some(self.today_provider.today().next_occurrence(week_day))
    }

    fn parse_date(&self, text: &str) -> Result<time::Date, DateParseError> {
        let parser = date_parse::chumsky::date_parser();
        let parsed = parser.parse(text)?;

        self.try_from_parsed_date(parsed)
    }

    fn try_from_parsed_date(&self, parsed_date: ParsedDate) -> Result<time::Date, DateParseError> {
        let ParsedDate { year, month, day } = parsed_date;
        let month = match month {
            None => self.today_provider.today().month().into(),
            Some(month) => u8::try_from(month)?,
        };

        let month = time::Month::try_from(month)?;

        let year = match year {
            None => self.today_provider.today().year(),
            Some(year) => year,
        };

        let day = u8::try_from(day)?;

        let date = time::Date::from_calendar_date(year, month, day)?;

        Ok(date)
    }
}


fn parse_to_weekday(text: &str) -> Option<time::Weekday> {
    match text.to_lowercase().as_str() {
        "m" | "mo" | "mon" | "mond" | "monda" | "monday" => Some(time::Weekday::Monday),
        "tu" | "tue" | "tues" | "tuesd" | "tuesda" | "tuesday" => Some(time::Weekday::Tuesday),
        "w" | "we" | "wed" | "wedn" | "wedne" | "wednes" | "wednesd" | "wednesda" | "wednesday" => Some(time::Weekday::Wednesday),
        "th" | "thu" | "thur" | "thurs" | "thursd" | "thursda" | "thursday" => Some(time::Weekday::Thursday),
        "f" | "fr" | "fri" | "frid" | "frida" | "friday" => Some(time::Weekday::Friday),
        "sa" | "sat" | "satu" | "satur" | "saturd" | "saturda" | "saturday" => Some(time::Weekday::Saturday),
        "su" | "sun" | "sund" | "sunda" | "sunday" => Some(time::Weekday::Sunday),
        _ => None,
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
            "unparsable", "-", "--", "tomorroww", "2034-",
            "2024-23-01", "2025-02-29", "-2-02-02",
            "t", "-2034", ""];

        for input in test_table_failure {
            let result = date_parser.parse(input);
            // ToDo: more precise error handling
            check!(result.is_err());
        }
    }

}
