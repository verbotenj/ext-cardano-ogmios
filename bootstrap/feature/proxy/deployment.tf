resource "kubernetes_deployment_v1" "ogmios_proxy" {
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
            name           = "metrics"
            container_port = local.prometheus_port
            protocol       = "TCP"
          }

          port {
            name           = "proxy"
            container_port = local.proxy_port
            protocol       = "TCP"
          }

          env {
            name  = "PROXY_NAMESPACE"
            value = var.namespace
          }

          env {
            name  = "PROXY_ADDR"
            value = local.proxy_addr
          }

          env {
            name  = "PROMETHEUS_ADDR"
            value = local.prometheus_addr
          }

          env {
            name  = "OGMIOS_PORT"
            value = var.ogmios_port
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
