locals {
  name          = "operator"
  role          = "operator"
  operator_port = 9817
  operator_addr = "0.0.0.0:${local.operator_port}"
}

variable "namespace" {
  type = string
}

variable "image" {
  type = string
}

variable "dns_zone" {
  type    = string
  default = "demeter.run"
}

variable "ingress_class" {
  type    = string
  default = "ogmios-v1"
}

variable "api_key_salt" {
  type    = string
  default = "ogmios-salt"
}

variable "dcu_per_frame" {
  type = map(string)
  default = {
    "mainnet" = "10"
    "preprod" = "5"
    "preview" = "5"
  }
}

variable "metrics_delay" {
  type    = number
  default = 30
}

variable "prometheus_url" {
  type    = string
  default = "http://prometheus-operated.demeter-system.svc.cluster.local:9090/api/v1"
}

variable "replicas" {
  type    = number
  default = 1
}

variable "resources" {
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
