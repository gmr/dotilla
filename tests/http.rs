mod utils;

use crate::utils::*;
use dotilla::http::*;
use std::net::{IpAddr, Ipv4Addr};
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_serve_ok() {
    let cancellation_token = CancellationToken::new();
    let port = get_ephemeral_port();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let task = tokio::spawn(serve(ip_addr, port, cancellation_token.clone()));
    sleep(Duration::from_millis(500)).await;
    assert!(!task.is_finished());
    cancellation_token.cancel();
    let result = task.await;
    if let Ok(Ok(_)) = result {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[tokio::test]
async fn test_serve_error() {
    let cancellation_token_one = CancellationToken::new();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = get_ephemeral_port();
    let first_task = tokio::spawn(async move {
        match serve(ip_addr, port, cancellation_token_one).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    let cancellation_token_two = CancellationToken::new();
    tokio::spawn(async move {
        match serve(ip_addr, port, cancellation_token_two).await {
            Ok(_) => assert!(false),
            Err(Error::ServeFailure { .. }) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    sleep(Duration::from_millis(500)).await;
    first_task.abort();
}
