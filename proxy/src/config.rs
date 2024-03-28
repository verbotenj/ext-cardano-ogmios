use std::{env, path::PathBuf, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_addr: String,
    pub proxy_namespace: String,
    pub proxy_tiers_path: PathBuf,
    pub proxy_tiers_poll_interval: Duration,
    pub prometheus_addr: String,
    pub ogmios_port: u16,
    pub ogmios_dns: String,
    pub ssl_crt_path: PathBuf,
    pub ssl_key_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: env::var("PROXY_ADDR").expect("PROXY_ADDR must be set"),
            proxy_namespace: env::var("PROXY_NAMESPACE").unwrap_or("ftr-ogmios-v1".into()),
            proxy_tiers_path: env::var("PROXY_TIERS_PATH")
                .map(|v| v.into())
                .expect("PROXY_TIERS_PATH must be set"),
            proxy_tiers_poll_interval: env::var("PROXY_TIERS_POLL_INTERVAL")
                .map(|v| {
                    Duration::from_secs(
                        v.parse::<u64>()
                            .expect("PROXY_TIERS_POLL_INTERVAL must be a number in seconds. eg: 2"),
                    )
                })
                .unwrap_or(Duration::from_secs(2)),
            prometheus_addr: env::var("PROMETHEUS_ADDR").expect("PROMETHEUS_ADDR must be set"),
            ssl_crt_path: env::var("SSL_CRT_PATH")
                .map(|e| e.into())
                .expect("SSL_CRT_PATH must be set"),
            ssl_key_path: env::var("SSL_KEY_PATH")
                .map(|e| e.into())
                .expect("SSL_KEY_PATH must be set"),
            ogmios_port: env::var("OGMIOS_PORT")
                .expect("OGMIOS_PORT must be set")
                .parse()
                .expect("OGMIOS_PORT must a number"),
            ogmios_dns: env::var("OGMIOS_DNS").expect("OGMIOS_DNS must be set"),
        }
    }
}
