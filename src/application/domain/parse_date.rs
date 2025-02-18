use time;
use chumsky::prelude::*;

use crate::application::domain::error::{DomainError, DomainResult, ParseError};
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
            .or(self.parse_date(text).map_err(|e| e.into()))
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

    fn parse_date(&self, text: &str) -> Result<time::Date, ParseError> {
        let parser = date_parser();
        let parsed = parser.parse(text)?;

        self.try_from_parsed_date(parsed)
    }

    fn try_from_parsed_date(&self, parsed_date: ParsedDate) -> Result<time::Date, ParseError> {
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


#[derive(Debug, PartialEq)]
struct ParsedDate {
    year: Option<i32>,
    month: Option<i32>,
    day: i32,
}

fn date_parser() -> impl Parser<char, ParsedDate, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| s.parse::<i32>().unwrap());

    let number_padded_zero = filter(|c: &char| *c == '0').repeated().at_most(1).ignored().ignore_then(number);
    let additional_number = just("-").ignored().ignore_then(number_padded_zero);

    // ToDo: zero padded years should not be accepted.
    number_padded_zero
        .then(additional_number.or_not())
        .then(additional_number.or_not())
        //.then_ignore(end())
        .map(|((first, second), third)|{
            match second {
                None => ParsedDate { year: None, month: None, day: first },
                Some(second) => match third {
                    None => ParsedDate { year: None, month: Some(first), day: second },
                    Some(third) => ParsedDate { year: Some(first), month: Some(second ), day: third },
                },
            }
        })
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use time::macros::date;
    use crate::adapters::time_providers::fake::FakeTodayProvider;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::parse_date::{date_parser, DateParser, ParsedDate};
    use chumsky::Parser;

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
            let_assert!(Err(DomainError::ParseError(_)) = result, "input: {}", input);
        }
    }

    #[test]
    fn test_chumsky_parse_date() {
        // ToDo: test failure cases?
        // ToDo: zero padded years should not be accepted.
        let test_table = [
            ("2025-02-09", ParsedDate{ year: Some(2025), month: Some(2), day: 9, }),
            ("2025-2-09", ParsedDate{ year: Some(2025), month: Some(2), day: 9, }),
            ("2025-02-9", ParsedDate{ year: Some(2025), month: Some(2), day: 9, }),
            ("02-9", ParsedDate{ year: None, month: Some(2), day: 9, }),
            ("2-09", ParsedDate{ year: None, month: Some(2), day: 9, }),
            ("09", ParsedDate{ year: None, month: None, day: 9, }),
            ("9", ParsedDate{ year: None, month: None, day: 9, }),
        ];

        for (input, expected_output) in test_table {
            let result = date_parser().parse(input);
            let_assert!(Ok(result) = result, "input: {}, expected output: {:?}", input, expected_output);
            check!(result == expected_output, "input: {}", input);
        }
    }
}
