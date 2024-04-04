use config::Config;
use dotenv::dotenv;
use leaky_bucket::RateLimiter;
use metrics::Metrics;
use prometheus::Registry;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use tiers::Tier;
use tokio::sync::RwLock;
use tracing::Level;

mod auth;
mod config;
mod limiter;
mod metrics;
mod proxy;
mod tiers;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let state = Arc::new(State::try_new()?);

    auth::start(state.clone());
    tiers::start(state.clone());

    let metrics = metrics::start(state.clone());
    let proxy_server = proxy::start(state.clone());

    tokio::join!(metrics, proxy_server);

    Ok(())
}

pub struct State {
    config: Config,
    metrics: Metrics,
    host_regex: Regex,
    consumers: RwLock<HashMap<String, Consumer>>,
    tiers: RwLock<HashMap<String, Tier>>,
    limiter: RwLock<HashMap<String, Vec<Arc<RateLimiter>>>>,
}
impl State {
    pub fn try_new() -> Result<Self, Box<dyn Error>> {
        let config = Config::new();
        let metrics = Metrics::try_new(Registry::default())?;
        let host_regex = Regex::new(r"(dmtr_[\w\d-]+)?\.?([\w-]+)-v([\d]).+")?;
        let consumers = Default::default();
        let tiers = Default::default();
        let limiter = Default::default();

        Ok(Self {
            config,
            metrics,
            host_regex,
            consumers,
            tiers,
            limiter,
        })
    }

    pub async fn get_consumer(&self, network: &str, version: &str, key: &str) -> Option<Consumer> {
        let consumers = self.consumers.read().await.clone();
        let hash_key = format!("{}.{}.{}", network, version, key);
        consumers.get(&hash_key).cloned()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Consumer {
    namespace: String,
    port_name: String,
    tier: String,
    key: String,
}
impl Consumer {
    pub fn new(namespace: String, port_name: String, tier: String, key: String) -> Self {
        Self {
            namespace,
            port_name,
            tier,
            key,
        }
    }
}
impl Display for Consumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.namespace, self.port_name)
    }
}
