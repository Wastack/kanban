use chumsky::error::Simple;
use thiserror::Error;
use time::error::ComponentRange;

#[derive(Debug, Error, Clone)]
pub enum DateParseError {
    #[error(transparent)]
    ComponentRange(#[from] ComponentRange),

    #[error("Chumsky error: {0:?}")]
    ChumskyError(Vec<Simple<char>>),
}

impl From<Vec<Simple<char>>> for DateParseError {
    fn from(value: Vec<Simple<char>>) -> Self {
        Self::ChumskyError(value)
    }
}