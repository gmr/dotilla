use dotilla::http::*;
use std::net::{IpAddr, Ipv4Addr};
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_serve_ok() {
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let task = tokio::spawn(async move {
        match serve(ip_addr, 64656).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    sleep(Duration::from_millis(500)).await;
    task.abort();
}

#[tokio::test]
async fn test_serve_error() {
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let first_task = tokio::spawn(async move {
        match serve(ip_addr, 64656).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    tokio::spawn(async move {
        match serve(ip_addr, 64656).await {
            Ok(_) => assert!(false),
            Err(Error::ServeFailure { .. }) => assert!(true),
            Err(_) => assert!(false),
        }
    });
    sleep(Duration::from_millis(500)).await;
    first_task.abort();
}
