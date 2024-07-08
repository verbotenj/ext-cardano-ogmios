locals {
  name           = "ogmios-${var.network}-${var.ogmios_version}-${var.salt}"
  image          = var.ogmios_image
  container_port = 1337
  default_args = [
    "--node-socket", "/ipc/node.socket",
    "--node-config", "/config/config.json",
    "--host", "0.0.0.0"
  ]
  container_args = var.ogmios_version == "5" ? local.default_args : concat(local.default_args, ["--include-cbor"])
}

resource "kubernetes_deployment_v1" "ogmios" {
  wait_for_rollout = false

  metadata {
    name      = local.name
    namespace = var.namespace
    labels = {
      "role"                               = "instance"
      "demeter.run/kind"                   = "OgmiosInstance"
      "cardano.demeter.run/network"        = var.network
      "cardano.demeter.run/ogmios-version" = var.ogmios_version
    }
  }
  spec {
    replicas = var.replicas

    strategy {
      rolling_update {
        max_surge       = 2
        max_unavailable = 0
      }
    }

    selector {
      match_labels = {
        "role"                               = "instance"
        "demeter.run/instance"               = local.name
        "cardano.demeter.run/network"        = var.network
        "cardano.demeter.run/ogmios-version" = var.ogmios_version
      }
    }
    template {
      metadata {
        name = local.name
        labels = {
          "role"                               = "instance"
          "demeter.run/instance"               = local.name
          "cardano.demeter.run/network"        = var.network
          "cardano.demeter.run/ogmios-version" = var.ogmios_version
        }
      }
      spec {
        restart_policy = "Always"

        security_context {
          fs_group = 1000
        }

        container {
          name              = "main"
          image             = local.image
          image_pull_policy = "IfNotPresent"
          args = local.container_args

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
            container_port = local.container_port
            name           = "api"
            protocol       = "TCP"
          }

          volume_mount {
            name       = "ipc"
            mount_path = "/ipc"
          }

          volume_mount {
            name       = "node-config"
            mount_path = "/config"
          }

          liveness_probe {
            http_get {
              path   = "/health"
              port   = "api"
              scheme = "HTTP"
            }
            initial_delay_seconds = 60
            period_seconds        = 30
            timeout_seconds       = 5
            success_threshold     = 1
            failure_threshold     = 2
          }
        }

        container {
          name  = "socat"
          image = "alpine/socat"
          args = [
            "UNIX-LISTEN:/ipc/node.socket,reuseaddr,fork,unlink-early",
            "TCP-CONNECT:${var.node_private_dns}"
          ]

          security_context {
            run_as_user  = 1000
            run_as_group = 1000
          }

          volume_mount {
            name       = "ipc"
            mount_path = "/ipc"
          }
        }

        volume {
          name = "ipc"
          empty_dir {}
        }

        volume {
          name = "node-config"

          config_map {
            name = "configs-${var.network}"
          }
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-profile"
          operator = "Exists"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-arch"
          operator = "Equal"
          value    = var.compute_arch
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
