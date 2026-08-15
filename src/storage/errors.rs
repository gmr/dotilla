use thiserror::Error;

use super::types::ValueError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error")]
    Database(#[from] fjall::Error),

    #[error("data codec decoding error")]
    Decoding { err: apache_avro::Error },

    #[error("data codec encoding error")]
    Encoding { err: apache_avro::Error },

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
