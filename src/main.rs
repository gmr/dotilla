use clap::Parser;
use std::panic;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tokio::signal;
use tokio::task::JoinSet;

use dotilla::{http::server, state};

/// Entry point for the application.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app_state = match state::AppState::initialize(cli.config).await {
        Ok(app_state) => app_state,
        Err(err) => startup_failure(err),
    };

    let mut join_set = JoinSet::new();
    join_set.spawn(signal_handler(app_state.clone()));
    join_set.spawn(server::serve(
        app_state.config.listen_address,
        app_state.config.port,
        app_state.clone(),
    ));

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => startup_failure(state::StartupError::Http { err }),
            Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
            Err(err) => startup_failure(state::StartupError::Task { err }),
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

/// Handles startup failures by printing the error and exiting with the appropriate exit code.
fn startup_failure(err: state::StartupError) -> ! {
    eprintln!("{err}");
    process::exit(err.exit_code());
}

/// Catch CTRL-C and SIGTERM signals to gracefully shut down the server.
async fn signal_handler(app_state: Arc<state::AppState>) -> Result<(), server::Error> {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use dotilla::config;

    #[test]
    fn exit_code_config() {
        let error = state::StartupError::Config {
            err: config::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "")),
        };
        assert_eq!(error.exit_code(), 2);
    }
    #[test]
    fn exit_code_http() {
        let error = state::StartupError::Http {
            err: server::Error::ServeFailure {
                err: std::io::Error::new(std::io::ErrorKind::Other, ""),
            },
        };
        assert_eq!(error.exit_code(), 7);
    }
}
