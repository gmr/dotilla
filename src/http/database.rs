use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use std::sync::Arc;

use crate::{state, storage::errors, storage::namespace};

/// Return information about the database, including per namespace details
pub async fn info(State(state): State<Arc<state::AppState>>) -> impl IntoResponse {
    let journal_count_future = state.database.journal_count();
    let keyspace_count_future = state.database.keyspace_count();
    let size_on_disk_future = state.database.size_on_disk();
    let write_buffer_size_future = state.database.write_buffer_size();
    match futures::try_join!(
        journal_count_future,
        keyspace_count_future,
        size_on_disk_future,
        write_buffer_size_future,
    ) {
        Ok((journal_count, keyspace_count, size_on_disk, write_buffer_size)) => {
            match get_namespaces_details(&state).await {
                Ok(namespaces) => {
                    let report = DatabaseInfo {
                        size_on_disk,
                        journal_count,
                        keyspace_count,
                        write_buffer_size,
                        namespaces,
                    };
                    (StatusCode::OK, Json(report).into_response())
                }
                Err(err) => super::utils::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database system error: {}", err),
                    "_db_info".to_string(),
                    None,
                ),
            }
        }
        Err(err) => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database system error: {}", err),
            "_db_info".to_string(),
            None,
        ),
    }
}

async fn get_namespaces_details(
    state: &Arc<state::AppState>,
) -> Result<Vec<namespace::Details>, errors::Error> {
    let namespaces = state.list_namespaces();
    let mut details = Vec::new();
    for name in namespaces {
        let ns = namespace::Namespace::get(&state.database, &name).await?;
        details.push(ns.details().await?);
    }
    Ok(details)
}

#[derive(serde::Serialize)]
struct DatabaseInfo {
    pub size_on_disk: u64,
    pub journal_count: usize,
    pub keyspace_count: usize,
    pub write_buffer_size: u64,
    pub namespaces: Vec<namespace::Details>,
}
