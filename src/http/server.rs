use std::net::SocketAddr;
use std::pin::pin;
use std::sync::Arc;

use compio::signal::{ctrl_c, unix};
use futures_util::future::select;
use thiserror::Error;

use super::routes::Router;
use crate::state;

pub struct Server {
    state: Arc<state::AppState>,
}

impl Server {
    pub fn new(state: Arc<state::AppState>) -> Self {
        Self { state }
    }

    pub async fn serve(&self) -> Result<(), Error> {
        let listener = self.bind().await?;
        match cyper_axum::serve(listener, Router::new(self.state.clone()).router)
            .with_graceful_shutdown(Self::signal_handler())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::ServeFailure { err: e }),
        }
    }

    async fn bind(&self) -> Result<compio::net::TcpListener, Error> {
        let addr = SocketAddr::new(self.state.config.listen_address, self.state.config.port);
        match compio::net::TcpListener::bind(addr).await {
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

    async fn signal_handler() {
        let ctrl_c_fut = async {
            ctrl_c().await.unwrap();
        };
        #[cfg(unix)]
        let terminate_fut = async {
            unix::signal(15).await.unwrap();
        };
        #[cfg(not(unix))]
        let terminate_fut = std::future::pending::<()>();

        let ctrl_c_pinned = pin!(ctrl_c_fut);
        let terminate_pinned = pin!(terminate_fut);

        select(ctrl_c_pinned, terminate_pinned).await;
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

    use std::time::Duration;

    use compio::net::TcpStream;
    use compio::time::sleep;
    use test_context::test_context;

    use super::*;
    use crate::test_helpers::TestContext;

    #[test_context(TestContext)]
    #[compio::test]
    async fn start_server_ok(ctx: &mut TestContext) {
        let state = ctx.state.clone();
        let task = compio::runtime::spawn(async move {
            match Server::new(state).serve().await {
                Ok(_) => (),
                Err(err) => panic!("expected success: {}", err),
            }
        });
        sleep(Duration::from_millis(500)).await;

        let state = ctx.state.clone();
        let _stream = TcpStream::connect(format!(
            "{}:{}",
            state.config.listen_address, state.config.port
        ))
        .await
        .unwrap();

        task.cancel().await;
    }

    #[test_context(TestContext)]
    #[compio::test]
    async fn exit_code_listen_failure(ctx: &mut TestContext) {
        // First server should start ok
        let state = ctx.state.clone();
        let task1 = compio::runtime::spawn(async move {
            match Server::new(state.clone()).serve().await {
                Ok(_) => (),
                Err(err) => panic!("expected success: {}", err),
            }
        });

        sleep(Duration::from_millis(500)).await;

        // Second server should fail
        let state = ctx.state.clone();
        match Server::new(state).serve().await {
            Ok(_) => panic!("expected failure"),
            Err(err) => {
                assert_eq!(err.exit_code(), 6);
            }
        }
        task1.cancel().await;
    }

    #[test]
    fn exit_code_serve_failure() {
        let error = Error::ServeFailure {
            err: std::io::Error::other(""),
        };
        assert_eq!(error.exit_code(), 7);
    }
}
