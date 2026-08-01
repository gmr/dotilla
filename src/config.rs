use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Load the configuration from a TOML file, validating it against the schema.
///
/// # Errors
///
/// Returns [`Error::Io`] if the file cannot be read or does not exist.
/// Returns [`Error::Toml`] if the file cannot be parsed.
/// Returns [`Error::DataDirectory`] if the data directory is not a directory.
/// Returns [`Error::PermissionCheck`] if the data directory is not writable.
pub fn load(path: impl AsRef<Path>) -> Result<Config, Error> {
    let path = shellexpand::path::tilde(path.as_ref());
    let content = fs::read_to_string(path)?;
    let mut config: Config = toml::from_str(&content)?;
    config.data_directory = shellexpand::path::tilde(&config.data_directory).into_owned();
    validate(&config)?;
    Ok(config)
}

/// Schema for the configuration file
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The data directory where internal files are stored.
    #[serde(default = "default_data_directory")]
    pub data_directory: PathBuf,

    /// The listen address for the server (IPv4 or IPv6), default is `127.0.0.1`.
    #[serde(default = "default_listen_address")]
    pub listen_address: IpAddr,

    /// The port for the server, default is `6465`.
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_data_directory() -> PathBuf {
    PathBuf::from("~/.dotilla")
}

fn default_listen_address() -> IpAddr {
    IpAddr::from([127, 0, 0, 1])
}

fn default_port() -> u16 {
    6465
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to read the config file.
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse the config file.
    #[error("failed to parse config: {0}")]
    Toml(#[from] toml::de::Error),

    /// Data directory is not a directory.
    #[error("data directory is not a directory {path:?}")]
    DataDirectory { path: PathBuf },

    /// Failed to write to the data directory.
    #[error("can not write to data directory {path:?}: {source}")]
    PermissionCheck {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    /// Map the error to an exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Io { .. } => 1,
            Error::Toml { .. } => 2,
            Error::DataDirectory { .. } => 3,
            Error::PermissionCheck { .. } => 4,
        }
    }
}

fn create_data_directory(path: impl AsRef<Path>) -> Result<fs::Metadata, Error> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|e| Error::PermissionCheck {
        path: path.to_path_buf(),
        source: e,
    })?;
    let metadata = fs::metadata(path).map_err(|e| Error::PermissionCheck {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(metadata)
}

fn validate(config: &Config) -> Result<(), Error> {
    // Stub function that currently only validates the data directory
    // will add other validations as needed.
    validate_data_directory(&config.data_directory)
}

fn validate_data_directory(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => create_data_directory(path)?,
    };

    if !metadata.is_dir() {
        return Err(Error::DataDirectory {
            path: path.to_path_buf(),
        });
    }

    let mut file = tempfile::tempfile_in(path).map_err(|e| Error::PermissionCheck {
        path: path.to_path_buf(),
        source: e,
    })?;
    file.write_all(b"dotilla")
        .map_err(|e| Error::PermissionCheck {
            path: path.to_path_buf(),
            source: e,
        })?;
    file.flush().map_err(|e| Error::PermissionCheck {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}
