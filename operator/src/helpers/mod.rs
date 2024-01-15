use kube::{
    api::{Patch, PatchParams, PostParams},
    core::{DynamicObject, ObjectMeta},
    discovery::ApiResource,
    Api, Client,
};
use serde_json::json;

use crate::Network;

pub fn http_route() -> ApiResource {
    ApiResource {
        group: "gateway.networking.k8s.io".into(),
        version: "v1".into(),
        api_version: "gateway.networking.k8s.io/v1".into(),
        kind: "HTTPRoute".into(),
        plural: "httproutes".into(),
    }
}

pub fn reference_grant() -> ApiResource {
    ApiResource {
        group: "gateway.networking.k8s.io".into(),
        version: "v1beta1".into(),
        api_version: "gateway.networking.k8s.io/v1beta1".into(),
        kind: "ReferenceGrant".into(),
        plural: "referencegrants".into(),
    }
}

pub fn kong_plugin() -> ApiResource {
    ApiResource {
        group: "configuration.konghq.com".into(),
        version: "v1".into(),
        api_version: "configuration.konghq.com/v1".into(),
        kind: "KongPlugin".into(),
        plural: "kongplugins".into(),
    }
}

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

pub fn get_http_route_name(name: &str) -> String {
    format!("ogmios-http-route-{}", name)
}

pub fn get_http_route_key_name(name: &str) -> String {
    format!("ogmios-http-route-key-{}", name)
}

pub fn get_auth_name(name: &str) -> String {
    format!("ogmios-auth-{name}")
}

pub fn get_host_key_name(name: &str) -> String {
    format!("ogmios-host-key-{name}")
}

pub fn get_acl_name(name: &str) -> String {
    format!("ogmios-acl-{name}")
}

pub fn ogmios_service_name(network: &Network, version: &u8) -> String {
    format!("ogmios-{network}-{version}")
}
