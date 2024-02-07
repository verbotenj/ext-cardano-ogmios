variable "network" {
  description = "cardano node network"
}

variable "namespace" {
  description = "the namespace where the resources will be created"
}

resource "kubernetes_config_map" "node-config" {
  metadata {
    namespace = var.namespace
    name      = "configs-${var.network}"
  }

  data = {
    "config.json" = "${file("${path.module}/${var.network}/config.json")}"
  }
}

output "cm_name" {
  value = "configs-${var.network}"
}
