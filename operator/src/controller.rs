use futures::StreamExt;
use kube::{
    api::ListParams,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tracing::{error, info, instrument};

use crate::{
    auth::handle_auth,
    gateway::{handle_http_route, handle_reference_grant},
    Error, Metrics, Network, Result, State,
};

pub static OGMIOS_PORT_FINALIZER: &str = "ogmiosports.demeter.run";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "OgmiosPort",
    group = "demeter.run",
    version = "v1alpha1",
    namespaced
)]
#[kube(status = "OgmiosPortStatus")]
#[kube(printcolumn = r#"
        {"name": "Network", "jsonPath": ".spec.network", "type": "string"},
        {"name": "Version", "jsonPath": ".spec.version", "type": "number"},
        {"name": "Endpoint URL", "jsonPath": ".status.endpointUrl",  "type": "string"},
        {"name": "Auth Token", "jsonPath": ".status.authToken", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct OgmiosPortSpec {
    pub network: Network,
    pub version: u8,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OgmiosPortStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

struct Context {
    pub client: Client,
    pub metrics: Metrics,
}
impl Context {
    pub fn new(client: Client, metrics: Metrics) -> Self {
        Self { client, metrics }
    }
}

async fn reconcile(crd: Arc<OgmiosPort>, ctx: Arc<Context>) -> Result<Action> {
    handle_reference_grant(ctx.client.clone(), &crd).await?;
    handle_http_route(ctx.client.clone(), &crd).await?;
    handle_auth(ctx.client.clone(), &crd).await?;

    Ok(Action::await_change())
}

fn error_policy(crd: Arc<OgmiosPort>, err: &Error, ctx: Arc<Context>) -> Action {
    error!(error = err.to_string(), "reconcile failed");
    ctx.metrics.reconcile_failure(&crd, err);
    Action::requeue(Duration::from_secs(5))
}

#[instrument("controller run", skip_all)]
pub async fn run(state: Arc<State>) {
    info!("listening crds running");

    let client = Client::try_default()
        .await
        .expect("failed to create kube client");

    let crds = Api::<OgmiosPort>::all(client.clone());
    if let Err(e) = crds.list(&ListParams::default().limit(1)).await {
        error!("CRD is not queryable; {e:?}. Is the CRD installed?");
        std::process::exit(1);
    }

    let ctx = Context::new(client, state.metrics.clone());

    Controller::new(crds, WatcherConfig::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile, error_policy, Arc::new(ctx))
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;
}
