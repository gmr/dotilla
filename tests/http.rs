use dotilla::http::server;
use dotilla::state::AppState;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

mod common;
use crate::common::config::*;

async fn create_app_state() -> Arc<AppState> {
    let cfg = write_config_with_ephemeral_port();
    AppState::initialize(cfg.path.clone()).await.unwrap()
}

#[tokio::test]
async fn test_serve_ok() {
    let app_state = create_app_state().await;
    let port = get_ephemeral_port();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let task = tokio::spawn(server::serve(ip_addr, port, app_state.clone()));
    sleep(Duration::from_millis(500)).await;
    assert!(!task.is_finished());
    app_state.cancellation_token.cancel();
    let result = task.await;
    if let Ok(Ok(_)) = result {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[tokio::test]
async fn test_serve_error() {
    let app_state_one = create_app_state().await;
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = get_ephemeral_port();
    let first_task = tokio::spawn(async move {
        match server::serve(ip_addr, port, app_state_one).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    let app_state_two = create_app_state().await;
    tokio::spawn(async move {
        match server::serve(ip_addr, port, app_state_two).await {
            Ok(_) => assert!(false),
            Err(server::Error::ServeFailure { .. }) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    sleep(Duration::from_millis(500)).await;
    first_task.abort();
}
