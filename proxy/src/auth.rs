use futures_util::TryStreamExt;
use operator::{
    kube::{
        api::ListParams,
        runtime::{
            watcher::{self, Config},
            WatchStreamExt,
        },
        Api, Client, ResourceExt,
    },
    OgmiosPort,
};
use std::{collections::HashMap, sync::Arc};
use tokio::pin;
use tracing::{error, instrument};

use crate::{Consumer, State};

#[instrument("auth background service", skip_all)]
pub fn start(state: Arc<State>) {
    tokio::spawn(async move {
        let client = Client::try_default()
            .await
            .expect("failed to create kube client");

        let api = Api::<OgmiosPort>::all(client.clone());
        update_auth(state.clone(), api.clone()).await;

        let stream = watcher::watcher(api.clone(), Config::default()).touched_objects();
        pin!(stream);

        loop {
            let result = stream.try_next().await;
            if let Err(err) = result {
                error!(error = err.to_string(), "fail crd auth watcher");
                continue;
            }

            update_auth(state.clone(), api.clone()).await;
        }
    });
}

async fn update_auth(state: Arc<State>, api: Api<OgmiosPort>) {
    let result = api.list(&ListParams::default()).await;
    if let Err(err) = result {
        error!(
            error = err.to_string(),
            "error to get crds while updating auth keys"
        );
        return;
    }

    let mut consumers = HashMap::new();
    for crd in result.unwrap().items.iter() {
        if crd.status.is_some() {
            let network = crd.spec.network.to_string();
            let version = crd.spec.version;
            let tier = crd.spec.throughput_tier.to_string();
            let key = crd.status.as_ref().unwrap().auth_token.clone();
            let namespace = crd.metadata.namespace.as_ref().unwrap().clone();
            let port_name = crd.name_any();

            let hash_key = format!("{}.{}.{}", network, version, key);
            let consumer = Consumer::new(namespace, port_name, tier, key);

            consumers.insert(hash_key, consumer);
        }
    }

    *state.consumers.write().await = consumers;
}
