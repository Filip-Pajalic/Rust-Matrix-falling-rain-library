use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixError {
    InvalidConfig(String),
    Resource(String),
}

impl MatrixError {
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig(message.into())
    }

    pub fn resource(message: impl Into<String>) -> Self {
        Self::Resource(message.into())
    }
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatrixError::InvalidConfig(message) => {
                write!(f, "invalid matrix rain config: {message}")
            }
            MatrixError::Resource(message) => write!(f, "matrix rain resource error: {message}"),
        }
    }
}

impl Error for MatrixError {}
