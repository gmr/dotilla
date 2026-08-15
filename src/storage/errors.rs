use thiserror::Error;

use super::types::ValueError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("data encoding/decoding error")]
    Avro(#[from] apache_avro::Error),

    #[error("database error")]
    Database(#[from] fjall::Error),

    #[error("failed to execute blocking operation")]
    IO(#[from] tokio::task::JoinError),

    #[error("namespace already exists")]
    NamespaceExists { namespace: String },

    #[error("invalid namespace name")]
    NamespaceInvalidName(#[from] ValueError),

    #[error("not found")]
    NotFound,

    #[error("value error")]
    ValueError,
}
