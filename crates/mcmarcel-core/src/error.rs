use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("missing parameter: {0}")]
    MissingParameter(String),
    #[error("invalid country pack: {0}")]
    Pack(String),
    #[error("production gate failed: {0}")]
    ProductionGate(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
