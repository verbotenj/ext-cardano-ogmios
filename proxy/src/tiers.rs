use notify::{Event, PollWatcher, RecursiveMode, Watcher};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{error::Error, fs, sync::Arc, time::Duration};
use tokio::runtime::{Handle, Runtime};
use tracing::{error, info, instrument, warn};

use crate::State;

#[derive(Debug, Clone, Deserialize)]
pub struct Tier {
    pub name: String,
    pub rates: Vec<TierRate>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TierRate {
    pub limit: usize,
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
}
pub fn deserialize_duration<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let value: String = Deserialize::deserialize(deserializer)?;
    let regex = Regex::new(r"([\d]+)([\w])").unwrap();
    let captures = regex.captures(&value);
    if captures.is_none() {
        return Err(<D::Error as serde::de::Error>::custom(
            "Invalid tier interval format",
        ));
    }

    let captures = captures.unwrap();
    let number = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
    let symbol = captures.get(2).unwrap().as_str();

    match symbol {
        "s" => Ok(Duration::from_secs(number)),
        "m" => Ok(Duration::from_secs(number * 60)),
        "h" => Ok(Duration::from_secs(number * 60 * 60)),
        "d" => Ok(Duration::from_secs(number * 60 * 60 * 24)),
        _ => Err(<D::Error as serde::de::Error>::custom(
            "Invalid symbol tier interval",
        )),
    }
}

#[instrument("tiers background service", skip_all)]
pub fn start(state: Arc<State>) {
    tokio::spawn(async move {
        if let Err(err) = update_tiers(state.clone()).await {
            error!(error = err.to_string(), "error to update tiers");
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);

        let watcher_config = notify::Config::default()
            .with_compare_contents(true)
            .with_poll_interval(state.config.proxy_tiers_poll_interval);

        let watcher_result = PollWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    runtime_handle()
                        .block_on(async { tx.send(event).await })
                        .unwrap();
                }
            },
            watcher_config,
        );
        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher tier");
            return;
        }

        let mut watcher = watcher_result.unwrap();
        let watcher_result =
            watcher.watch(&state.config.proxy_tiers_path, RecursiveMode::Recursive);
        if let Err(err) = watcher_result {
            error!(error = err.to_string(), "error to watcher tier");
            return;
        }

        loop {
            let result = rx.recv().await;
            if result.is_some() {
                if let Err(err) = update_tiers(state.clone()).await {
                    error!(error = err.to_string(), "error to update tiers");
                    continue;
                }

                info!("tiers modified");
            }
        }
    });
}

async fn update_tiers(state: Arc<State>) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&state.config.proxy_tiers_path)?;

    let value: Value = toml::from_str(&contents)?;
    let tiers_value: Option<&Value> = value.get("tiers");
    if tiers_value.is_none() {
        warn!("tiers not configured on toml");
        return Ok(());
    }

    let tiers = serde_json::from_value::<Vec<Tier>>(tiers_value.unwrap().to_owned())?;

    *state.tiers.write().await = tiers
        .into_iter()
        .map(|tier| (tier.name.clone(), tier))
        .collect();

    state.limiter.write().await.clear();

    Ok(())
}

fn runtime_handle() -> Handle {
    match Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            let rt = Runtime::new().unwrap();
            rt.handle().clone()
        }
    }
}
