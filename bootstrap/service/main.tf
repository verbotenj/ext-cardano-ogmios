variable "namespace" {
  description = "the namespace where the resources will be created"
}

variable "network" {
  description = "cardano node network"
}

variable "ogmios_version" {
  type = string

  validation {
    condition     = contains(["5", "6"], var.ogmios_version)
    error_message = "Invalid version. Allowed values are 5 or 6."
  }
}
