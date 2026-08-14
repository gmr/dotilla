use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use std::sync::Arc;

use crate::{state, storage, storage::namespace};

/// Deletes a namespace
pub async fn delete(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match namespace::delete(&state.db, params.namespace.as_ref()).await {
        Ok(_) => (StatusCode::NO_CONTENT, "".into_response()),
        Err(err) => error_response(err),
    }
}

/// Returns details about a namespace
pub async fn get(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match namespace::get(&state.db, params.namespace.as_ref()).await {
        Ok(details) => (StatusCode::OK, Json(details).into_response()),
        Err(err) => error_response(err),
    }
}

/// Validates a namespace exists
pub async fn head(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match namespace::get(&state.db, params.namespace.as_ref()).await {
        Ok(_) => (StatusCode::OK, "".into_response()),
        Err(err) => error_response(err),
    }
}

/// Return a list of namespaces
pub async fn list(State(state): State<Arc<state::AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(storage::namespace::list(&state.db)).into_response(),
    )
}

/// Query the database using Cypher
pub async fn post(
    State(_app_state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
    _body: String,
) -> impl IntoResponse {
    super::utils::not_implemented(
        format!(
            "Error querying database `{0}`: Not Implemented",
            params.namespace
        ),
        params.namespace.to_string(),
    )
}

/// Create a new namespace
pub async fn put(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
    payload: Json<CreateBody>,
) -> impl IntoResponse {
    match namespace::create(
        &state.db,
        params.namespace.as_ref(),
        payload.locale.clone(),
        payload.case_insensitive,
        payload.collation_strength.clone(),
    )
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(CreateOkResponse { ok: true }).into_response(),
        ),
        Err(err) => error_response(err),
    }
}

/// Query parameters for the namespace endpoints.
#[derive(serde::Deserialize)]
pub struct QueryParams {
    namespace: storage::types::NamespaceName,
}

/// Body for creating a namespace
#[derive(serde::Deserialize)]
pub struct CreateBody {
    pub locale: Option<String>,
    pub case_insensitive: Option<bool>,
    pub collation_strength: Option<storage::types::CollationStrength>,
}

/// Response body for creating a namespace
#[derive(serde::Serialize)]
struct CreateOkResponse {
    ok: bool,
}

/// Return an error response based on the error returned from the storage layer
fn error_response(error: namespace::Error) -> (StatusCode, Response<Body>) {
    match error {
        namespace::Error::AlreadyExists { namespace } => super::utils::error_response(
            StatusCode::BAD_REQUEST.to_string(),
            StatusCode::BAD_REQUEST,
            format!("Namespace `{}` already exists", namespace),
            namespace.to_string(),
            None,
        ),
        namespace::Error::Database(_err) => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal database error".to_string(),
            "".to_string(),
            None,
        ),
        namespace::Error::LoadConfig { namespace, err } => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Failed to load config for namespace `{}`: {}",
                namespace, err
            ),
            namespace.to_string(),
            None,
        ),
        namespace::Error::NotFound { namespace } => super::utils::error_response(
            StatusCode::NOT_FOUND.to_string(),
            StatusCode::NOT_FOUND,
            format!("Namespace `{}` not found", namespace),
            namespace.to_string(),
            None,
        ),
        namespace::Error::Open { err: _err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error opening namespace".to_string(),
            "".to_string(),
            None,
        ),
        namespace::Error::Save { err: _err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving internal state".to_string(),
            "".to_string(),
            None,
        ),
        namespace::Error::System { err: _err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving internal state".to_string(),
            "".to_string(),
            None,
        ),
    }
}
