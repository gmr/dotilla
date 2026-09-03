use std::io::Write;
use std::time::Duration;

use assert_cmd::Command;
use compio::runtime::spawn;
use compio::time::sleep;
use predicates::prelude::*;
use tempfile::tempdir;
use test_context::test_context;

use dotilla::http::server;
use dotilla::test_helpers::TestContext;

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

#[test_context(TestContext)]
#[compio::test]
async fn main_exits_10_database_error(ctx: &mut TestContext) {
    let state = ctx.state.clone();
    let first_server = spawn(async move { server::Server::new(state).serve().await });
    sleep(Duration::from_millis(500)).await;
    assert!(!first_server.is_finished());
    Command::cargo_bin("dotilla")
        .unwrap()
        .arg("--config")
        .arg(ctx.temp_dir.path().join("config.toml"))
        .assert()
        .failure()
        .code(10);
    let _ = first_server.cancel().await;
}
