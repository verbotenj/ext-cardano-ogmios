variable "namespace" {
  type = string
}

variable "dns_zone" {
  type    = string
  default = "demeter.run"
}

variable "cluster_issuer" {
  type    = string
  default = "letsencrypt"
}

variable "extension_name" {
  type    = string
  default = "ogmios-m1"
}

variable "cloud_provider" {
  type    = string
  default = "aws"
}

variable "networks" {
  type    = list(string)
  default = ["mainnet", "preprod", "preview", "vector-testnet"]
}

variable "versions" {
  type    = list(string)
  default = ["5", "6"]
}

// operator settings

variable "operator_image_tag" {
  type = string
}

variable "api_key_salt" {
  type    = string
  default = "ogmios-salt"
}

variable "dcu_per_frame" {
  type = map(string)
  default = {
    "mainnet"        = "10"
    "preprod"        = "5"
    "preview"        = "5"
    "vector-testnet" = "5"
  }
}

variable "metrics_delay" {
  type    = number
  default = 60
}

variable "prometheus_url" {
  type    = string
  default = "http://prometheus-operated.demeter-system.svc.cluster.local:9090/api/v1"
}

variable "operator_resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits : {
      cpu : "50m",
      memory : "250Mi"
    }
    requests : {
      cpu : "50m",
      memory : "250Mi"
    }
  }
}
// proxy

# variable "proxy_image_tag" {
#   type = string
# }

# variable "proxy_replicas" {
#   type    = number
#   default = 1
# }

variable "proxy_green_image_tag" {
  type = string
}

variable "proxy_green_replicas" {
  type    = number
  default = 1
}

variable "proxy_blue_image_tag" {
  type = string
}

variable "proxy_blue_replicas" {
  type    = number
  default = 1
}


variable "proxy_resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits : {
      cpu : "50m",
      memory : "250Mi"
    }
    requests : {
      cpu : "50m",
      memory : "250Mi"
    }
  }
}

variable "instances" {
  type = map(object({
    salt             = string
    network          = string
    ogmios_image     = string
    node_private_dns = string
    ogmios_version   = string
    compute_arch     = string
    replicas         = number
    resources = optional(object({
      limits = object({
        cpu    = string
        memory = string
      })
      requests = object({
        cpu    = string
        memory = string
      })
    }))
  }))
}
