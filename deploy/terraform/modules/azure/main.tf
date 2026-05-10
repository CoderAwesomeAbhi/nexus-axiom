variable "region"     { default = "eastus" }
variable "image_tag"  { default = "latest" }
variable "nexus_mode" { default = "audit" }

provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "nexus" {
  name     = "nexus-axiom"
  location = var.region
}

# AKS cluster — Ubuntu node pool supports eBPF
resource "azurerm_kubernetes_cluster" "nexus" {
  name                = "nexus-axiom"
  location            = azurerm_resource_group.nexus.location
  resource_group_name = azurerm_resource_group.nexus.name
  dns_prefix          = "nexus-axiom"

  default_node_pool {
    name       = "default"
    node_count = 1
    vm_size    = "Standard_D2s_v3"
    os_sku     = "Ubuntu"  # Required for eBPF BTF support
  }

  identity {
    type = "SystemAssigned"
  }
}

output "info" {
  value = {
    cluster        = azurerm_kubernetes_cluster.nexus.name
    resource_group = azurerm_resource_group.nexus.name
    region         = var.region
    note           = "Deploy with: helm install nexus-axiom ../helm/nexus-axiom"
  }
  sensitive = false
}
