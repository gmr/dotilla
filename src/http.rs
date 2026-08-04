use axum::{
    Router,
    extract::{FromRequestParts, Path, Request, State},
    http::request::Parts,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use thiserror::Error;
use tower_http::timeout::TimeoutLayer;

use crate::{state, storage};

/// Runs the HTTP server until it exits or fails to bind/serve.
///
/// # Errors
///
/// Returns [`Error::ListenFailure`] if the server fails to bind to the specified address.
/// Returns [`Error::ServeFailure`] if the server fails to serve.
pub async fn serve(
    listen_address: std::net::IpAddr,
    port: u16,
    app_state: Arc<state::AppState>,
) -> Result<(), Error> {
    let addr = SocketAddr::new(listen_address, port);
    let listener = bind_listener(addr).await?;
    start_http_server(listener, app_state).await
}

/// Binds the TCP listener to the specified address.
async fn bind_listener(addr: SocketAddr) -> Result<tokio::net::TcpListener, Error> {
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!(
                "Dotilla v{} listening on {addr:?}",
                env!("CARGO_PKG_VERSION")
            );
            Ok(listener)
        }
        Err(e) => Err(Error::ListenFailure { addr, err: e }),
    }
}

/// Creates the router for the HTTP server.
fn create_router(app_state: Arc<state::AppState>) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/health", get(handle_health))
        .route("/{db}", post(handle_cypher_query).put(create_database))
        .fallback(handle_404)
        .layer((
            middleware::from_fn(add_response_headers),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
        ))
        .with_state(app_state)
}

/// Start the HTTP server
async fn start_http_server(
    listener: tokio::net::TcpListener,
    app_state: Arc<state::AppState>,
) -> Result<(), Error> {
    let cancellation_token = app_state.cancellation_token.clone();
    match axum::serve(listener, create_router(app_state))
        .with_graceful_shutdown(cancellation_token.cancelled_owned())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::ServeFailure { err: e }),
    }
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
            Error::ListenFailure { .. } => 6,
            Error::ServeFailure { .. } => 7,
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
    let response = ErrorResponse {
        type_: "about:blank".to_string(),
        title: "Not Found".to_string(),
        status: StatusCode::NOT_FOUND.as_u16(),
        detail: format!("The requested resource {path:?} does not exist."),
        hint: None,
        instance: path,
    };
    (StatusCode::NOT_FOUND, Json(json!(&response)))
}

#[derive(serde::Deserialize)]
struct QueryParams {
    db: storage::DatabaseName,
}

async fn create_database(
    State(app_state): State<Arc<state::AppState>>,
    ValidatedPath(params): ValidatedPath<QueryParams>,
) -> impl IntoResponse {
    let state = app_state.clone();
    match storage::create_keyspace(&state.db, params.db.to_string()).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({"created": "ok"}))),
        Err(err) => {
            let response = ErrorResponse {
                type_: "about:blank".to_string(),
                title: "Database Already Exists".to_string(),
                status: StatusCode::PRECONDITION_FAILED.as_u16(),
                detail: format!("Error creating database `{0}`: {1}", params.db, err),
                instance: params.db.to_string(),
                hint: None,
            };
            (StatusCode::PRECONDITION_FAILED, Json(json!(&response)))
        }
    }
}

async fn handle_cypher_query(
    State(app_state): State<Arc<state::AppState>>,
    ValidatedPath(params): ValidatedPath<QueryParams>,
    body: String,
) -> impl IntoResponse {
    eprintln!("Handling cypher query request for {}", params.db);
    let state = app_state.clone();
    return match storage::keyspace(&state.db, params.db.to_string()).await {
        Ok(_keyspace) => {
            eprintln!("Would query in keyspace {:?}: {body:?}", params.db);
            (
                StatusCode::OK,
                Json(json!({
                    "keyspace": params.db
                })),
            )
        }
        Err(err) => {
            let hint = format!("Create a new database using PUT `/{0}`", params.db);
            let response = ErrorResponse {
                type_: "about:blank".to_string(),
                title: "Not Found".to_string(),
                status: StatusCode::NOT_FOUND.as_u16(),
                detail: format!("Error querying database `{0}`: {1}", params.db, err),
                instance: params.db.to_string(),
                hint: Some(hint),
            };
            (StatusCode::NOT_FOUND, Json(json!(&response)))
        }
    };
}

/// Returns a standardized RFC 7807 JSON Problem Details error response
#[derive(Serialize, Debug)]
struct ErrorResponse {
    #[serde(rename = "type")]
    type_: String,
    title: String,
    status: u16,
    detail: String,
    instance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
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
                let body = ErrorResponse {
                    type_: "about:blank".to_string(),
                    title: "Bad Request".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                    detail: rejection.to_string(),
                    instance: parts.uri.path().to_string(),
                    hint: None,
                };
                Err((StatusCode::BAD_REQUEST, Json(body)).into_response())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{config, cypher, state, storage};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Mutex;
    use tokio::time::{Duration, sleep};
    use tokio_util::sync::CancellationToken;
    use tower::ServiceExt;

    fn build_state() -> Arc<state::AppState> {
        let data_dir = tempfile::tempdir().unwrap();
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 0);
        let occupied = std::net::TcpListener::bind(addr).unwrap();
        let port = occupied.local_addr().unwrap().port();
        let config = config::Config {
            data_directory: data_dir.path().to_path_buf(),
            listen_address: ip_addr,
            port: port,
        };
        Arc::new(state::AppState {
            cancellation_token: CancellationToken::new(),
            cypher_parser: Mutex::new(cypher::build_cypher_parser().unwrap()),
            db: storage::open(&config).unwrap(),
        })
    }

    #[tokio::test]
    async fn bind_listener_ok() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 0);
        assert!(bind_listener(addr).await.is_ok());
    }

    #[tokio::test]
    async fn bind_listener_err() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 32768);
        let listener = bind_listener(addr).await;
        assert!(listener.is_ok());
        match bind_listener(addr).await {
            Ok(_) => assert!(false),
            Err(
                ref error @ Error::ListenFailure {
                    addr: bound_addr,
                    ref err,
                },
            ) => {
                assert_eq!(bound_addr, addr);
                assert!(err.kind() == std::io::ErrorKind::AddrInUse);
                assert_eq!(error.exit_code(), 6);
            }
            Err(_) => assert!(false),
        }
        sleep(Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn start_http_server_ok() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 0);
        let listener = bind_listener(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app_state = build_state();

        let task = tokio::spawn(async move {
            match start_http_server(listener, app_state.clone()).await {
                Ok(_) => assert!(true),
                Err(_) => assert!(false),
            }
        });
        sleep(Duration::from_millis(500)).await;
        let resp = reqwest::get(format!("http://{}/health", addr))
            .await
            .unwrap();
        assert!(resp.status().is_success());
        sleep(Duration::from_millis(500)).await;
        task.abort();
    }

    #[test]
    fn exit_code_listen_failure() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 65535);
        let error = Error::ListenFailure {
            addr: addr,
            err: std::io::Error::new(std::io::ErrorKind::Other, ""),
        };
        assert_eq!(error.exit_code(), 6);
    }

    #[test]
    fn exit_code_serve_failure() {
        let error = Error::ServeFailure {
            err: std::io::Error::new(std::io::ErrorKind::Other, ""),
        };
        assert_eq!(error.exit_code(), 7);
    }

    #[tokio::test]
    async fn test_router_health() {
        let app_state = build_state();
        let router = create_router(app_state.clone());
        let req = Request::get("/health").body(Body::empty()).unwrap();
        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_index() {
        let app_state = build_state();
        let router = create_router(app_state.clone());
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
        let app_state = build_state();
        let router = create_router(app_state.clone());
        let req = Request::get("/not_found/foo").body(Body::empty()).unwrap();
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
                path = "/not_found/foo"
            )
        );
        assert_eq!(body["instance"], "/not_found/foo".to_string());
    }
}
