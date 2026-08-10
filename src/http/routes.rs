use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get},
};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

use crate::state;

/// Creates the router for the HTTP server.
pub fn create(app_state: Arc<state::AppState>) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/_all_namespaces", get(super::namespace::list))
        .route("/_db_info", get(super::database::info))
        .route("/_health", get(handle_health))
        .route(
            "/{namespace}",
            delete(super::namespace::delete)
                .get(super::namespace::get)
                .head(super::namespace::head)
                .post(super::namespace::post)
                .put(super::namespace::put),
        )
        .fallback(handle_404)
        .layer((
            middleware::from_fn(add_response_headers),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
        ))
        .with_state(app_state)
}

// --- Base Route Handlers ---

async fn handle_index() -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn handle_health() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

async fn handle_404(req: Request) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    (
        StatusCode::NOT_FOUND,
        super::utils::error_response(
            StatusCode::NOT_FOUND.to_string(),
            StatusCode::NOT_FOUND,
            format!("The requested resource {path:?} does not exist."),
            path,
            None,
        ),
    )
}

// --- Middleware ---

static SERVER_VERSION: LazyLock<HeaderValue> = LazyLock::new(|| {
    HeaderValue::from_str(&format!("Dotilla/{}", env!("CARGO_PKG_VERSION")))
        .expect("Failed to parse server version")
});

async fn add_response_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("Server", SERVER_VERSION.clone());
    response
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_router_health() {
        let app_state = crate::test_helpers::build_state().await;
        let router = create(app_state.clone());
        let req = Request::get("/_health").body(Body::empty()).unwrap();
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_index() {
        let app_state = crate::test_helpers::build_state().await;
        let router = create(app_state.clone());
        let req = Request::get("/").body(Body::empty()).unwrap();
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["name"], env!("CARGO_PKG_NAME"));
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_router_file_not_found() {
        let app_state = crate::test_helpers::build_state().await;
        let router = create(app_state.clone());
        let req = Request::get("/not_found/foo").body(Body::empty()).unwrap();
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["type"], "about:blank".to_string());
        assert_eq!(body["title"], StatusCode::NOT_FOUND.to_string());
        assert_eq!(body["status"], StatusCode::NOT_FOUND.as_u16());
        assert_eq!(
            body["detail"],
            format!(
                "The requested resource {path:?} does not exist.",
                path = "/not_found/foo"
            )
        );
        assert_eq!(body["instance"], "/not_found/foo".to_string());
    }
}
