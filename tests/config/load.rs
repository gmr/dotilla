use dotilla::config::*;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn load_ok() {
    let tmp_dir = tempdir().expect("failed to create temp dir");
    let config_path = tmp_dir.path().join("config.toml");
    let mut file = std::fs::File::create(&config_path).expect("failed to create config file");
    writeln!(file, "data_directory = \"{}\"", tmp_dir.path().display())
        .expect("failed to write config file");

    let config = load(&config_path).expect("failed to load config");
    assert_eq!(config.data_directory, tmp_dir.path());
}

#[test]
fn load_err() {
    let tmp_dir = tempdir().expect("failed to create temp dir");
    let config_path = tmp_dir.path().join("config.toml");
    match load(&config_path) {
        Err(Error::Io(err)) => {
            assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        }
        _ => panic!("unexpected error"),
    }
}
