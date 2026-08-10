use std::sync::Mutex;
use thiserror::Error;
use tokio::task::{JoinError, spawn_blocking};

use super::types::{Database, DatabaseInfo, NamespaceDetails};

/// Initializes the database and returns a mutex-wrapped database handle.
pub async fn initialize(config: &crate::config::Config) -> Result<Database, Error> {
    let db = match fjall::Database::builder(config.data_directory.clone()).open() {
        Ok(db) => db,
        Err(err) => return Err(Error::Internal(err)),
    };
    let system = match db.keyspace("system", fjall::KeyspaceCreateOptions::default) {
        Ok(system) => system,
        Err(err) => return Err(Error::Internal(err)),
    };
    match super::namespace::fetch_all(&db, &system).await {
        Ok(namespaces) => Ok(Database {
            db,
            system,
            default_locale: config.default_locale.clone(),
            namespaces: Mutex::new(namespaces),
        }),
        Err(err) => Err(Error::System(err)),
    }
}

pub async fn info(database: &Database) -> Result<DatabaseInfo, Error> {
    let size_on_disk = database.db.disk_space().unwrap();
    let journal_count = database.db.journal_count();
    let mut namespaces: Vec<NamespaceDetails> = Vec::new();
    for namespace in super::namespace::list(database) {
        if let Ok(details) = super::namespace::get(database, &namespace.to_string()).await {
            namespaces.push(details)
        };
    }
    Ok(DatabaseInfo {
        size_on_disk,
        journal_count,
        namespaces,
    })
}

/// Delete a keyspace from the database
pub async fn delete_keyspace(db: &fjall::Database, keyspace: fjall::Keyspace) -> Result<(), Error> {
    let db = db.clone();
    match spawn_blocking(move || db.delete_keyspace(keyspace)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(Error::Internal(err)),
        Err(err) => Err(Error::IO(err)),
    }
}

/// Open / Create a keyspace in the database
pub async fn open_keyspace(db: &fjall::Database, name: &str) -> Result<fjall::Keyspace, Error> {
    let db = db.clone();
    let namespace = name.to_string();
    match spawn_blocking(move || db.keyspace(&namespace, fjall::KeyspaceCreateOptions::default))
        .await
    {
        Ok(Ok(keyspace)) => Ok(keyspace),
        Ok(Err(err)) => Err(Error::Internal(err)),
        Err(err) => Err(Error::IO(err)),
    }
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to execute blocking operation")]
    IO(#[from] JoinError),

    #[error("Internal error")]
    Internal(#[from] fjall::Error),

    #[error("System error")]
    System(#[from] super::namespace::Error),
}

impl Error {
    /// Map the error to an exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::IO { .. } => 0, // Non-exiting error
            Error::Internal { .. } => 11,
            Error::System { .. } => 10,
        }
    }
}
