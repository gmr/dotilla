use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::storage::{database, errors, namespace};
use crate::{config, cypher};

/// Used to carry the runtime state of the app across modules and requests
pub struct AppState {
    /// Used when the app is shutting down to cancel pending tasks
    pub cancellation_token: CancellationToken,

    /// Application Configuration
    pub config: crate::config::Config,

    /// Used to parse Cypher queries
    pub cypher_parser: cypher::Parser,

    /// The handle to the database system
    pub database: Arc<database::Database>,

    /// The handle to the namespace cache
    pub namespaces: Mutex<HashMap<namespace::Name, Arc<namespace::Namespace>>>,
}

impl AppState {
    pub async fn initialize(config_path: PathBuf) -> Result<Arc<Self>, StartupError> {
        let config = config::load(config_path)?;
        let db = Arc::new(database::Database::initialize(&config).await?);
        let namespaces = namespace::load_all(&db).await?;
        Ok(Arc::new(Self {
            cancellation_token: CancellationToken::new(),
            config: config.clone(),
            cypher_parser: cypher::Parser::new().unwrap(),
            database: db,
            namespaces: Mutex::new(namespaces),
        }))
    }

    pub fn get_namespace(&self, name: &str) -> Option<Arc<namespace::Namespace>> {
        self.namespaces.lock().unwrap().get(name).cloned()
    }

    pub fn list_namespaces(&self) -> Vec<String> {
        self.namespaces
            .lock()
            .unwrap()
            .keys()
            .map(|k| k.to_string())
            .collect()
    }

    pub fn maybe_add_namespace(&self, ns: Arc<namespace::Namespace>) -> Arc<namespace::Namespace> {
        // Minor race condition
        let mut guard = self.namespaces.lock().unwrap();
        match guard.get(&ns.name) {
            Some(ns) => ns.clone(),
            None => {
                guard.insert(ns.name.clone(), ns.clone());
                ns
            }
        }
    }

    pub fn remove_namespace(&self, name: &str) {
        self.namespaces.lock().unwrap().remove(name);
    }
}

/// Errors that can occur during startup.
#[derive(Debug, Error)]
pub enum StartupError {
    /// Error reading or validating the configuration file
    #[error("Configuration error: {0}")]
    Config(#[from] config::Error),

    /// Error opening the database
    #[error("Graph database initialization error: {0}")]
    Database(#[from] database::Error),

    /// Error loading namespaces
    #[error("Error loading namespaces: {0}")]
    Namespaces(#[from] errors::Error),

    /// Error parsing Cypher queries
    #[error("Error initializing Cypher parser: {0}")]
    Cypher(#[from] tree_sitter::LanguageError),
}

impl StartupError {
    /// Returns the exit code for the error.
    pub fn exit_code(&self) -> i32 {
        match self {
            StartupError::Config(err) => err.exit_code(),
            StartupError::Database(err) => err.exit_code(),
            StartupError::Namespaces(..) => 6,
            StartupError::Cypher(..) => 7,
        }
    }
}
