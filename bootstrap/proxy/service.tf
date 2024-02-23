resource "kubernetes_service_v1" "proxy_service" {
  metadata {
    name      = local.name
    namespace = var.namespace
    annotations = {
      "service.beta.kubernetes.io/aws-load-balancer-nlb-target-type" : "instance"
      "service.beta.kubernetes.io/aws-load-balancer-scheme" : "internet-facing"
      "service.beta.kubernetes.io/aws-load-balancer-type" : "external"
      "service.beta.kubernetes.io/aws-load-balancer-healthcheck-protocol" : "HTTPS"
      "service.beta.kubernetes.io/aws-load-balancer-healthcheck-path" : "/healthz"
    }
  }

  spec {
    load_balancer_class = "service.k8s.aws/nlb"
    selector = {
      role = local.role
    }

    port {
      name        = "proxy"
      port        = 443
      target_port = local.proxy_port
      protocol    = "TCP"
    }

    type = "LoadBalancer"
  }
}
