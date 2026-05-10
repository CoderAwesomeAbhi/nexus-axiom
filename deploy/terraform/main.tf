terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
}

variable "cloud" {
  description = "Target cloud: aws | gcp | azure"
  default     = "aws"
}

variable "region" {
  default = "us-east-1"
}

variable "image_tag" {
  default = "latest"
}

variable "nexus_mode" {
  description = "audit or enforce"
  default     = "audit"
}

# ── AWS ──────────────────────────────────────────────────────────────────────
module "aws" {
  source     = "./modules/aws"
  count      = var.cloud == "aws" ? 1 : 0
  region     = var.region
  image_tag  = var.image_tag
  nexus_mode = var.nexus_mode
}

# ── GCP ──────────────────────────────────────────────────────────────────────
module "gcp" {
  source     = "./modules/gcp"
  count      = var.cloud == "gcp" ? 1 : 0
  region     = var.region
  image_tag  = var.image_tag
  nexus_mode = var.nexus_mode
}

# ── Azure ─────────────────────────────────────────────────────────────────────
module "azure" {
  source     = "./modules/azure"
  count      = var.cloud == "azure" ? 1 : 0
  region     = var.region
  image_tag  = var.image_tag
  nexus_mode = var.nexus_mode
}

output "deployment_info" {
  value = var.cloud == "aws" ? (
    length(module.aws) > 0 ? module.aws[0].info : {}
  ) : var.cloud == "gcp" ? (
    length(module.gcp) > 0 ? module.gcp[0].info : {}
  ) : (
    length(module.azure) > 0 ? module.azure[0].info : {}
  )
}
