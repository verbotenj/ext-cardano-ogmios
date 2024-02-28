use std::{collections::HashMap, sync::Arc};

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
use tokio::{pin, sync::RwLock};
use tracing::error;

use crate::{Consumer, State};

pub async fn start(state: Arc<RwLock<State>>) {
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
}

async fn update_auth(state: Arc<RwLock<State>>, api: Api<OgmiosPort>) {
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
            let auth_token = crd.status.as_ref().unwrap().auth_token.clone();
            let namespace = crd.metadata.namespace.as_ref().unwrap().clone();
            let port_name = crd.name_any();

            let hash_key = format!("{}.{}.{}", network, version, auth_token);
            let consumer = Consumer::new(namespace, port_name);

            consumers.insert(hash_key, consumer);
        }
    }
    state.write().await.consumers = consumers;
}
