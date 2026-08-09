use fjall::Database;
use thiserror::Error;

pub fn initialize(config: &crate::config::Config) -> Result<fjall::Database, Error> {
    let path = config.data_directory.clone().join("data");
    match Database::builder(path).open() {
        Ok(db) => Ok(db),
        Err(e) => Err(Error::Open(e)),
    }
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create database")]
    Create { keyspace: String, err: fjall::Error },

    #[error("Database already exists")]
    KeyspaceAlreadyExists { keyspace: String },

    #[error("Invalid database")]
    InvalidKeyspace {
        keyspace: String,
        err: Option<fjall::Error>,
    },

    #[error("Failed to open database")]
    Open(#[from] fjall::Error),

    #[error("Failed to save databases")]
    SaveKeyspaces { err: fjall::Error },
}

impl Error {
    /// Map the error to an exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Create { .. } => 0, // Not an exitable failure
            Error::KeyspaceAlreadyExists { .. } => 0,
            Error::InvalidKeyspace { .. } => 0,
            Error::Open { .. } => 10,
            Error::SaveKeyspaces { .. } => 0,
        }
    }
}
