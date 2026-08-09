use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{config, cypher, state, storage};

pub fn build_state() -> Arc<state::AppState> {
    let data_dir = tempfile::tempdir().unwrap();
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let addr = SocketAddr::new(ip_addr, 0);
    let occupied = std::net::TcpListener::bind(addr).unwrap();
    let port = occupied.local_addr().unwrap().port();
    let config = config::Config {
        data_directory: data_dir.path().to_path_buf(),
        listen_address: ip_addr,
        port,
        default_locale: "und".to_string(),
    };
    Arc::new(state::AppState {
        cancellation_token: CancellationToken::new(),
        config: config.clone(),
        cypher_parser: Mutex::new(cypher::build_cypher_parser().unwrap()),
        registry: storage::database::registry(),
        system: storage::database::open_system(&config).unwrap(),
    })
}
