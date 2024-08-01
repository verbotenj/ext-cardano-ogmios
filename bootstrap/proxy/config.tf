locals {
  config_map_name = var.environment != null ? "${var.environment}-proxy-config" : "proxy-config"

  tiers = [
    {
      "name"            = "0",
      "max_connections" = 2
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = 500
        }
      ]
    },
    {
      "name"            = "1",
      "max_connections" = 5
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = 500
        }
      ]
    },
    {
      "name"            = "2",
      "max_connections" = 250
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = 500
        }
      ]
    },
    {
      "name"            = "3",
      "max_connections" = 250
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = 500
        }
      ]
    }
  ]
}

resource "kubernetes_config_map" "proxy" {
  metadata {
    namespace = var.namespace
    name      = local.config_map_name
  }

  data = {
    "tiers.toml" = "${templatefile("${path.module}/proxy-config.toml.tftpl", { tiers = local.tiers })}"
  }
}
