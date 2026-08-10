use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use std::sync::Arc;

use crate::{state, storage::database};

/// Return information about the database, including per namespace details
pub async fn info(State(state): State<Arc<state::AppState>>) -> impl IntoResponse {
    match database::info(&state.db).await {
        Ok(info) => (StatusCode::OK, Json(info).into_response()),
        Err(_err) => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database system error".to_string(),
            "".to_string(),
            None,
        ),
    }
}
