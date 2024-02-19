resource "kubernetes_service_v1" "operator_service" {
  metadata {
    name      = local.operator_name
    namespace = var.namespace
  }

  spec {
    selector = {
      role = local.operator_role
    }

    port {
      name        = "operator"
      port        = local.operator_port
      target_port = local.operator_port
      protocol    = "TCP"
    }

    type = "ClusterIP"
  }
}
