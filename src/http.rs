use crate::config::Config;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use std::net::SocketAddr;

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn serve(config: &Config) {
    let router = Router::new()
        .route(
            "/",
            get(|| async { format!("Dotilla v{}\n", env!("CARGO_PKG_VERSION")) }),
        )
        .fallback(handler_404);

    let addr = SocketAddr::new(config.listen_address, config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
