mod config;
mod http;

use clap::Parser;
use std::path::PathBuf;
use std::process;
use thiserror::Error;

/// Entry point for the application.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = match config::load(cli.config) {
        Ok(config) => config,
        Err(err) => startup_failure(StartupError::Config { err }),
    };

    if cli.debug {
        println!("debug mode enabled");
    }

    if let Err(err) = http::serve(&config).await {
        startup_failure(StartupError::Http { err });
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
        }
    }
}

/// Handles startup failures by printing the error and exiting with the appropriate exit code.
fn startup_failure(err: StartupError) -> ! {
    eprintln!("{err}");
    process::exit(err.exit_code());
}
