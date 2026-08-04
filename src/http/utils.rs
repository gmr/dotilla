use axum::http::StatusCode;
use axum::response::Json;

pub fn not_implemented(
    detail: String,
    instance: String,
) -> (StatusCode, Json<super::types::ErrorResponse>) {
    let response = super::types::ErrorResponse {
        type_: "about:blank".to_string(),
        title: "Not Implemented".to_string(),
        status: StatusCode::NOT_IMPLEMENTED.as_u16(),
        detail,
        instance,
        hint: None,
    };
    (StatusCode::NOT_IMPLEMENTED, Json(response))
}
