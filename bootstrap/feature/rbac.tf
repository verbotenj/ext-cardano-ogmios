resource "kubernetes_cluster_role" "cluster_role" {
  metadata {
    name = var.namespace
  }

  rule {
    api_groups = ["demeter.run"]
    resources  = ["ogmiosports", "ogmiosports/status", "ogmiosports/finalizers"]
    verbs      = ["get", "list", "watch", "patch", "update"]
  }

  rule {
    api_groups = ["events.k8s.io"]
    resources  = ["events"]
    verbs      = ["create"]
  }
}

resource "kubernetes_cluster_role_binding" "cluster_role_binding" {
  metadata {
    name = var.namespace
  }
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = var.namespace
  }
  subject {
    kind      = "ServiceAccount"
    name      = "default"
    namespace = var.namespace
  }
}
