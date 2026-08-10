use serde::{Deserialize, Serialize};
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
    let namespaces = fetch_namespaces(db.clone(), system.clone()).await?;
    Ok(Database {
        db,
        system,
        default_locale: config.default_locale.clone(),
        namespaces: Mutex::new(namespaces),
    })
}

pub fn all_namespaces(database: &Database) -> Vec<super::types::DatabaseName> {
    database
        .namespaces
        .lock()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<super::types::DatabaseName>>()
}

pub async fn create_namespace(
    database: &Database,
    name: &str,
    locale: Option<String>,
    case_insensitive: Option<bool>,
    collation_strength: Option<CollationStrength>,
) -> Result<(), Error> {
    let namespace_name = super::types::DatabaseName::new(name);
    let config = NamespaceConfig {
        locale: locale.unwrap_or(database.default_locale.to_string()),
        case_insensitive: case_insensitive.unwrap_or(false),
        collation_strength: collation_strength.unwrap_or(CollationStrength::Primary),
    };
    let namespace = match fetch_namespace(
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
        Err(_) => {
            let cfg = config.clone();
            Namespace {
                name: namespace_name,
                locale: cfg.locale,
                case_insensitive: cfg.case_insensitive,
                collation_strength: cfg.collation_strength,
                nodes: create_keyspace(database.db.clone(), "nodes").await?,
                edges: create_keyspace(database.db.clone(), "edges").await?,
                vectors: create_keyspace(database.db.clone(), "vectors").await?,
            }
        }
    };
    let serialized_config = serde_json::to_string(&config).unwrap();
    database.system.insert(name, serialized_config).unwrap();
    database
        .namespaces
        .lock()
        .unwrap()
        .insert(super::types::DatabaseName::new(name), namespace);
    Ok(())
}

pub async fn delete_namespace(database: &Database, name: &str) -> Result<(), Error> {
    let namespace = fetch_namespace(
        database.db.clone(),
        database.system.clone(),
        name.to_string(),
    )
    .await?;
    database.db.delete_keyspace(namespace.edges)?;
    database.db.delete_keyspace(namespace.nodes)?;
    database.db.delete_keyspace(namespace.vectors)?;
    database
        .namespaces
        .lock()
        .unwrap()
        .remove(&super::types::DatabaseName::new(name))
        .ok_or_else(|| Error::NotFound {
            namespace: name.to_string(),
        })?;
    database.system.remove(name).unwrap();
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyspaceDetails {
    pub size_on_disk: u64,
    pub item_count: usize,
    pub wasted_space: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamespaceDetails {
    pub name: String,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub nodes: KeyspaceDetails,
    pub edges: KeyspaceDetails,
    pub vectors: KeyspaceDetails,
}

pub async fn get_namespace(database: &Database, name: &str) -> Result<NamespaceDetails, Error> {
    let namespace = fetch_namespace(
        database.db.clone(),
        database.system.clone(),
        name.to_string(),
    )
    .await?;
    let keys = keyspace_names(name);

    let nodes = database
        .db
        .keyspace(&keys.nodes, fjall::KeyspaceCreateOptions::default)?;
    let edges = database
        .db
        .keyspace(&keys.edges, fjall::KeyspaceCreateOptions::default)?;
    let vectors = database
        .db
        .keyspace(&keys.vectors, fjall::KeyspaceCreateOptions::default)?;

    Ok(NamespaceDetails {
        name: name.to_string(),
        locale: namespace.locale.clone(),
        case_insensitive: namespace.case_insensitive,
        collation_strength: namespace.collation_strength,
        nodes: KeyspaceDetails {
            size_on_disk: nodes.disk_space(),
            item_count: nodes.approximate_len(),
            wasted_space: nodes.fragmented_blob_bytes(),
        },
        edges: KeyspaceDetails {
            size_on_disk: edges.disk_space(),
            item_count: edges.approximate_len(),
            wasted_space: edges.fragmented_blob_bytes(),
        },
        vectors: KeyspaceDetails {
            size_on_disk: vectors.disk_space(),
            item_count: vectors.approximate_len(),
            wasted_space: vectors.fragmented_blob_bytes(),
        },
    })
}

#[derive(serde::Serialize)]
pub struct DbInfo {
    pub size_on_disk: u64,
    pub journal_count: usize,
    pub namespaces: Vec<NamespaceDetails>,
}

pub async fn db_info(database: &Database) -> Result<DbInfo, Error> {
    let size_on_disk = database.db.disk_space().unwrap();
    let journal_count = database.db.journal_count();

    let mut namespaces: Vec<NamespaceDetails> = Vec::new();
    for namespace in all_namespaces(database) {
        let details = get_namespace(database, &namespace.to_string()).await?;
        namespaces.push(details);
    }
    Ok(DbInfo {
        size_on_disk,
        journal_count,
        namespaces,
    })
}

// Internal Methods

async fn create_keyspace(db: fjall::Database, name: &str) -> Result<fjall::Keyspace, Error> {
    match db.keyspace(name, fjall::KeyspaceCreateOptions::default) {
        Ok(keyspace) => Ok(keyspace),
        Err(err) => Err(Error::Open(err)),
    }
}

// Get a namespace by name
async fn fetch_namespace(
    db: fjall::Database,
    system: fjall::Keyspace,
    name: String,
) -> Result<Namespace, Error> {
    let names = keyspace_names(&name);
    let namespace = name.clone();
    match spawn_blocking(move || system.get(&namespace)).await {
        Ok(Ok(None)) => Err(Error::NotFound { namespace: name }),
        Ok(Ok(Some(bytes))) => {
            let config: NamespaceConfig = serde_json::from_slice(&bytes).unwrap();
            Ok(Namespace {
                name: super::types::DatabaseName::new(&name),
                locale: config.locale,
                case_insensitive: config.case_insensitive,
                collation_strength: config.collation_strength,
                nodes: open_namespace(db.clone(), &names.nodes)?,
                edges: open_namespace(db.clone(), &names.edges)?,
                vectors: open_namespace(db.clone(), &names.vectors)?,
            })
        }
        Ok(Err(err)) => Err(Error::Open(err)),
        Err(err) => Err(Error::System(err)),
    }
}

/// Return the namespaces from the system keyspace
async fn fetch_namespaces(
    db: fjall::Database,
    system: fjall::Keyspace,
) -> Result<HashMap<super::types::DatabaseName, Namespace>, Error> {
    let mut namespaces: HashMap<super::types::DatabaseName, Namespace> = HashMap::new();
    for guard in system.iter() {
        let key_bytes = guard.key()?;
        let key = std::str::from_utf8(&key_bytes).expect("valid utf8");
        namespaces.insert(
            super::types::DatabaseName::new(key),
            fetch_namespace(db.clone(), system.clone(), key.to_string())
                .await
                .unwrap(),
        );
    }
    Ok(namespaces)
}

/// Return struct of the keyspace names for a namespace
fn keyspace_names(name: &str) -> KeyspaceNames {
    let namespace = name.to_string();
    KeyspaceNames {
        nodes: format!("{}\0nodes", namespace),
        edges: format!("{}\0edges", namespace),
        vectors: format!("{}\0vectors", namespace),
    }
}

fn open_namespace(db: fjall::Database, name: &str) -> Result<fjall::Keyspace, Error> {
    match db.keyspace(name, fjall::KeyspaceCreateOptions::default) {
        Ok(keyspace) => Ok(keyspace),
        Err(err) => Err(Error::Internal { err }),
    }
}

/// Top level database handle with the core database and system keyspace, and a map of namespaces.
pub struct Database {
    pub db: fjall::Database,
    pub system: fjall::Keyspace,
    pub default_locale: String,
    pub namespaces: Mutex<HashMap<super::types::DatabaseName, Namespace>>,
}

/// Namespace for a database with handles to the keyspaces to the different keyspace types
pub struct Namespace {
    pub name: super::types::DatabaseName,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub nodes: fjall::Keyspace,
    pub edges: fjall::Keyspace,
    pub vectors: fjall::Keyspace,
}

#[non_exhaustive]
#[repr(u8)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CollationStrength {
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
    Quaternary = 3,
    Identical = 7,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NamespaceConfig {
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
}

struct KeyspaceNames {
    nodes: String,
    edges: String,
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

    #[error("Failed to load namespace config")]
    LoadConfig {
        namespace: String,
        err: serde_json::Error,
    },

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
            Error::LoadConfig { .. } => 0,
            Error::NotFound { .. } => 0,
            Error::Open { .. } => 10,
            Error::SaveNamespaces { .. } => 0,
            Error::System { .. } => 0,
        }
    }
}
