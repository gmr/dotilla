use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use tree_sitter::Parser;

use crate::http::server;
use crate::storage::{database, errors, namespace};
use crate::{config, cypher};

/// Used to carry the runtime state of the app across modules and requests
pub struct AppState {
    /// Used when the app is shutting down to cancel pending tasks
    pub cancellation_token: CancellationToken,

    /// Application Configuration
    pub config: crate::config::Config,

    /// Used to parse Cypher queries
    pub cypher_parser: Mutex<Parser>,

    /// The handle to the database system
    pub database: Arc<database::Database>,

    /// The handle to the namespace cache
    pub namespaces: Mutex<HashMap<namespace::Name, namespace::Namespace>>,
}

impl AppState {
    pub async fn initialize(config_path: PathBuf) -> Result<Arc<Self>, StartupError> {
        let config = config::load(config_path).map_err(|e| StartupError::Config { err: e })?;
        let db = Arc::new(
            database::Database::initialize(&config)
                .await
                .map_err(|e| StartupError::Database { err: e })?,
        );
        let namespaces = namespace::load_all(&db).await?;
        Ok(Arc::new(Self {
            cancellation_token: CancellationToken::new(),
            config: config.clone(),
            cypher_parser: Mutex::new(cypher::build_cypher_parser().unwrap()),
            database: db,
            namespaces: Mutex::new(namespaces),
        }))
    }

    pub fn add_namespace(&self, ns: namespace::Namespace) {
        self.namespaces.lock().unwrap().insert(ns.name.clone(), ns);
    }

    pub fn list_namespaces(&self) -> Vec<String> {
        self.namespaces
            .lock()
            .unwrap()
            .keys()
            .map(|k| k.to_string())
            .collect()
    }

    pub fn remove_namespace(&self, ns: namespace::Namespace) {
        self.namespaces.lock().unwrap().remove(&ns.name);
    }
}

/// Errors that can occur during startup.
#[derive(Debug, Error)]
pub enum StartupError {
    /// Error reading or validating the configuration file
    #[error("Configuration error: {err}")]
    Config {
        #[from]
        err: config::Error,
    },

    /// Error spawning multiple tasks
    #[error("Error spawning multiple tasks: {err}")]
    Task {
        #[from]
        err: tokio::task::JoinError,
    },

    /// Error starting the HTTP server
    #[error("HTTP Server error: {err}")]
    Http {
        #[from]
        err: server::Error,
    },

    /// Error opening the database
    #[error("Graph database initialization error: {err}")]
    Database {
        #[from]
        err: database::Error,
    },

    /// Error loading namespaces
    #[error("Error loading namespaces: {err}")]
    Namespaces {
        #[from]
        err: errors::Error,
    },
}

impl StartupError {
    /// Returns the exit code for the error.
    pub fn exit_code(&self) -> i32 {
        match self {
            StartupError::Config { err } => err.exit_code(),
            StartupError::Http { err } => err.exit_code(),
            StartupError::Task { .. } => 1,
            StartupError::Database { err } => err.exit_code(),
            StartupError::Namespaces { .. } => 5,
        }
    }
}
