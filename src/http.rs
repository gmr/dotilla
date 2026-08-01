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
    let router = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .fallback(handle_404)
        .layer(middleware::from_fn(add_response_headers));
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
    let host = req
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    (
        StatusCode::NOT_FOUND,
        error_response(
            format!("http://{host}"),
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
