use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use std::sync::Arc;

use crate::{state, storage::errors, storage::namespace};

/// Deletes a namespace
pub async fn delete(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    let ns = match namespace::Namespace::get(&state.database, params.namespace.as_ref()).await {
        Ok(ns) => ns,
        Err(err) => return error_response(err, params.namespace.as_ref()),
    };
    match ns.delete().await {
        Ok(_) => {
            state.remove_namespace(ns);
            (StatusCode::NO_CONTENT, "".into_response())
        }
        Err(err) => error_response(err, params.namespace.as_ref()),
    }
}

/// Returns details about a namespace
pub async fn get(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match namespace::Namespace::get(&state.database, params.namespace.as_ref()).await {
        Ok(ns) => match ns.details().await {
            Ok(details) => (StatusCode::OK, Json(details).into_response()),
            Err(err) => error_response(err, params.namespace.as_ref()),
        },
        Err(err) => error_response(err, params.namespace.as_ref()),
    }
}

/// Validates a namespace exists
pub async fn head(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match namespace::Namespace::get(&state.database, params.namespace.as_ref()).await {
        Ok(_) => (StatusCode::OK, "".into_response()),
        Err(err) => error_response(err, params.namespace.as_ref()),
    }
}

/// Return a list of namespaces
pub async fn list(State(state): State<Arc<state::AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(state.list_namespaces()).into_response(),
    )
}

/// Query the database using Cypher
pub async fn post(
    State(_app_state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
    _body: String,
) -> impl IntoResponse {
    super::utils::not_implemented(
        "Error querying database: Not Implemented".to_string(),
        params.namespace.to_string(),
    )
}

/// Create a new namespace
pub async fn put(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
    payload: Json<CreateBody>,
) -> impl IntoResponse {
    match namespace::Namespace::create(
        &state.database,
        params.namespace.as_ref(),
        payload.locale.clone(),
        payload.case_insensitive,
        payload.collation_strength.clone(),
    )
    .await
    {
        Ok(ns) => {
            state.add_namespace(ns);
            (
                StatusCode::CREATED,
                Json(CreateOkResponse { ok: true }).into_response(),
            )
        }
        Err(err) => error_response(err, params.namespace.as_ref()),
    }
}

/// Query parameters for the namespace endpoints.
#[derive(serde::Deserialize)]
pub struct QueryParams {
    namespace: namespace::Name,
}

/// Body for creating a namespace
#[derive(serde::Deserialize)]
pub struct CreateBody {
    pub locale: Option<String>,
    pub case_insensitive: Option<bool>,
    pub collation_strength: Option<namespace::CollationStrength>,
}

/// Response body for creating a namespace
#[derive(serde::Serialize)]
struct CreateOkResponse {
    ok: bool,
}

/// Return an error response based on the error returned from the storage layer
fn error_response(error: errors::Error, namespace: &str) -> (StatusCode, Response<Body>) {
    match error {
        errors::Error::Avro(err) => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Encoding / decoding error for namespace: {}", err),
            namespace.to_string(),
            None,
        ),
        errors::Error::Database(_err) => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal database error".to_string(),
            "".to_string(),
            None,
        ),
        errors::Error::IO(err) => super::utils::error_response(
            StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal IO error processing namespace: {}", err),
            namespace.to_string(),
            None,
        ),
        errors::Error::NamespaceExists { namespace } => super::utils::error_response(
            StatusCode::BAD_REQUEST.to_string(),
            StatusCode::BAD_REQUEST,
            "Namespace already exists".to_string(),
            namespace.to_string(),
            None,
        ),
        errors::Error::NamespaceInvalidName(err) => super::utils::error_response(
            StatusCode::BAD_REQUEST.to_string(),
            StatusCode::BAD_REQUEST,
            format!("Invalid namespace name: {}", err),
            namespace.to_string(),
            None,
        ),
        errors::Error::NotFound => super::utils::error_response(
            StatusCode::NOT_FOUND.to_string(),
            StatusCode::NOT_FOUND,
            "Namespace not found".to_string(),
            namespace.to_string(),
            None,
        ),
        _ => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected error".to_string(),
            namespace.to_string(),
            None,
        ),
    }
}
