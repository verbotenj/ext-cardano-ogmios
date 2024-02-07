use kube::{
    api::{Patch, PatchParams, PostParams},
    core::{DynamicObject, ObjectMeta},
    discovery::ApiResource,
    Api, Client,
};
use serde_json::json;

use crate::{get_config, Network};

pub fn kong_consumer() -> ApiResource {
    ApiResource {
        group: "configuration.konghq.com".into(),
        version: "v1".into(),
        api_version: "configuration.konghq.com/v1".into(),
        kind: "KongConsumer".into(),
        plural: "kongconsumers".into(),
    }
}

pub async fn get_resource(
    client: Client,
    namespace: &str,
    api_resource: &ApiResource,
    name: &str,
) -> Result<Option<DynamicObject>, kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, api_resource);

    api.get_opt(name).await
}

pub async fn create_resource(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    metadata: ObjectMeta,
    data: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let post_params = PostParams::default();

    let mut dynamic = DynamicObject::new("", &api_resource);
    dynamic.data = data;
    dynamic.metadata = metadata;
    api.create(&post_params, &dynamic).await?;
    Ok(())
}

pub async fn patch_resource(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let patch_params = PatchParams::default();
    api.patch(name, &patch_params, &Patch::Merge(payload))
        .await?;
    Ok(())
}

pub async fn patch_resource_status(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let status = json!({ "status": payload });
    let patch_params = PatchParams::default();
    api.patch_status(name, &patch_params, &Patch::Merge(status))
        .await?;
    Ok(())
}

pub fn build_hostname(network: &Network, key: &str) -> (String, String) {
    let config = get_config();
    let ingress_class = &config.ingress_class;
    let dns_zone = &config.dns_zone;

    let hostname = format!("{network}.{ingress_class}.{dns_zone}");
    let hostname_key = format!("{key}.{network}.{ingress_class}.{dns_zone}");

    (hostname, hostname_key)
}
