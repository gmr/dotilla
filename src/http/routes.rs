use std::sync::Arc;
use std::sync::LazyLock;

use axum;
use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get},
};
use serde_json::{Value, json};

use crate::state;

static SERVER_VERSION: LazyLock<HeaderValue> = LazyLock::new(|| {
    HeaderValue::from_str(&format!("Dotilla/{}", env!("CARGO_PKG_VERSION")))
        .expect("Failed to parse server version")
});

pub struct Router {
    pub router: axum::Router,
}

impl Router {
    pub fn new(state: Arc<state::AppState>) -> Self {
        Self {
            router: Self::create_router(state),
        }
    }

    fn create_router(state: Arc<state::AppState>) -> axum::Router {
        // @TODO: Implement a timeout function for the middleware
        axum::Router::new()
            .route("/", get(Self::handle_index))
            .route("/_all_namespaces", get(super::namespace::list))
            .route("/_db_info", get(super::database::info))
            .route("/_health", get(Self::handle_health))
            .route(
                "/{namespace}",
                delete(super::namespace::delete)
                    .get(super::namespace::get)
                    .head(super::namespace::head)
                    .post(super::namespace::post)
                    .put(super::namespace::put),
            )
            .fallback(Self::handle_404)
            .layer((middleware::from_fn(Self::add_response_headers),))
            .with_state(state.clone())
    }

    async fn add_response_headers(req: Request, next: Next) -> Response {
        let mut response = next.run(req).await;
        response
            .headers_mut()
            .insert("Server", SERVER_VERSION.clone());
        response
    }

    async fn handle_404(req: Request) -> impl IntoResponse {
        let path = req.uri().path().to_string();
        (
            StatusCode::NOT_FOUND,
            super::utils::error_response(
                StatusCode::NOT_FOUND.to_string(),
                StatusCode::NOT_FOUND,
                "The requested resource does not exist.".to_string(),
                path,
                None,
            ),
        )
    }

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
}

#[cfg(test)]
mod tests {

    use axum_test::TestServer;
    use test_context::test_context;

    use super::*;
    use crate::test_helpers::TestContext;

    #[test_context(TestContext)]
    #[compio::test]
    async fn health(ctx: &mut TestContext) {
        let server = TestServer::new(Router::new(ctx.state.clone()).router);
        let response = server.get("/_health").await;
        response.assert_status(StatusCode::OK);
    }

    #[test_context(TestContext)]
    #[compio::test]
    async fn index(ctx: &mut TestContext) {
        let server = TestServer::new(Router::new(ctx.state.clone()).router);
        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);
        response.assert_json(&json!({
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
        }));
    }

    #[test_context(TestContext)]
    #[compio::test]
    async fn not_found(ctx: &mut TestContext) {
        let server = TestServer::new(Router::new(ctx.state.clone()).router);
        let response = server.get("/not-found/foo").await;
        response.assert_status(StatusCode::NOT_FOUND);
        response.assert_json(&json!({
            "type": "about:blank",
            "title":  StatusCode::NOT_FOUND.to_string(),
            "status": StatusCode::NOT_FOUND.as_u16(),
            "detail":"The requested resource does not exist.",
            "instance": "/not-found/foo",
        }));
    }
}
