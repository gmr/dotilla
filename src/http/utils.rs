use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};

/// Returns a `NOT_IMPLEMENTED` error response with the given detail and instance.
pub fn not_implemented(detail: String, instance: String) -> (StatusCode, Response<Body>) {
    error_response(
        StatusCode::NOT_IMPLEMENTED.to_string(),
        StatusCode::NOT_IMPLEMENTED,
        detail,
        instance,
        None,
    )
}

/// Returns an error response with the given title, status, detail, instance, and hint.
pub fn error_response(
    title: String,
    status: StatusCode,
    detail: String,
    instance: String,
    hint: Option<String>,
) -> (StatusCode, Response<Body>) {
    let response = super::types::ErrorResponse {
        type_: "about:blank".to_string(),
        title,
        status: status.as_u16(),
        detail,
        instance,
        hint,
    };
    (status, Json(response).into_response())
}
