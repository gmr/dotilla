use std::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tree_sitter::Parser;

/// Used to carry the runtime state of the app across modules and requests
pub struct AppState {
    /// Used when the app is shutting down to cancel pending tasks
    pub cancellation_token: CancellationToken,

    /// Application Configuration
    pub config: crate::config::Config,

    /// Used to parse Cypher queries
    pub cypher_parser: Mutex<Parser>,

    /// The handle to the database system
    pub db: crate::storage::types::Database,
}
