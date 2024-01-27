use std::{io};
use thiserror::Error;

pub type DomainResult<T> = Result<T, DomainError>;

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