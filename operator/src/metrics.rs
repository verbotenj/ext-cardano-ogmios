use std::sync::Arc;

use chrono::Utc;
use kube::{Resource, ResourceExt};
use prometheus::{opts, IntCounterVec, Registry};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use tracing::{error, info, instrument};

use crate::{get_config, Error, OgmiosPort, State};

#[derive(Clone)]
pub struct Metrics {
    pub dcu: IntCounterVec,
    pub reconcile_failures: IntCounterVec,
    pub metrics_failures: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        let dcu = IntCounterVec::new(
            opts!("dmtr_consumed_dcus", "quantity of dcu consumed",),
            &["project", "service", "service_type", "tenancy"],
        )
        .unwrap();

        let reconcile_failures = IntCounterVec::new(
            opts!(
                "ogmios_operator_reconciliation_errors_total",
                "reconciliation errors",
            ),
            &["instance", "error"],
        )
        .unwrap();

        let metrics_failures = IntCounterVec::new(
            opts!(
                "ogmios_metrics_controller_errors_total",
                "errors to calculation metrics",
            ),
            &["error"],
        )
        .unwrap();

        Metrics {
            reconcile_failures,
            dcu,
            metrics_failures,
        }
    }
}

impl Metrics {
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.dcu.clone()))?;
        registry.register(Box::new(self.metrics_failures.clone()))?;
        registry.register(Box::new(self.reconcile_failures.clone()))?;

        Ok(self)
    }

    pub fn reconcile_failure(&self, crd: &OgmiosPort, e: &Error) {
        self.reconcile_failures
            .with_label_values(&[crd.name_any().as_ref(), e.metric_label().as_ref()])
            .inc()
    }

    pub fn metrics_failure(&self, e: &Error) {
        self.metrics_failures
            .with_label_values(&[e.metric_label().as_ref()])
            .inc()
    }

    pub fn count_dcu_consumed(&self, project: &str, network: &str, dcu: f64) {
        let service = format!("{}-{}", OgmiosPort::kind(&()), network);
        let service_type = format!("{}.{}", OgmiosPort::plural(&()), OgmiosPort::group(&()));
        let tenancy = "proxy";

        let dcu: u64 = dcu.ceil() as u64;

        self.dcu
            .with_label_values(&[project, &service, &service_type, tenancy])
            .inc_by(dcu);
    }
}

#[instrument("metrics collector run", skip_all)]
pub async fn run_metrics_collector(state: Arc<State>) {
    tokio::spawn(async move {
        info!("collecting metrics running");

        let config = get_config();
        let client = reqwest::Client::builder().build().unwrap();
        let project_regex = Regex::new(r"prj-(.+)\..+").unwrap();
        let network_regex = Regex::new(r"([\w-]+)-.+").unwrap();
        let mut last_execution = Utc::now();

        loop {
            tokio::time::sleep(config.metrics_delay).await;

            let end = Utc::now();
            let start = (end - last_execution).num_seconds();

            last_execution = end;

            let query = format!(
                "sum by (consumer, route) (increase(ogmios_proxy_ws_total_frame[{start}s] @ {}))",
                end.timestamp_millis() / 1000
            );

            let result = client
                .get(format!("{}/query?query={query}", config.prometheus_url))
                .send()
                .await;

            if let Err(err) = result {
                error!(error = err.to_string(), "error to make prometheus request");
                state
                    .metrics
                    .metrics_failure(&Error::HttpError(err.to_string()));
                continue;
            }

            let response = result.unwrap();
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                error!(status = status.to_string(), "request status code fail");
                state.metrics.metrics_failure(&Error::HttpError(format!(
                    "Prometheus request error. Status: {} Query: {}",
                    status, query
                )));
                continue;
            }

            let response = response.json::<PrometheusResponse>().await.unwrap();
            for result in response.data.result {
                if result.value == 0.0
                    || result.metric.consumer.is_none()
                    || result.metric.route.is_none()
                {
                    continue;
                }

                let consumer = result.metric.consumer.unwrap();
                let project_captures = project_regex.captures(&consumer);
                if project_captures.is_none() {
                    continue;
                }
                let project_captures = project_captures.unwrap();
                let project = project_captures.get(1).unwrap().as_str();

                let route = result.metric.route.unwrap();
                let network_captures = network_regex.captures(&route);
                if network_captures.is_none() {
                    continue;
                }
                let network_captures = network_captures.unwrap();
                let network = network_captures.get(1).unwrap().as_str();

                let dcu_per_frame = config.dcu_per_frame.get(network);
                if dcu_per_frame.is_none() {
                    let error = Error::ConfigError(format!(
                        "dcu_per_frame not configured to {} network",
                        network
                    ));
                    error!(error = error.to_string());
                    state.metrics.metrics_failure(&error);
                    continue;
                }
                let dcu_per_frame = dcu_per_frame.unwrap();

                let dcu = result.value * dcu_per_frame;
                state.metrics.count_dcu_consumed(project, network, dcu);
            }
        }
    });
}

#[derive(Debug, Deserialize)]
struct PrometheusDataResultMetric {
    consumer: Option<String>,
    route: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PrometheusDataResult {
    metric: PrometheusDataResultMetric,
    #[serde(deserialize_with = "deserialize_value")]
    value: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrometheusData {
    result: Vec<PrometheusDataResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    data: PrometheusData,
}

fn deserialize_value<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<serde_json::Value> = Deserialize::deserialize(deserializer)?;
    Ok(value.into_iter().as_slice()[1]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap())
}
