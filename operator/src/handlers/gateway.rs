use kube::{core::ObjectMeta, Client, Resource, ResourceExt};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use tracing::info;

use crate::{
    create_resource, get_acl_name, get_auth_name, get_config, get_http_route_key_name,
    get_http_route_name, get_resource, http_route, ogmios_service_name, patch_resource,
    reference_grant, Error, OgmiosPort,
};

pub async fn handle_http_route(client: &Client, crd: &OgmiosPort) -> Result<String, Error> {
    let namespace = crd.namespace().unwrap();
    let ogmios_service = ogmios_service_name(&crd.spec.network, &crd.spec.version);

    let name = get_http_route_name(&crd.name_any());
    let host_name = build_hostname(&crd.name_any(), &project_id(&namespace), None);
    let http_route = http_route();

    let result = get_resource(client.clone(), &namespace, &http_route, &name).await?;

    let (metadata, data, raw) = build_route(&name, &host_name, crd, &ogmios_service)?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating http route");
        patch_resource(client.clone(), &namespace, http_route, &name, raw).await?;
    } else {
        info!(resource = crd.name_any(), "Creating http route");
        create_resource(client.clone(), &namespace, http_route, metadata, data).await?;
    }

    Ok(host_name)
}

pub async fn handle_http_route_key(
    client: &Client,
    crd: &OgmiosPort,
    key: &str,
) -> Result<String, Error> {
    let namespace = crd.namespace().unwrap();
    let ogmios_service = ogmios_service_name(&crd.spec.network, &crd.spec.version);

    let name = get_http_route_key_name(&crd.name_any());
    let host_name = build_hostname(&crd.name_any(), &project_id(&namespace), Some(key));
    let http_route = http_route();

    let result = get_resource(client.clone(), &namespace, &http_route, &name).await?;

    let (metadata, data, raw) = build_route(&name, &host_name, crd, &ogmios_service)?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating http route");
        patch_resource(client.clone(), &namespace, http_route, &name, raw).await?;
    } else {
        info!(resource = crd.name_any(), "Creating http route");
        create_resource(client.clone(), &namespace, http_route, metadata, data).await?;
    }

    Ok(host_name)
}

pub async fn handle_reference_grant(client: &Client, crd: &OgmiosPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let ogmios_service = ogmios_service_name(&crd.spec.network, &crd.spec.version);

    let name = format!("{}-{}-http", namespace, crd.name_any());
    let reference_grant = reference_grant();
    let config = get_config();

    let result = get_resource(client.clone(), &config.namespace, &reference_grant, &name).await?;

    let (metadata, data, raw) = build_grant(&name, &ogmios_service, &namespace)?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating reference grant");
        patch_resource(
            client.clone(),
            &config.namespace,
            reference_grant,
            &name,
            raw,
        )
        .await?;
    } else {
        info!(resource = crd.name_any(), "Creating reference grant");
        create_resource(
            client.clone(),
            &config.namespace,
            reference_grant,
            metadata,
            data,
        )
        .await?;
    }
    Ok(())
}

fn build_route(
    name: &str,
    hostname: &str,
    crd: &OgmiosPort,
    ogmios_service: &str,
) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let config = get_config();
    let http_route = http_route();
    let plugins = format!(
        "{},{}",
        get_auth_name(&crd.name_any()),
        get_acl_name(&crd.name_any()),
    );

    let metadata = ObjectMeta::deserialize(&json!({
      "name": name,
      "labels": {
        "demeter.run/instance": name,
        "demeter.run/tenancy": "project",
        "demeter.run/kind": "http-route"
      },
      "annotations": {
        "konghq.com/plugins": plugins,
      },
      "ownerReferences": [
        {
          "apiVersion": OgmiosPort::api_version(&()).to_string(), // @TODO: try to grab this from the owner
          "kind": OgmiosPort::kind(&()).to_string(), // @TODO: try to grab this from the owner
          "name": crd.name_any(),
          "uid": crd.uid()
        }
      ]
    }))?;

    let data = json!({
      "spec": {
        "hostnames": [hostname],
        "parentRefs": [
          {
            "name": config.ingress_class,
            "namespace": config.namespace
          }
        ],
        "rules": [
          {
            "backendRefs": [
              {
                "kind": "Service",
                "name": ogmios_service,
                "port": config.http_port.parse::<i32>()?,
                "namespace": config.namespace
              }
            ]
          }
        ]
      }
    });

    let raw = json!({
      "apiVersion": http_route.api_version,
      "kind": http_route.kind,
      "metadata": metadata,
      "spec": data["spec"]
    });

    Ok((metadata, data, raw))
}

fn build_grant(
    name: &str,
    ogmios_service: &str,
    project_namespace: &str,
) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let reference_grant = reference_grant();
    let http_route = http_route();

    let metadata = ObjectMeta::deserialize(&json!({
      "name": name,
    }))?;

    let data = json!({
      "spec": {
        "from": [
              {
                  "group": http_route.group,
                  "kind": http_route.kind,
                  "namespace": project_namespace,
              },
            ],
        "to": [
            {
                "group": "",
                "kind": "Service",
                "name": ogmios_service,
            },
        ],
      }
    });

    let raw = json!({
      "apiVersion": reference_grant.api_version,
      "kind": reference_grant.kind,
      "metadata": metadata,
      "spec": data["spec"]
    });

    Ok((metadata, data, raw))
}

fn build_hostname(crd_name: &str, project_id: &str, key: Option<&str>) -> String {
    let config = get_config();
    let ingress_class = &config.ingress_class;
    let dns_zone = &config.dns_zone;

    if let Some(key) = key {
        return format!("{key}.{crd_name}-{project_id}.{ingress_class}.{dns_zone}");
    }

    format!("{crd_name}-{project_id}.{ingress_class}.{dns_zone}")
}

fn project_id(namespace: &str) -> String {
    namespace.split_once('-').unwrap().1.to_string()
}
