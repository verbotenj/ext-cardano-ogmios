locals {
  tiers = [
    {
      "name" = "0",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(60 * 60 * 24 / var.replicas)
        }
      ]
    },
    {
      "name" = "1",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(300 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(300 * 60 * 24 / var.replicas)
        }
      ]
    },
    {
      "name" = "2",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(2400 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(2400 * 60 * 24 / var.replicas)
        }
      ]
    },
    {
      "name" = "3",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(4800 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(4800 * 60 * 24 / var.replicas)
        }
      ]
    }
  ]
}

resource "kubernetes_config_map" "proxy" {
  metadata {
    namespace = var.namespace
    name      = "proxy-config"
  }

  data = {
    "tiers.toml" = "${templatefile("${path.module}/proxy-config.toml.tftpl", { tiers = local.tiers })}"
  }
}
