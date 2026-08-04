use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::{state, storage};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    db: storage::types::DatabaseName,
}

pub async fn create(
    State(_app_state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    super::utils::not_implemented(
        format!("Error creating database `{0}`: Not Implemented", params.db),
        params.db.to_string(),
    )
}

pub async fn delete(
    State(_app_state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    super::utils::not_implemented(
        format!("Error deleting database `{0}`: Not Implemented", params.db),
        params.db.to_string(),
    )
}

pub async fn head(
    State(_app_state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    super::utils::not_implemented(
        format!(
            "Error retrieving database information `{0}`: Not Implemented",
            params.db
        ),
        params.db.to_string(),
    )
}

pub async fn post(
    State(_app_state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
    _body: String,
) -> impl IntoResponse {
    super::utils::not_implemented(
        format!("Error querying database `{0}`: Not Implemented", params.db),
        params.db.to_string(),
    )
}
