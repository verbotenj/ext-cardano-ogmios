resource "kubernetes_service_v1" "proxy_service" {
  metadata {
    name      = local.name
    namespace = var.namespace
  }

  spec {
    selector = {
      role = local.role
    }

    port {
      name        = "metrics"
      port        = local.prometheus_port
      target_port = local.prometheus_port
      protocol    = "TCP"
    }

    port {
      name        = "proxy"
      port        = local.proxy_port
      target_port = local.proxy_port
      protocol    = "TCP"
    }

    type = "ClusterIP"
  }
}
