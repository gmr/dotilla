use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use test_context::AsyncTestContext;

use crate::{config, state};

pub struct TestContext {
    pub state: Arc<state::AppState>,
    pub temp_dir: tempfile::TempDir,
}

impl AsyncTestContext for TestContext {
    async fn setup() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip_addr, 0);
        let occupied = std::net::TcpListener::bind(addr).unwrap();
        let port = occupied.local_addr().unwrap().port();
        let config = config::Config {
            data_directory: temp_dir.path().to_path_buf(),
            listen_address: ip_addr,
            port,
            default_locale: "und".to_string(),
            sync_mode: config::SyncMode::All,
        };
        let config_path = temp_dir.path().join("config.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(file, "{}", toml::to_string(&config).unwrap()).unwrap();
        file.flush().unwrap();
        let state = state::AppState::initialize(config_path.to_path_buf())
            .await
            .unwrap();
        TestContext { state, temp_dir }
    }
}
