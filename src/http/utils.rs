use axum::http::StatusCode;
use axum::response::Json;

pub fn not_implemented(
    detail: String,
    instance: String,
) -> (StatusCode, Json<super::types::ErrorResponse>) {
    error_response(
        "Not Implemented".to_string(),
        StatusCode::NOT_IMPLEMENTED,
        detail,
        instance,
        None,
    )
}

pub fn error_response(
    title: String,
    status: StatusCode,
    detail: String,
    instance: String,
    hint: Option<String>,
) -> (StatusCode, Json<super::types::ErrorResponse>) {
    let response = super::types::ErrorResponse {
        type_: "about:blank".to_string(),
        title,
        status: status.as_u16(),
        detail,
        instance,
        hint,
    };
    (StatusCode::NOT_IMPLEMENTED, Json(response))
}
