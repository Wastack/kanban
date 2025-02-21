use std::ops::Range;
use chumsky::{text, Parser};
use chumsky::error::Simple;
use chumsky::prelude::{choice, end, filter, just, one_of};

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedDateAst {
    Today,
    Tomorrow,
    RelativeWeekDay(time::Weekday),
    ParsedDate(ParsedDate)
}

impl ParsedDateAst {
    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        choice((
            just("today").or(just("now")).to(Self::Today),
            just("tomorrow").to(Self::Tomorrow),
            Self::weekday_parser(),
            ParsedDate::parser().map(Self::ParsedDate),
        )).then_ignore(end())
    }

    fn weekday_parser() -> impl Parser<char, Self, Error = Simple<char>> {
        filter(char::is_ascii_alphabetic)
            .repeated()
            .at_least(1)
            .collect::<String>()
            .try_map(Self::text2weekday)
            .map(ParsedDateAst::RelativeWeekDay)
    }

    fn text2weekday(text: String, span: Range<usize>) -> Result<time::Weekday, Simple<char>> {
        match text.to_lowercase().as_str() {
            "m" | "mo" | "mon" | "mond" | "monda" | "monday" => Ok(time::Weekday::Monday),
            "tu" | "tue" | "tues" | "tuesd" | "tuesda" | "tuesday" => Ok(time::Weekday::Tuesday),
            "w" | "we" | "wed" | "wedn" | "wedne" | "wednes" | "wednesd" | "wednesda" | "wednesday" => Ok(time::Weekday::Wednesday),
            "th" | "thu" | "thur" | "thurs" | "thursd" | "thursda" | "thursday" => Ok(time::Weekday::Thursday),
            "f" | "fr" | "fri" | "frid" | "frida" | "friday" => Ok(time::Weekday::Friday),
            "sa" | "sat" | "satu" | "satur" | "saturd" | "saturda" | "saturday" => Ok(time::Weekday::Saturday),
            "su" | "sun" | "sund" | "sunda" | "sunday" => Ok(time::Weekday::Sunday),
            _ => Err(Simple::custom(span, "not a weekday")),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedDate {
    pub year: Option<i32>,
    pub month: Option<time::Month>,
    pub day: u8,
}

impl ParsedDate {
    fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        let number = text::int(10).map(|s: String| s.parse::<i32>().unwrap());

        // Parser that remembers if there were any padding zeroes
        let number_padded_not_ignored_zero= filter(|c: &char| *c == '0')
            .repeated()
            .at_most(1)
            .map(|v| !v.is_empty())
            .then(number);

        let number_padded_zero = filter(|c: &char| *c == '0').repeated().at_most(1).ignored().ignore_then(number);
        let additional_number = one_of("-./").ignored().ignore_then(number_padded_zero);

        // ToDo: zero padded years should not be accepted.
        number_padded_not_ignored_zero
            .then(additional_number.clone().or_not())
            .then(additional_number.clone().or_not())
            .then_ignore(end())
            .try_map(|(((first_had_padding_zero ,first), second), third), span|{
                let parsed_date = match second {
                    None => ParsedDate {
                        year: None,
                        month: None,
                        day: Self::i32_to_u8(first, span)?,
                    },
                    Some(second) => match third {
                        None => ParsedDate {
                            year: None,
                            month: Some(Self::i32_to_month(first, span.clone())?),
                            day: Self::i32_to_u8(second, span)?,
                        },
                        Some(third) => {
                            // Now we know that `first` is a year. In this case, padding zeroes should
                            // not have been accepted.
                            if first_had_padding_zero {
                                return Err(Simple::custom(span, "Year component started with zero digit"));
                            }

                            ParsedDate {
                                year: Some(first),
                                month: Some(Self::i32_to_month(second, span.clone())?),
                                day: Self::i32_to_u8(third, span)?,
                            }
                        },
                    },
                };

                Ok(parsed_date)
            })
    }

    fn i32_to_u8(num: i32, span: Range<usize>) -> Result<u8, Simple<char>> {
        u8::try_from(num)
            .map_err(|e| Simple::custom(span, e.to_string()))
    }

    fn i32_to_month(month: i32, span: Range<usize>) -> Result<time::Month, Simple<char>> {
        u8::try_from(month)
            .map_err(|e| Simple::custom(span.clone(), e.to_string()))
            .and_then(|month| time::Month::try_from(month)
                .map_err(|e| Simple::custom(span, e.to_string())))
    }
}


#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use crate::application::domain::date_parse::parser::{ParsedDate};
    use chumsky::Parser;
    use time::Month;

    #[test]
    fn test_chumsky_parse_date() {
        // ToDo: test failure cases?
        let test_table = [
            ("2025-02-09", ParsedDate{ year: Some(2025), month: Some(Month::February), day: 9, }),
            ("2025.02.09", ParsedDate{ year: Some(2025), month: Some(Month::February), day: 9, }),
            ("2025/02/09", ParsedDate{ year: Some(2025), month: Some(Month::February), day: 9, }),
            ("2025-2-09", ParsedDate{ year: Some(2025), month: Some(Month::February), day: 9, }),
            ("2025-02-9", ParsedDate{ year: Some(2025), month: Some(Month::February), day: 9, }),
            ("02-9", ParsedDate{ year: None, month: Some(Month::February), day: 9, }),
            ("2-09", ParsedDate{ year: None, month: Some(Month::February), day: 9, }),
            ("09", ParsedDate{ year: None, month: None, day: 9, }),
            ("9", ParsedDate{ year: None, month: None, day: 9, }),
        ];

        for (input, expected_output) in test_table {
            let result = ParsedDate::parser().parse(input);
            let_assert!(Ok(result) = result, "input: {}, expected output: {:?}", input, expected_output);
            check!(result == expected_output, "input: {}", input);
        }
    }
}
