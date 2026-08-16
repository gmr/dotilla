use axum::http::StatusCode;
use axum::response::Response;
use axum::{
    Json,
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::IntoResponse,
};
use serde::Serialize;

/// Returns a standardized RFC 7807 JSON Problem Details error response
#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

pub struct ValidatedPath<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedPath<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::<T>::from_request_parts(parts, state).await {
            Ok(Path(value)) => Ok(ValidatedPath(value)),
            Err(rejection) => {
                let path = parts.uri.path();
                let path = path.strip_prefix('/').unwrap_or(path);
                let body = ErrorResponse {
                    type_: "about:blank".to_string(),
                    title: "Bad Request".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                    detail: rejection.to_string(),
                    instance: path.to_string(),
                    hint: None,
                };
                Err((StatusCode::BAD_REQUEST, Json(body)).into_response())
            }
        }
    }
}
