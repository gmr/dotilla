use std::path::PathBuf;
use std::process;

use clap::Parser;
use thiserror::Error;

use dotilla::{http::server, state};

/// Entry point for the application.
#[compio::main]
async fn main() {
    let cli = Cli::parse();
    let app_state = match state::AppState::initialize(cli.config).await {
        Ok(app_state) => app_state,
        Err(err) => startup_failure(err.into()),
    };
    match server::Server::new(app_state.clone()).serve().await {
        Ok(_) => (),
        Err(err) => startup_failure(Error::HTTPServer(err)),
    }
}

/// Handles startup failures by printing the error and exiting with the appropriate exit code.
fn startup_failure(err: Error) -> ! {
    eprintln!("{err}");
    process::exit(err.exit_code());
}

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
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Startup(err) => err.exit_code(),
            Error::HTTPServer(err) => err.exit_code(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dotilla::config;

    #[test]
    fn exit_code_config() {
        let error = Error::Startup(state::StartupError::Config(config::Error::Io(
            std::io::Error::other(""),
        )));
        assert_eq!(error.exit_code(), 2);
    }
    #[test]
    fn exit_code_http() {
        let error = Error::HTTPServer(server::Error::ServeFailure {
            err: std::io::Error::other(""),
        });
        assert_eq!(error.exit_code(), 7);
    }
}
