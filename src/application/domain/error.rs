use std::{error, fmt};

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Clone)]
pub struct DomainError {
    description: String,
}

impl DomainError {
    pub fn new(description: &str) -> DomainError {
        DomainError {
            description: description.to_string(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DomainError: {}", self.description)
    }
}

impl error::Error for DomainError {
    fn description(&self) -> &str {
        &self.description
    }
}