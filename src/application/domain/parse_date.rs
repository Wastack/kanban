use time;
use time::macros::format_description;
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::ports::time::TodayProvider;

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
    pub(crate) fn parse(&self, text: &str) -> DomainResult<time::Date> {
        self.parse_as_relative_day(text)
            .or(self.parse_as_relative_weekday(text))
            .or(self.parse_as_date(text))
    }

    fn parse_as_relative_day(&self, text: &str) -> DomainResult<time::Date> {
        match text.to_lowercase().as_str() {
            "today" | "now" => {
                Ok(self.today_provider.today())
            },
            "tomorrow" => {
                Ok(self.today_provider.today().next_day().unwrap())
            },
            _ => Err(DomainError::InternalError(String::from("not a relative day")))
        }
    }
    fn parse_as_relative_weekday(&self, text: &str) -> DomainResult<time::Date> {
        let week_day = parse_to_weekday(text)?;

        Ok(self.today_provider.today().next_occurrence(week_day))
    }

    fn parse_as_date(&self, text: &str) -> DomainResult<time::Date> {
        let mut text = String::from(text);
        let parts = text.chars().filter(|c| *c == '-').count() + 1;
        if parts < 3 {
            if parts == 2 {
                text = format!("{}-{}", self.today_provider.today().year(), text)
            } else if parts == 1 {
                text = format!("{}-{}-{}", self.today_provider.today().year(), self.today_provider.today().month(), text)
            }
        }
        let format = format_description!("[year]-[month]-[day]");
        time::Date::parse(&text, format).map_err(|err| err.into())
    }
}



fn parse_to_weekday(text: &str) -> DomainResult<time::Weekday> {
    match text.to_lowercase().as_str() {
        "m" | "mo" | "mon" | "mond" | "monda" | "monday" => Ok(time::Weekday::Monday),
        "tu" | "tue" | "tues" | "tuesd" | "tuesda" | "tuesday" => Ok(time::Weekday::Tuesday),
        "w" | "we" | "wed" | "wedn" | "wedne" | "wednes" | "wednesd" | "wednesda" | "wednesday" => Ok(time::Weekday::Wednesday),
        "th" | "thu" | "thur" | "thurs" | "thursd" | "thursda" | "thursday" => Ok(time::Weekday::Thursday),
        "f" | "fr" | "fri" | "frid" | "frida" | "friday" => Ok(time::Weekday::Friday),
        "sa" | "sat" | "satu" | "satur" | "saturd" | "saturda" | "saturday" => Ok(time::Weekday::Saturday),
        "su" | "sun" | "sund" | "sunda" | "sunday" => Ok(time::Weekday::Sunday),
        _ => Err(DomainError::InternalError(String::from("not a weekday")))
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use time::macros::date;
    use crate::adapters::time_providers::fake::FakeTodayProvider;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::parse_date::DateParser;

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
            // ToDo: implement missing padding zeros, e.g. 2-4
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
            let_assert!(Err(DomainError::ParseError(_)) = result, "input: {}", input);
        }
    }
}

