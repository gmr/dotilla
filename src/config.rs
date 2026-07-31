use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_data_directory")]
    pub data_directory: String,
    #[serde(default = "default_listen_address")]
    pub listen_address: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_data_directory() -> String {
    "~/.dotilla".to_string()
}

fn default_listen_address() -> IpAddr {
    IpAddr::from([127, 0, 0, 1])
}

fn default_port() -> u16 {
    6465
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("data directory is not a directory: {path:?}")]
    DataDirectory { path: std::path::PathBuf },
    #[error("can not write to data directory: {path:?}")]
    PermissionCheck { path: std::path::PathBuf },
}

pub fn load(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}

fn validate_data_directory(path: impl AsRef<Path>) -> Result<(), ValidationError> {
    let path = path.as_ref();

    let metadata = fs::metadata(path).map_err(|_| ValidationError::DataDirectory {
        path: path.to_path_buf(),
    })?;

    if !metadata.is_dir() {
        return Err(ValidationError::DataDirectory {
            path: path.to_path_buf(),
        });
    }

    let mut file = tempfile::tempfile_in(path).map_err(|_| ValidationError::PermissionCheck {
        path: path.to_path_buf(),
    })?;
    file.write(b"dotilla")
        .map_err(|_| ValidationError::PermissionCheck {
            path: path.to_path_buf(),
        })?;
    file.flush().map_err(|_| ValidationError::PermissionCheck {
        path: path.to_path_buf(),
    })?;

    Ok(())
}

pub fn validate(config: &Config) -> Result<(), ValidationError> {
    validate_data_directory(&config.data_directory)
}
