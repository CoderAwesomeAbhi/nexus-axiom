variable "region"     { default = "us-central1" }
variable "image_tag"  { default = "latest" }
variable "nexus_mode" { default = "audit" }
variable "project"    { default = "" }

provider "google" {
  region  = var.region
  project = var.project
}

# GKE cluster for eBPF (requires Container-Optimized OS or Ubuntu node image)
resource "google_container_cluster" "nexus" {
  name     = "nexus-axiom"
  location = var.region

  # Minimal cluster — node pool defined separately
  remove_default_node_pool = true
  initial_node_count       = 1

  workload_identity_config {
    workload_pool = "${var.project}.svc.id.goog"
  }
}

resource "google_container_node_pool" "nexus" {
  name     = "nexus-nodes"
  cluster  = google_container_cluster.nexus.name
  location = var.region

  node_count = 1

  node_config {
    # Ubuntu node image supports eBPF/BTF
    image_type   = "UBUNTU_CONTAINERD"
    machine_type = "e2-standard-2"

    oauth_scopes = [
      "https://www.googleapis.com/auth/logging.write",
      "https://www.googleapis.com/auth/monitoring",
    ]
  }
}

output "info" {
  value = {
    cluster  = google_container_cluster.nexus.name
    endpoint = google_container_cluster.nexus.endpoint
    region   = var.region
    note     = "Deploy with: helm install nexus-axiom ../helm/nexus-axiom"
  }
}
