use clap::Parser;
use dotilla::{config, http};
use std::panic;
use std::path::PathBuf;
use std::process;
use thiserror::Error;
use tokio::signal;
use tokio::task::{JoinError, JoinSet};
use tokio_util::sync::CancellationToken;

/// Entry point for the application.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = match config::load(cli.config) {
        Ok(config) => config,
        Err(err) => startup_failure(StartupError::Config { err }),
    };

    if cli.debug {
        println!("Debug mode enabled");
    }

    let cancellation_token = CancellationToken::new();

    let mut join_set = JoinSet::new();

    join_set.spawn(signal_handler(cancellation_token.clone()));
    join_set.spawn(http::serve(
        config.listen_address,
        config.port,
        cancellation_token.clone(),
    ));

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => startup_failure(StartupError::Http { err }),
            Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
            Err(err) => startup_failure(StartupError::Task { err }),
        }
    }
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

/// Errors that can occur during startup.
#[derive(Debug, Error)]
enum StartupError {
    /// Error reading or validating the configuration file
    #[error("Configuration error: {err}")]
    Config {
        #[from]
        err: config::Error,
    },

    /// Error spawning multiple tasks
    #[error("Error spawning multiple tasks: {err}")]
    Task {
        #[from]
        err: JoinError,
    },

    /// Error starting the HTTP server
    #[error("HTTP Server error: {err}")]
    Http {
        #[from]
        err: http::Error,
    },
}

impl StartupError {
    /// Returns the exit code for the error.
    fn exit_code(&self) -> i32 {
        match self {
            StartupError::Config { err } => err.exit_code(),
            StartupError::Http { err } => err.exit_code(),
            StartupError::Task { .. } => 1,
        }
    }
}

/// Handles startup failures by printing the error and exiting with the appropriate exit code.
fn startup_failure(err: StartupError) -> ! {
    eprintln!("{err}");
    process::exit(err.exit_code());
}

async fn signal_handler(token: CancellationToken) -> Result<(), http::Error> {
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
            token.cancel();
        },
        _ = terminate => {
            eprintln!("SIGTERM caught, shutting down");
            token.cancel();
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_config() {
        let error = StartupError::Config {
            err: config::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "")),
        };
        assert_eq!(error.exit_code(), 2);
    }
    #[test]
    fn exit_code_http() {
        let error = StartupError::Http {
            err: http::Error::ServeFailure {
                err: std::io::Error::new(std::io::ErrorKind::Other, ""),
            },
        };
        assert_eq!(error.exit_code(), 7);
    }
}
