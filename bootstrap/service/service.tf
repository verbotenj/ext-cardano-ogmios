locals {
  name = "ogmios-${var.network}-${var.ogmios_version}"
  port = 1337
}

resource "kubernetes_service_v1" "well_known_service" {
  metadata {
    name      = local.name
    namespace = var.namespace
  }

  spec {
    selector = {
      "cardano.demeter.run/network"        = var.network
      "cardano.demeter.run/ogmios-version" = var.ogmios_version
    }

    port {
      name        = "api"
      port        = local.port
      target_port = local.port
      protocol    = "TCP"
    }

    type = "ClusterIP"
  }
}
