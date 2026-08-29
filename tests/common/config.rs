#![allow(dead_code)]

use dotilla::config;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

pub fn get_ephemeral_port() -> u16 {
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    occupied.local_addr().unwrap().port()
}

pub fn write_config(port: Option<u16>) -> TestConfig {
    let tmp_dir = tempdir().expect("failed to create temp dir");
    let config_path = tmp_dir.path().join("config.toml");
    let mut file = std::fs::File::create(&config_path).expect("failed to create config file");
    writeln!(file, "data_directory = \"{}\"", tmp_dir.path().display())
        .expect("failed to write config file");
    writeln!(file, "listen_address = \"127.0.0.1\"").expect("failed to write config file");
    if let Some(port) = port {
        writeln!(file, "port = {}", port).expect("failed to write config file");
    }
    writeln!(file, "sync_mode = \"buffer\"").expect("failed to write config file");
    TestConfig {
        _tempdir: tmp_dir,
        _file: file,
        path: config_path,
        port,
        _sync_mode: config::SyncMode::Buffer,
    }
}

pub fn write_config_with_ephemeral_port() -> TestConfig {
    let port = get_ephemeral_port();
    write_config(Some(port))
}

pub struct TestConfig {
    _tempdir: tempfile::TempDir,
    _file: std::fs::File,
    pub path: PathBuf,
    pub port: Option<u16>,
    _sync_mode: config::SyncMode,
}
