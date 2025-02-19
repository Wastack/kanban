use chumsky::{text, Parser};
use chumsky::error::Simple;
use chumsky::prelude::{filter, one_of};

#[derive(Debug, PartialEq)]
pub struct ParsedDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: i32,
}

pub fn date_parser() -> impl Parser<char, ParsedDate, Error = Simple<char>> {
    let number = text::int(10).map(|s: String| s.parse::<i32>().unwrap());

    let number_padded_zero = filter(|c: &char| *c == '0').repeated().at_most(1).ignored().ignore_then(number);
    let additional_number = one_of("-./").ignored().ignore_then(number_padded_zero);

    // ToDo: zero padded years should not be accepted.
    number_padded_zero
        .then(additional_number.clone().or_not())
        .then(additional_number.clone().or_not())
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
    use crate::application::domain::date_parse::chumsky::{date_parser, ParsedDate};
    use chumsky::Parser;

    #[test]
    fn test_chumsky_parse_date() {
        // ToDo: test failure cases?
        // ToDo: zero padded years should not be accepted.
        let test_table = [
            ("2025-02-09", ParsedDate{ year: Some(2025), month: Some(2), day: 9, }),
            ("2025.02.09", ParsedDate{ year: Some(2025), month: Some(2), day: 9, }),
            ("2025/02/09", ParsedDate{ year: Some(2025), month: Some(2), day: 9, }),
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
