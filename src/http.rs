use crate::config::Config;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

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

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.listen_address, config.port))
            .await
            .unwrap();
    axum::serve(listener, router).await.unwrap();
}
