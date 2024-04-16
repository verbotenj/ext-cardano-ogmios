resource "kubernetes_manifest" "operator_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = local.name
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = local.proxy_labels
      }
      podMetricsEndpoints = [
        {
          port = "metrics",
          path = "/metrics"
        }
      ]
    }
  }
}
