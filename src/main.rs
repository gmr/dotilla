mod config;
mod http;

use clap::Parser;
use config::ConfigError;
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
        Err(err) => {
            eprintln!("{err}");
            process::exit(err.exit_code());
        }
    }

    http::serve(&config).await;
}
