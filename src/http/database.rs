use crate::{state, storage, storage::database};
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    db: storage::types::DatabaseName,
}

pub async fn delete(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match storage::database::delete_namespace(&state.db, &params.db.to_string()).await {
        Ok(_) => (StatusCode::NO_CONTENT, "".into_response()),
        Err(err) => error_response(err),
    }
}

pub async fn get(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match storage::database::get_namespace(&state.db, &params.db.to_string()).await {
        Ok(details) => (StatusCode::OK, Json(details).into_response()),
        Err(err) => error_response(err),
    }
}

pub async fn head(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    match storage::database::get_namespace(&state.db, &params.db.to_string()).await {
        Ok(_) => (StatusCode::OK, "".into_response()),
        Err(err) => error_response(err),
    }
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

pub async fn all_dbs(State(state): State<Arc<state::AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(storage::database::all_namespaces(&state.db)).into_response(),
    )
}

pub async fn db_info(State(state): State<Arc<state::AppState>>) -> impl IntoResponse {
    match storage::database::db_info(&state.db).await {
        Ok(info) => (StatusCode::OK, Json(info).into_response()),
        Err(err) => error_response(err),
    }
}

#[derive(serde::Deserialize)]

pub struct CreateBody {
    pub locale: Option<String>,
    pub case_insensitive: Option<bool>,
    pub collation_strength: Option<database::CollationStrength>,
}

#[derive(serde::Serialize)]
struct CreateOkResponse {
    ok: bool,
}

pub async fn put(
    State(state): State<Arc<state::AppState>>,
    super::types::ValidatedPath(params): super::types::ValidatedPath<QueryParams>,
    payload: Json<CreateBody>,
) -> impl IntoResponse {
    match database::create_namespace(
        &state.db,
        &params.db.to_string(),
        payload.locale.clone(),
        payload.case_insensitive,
        payload.collation_strength.clone(),
    )
    .await
    {
        Ok(()) => (
            StatusCode::CREATED,
            Json(CreateOkResponse { ok: true }).into_response(),
        ),
        Err(err) => error_response(err),
    }
}

fn error_response(error: database::Error) -> (StatusCode, Response<Body>) {
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
        database::Error::LoadConfig { namespace, err } => super::utils::error_response(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Failed to load config for database `{}`: {}",
                namespace, err
            ),
            namespace.to_string(),
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
