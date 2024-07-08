locals {
  network_version_combinations = [
    for combo in setproduct(var.networks, var.versions) : {
      network = combo[0]
      version = combo[1]
    }
  ]
}

resource "kubernetes_namespace" "namespace" {
  metadata {
    name = var.namespace
  }
}

module "ogmios_v1_feature" {
  depends_on         = [kubernetes_namespace.namespace]
  source             = "./feature"
  namespace          = var.namespace
  operator_image_tag = var.operator_image_tag
  metrics_delay      = var.metrics_delay
  extension_name     = var.extension_name
  api_key_salt       = var.api_key_salt
  dcu_per_frame      = var.dcu_per_frame
}

module "ogmios_v1_proxy" {
  depends_on      = [kubernetes_namespace.namespace]
  source          = "./proxy"
  namespace       = var.namespace
  replicas        = var.proxy_blue_replicas
  proxy_image_tag = var.proxy_blue_image_tag
  extension_name  = var.extension_name
  networks        = var.networks
  name            = "proxy"
}

module "ogmios_v1_proxy_green" {
  depends_on      = [kubernetes_namespace.namespace]
  source          = "./proxy"
  namespace       = var.namespace
  replicas        = var.proxy_green_replicas
  proxy_image_tag = var.proxy_green_image_tag
  extension_name  = var.extension_name
  networks        = ["mainnet", "preprod", "preview", "vector-testnet"]
  environment     = "green"
  name            = "proxy-green"
}

// mainnet

module "ogmios_configs" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = { for network in var.networks : "${network}" => network }

  source    = "./configs"
  namespace = var.namespace
  network   = each.value
}

module "ogmios_instances" {
  depends_on = [kubernetes_namespace.namespace, module.ogmios_configs]
  for_each   = var.instances
  source     = "./instance"

  namespace        = var.namespace
  salt             = each.value.salt
  network          = each.value.network
  ogmios_image     = each.value.ogmios_image
  node_private_dns = each.value.node_private_dns
  ogmios_version   = each.value.ogmios_version
  compute_arch     = each.value.compute_arch
  replicas         = each.value.replicas
}

module "ogmios_services" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = { for i, nv in local.network_version_combinations : "${nv.network}-${nv.version}" => nv }
  source     = "./service"

  namespace      = var.namespace
  ogmios_version = each.value.version
  network        = each.value.network
}


