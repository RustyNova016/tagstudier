use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TagstudioDb(#[from] tagstudio_db::Error),

    #[error(transparent)]
    LibraryConnection(#[from] tagstudio_db::TSPoolError),

    #[error(transparent)]
    Sqlx(#[from] tagstudio_db::sqlx::Error),

    #[error("Incorrect input for CLI: {0}")]
    CliInput(String, String),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("Max tries for request")]
    MaxTries,
}
