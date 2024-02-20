use std::fmt::{self, Display, Formatter};

use prometheus::Registry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use kube;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kube Error: {0}")]
    KubeError(#[source] kube::Error),

    #[error("Argon Error: {0}")]
    ArgonError(String),

    #[error("Bech32 Error: {0}")]
    Bech32Error(#[source] bech32::Error),

    #[error("Deserialize Error: {0}")]
    DeserializeError(#[source] serde_json::Error),

    #[error("Parse Int error: {0}")]
    ParseIntError(#[source] std::num::ParseIntError),

    #[error("Http Request error: {0}")]
    HttpError(String),

    #[error("Config Error: {0}")]
    ConfigError(String),
}
impl Error {
    pub fn metric_label(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}
impl From<kube::Error> for Error {
    fn from(value: kube::Error) -> Self {
        Error::KubeError(value)
    }
}
impl From<argon2::Error> for Error {
    fn from(value: argon2::Error) -> Self {
        Error::ArgonError(value.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::DeserializeError(value)
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Error::ParseIntError(value)
    }
}
impl From<bech32::Error> for Error {
    fn from(value: bech32::Error) -> Self {
        Error::Bech32Error(value)
    }
}

#[derive(Clone, Default)]
pub struct State {
    registry: Registry,
    pub metrics: Metrics,
}
impl State {
    pub fn new() -> Self {
        let registry = Registry::default();
        let metrics = Metrics::default().register(&registry).unwrap();
        Self { registry, metrics }
    }

    pub fn metrics_collected(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub enum Network {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "preprod")]
    Preprod,
    #[serde(rename = "preview")]
    Preview,
    #[serde(rename = "sanchonet")]
    Sanchonet,
}
impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Preprod => write!(f, "preprod"),
            Network::Preview => write!(f, "preview"),
            Network::Sanchonet => write!(f, "sanchonet"),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod controller;
pub use crate::controller::*;

pub mod metrics;
pub use metrics::*;

mod utils;
pub use utils::*;

mod config;
pub use config::*;
