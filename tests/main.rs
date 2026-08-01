use assert_cmd::Command;
use dotilla::http::*;
use predicates::prelude::*;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::Stdio;
use tempfile::tempdir;
use tokio::time::{Duration, sleep};

#[test]
fn main_ok_debug() {
    let cfg = write_config_with_ephemeral_port();

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dotilla"))
        .arg("--config")
        .arg(&cfg.path)
        .arg("--debug")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    std::thread::sleep(Duration::from_millis(200));
    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(String::from_utf8_lossy(&output.stdout).contains("Debug mode enabled"));
}

#[test]
fn main_exits_1_on_missing_config() {
    Command::cargo_bin("dotilla")
        .unwrap()
        .arg("--config")
        .arg("/nonexistent/path.toml")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Configuration error"));
}

#[test]
fn main_exits_3_on_invalid_data_directory() {
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
        .code(3);
}

#[tokio::test]
async fn main_exits_5_on_port_in_use() {
    let cfg = write_config_with_ephemeral_port();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let first_server = tokio::spawn(async move {
        match serve(ip_addr, cfg.port).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    sleep(Duration::from_millis(500)).await;
    Command::cargo_bin("dotilla")
        .unwrap()
        .arg("--config")
        .arg(&cfg.path)
        .assert()
        .failure()
        .code(5);
    first_server.abort();
}

struct TestConfig {
    _tempdir: tempfile::TempDir,
    _cfgfile: std::fs::File,
    path: PathBuf,
    port: u16,
}

fn write_config_with_ephemeral_port() -> TestConfig {
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = occupied.local_addr().unwrap().port();
    let tmp_dir = tempdir().expect("failed to create temp dir");
    let config_path = tmp_dir.path().join("config.toml");
    let mut file = std::fs::File::create(&config_path).expect("failed to create config file");
    writeln!(file, "data_directory = \"{}\"", tmp_dir.path().display())
        .expect("failed to write config file");
    writeln!(file, "listen_address = \"127.0.0.1\"").expect("failed to write config file");
    writeln!(file, "port = {}", port).expect("failed to write config file");
    TestConfig {
        _tempdir: tmp_dir,
        _cfgfile: file,
        path: config_path,
        port,
    }
}
