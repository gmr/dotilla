use axum::http::StatusCode;
use axum::response::Json;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::{state, storage, storage::database};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    db: storage::types::DatabaseName,
}

pub async fn create(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match database::create_namespace(&state.db, &params.db.to_string()).await {
        Ok(()) => (StatusCode::CREATED, Json("{\"ok\": true}").into_response()),
        Err(err) => {
            let response = error_response(err);
            (response.0, response.1.into_response()) // (StatusCode, Response<Body>)
        }
    }
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

fn error_response(error: database::Error) -> (StatusCode, Json<super::types::ErrorResponse>) {
    match error {
        database::Error::AlreadyExists { namespace } => super::utils::error_response(
            "Bad Request".to_string(),
            StatusCode::BAD_REQUEST,
            format!("Database `{}` already exists", namespace),
            namespace.to_string(),
            None,
        ),
        database::Error::Create { namespace, err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create database `{}`: {}", namespace, err),
            namespace.to_string(),
            None,
        ),
        database::Error::Internal { err: _err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal database error".to_string(),
            "".to_string(),
            None,
        ),
        database::Error::NotFound { namespace } => super::utils::error_response(
            "Not Found".to_string(),
            StatusCode::NOT_FOUND,
            format!("Database `{}` not found", namespace),
            namespace.to_string(),
            None,
        ),
        database::Error::Open(_) => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error opening the database".to_string(),
            "".to_string(),
            None,
        ),
        database::Error::SaveNamespaces { err: _err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving internal state".to_string(),
            "".to_string(),
            None,
        ),
        database::Error::System(_) => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database system error".to_string(),
            "".to_string(),
            None,
        ),
    }
}
