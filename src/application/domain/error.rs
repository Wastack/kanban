use std::{io};
use std::num::TryFromIntError;
use chumsky::prelude::Simple;
use nonempty_collections::NEVec;
use thiserror::Error;
use time::error::ComponentRange;

pub type DomainResult<T> = Result<T, DomainError>;
pub type DomainResultMultiError<T> = Result<T, NEVec<DomainError>>;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Index `{0}` is out of range")]
    IndexOutOfRange(usize),
    #[error("Editor failed with error: {source}")]
    EditorError {
        #[from]
        source: io::Error
    },

    #[error("Invalid board: {0}")]
    InvalidBoard(String),

    #[error("History is empty")]
    EmptyHistory,

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error(transparent)]
    IntError(#[from] TryFromIntError),

    #[error(transparent)]
    ComponentRange(#[from] ComponentRange),

    #[error("Chumsky error: {0:?}")]
    ChumskyError(Vec<Simple<char>>),
}

impl From<Vec<Simple<char>>> for ParseError {
    fn from(value: Vec<Simple<char>>) -> Self {
        Self::ChumskyError(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Error};
    use crate::application::domain::error::DomainError;

    impl DomainError {
        pub(crate) fn clone_for_testing(&self) -> DomainError {
            match self {
                DomainError::IndexOutOfRange(e) => DomainError::IndexOutOfRange(*e),
                DomainError::EditorError { source} => DomainError::EditorError {
                    // Here we lose the error message
                    source: Error::from(source.kind().clone()),
                },
                DomainError::InvalidBoard(e) => DomainError::InvalidBoard(e.clone()),
                DomainError::EmptyHistory => DomainError::EmptyHistory,
                DomainError::ParseError(e) => DomainError::ParseError(e.clone()),
                DomainError::InternalError(e) => DomainError::InternalError(e.clone()),
            }
        }
    }
}