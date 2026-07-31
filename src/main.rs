mod config;
mod http;

use clap::Parser;
use config::{ConfigError, ValidationError};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = "dotilla.toml")]
    config: PathBuf,

    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    println!("Dotilla v{}", env!("CARGO_PKG_VERSION"));

    let cli = Cli::parse();

    let config = match config::load(cli.config) {
        Ok(config) => config,
        Err(ConfigError::Io(error)) => {
            eprintln!("Could not read config.toml: {error}");
            process::exit(1);
        }
        Err(ConfigError::Toml(error)) => {
            eprintln!("Could not parse config.toml: {error}");
            process::exit(2);
        }
    };

    if cli.debug {
        println!("debug mode enabled");
    }

    match config::validate(&config) {
        Ok(_) => {}
        Err(ValidationError::DataDirectory { path }) => {
            eprintln!("Data directory is not a directory: {path:?}");
            process::exit(3);
        }
        Err(ValidationError::PermissionCheck { path }) => {
            eprintln!("Can not write to data directory: {path:?}");
            process::exit(4);
        }
    }

    http::serve(&config).await;
}
