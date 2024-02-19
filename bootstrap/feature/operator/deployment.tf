resource "kubernetes_deployment_v1" "ogmios_operator" {
  wait_for_rollout = false

  metadata {
    name      = local.name
    namespace = var.namespace
    labels = {
      role = local.role
    }
  }
  spec {
    replicas = var.replicas
    selector {
      match_labels = {
        role = local.role
      }
    }
    template {
      metadata {
        name = local.name
        labels = {
          role = local.role
        }
      }
      spec {
        container {
          name              = "main"
          image             = var.image
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
            name  = "INGRESS_CLASS"
            value = var.ingress_class
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
