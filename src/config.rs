use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use strum::{Display, EnumString};
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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

    #[serde(default = "default_locale")]
    pub default_locale: String,

    #[serde(default = "default_sync_mode")]
    pub sync_mode: SyncMode,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_directory: default_data_directory(),
            listen_address: default_listen_address(),
            port: default_port(),
            default_locale: default_locale(),
            sync_mode: default_sync_mode(),
        }
    }
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

fn default_locale() -> String {
    "und".to_string()
}

fn default_sync_mode() -> SyncMode {
    SyncMode::All
}

#[derive(Clone, Debug, Default, Deserialize, Display, EnumString, PartialEq, Serialize)]
pub enum SyncMode {
    #[serde(rename = "buffer")]
    #[strum(serialize = "buffer")]
    Buffer,
    #[serde(rename = "data")]
    #[strum(serialize = "data")]
    Data,
    #[default]
    #[serde(rename = "all")]
    #[strum(serialize = "all")]
    All,
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
            Error::Io { .. } => 2,
            Error::Toml { .. } => 3,
            Error::DataDirectory { .. } => 4,
            Error::PermissionCheck { .. } => 5,
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

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn default_data_directory_is_dotilla_home() {
        assert_eq!(default_data_directory(), PathBuf::from("~/.dotilla"));
    }

    #[test]
    fn exit_code_io() {
        let error = Error::Io(std::io::Error::other(""));
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn exit_code_toml() {
        let toml_error = toml::from_str::<Config>("not valid toml").unwrap_err();
        let error = Error::Toml(toml_error);
        assert_eq!(error.exit_code(), 3);
    }

    #[test]
    fn exit_code_data_directory() {
        let error = Error::DataDirectory {
            path: PathBuf::from("/some/path"),
        };
        assert_eq!(error.exit_code(), 4);
    }

    #[test]
    fn exit_code_permission_check() {
        let error = Error::PermissionCheck {
            path: PathBuf::from("/some/path"),
            source: std::io::Error::other(""),
        };
        assert_eq!(error.exit_code(), 5);
    }

    #[test]
    fn validate_data_directory_ok() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        assert!(validate_data_directory(tmp_dir.path()).is_ok());
    }

    #[test]
    fn validate_data_directory_is_created() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path().to_path_buf();
        fs::remove_dir(&path).ok();
        assert!(validate_data_directory(&path).is_ok());
        let metadata = std::fs::metadata(&path).expect("failed to get metadata");
        assert!(metadata.is_dir());
    }

    #[test]
    fn validate_data_directory_is_not_a_dir() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let tmp_file = NamedTempFile::new_in(&tmp_dir).expect("failed to create temp file");
        match validate_data_directory(tmp_file.path()) {
            Err(Error::DataDirectory { path }) => assert_eq!(path, tmp_file.path()),
            _ => panic!("expected DataDirectory error"),
        }
    }

    #[test]
    #[cfg(unix)]
    fn validate_data_directory_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path();
        let mut perms = fs::metadata(path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_mode(0o500);
        fs::set_permissions(path, perms).expect("failed to set permissions");

        let result = validate_data_directory(path);

        // Restore permissions so the tempdir can clean itself up on drop.
        let mut perms = fs::metadata(path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms).expect("failed to restore permissions");

        match result {
            Err(Error::PermissionCheck { path: err_path, .. }) => assert_eq!(err_path, path),
            other => panic!("expected PermissionCheck error, got {other:?}"),
        }
    }

    #[test]
    fn create_data_directory_creates_nested_dirs() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path().join("a").join("b").join("c");
        let metadata = create_data_directory(&path).expect("failed to create data directory");
        assert!(metadata.is_dir());
    }

    #[test]
    fn create_data_directory_fails_when_parent_is_a_file() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let tmp_file = NamedTempFile::new_in(&tmp_dir).expect("failed to create temp file");
        let path = tmp_file.path().join("subdir");
        match create_data_directory(&path) {
            Err(Error::PermissionCheck { path: err_path, .. }) => assert_eq!(err_path, path),
            other => panic!("expected PermissionCheck error, got {other:?}"),
        }
    }

    #[test]
    fn validate_ok() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let config = Config {
            data_directory: tmp_dir.path().to_path_buf(),
            listen_address: default_listen_address(),
            default_locale: "und".to_string(),
            port: default_port(),
            sync_mode: default_sync_mode(),
        };
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn validate_propagates_data_directory_error() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let tmp_file = NamedTempFile::new_in(&tmp_dir).expect("failed to create temp file");
        let config = Config {
            data_directory: tmp_file.path().to_path_buf(),
            listen_address: default_listen_address(),
            default_locale: "und".to_string(),
            port: default_port(),
            sync_mode: default_sync_mode(),
        };
        match validate(&config) {
            Err(Error::DataDirectory { path }) => assert_eq!(path, tmp_file.path()),
            _ => panic!("expected DataDirectory error"),
        }
    }
}
