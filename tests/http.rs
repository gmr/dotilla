mod utils;

use crate::utils::*;
use dotilla::http::*;
use dotilla::state::AppState;
use dotilla::{config, cypher, state, storage};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

fn test_app_state() -> Arc<AppState> {
    let data_dir = tempfile::tempdir().unwrap();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let addr = SocketAddr::new(ip_addr, 0);
    let occupied = std::net::TcpListener::bind(addr).unwrap();
    let port = occupied.local_addr().unwrap().port();
    let config = config::Config {
        data_directory: data_dir.path().to_path_buf(),
        listen_address: ip_addr,
        port: port,
    };
    Arc::new(state::AppState {
        cancellation_token: CancellationToken::new(),
        cypher_parser: Mutex::new(cypher::build_cypher_parser().unwrap()),
        db: storage::open(&config).unwrap(),
    })
}

#[tokio::test]
async fn test_serve_ok() {
    let app_state = test_app_state();
    let port = get_ephemeral_port();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let task = tokio::spawn(serve(ip_addr, port, app_state.clone()));
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
    let app_state_one = test_app_state();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = get_ephemeral_port();
    let first_task = tokio::spawn(async move {
        match serve(ip_addr, port, app_state_one).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    let app_state_two = test_app_state();
    tokio::spawn(async move {
        match serve(ip_addr, port, app_state_two).await {
            Ok(_) => assert!(false),
            Err(Error::ServeFailure { .. }) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    sleep(Duration::from_millis(500)).await;
    first_task.abort();
}
