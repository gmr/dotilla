use dotilla::http::server;
use dotilla::state::AppState;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use crate::common::config::*;

async fn create_app_state() -> Arc<AppState> {
    let cfg = write_config_with_ephemeral_port();
    AppState::initialize(cfg.path.clone()).await.unwrap()
}

#[tokio::test]
async fn serve_ok() {
    let app_state = create_app_state().await;
    let port = get_ephemeral_port();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let task = tokio::spawn(server::serve(ip_addr, port, app_state.clone()));
    sleep(Duration::from_millis(500)).await;
    assert!(!task.is_finished());
    app_state.cancellation_token.cancel();
    let result = task.await;
    result
        .expect("server task panicked")
        .expect("serve returned error");
}

#[tokio::test]
async fn serve_error() {
    let app_state_one = create_app_state().await;
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = get_ephemeral_port();
    let first_task = tokio::spawn(async move {
        if let Err(err) = server::serve(ip_addr, port, app_state_one).await {
            panic!("expected success: {}", err);
        }
    });
    let app_state_two = create_app_state().await;
    tokio::spawn(async move {
        match server::serve(ip_addr, port, app_state_two).await {
            Ok(_) => panic!("expected failure"),
            Err(server::Error::ServeFailure { .. }) => (),
            Err(err) => panic!("expected ServerFailure: {}", err),
        }
    });
    sleep(Duration::from_millis(500)).await;
    first_task.abort();
}
