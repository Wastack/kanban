use std::{io};
use nonempty_collections::NEVec;
use thiserror::Error;

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

    #[error("Not implemented")]
    NotImplemented
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
                DomainError::NotImplemented => DomainError::NotImplemented,
            }
        }
    }
}