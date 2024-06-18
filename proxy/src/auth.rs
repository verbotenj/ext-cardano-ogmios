use futures_util::TryStreamExt;
use operator::{
    kube::{
        runtime::watcher::{self, Config, Event},
        Api, Client, ResourceExt,
    },
    OgmiosPort,
};
use std::{collections::HashMap, sync::Arc};
use tokio::pin;
use tracing::{error, info, instrument};

use crate::{Consumer, State};

#[instrument("auth background service", skip_all)]
pub fn start(state: Arc<State>) {
    tokio::spawn(async move {
        let client = Client::try_default()
            .await
            .expect("failed to create kube client");

        let api = Api::<OgmiosPort>::all(client.clone());
        let stream = watcher::watcher(api.clone(), Config::default());
        pin!(stream);

        loop {
            let result = stream.try_next().await;
            match result {
                // Stream restart, also run on startup.
                Ok(Some(Event::Restarted(crds))) => {
                    info!("auth: Watcher restarted, reseting consumers");
                    let consumers: HashMap<String, Consumer> = crds
                        .iter()
                        .map(|crd| {
                            let consumer = Consumer::from(crd);
                            (consumer.key.clone(), consumer)
                        })
                        .collect();
                    *state.consumers.write().await = consumers;

                    // When the watcher is restarted, we reset the limiter because a user
                    // could have changed the tier on the watcher restart.
                    state.limiter.write().await.clear();
                }
                // New port created or updated.
                Ok(Some(Event::Applied(crd))) => match crd.status {
                    Some(_) => {
                        info!("auth: Adding new consumer: {}", crd.name_any());
                        let consumer = Consumer::from(&crd);
                        state.limiter.write().await.remove(&consumer.key);
                        state
                            .consumers
                            .write()
                            .await
                            .insert(consumer.key.clone(), consumer);
                    }
                    None => {
                        // New ports are created without status. When the status is added, a new
                        // Applied event is triggered.
                        info!("auth: New port created: {}", crd.name_any());
                    }
                },
                // Port deleted.
                Ok(Some(Event::Deleted(crd))) => {
                    info!(
                        "auth: Port deleted, removing from state: {}",
                        crd.name_any()
                    );
                    let consumer = Consumer::from(&crd);
                    state.consumers.write().await.remove(&consumer.key);
                    state.limiter.write().await.remove(&consumer.key);
                }
                // Empty response from stream. Should never happen.
                Ok(None) => {
                    error!("auth: Empty response from watcher.");
                    continue;
                }
                // Unexpected error when streaming CRDs.
                Err(err) => {
                    error!(error = err.to_string(), "auth: Failed to update crds.");
                    std::process::exit(1);
                }
            }
        }
    });
}
