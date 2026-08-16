use clap::Parser;
use std::panic;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tokio::task::JoinSet;

use dotilla::{http::server, state};

/// Entry point for the application.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let app_state = match state::AppState::initialize(cli.config).await {
        Ok(app_state) => app_state,
        Err(err) => startup_failure(err.into()),
    };
    match serve(app_state).await {
        Ok(_) => {}
        Err(err) => startup_failure(err),
    }
}

/// Start the HTTP server and handle incoming requests.
async fn serve(state: Arc<state::AppState>) -> Result<(), Error> {
    let mut join_set = JoinSet::new();
    let sh = signal_handler(state.clone());
    join_set.spawn(async {
        sh.await;
        Ok(())
    });
    join_set.spawn(server::serve(
        state.config.listen_address,
        state.config.port,
        state.clone(),
    ));
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => return Err(Error::HTTPServer(err)),
            Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
            Err(err) => return Err(Error::Task(err)),
        }
    }
    Ok(())
}

/// Dotilla is a Graph database server that uses HTTP to serve graph data.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, value_name = "FILE", default_value = "dotilla.toml")]
    config: PathBuf,

    /// Enable debug mode, increases output verbosity
    #[arg(short, long)]
    debug: bool,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    HTTPServer(#[from] server::Error),

    #[error("Startup error: {0}")]
    Startup(#[from] state::StartupError),

    #[error("Task error: {0}")]
    Task(#[from] tokio::task::JoinError),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Startup(err) => err.exit_code(),
            Error::Task(_) => 1,
            Error::HTTPServer(err) => err.exit_code(),
        }
    }
}

/// Handles startup failures by printing the error and exiting with the appropriate exit code.
fn startup_failure(err: Error) -> ! {
    eprintln!("{err}");
    process::exit(err.exit_code());
}

/// Catch CTRL-C and SIGTERM signals to gracefully shut down the server.
async fn signal_handler(app_state: Arc<state::AppState>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            eprintln!("CTRL-C caught, shutting down");
            app_state.cancellation_token.cancel();
        },
        _ = terminate => {
            eprintln!("SIGTERM caught, shutting down");
            app_state.cancellation_token.cancel();
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dotilla::config;

    #[test]
    fn exit_code_config() {
        let error = Error::Startup(state::StartupError::Config(config::Error::Io(
            std::io::Error::new(std::io::ErrorKind::Other, ""),
        )));
        assert_eq!(error.exit_code(), 2);
    }
    #[test]
    fn exit_code_http() {
        let error = Error::HTTPServer(server::Error::ServeFailure {
            err: std::io::Error::new(std::io::ErrorKind::Other, ""),
        });
        assert_eq!(error.exit_code(), 7);
    }
}
