//! Domain errors

use thiserror::Error;

/// Network domain errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    General(String),
}