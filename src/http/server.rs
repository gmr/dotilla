use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;

use crate::state;

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

/// Start the HTTP server
async fn start_http_server(
    listener: tokio::net::TcpListener,
    app_state: Arc<state::AppState>,
) -> Result<(), Error> {
    let cancellation_token = app_state.cancellation_token.clone();
    match axum::serve(listener, super::routes::create(app_state))
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

#[cfg(test)]
mod tests {

    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use test_context::test_context;
    use tokio::time::{Duration, sleep};

    use crate::test_helpers::TestContext;

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

    #[test_context(TestContext)]
    #[tokio::test]
    async fn start_http_server_ok(ctx: &mut TestContext) {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 0);
        let listener = bind_listener(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = ctx.state.clone();

        let task = tokio::spawn(async move {
            match start_http_server(listener, state).await {
                Ok(_) => assert!(true),
                Err(_) => assert!(false),
            }
        });
        sleep(Duration::from_millis(500)).await;
        let resp = reqwest::get(format!("http://{}/_health", addr))
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
}
