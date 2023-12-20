use std::{sync::Arc, thread::sleep, time::Duration};

use kube::ResourceExt;
use prometheus::{opts, IntCounterVec, Registry};

use crate::{Error, OgmiosPort, State};

#[derive(Clone)]
pub struct Metrics {
    pub reconcile_failures: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        let reconcile_failures = IntCounterVec::new(
            opts!(
                "crd_controller_reconciliation_errors_total",
                "reconciliation errors",
            ),
            &["instance", "error"],
        )
        .unwrap();

        Metrics { reconcile_failures }
    }
}

impl Metrics {
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.reconcile_failures.clone()))?;

        Ok(self)
    }

    pub fn reconcile_failure(&self, crd: &OgmiosPort, e: &Error) {
        self.reconcile_failures
            .with_label_values(&[crd.name_any().as_ref(), e.metric_label().as_ref()])
            .inc()
    }
}

pub async fn run_metrics_collector(_state: Arc<State>) -> Result<(), Error> {
    loop {
        sleep(Duration::from_secs(6))
    }
}
