//! Domain errors

use thiserror::Error;

/// Network domain errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    General(String),
    
    #[error("Configuration generation error: {0}")]
    ConfigurationError(String),
}

impl From<std::fmt::Error> for NetworkError {
    fn from(err: std::fmt::Error) -> Self {
        NetworkError::ConfigurationError(err.to_string())
    }
}