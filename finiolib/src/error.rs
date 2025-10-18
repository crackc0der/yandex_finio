//! Единый тип ошибок публичного API.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinioError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("XML error: {0}")]
    Xml(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(&'static str),
}

pub type Result<T> = std::result::Result<T, FinioError>;
