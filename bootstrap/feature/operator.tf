resource "kubernetes_deployment_v1" "ogmios_operator" {
  wait_for_rollout = false

  metadata {
    name      = local.operator_name
    namespace = var.namespace
    labels = {
      role = local.operator_role
    }
  }
  spec {
    replicas = 1

    selector {
      match_labels = {
        role = local.operator_role
      }
    }
    template {
      metadata {
        name = local.operator_name
        labels = {
          role = local.operator_role
        }
      }
      spec {
        container {
          name              = "main"
          image             = "ghcr.io/demeter-run/ext-cardano-ogmios-operator:${var.operator_image_tag}"
          image_pull_policy = "IfNotPresent"

          resources {
            limits = {
              cpu    = var.resources.limits.cpu
              memory = var.resources.limits.memory
            }
            requests = {
              cpu    = var.resources.requests.cpu
              memory = var.resources.requests.memory
            }
          }

          port {
            name           = "operator"
            container_port = local.operator_port
            protocol       = "TCP"
          }

          env {
            name  = "ADDR"
            value = local.operator_addr
          }

          env {
            name  = "DNS_ZONE"
            value = var.dns_zone
          }

          env {
            name  = "EXTENSION_NAME"
            value = var.extension_name
          }

          env {
            name  = "API_KEY_SALT"
            value = var.api_key_salt
          }

          env {
            name  = "DCU_PER_FRAME"
            value = "mainnet=${var.dcu_per_frame["mainnet"]},preprod=${var.dcu_per_frame["preprod"]},preview=${var.dcu_per_frame["preview"]}"
          }

          env {
            name  = "METRICS_DELAY"
            value = var.metrics_delay
          }

          env {
            name  = "PROMETHEUS_URL"
            value = var.prometheus_url
          }

        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-profile"
          operator = "Equal"
          value    = "general-purpose"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-arch"
          operator = "Equal"
          value    = "x86"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/availability-sla"
          operator = "Equal"
          value    = "consistent"
        }
      }
    }
  }
}
