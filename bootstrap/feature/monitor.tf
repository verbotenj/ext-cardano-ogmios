resource "kubernetes_manifest" "operator_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = "operator"
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          role = "operator"
        }
      }
      podMetricsEndpoints = [
        {
          port = "operator",
          path = "/metrics"
        }
      ]
    }
  }
}

resource "kubernetes_manifest" "instance_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = "instance"
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          role = "instance"
        }
      }
      podMetricsEndpoints = [
        {
          port = "api",
          path = "/metrics"
        }
      ]
    }
  }
}
