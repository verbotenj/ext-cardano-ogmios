use futures::StreamExt;
use kube::{
    runtime::{
        controller::Action,
        finalizer::{finalizer, Event},
        watcher::Config as WatcherConfig,
        Controller,
    },
    Api, Client, CustomResource, ResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

use crate::{Error, Metrics, Result, State};

pub static OGMIOS_PORT_FINALIZER: &str = "ogmiosports.demeter.run";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "OgmiosPort",
    group = "demeter.run",
    version = "v1alpha1",
    namespaced
)]
#[kube(status = "OgmiosPortStatus")]

pub struct OgmiosPortSpec {}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct OgmiosPortStatus {}

impl OgmiosPort {
    async fn reconcile(&self, _ctx: Arc<Context>) -> Result<Action> {
        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    async fn cleanup(&self, _: Arc<Context>) -> Result<Action> {
        Ok(Action::await_change())
    }
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
    let ns = crd.namespace().unwrap();
    let crds: Api<OgmiosPort> = Api::namespaced(ctx.client.clone(), &ns);

    finalizer(&crds, OGMIOS_PORT_FINALIZER, crd, |event| async {
        match event {
            Event::Apply(crd) => crd.reconcile(ctx.clone()).await,
            Event::Cleanup(crd) => crd.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| Error::FinalizerError(Box::new(e)))
}

fn error_policy(crd: Arc<OgmiosPort>, err: &Error, ctx: Arc<Context>) -> Action {
    ctx.metrics.reconcile_failure(&crd, err);
    Action::requeue(Duration::from_secs(5))
}

pub async fn run(state: Arc<State>) -> Result<(), Error> {
    let client = Client::try_default().await?;
    let crds = Api::<OgmiosPort>::all(client.clone());
    let ctx = Context::new(client, state.metrics.clone());

    Controller::new(crds, WatcherConfig::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile, error_policy, Arc::new(ctx))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}
