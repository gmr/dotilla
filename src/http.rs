use crate::config::Config;
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
};
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::LazyLock;
use thiserror::Error;

/// Runs the HTTP server until it exits or fails to bind/serve.
///
/// # Errors
///
/// Returns [`Error::ListenFailure`] if the server fails to bind to the specified address.
/// Returns [`Error::ServeFailure`] if the server fails to serve.
pub async fn serve(config: &Config) -> Result<(), Error> {
    let router = create_router();
    let addr = SocketAddr::new(config.listen_address, config.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Error::ListenFailure { addr, err: e })?;
    println!(
        "Dotilla v{} listening on {addr:?}",
        env!("CARGO_PKG_VERSION")
    );
    axum::serve(listener, router)
        .await
        .map_err(|e| Error::ServeFailure { err: e })
}

/// Creates the router for the HTTP server.
fn create_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .fallback(handle_404)
        .layer(middleware::from_fn(add_response_headers))
}

/// Errors that can occur while starting or running the HTTP server.
#[derive(Debug, Error)]
pub enum Error {
    /// The server failed to bind to the specified address.
    #[error("Failed to bind to {addr:?}: {err}")]
    ListenFailure {
        addr: SocketAddr,
        err: std::io::Error,
    },

    /// The server failed to start.
    #[error("Failed to start http server: {err}")]
    ServeFailure { err: std::io::Error },
}

impl Error {
    /// Returns the exit code for each error type.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::ListenFailure { .. } => 5,
            Error::ServeFailure { .. } => 6,
        }
    }
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

// --- Route Handlers ---

async fn index() -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

async fn handle_404(req: Request) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    (
        StatusCode::NOT_FOUND,
        error_response(
            "about:blank",
            "Not Found",
            StatusCode::NOT_FOUND.as_u16(),
            format!("The requested resource {path:?} does not exist."),
            path,
        ),
    )
}

/// Returns a standardized RFC 7807 JSON Problem Details error response
fn error_response(
    error_type: impl Into<String>,
    title: impl Into<String>,
    status: u16,
    detail: impl Into<String>,
    instance: impl Into<String>,
) -> Json<Value> {
    Json(json!({
        "type": error_type.into(),
        "title": title.into(),
        "status": status,
        "detail": detail.into(),
        "instance": instance.into(),
    }))
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tower::ServiceExt;

    #[test]
    fn exit_code_listen_failure() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 65535);
        let error = Error::ListenFailure {
            addr: addr,
            err: std::io::Error::new(std::io::ErrorKind::Other, ""),
        };
        assert_eq!(error.exit_code(), 5);
    }

    #[test]
    fn exit_code_serve_failure() {
        let error = Error::ServeFailure {
            err: std::io::Error::new(std::io::ErrorKind::Other, ""),
        };
        assert_eq!(error.exit_code(), 6);
    }

    #[tokio::test]
    async fn test_router_health() {
        let router = create_router();
        let req = Request::get("/health").body(Body::empty()).unwrap();
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_index() {
        let router = create_router();
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
        let router = create_router();
        let req = Request::get("/not_found").body(Body::empty()).unwrap();
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["type"], "about:blank".to_string());
        assert_eq!(body["title"], "Not Found".to_string());
        assert_eq!(body["status"], StatusCode::NOT_FOUND.as_u16());
        assert_eq!(
            body["detail"],
            format!(
                "The requested resource {path:?} does not exist.",
                path = "/not_found"
            )
        );
        assert_eq!(body["instance"], "/not_found".to_string());
    }
}
