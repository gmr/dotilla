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

static SERVER_VERSION: LazyLock<HeaderValue> = LazyLock::new(|| {
    HeaderValue::from_str(&format!("Dotilla/{}", env!("CARGO_PKG_VERSION")))
        .expect("Failed to parse server version")
});

pub fn error_response(
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

async fn add_response_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("Server", SERVER_VERSION.clone());
    response
}

pub async fn index() -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

pub async fn serve(config: &Config) {
    let router = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .fallback(handle_404)
        .layer(middleware::from_fn(add_response_headers));
    let addr = SocketAddr::new(config.listen_address, config.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr:?}: {e}"));
    println!(
        "Dotilla v{} listening on {addr:?}",
        env!("CARGO_PKG_VERSION")
    );
    axum::serve(listener, router)
        .await
        .expect("Failed to start the axum server");
}
