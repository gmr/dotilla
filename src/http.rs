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
    error_type: String,
    title: String,
    status: u16,
    detail: String,
    instance: String,
) -> Json<Value> {
    Json(json!({
        "type": error_type,
        "title": title,
        "status": status,
        "detail": detail,
        "instance": instance,
    }))
}

async fn handler_404(req: Request) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        error_response(
            "https://ietf.org".to_string(),
            "Not Found".to_string(),
            StatusCode::NOT_FOUND.as_u16(),
            format!(
                "The requested resource {0} does not exist.",
                req.uri().path()
            ),
            req.uri().path().to_string(),
        ),
    )
}

async fn add_response_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .append("Server", SERVER_VERSION.clone());
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
        .layer(middleware::from_fn(add_response_headers))
        .route("/", get(index))
        .route("/health", get(health))
        .fallback(handler_404);

    let addr = SocketAddr::new(config.listen_address, config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
