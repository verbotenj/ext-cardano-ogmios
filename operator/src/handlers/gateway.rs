use kube::{core::ObjectMeta, Client, CustomResourceExt, Resource, ResourceExt};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use tracing::info;

use crate::{
    create_resource, get_acl_name, get_auth_name, get_config, get_resource, http_route,
    ogmios_service_name, patch_resource, patch_resource_status, reference_grant, Error, OgmiosPort,
    OgmiosPortStatus,
};

pub async fn handle_http_route(client: Client, crd: &OgmiosPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let ogmios_service = ogmios_service_name(&crd.spec.network, &crd.spec.version);

    let name = format!("ogmios-{}", crd.name_any());
    let host_name = build_host(&crd.name_any(), &namespace_to_slug(&namespace));
    let http_route = http_route();
    let ogmios_port = OgmiosPort::api_resource();

    let result = get_resource(client.clone(), &namespace, &http_route, &name).await?;

    let (metadata, data, raw) = build_route(&name, &host_name, crd, &ogmios_service)?;

    if result.is_some() {
        info!(resource = crd.name_any(), "Updating http route");
        patch_resource(client.clone(), &namespace, http_route, &name, raw).await?;
    } else {
        info!(resource = crd.name_any(), "Creating http route");
        create_resource(client.clone(), &namespace, http_route, metadata, data).await?;
    }

    let status = OgmiosPortStatus {
        endpoint_url: Some(format!("https://{}", host_name)),
        ..Default::default()
    };
    patch_resource_status(
        client.clone(),
        &namespace,
        ogmios_port,
        &crd.name_any(),
        serde_json::to_value(status)?,
    )
    .await?;
    Ok(())
}

pub async fn handle_reference_grant(client: Client, crd: &OgmiosPort) -> Result<(), Error> {
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
        // we need to get the deserialized payload
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
    owner: &OgmiosPort,
    ogmios_service: &str,
) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let config = get_config();
    let http_route = http_route();
    let plugins = format!(
        "{},{}",
        get_auth_name(&owner.name_any()),
        get_acl_name(&owner.name_any()),
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
          "name": owner.name_any(),
          "uid": owner.uid()
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

fn build_host(name: &str, project_slug: &str) -> String {
    let config = get_config();

    format!(
        "{}-{}.{}.{}",
        name, project_slug, config.ingress_class, config.dns_zone
    )
}

fn namespace_to_slug(namespace: &str) -> String {
    namespace.split_once('-').unwrap().1.to_string()
}
