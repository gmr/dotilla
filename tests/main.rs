use assert_cmd::Command;
use dotilla::http::server;
use dotilla::state::AppState;
use predicates::prelude::*;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use tempfile::tempdir;
use tokio::time::{Duration, sleep};

mod common;
use crate::common::config::*;

#[test]
fn main_exits_2_on_missing_config() {
    Command::cargo_bin("dotilla")
        .unwrap()
        .arg("--config")
        .arg("/nonexistent/path.toml")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Configuration error"));
}

#[test]
fn main_exits_4_on_invalid_data_directory() {
    let tmp_dir = tempdir().unwrap();
    let bad_data_dir = tmp_dir.path().join("not_a_dir");
    std::fs::write(&bad_data_dir, "").unwrap();

    let config_path = tmp_dir.path().join("config.toml");
    let mut file = std::fs::File::create(&config_path).unwrap();
    writeln!(file, "data_directory = \"{}\"", bad_data_dir.display()).unwrap();

    Command::cargo_bin("dotilla")
        .unwrap()
        .arg("--config")
        .arg(&config_path)
        .assert()
        .failure()
        .code(4);
}

#[tokio::test]
async fn main_exits_11_database_error() {
    let cfg = write_config_with_ephemeral_port();
    let app_state = AppState::initialize(cfg.path.clone()).await.unwrap();
    let port = cfg.port.unwrap();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let first_server = tokio::spawn(async move { server::serve(ip_addr, port, app_state).await });
    sleep(Duration::from_millis(500)).await;
    assert!(!first_server.is_finished());
    Command::cargo_bin("dotilla")
        .unwrap()
        .arg("--config")
        .arg(&cfg.path)
        .assert()
        .failure()
        .code(11);
    first_server.abort();
}
