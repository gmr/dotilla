use crate::config::Config;
use fjall::Database;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::{JoinError, spawn_blocking};

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

    #[error("System error")]
    System { err: JoinError },
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
            Error::System { .. } => 0,
        }
    }
}

/// Opens the database for the given configuration
pub fn open(config: &Config) -> Result<Database, Error> {
    let database_path = config.data_directory.join("system.dat");
    match Database::builder(database_path).open() {
        Ok(db) => Ok(db),
        Err(e) => Err(Error::Open(e)),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Keyspace {
    name: String,
    created_at: std::time::SystemTime,
}

async fn get_keyspace(db: &Database, name: String) -> Result<fjall::Keyspace, Error> {
    let result = db.keyspace(&name, fjall::KeyspaceCreateOptions::default);
    match result {
        Ok(system) => Ok(system),
        Err(err) => Err(Error::InvalidKeyspace {
            keyspace: name.to_string(),
            err: Some(err),
        }),
    }
}

async fn keyspaces(db: &Database) -> Result<Vec<Keyspace>, Error> {
    let name = "system";
    let system = get_keyspace(db, name.to_string()).await?;

    let sys = system.clone();
    match spawn_blocking(move || sys.get("keyspaces")).await {
        Ok(Ok(Some(bytes))) => {
            let keyspaces: Vec<Keyspace> = serde_json::from_slice(&bytes).unwrap_or_default();
            Ok(keyspaces)
        }
        Ok(Ok(None)) => Ok(vec![]),
        Ok(Err(err)) => Err(Error::InvalidKeyspace {
            keyspace: name.to_string(),
            err: Some(err),
        }),
        Err(err) => Err(Error::System { err }),
    }
}

async fn save_keyspaces(db: &Database, keyspaces: Vec<Keyspace>) -> Result<(), Error> {
    let name = "system";
    let system = get_keyspace(db, name.to_string()).await?;
    let sys = system.clone();
    let serialized = serde_json::to_string(&keyspaces).unwrap();
    match spawn_blocking(move || sys.insert("keyspaces", serialized)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(Error::SaveKeyspaces { err }),
        Err(err) => Err(Error::System { err }),
    }
}

pub async fn create_keyspace(db: &Database, name: String) -> Result<fjall::Keyspace, Error> {
    let mut keyspaces = keyspaces(db).await?;
    if keyspaces.iter().any(|k| k.name == name) {
        return Err(Error::KeyspaceAlreadyExists {
            keyspace: name.to_string(),
        });
    }

    let keyspace = match db.keyspace(&name, fjall::KeyspaceCreateOptions::default) {
        Ok(result) => result,
        Err(err) => {
            return Err(Error::Create {
                keyspace: name.to_string(),
                err,
            });
        }
    };

    // Append the keyspace to the list of keyspaces
    keyspaces.push(Keyspace {
        name: name.to_string(),
        created_at: std::time::SystemTime::now(),
    });

    save_keyspaces(db, keyspaces).await?;

    Ok(keyspace)
}

// We only allow for keyspaces that are tracked in the system database
pub async fn keyspace(db: &Database, name: String) -> Result<fjall::Keyspace, Error> {
    let keyspaces = match keyspaces(db).await {
        Ok(keyspaces) => keyspaces,
        Err(err) => return Err(err),
    };
    let found = keyspaces.iter().find(|k| k.name == name);
    match found {
        Some(_) => get_keyspace(db, name).await,
        None => Err(Error::InvalidKeyspace {
            keyspace: name.to_string(),
            err: None,
        }),
    }
}
