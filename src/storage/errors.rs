use thiserror::Error;

use super::types::ValueError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("data encoding/decoding error: {0}")]
    Avro(#[from] apache_avro::Error),

    #[error("database error: {0}")]
    Database(#[from] fjall::Error),

    #[error("failed to execute blocking operation: {0}")]
    IO(#[from] tokio::task::JoinError),

    #[error("namespace already exists")]
    NamespaceExists { namespace: String },

    #[error("invalid namespace name: {0}")]
    NamespaceInvalidName(#[from] ValueError),

    #[error("not found")]
    NotFound,

    #[error("UTF-8 decoding error: {0}")]
    UTF8(#[from] std::string::FromUtf8Error),

    #[error("value error")]
    ValueError,
}
