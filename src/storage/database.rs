use thiserror::Error;
use tokio::task::spawn_blocking;

use super::errors;
use super::keyspace::Keyspace;

pub struct Database {
    pub handle: fjall::Database,
    /// taken only at the top of create/delete, never in helpers they call — not reentrant
    pub namespace_lock: tokio::sync::Mutex<()>,
    pub system: Keyspace,
    pub default_locale: String,
}

impl Database {
    /// Initializes the database
    pub async fn initialize(config: &crate::config::Config) -> Result<Self, Error> {
        let db = fjall::Database::builder(config.data_directory.clone()).open()?;
        let system = db.keyspace("system", fjall::KeyspaceCreateOptions::default)?;
        Ok(Self {
            handle: db,
            namespace_lock: tokio::sync::Mutex::new(()),
            system: Keyspace {
                name: "system".to_string(),
                handle: system,
            },
            default_locale: config.default_locale.clone(),
        })
    }

    /// Returns the number of journal files in the database.
    pub async fn journal_count(&self) -> Result<usize, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.journal_count()).await?)
    }

    /// Returns the number of keyspaces in the database.
    pub async fn keyspace_count(&self) -> Result<usize, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.keyspace_count()).await?)
    }

    /// Returns the approximate size of the database on disk.
    pub async fn size_on_disk(&self) -> Result<u64, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.disk_space()).await??)
    }

    /// Returns the approximate size of the write buffer.
    pub async fn write_buffer_size(&self) -> Result<u64, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.write_buffer_size()).await?)
    }
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error")]
    Internal(#[from] fjall::Error),

    #[error("System error")]
    System,
}

impl Error {
    /// Map the error to an exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Internal { .. } => 11,
            Error::System => 10,
        }
    }
}
