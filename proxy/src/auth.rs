use std::sync::Arc;

use futures_util::TryStreamExt;
use operator::{
    kube::{
        api::ListParams,
        runtime::{
            watcher::{self, Config},
            WatchStreamExt,
        },
        Api, Client,
    },
    OgmiosPort,
};
use tokio::{pin, sync::RwLock};
use tracing::error;

use crate::State;

pub async fn start(state: Arc<RwLock<State>>) {
    let client = Client::try_default()
        .await
        .expect("failed to create kube client");

    let api = Api::<OgmiosPort>::all(client.clone());
    let result = api.list(&ListParams::default()).await;
    if let Err(err) = result {
        error!(error = err.to_string(), "error to get crds");
        std::process::exit(1);
    }

    for crd in result.unwrap().items.iter() {
        state.write().await.add_auth_token(crd);
    }

    let stream = watcher::watcher(api, Config::default()).applied_objects();
    pin!(stream);

    loop {
        let result = stream.try_next().await;
        if let Err(err) = result {
            error!(error = err.to_string(), "fail crd auth watcher");
            continue;
        }
        if let Some(crd) = result.unwrap() {
            state.write().await.add_auth_token(&crd);
        }
    }
}
