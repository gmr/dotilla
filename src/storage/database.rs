use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use tokio::task::spawn_blocking;

/// Initializes the database and returns a mutex-wrapped database handle.
pub async fn initialize(config: &crate::config::Config) -> Result<Database, Error> {
    let db = match fjall::Database::builder(config.data_directory.clone()).open() {
        Ok(db) => db,
        Err(e) => return Err(Error::Open(e)),
    };
    let system = match db.keyspace("system", fjall::KeyspaceCreateOptions::default) {
        Ok(system) => system,
        Err(e) => return Err(Error::Open(e)),
    };
    let namespaces = get_namespaces(db.clone(), system.clone()).await?;
    Ok(Database {
        db,
        system,
        namespaces: Mutex::new(namespaces),
    })
}

pub async fn create_namespace(database: &Database, name: &str) -> Result<(), Error> {
    let namespace_name = super::types::DatabaseName::new(name);
    let namespace = match get_namespace(
        database.db.clone(),
        database.system.clone(),
        name.to_string(),
    )
    .await
    {
        Ok(_) => {
            return Err(Error::AlreadyExists {
                namespace: name.to_string(),
            });
        }
        Err(_) => Namespace {
            name: namespace_name,
            nodes: create_keyspace(database.db.clone(), "nodes").await?,
            edges: create_keyspace(database.db.clone(), "edges").await?,
            labels: create_keyspace(database.db.clone(), "labels").await?,
            vectors: create_keyspace(database.db.clone(), "vectors").await?,
        },
    };
    database
        .namespaces
        .lock()
        .unwrap()
        .insert(super::types::DatabaseName::new(name), namespace);
    save_namespaces(database).await?;
    Ok(())
}

async fn create_keyspace(db: fjall::Database, name: &str) -> Result<fjall::Keyspace, Error> {
    match db.keyspace(name, fjall::KeyspaceCreateOptions::default) {
        Ok(keyspace) => Ok(keyspace),
        Err(err) => Err(Error::Open(err)),
    }
}

// Get a namespace by name, blocking the current thread to avoid async overhead.
async fn get_namespace(
    db: fjall::Database,
    system: fjall::Keyspace,
    name: String,
) -> Result<Namespace, Error> {
    let names = keyspace_names(&name);
    let namespace = name.clone();
    match spawn_blocking(move || system.get(&namespace)).await {
        Ok(Ok(None)) => Err(Error::NotFound { namespace: name }),
        Ok(Ok(_bytes)) => Ok(Namespace {
            name: super::types::DatabaseName::new(&name),
            nodes: open_namespace(db.clone(), &names.nodes)?,
            edges: open_namespace(db.clone(), &names.edges)?,
            labels: open_namespace(db.clone(), &names.labels)?,
            vectors: open_namespace(db.clone(), &names.vectors)?,
        }),
        Ok(Err(err)) => Err(Error::Open(err)),
        Err(err) => Err(Error::System(err)),
    }
}

/// Return struct of the keyspace names for a namespace
fn keyspace_names(name: &str) -> KeyspaceNames {
    let namespace = name.to_string();
    KeyspaceNames {
        nodes: format!("{}\0nodes", namespace),
        edges: format!("{}\0edges", namespace),
        labels: format!("{}\0labels", namespace),
        vectors: format!("{}\0vectors", namespace),
    }
}

/// Return the namespaces from the system keyspace
async fn get_namespaces(
    db: fjall::Database,
    system: fjall::Keyspace,
) -> Result<HashMap<super::types::DatabaseName, Namespace>, Error> {
    match system.get("namespaces") {
        Ok(None) => Ok(HashMap::new()),
        Ok(Some(bytes)) => {
            let keyspaces: Vec<String> = serde_json::from_slice(&bytes).unwrap_or_default();
            let mut namespaces = HashMap::new();
            for name in keyspaces {
                let namespace = get_namespace(db.clone(), system.clone(), name.clone()).await?;
                namespaces.insert(super::types::DatabaseName(name), namespace);
            }
            Ok(namespaces)
        }
        Err(err) => Err(Error::Internal { err }),
    }
}

fn open_namespace(db: fjall::Database, name: &str) -> Result<fjall::Keyspace, Error> {
    match db.keyspace(name, fjall::KeyspaceCreateOptions::default) {
        Ok(keyspace) => Ok(keyspace),
        Err(err) => Err(Error::Internal { err }),
    }
}

async fn save_namespaces(database: &Database) -> Result<(), Error> {
    let namespaces: Vec<String> = database
        .namespaces
        .lock()
        .unwrap()
        .keys()
        .map(|n| n.to_string())
        .collect();

    let serialized = serde_json::to_string(&namespaces).unwrap();
    let system = database.system.clone();
    match spawn_blocking(move || system.insert("namespaces", serialized)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(Error::SaveNamespaces { err }),
        Err(err) => Err(Error::System(err)),
    }
}

/// Top level database handle with the core database and system keyspace, and a map of namespaces.
pub struct Database {
    pub db: fjall::Database,
    pub system: fjall::Keyspace,
    pub namespaces: Mutex<HashMap<super::types::DatabaseName, Namespace>>,
}

/// Namespace for a database with handles to the keyspaces to the different keyspace types
pub struct Namespace {
    pub name: super::types::DatabaseName,
    pub nodes: fjall::Keyspace,
    pub edges: fjall::Keyspace,
    pub labels: fjall::Keyspace,
    pub vectors: fjall::Keyspace,
}

struct KeyspaceNames {
    nodes: String,
    edges: String,
    labels: String,
    vectors: String,
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Namespace already exists")]
    AlreadyExists { namespace: String },

    #[error("Failed to create database")]
    Create {
        namespace: String,
        err: fjall::Error,
    },

    #[error("Internal error")]
    Internal { err: fjall::Error },

    #[error("Namespace not found")]
    NotFound { namespace: String },

    #[error("Failed to open database")]
    Open(#[from] fjall::Error),

    #[error("Failed to save namespaces")]
    SaveNamespaces { err: fjall::Error },

    #[error("Failed to save databases")]
    System(#[from] tokio::task::JoinError),
}

impl Error {
    /// Map the error to an exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::AlreadyExists { .. } => 0, // Not an exitable failure
            Error::Create { .. } => 0,
            Error::Internal { .. } => 0,
            Error::NotFound { .. } => 0,
            Error::Open { .. } => 10,
            Error::SaveNamespaces { .. } => 0,
            Error::System { .. } => 0,
        }
    }
}
