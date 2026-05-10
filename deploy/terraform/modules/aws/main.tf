variable "region"     { default = "us-east-1" }
variable "image_tag"  { default = "latest" }
variable "nexus_mode" { default = "audit" }

provider "aws" {
  region = var.region
}

# Use default VPC for simplicity
data "aws_vpc" "default" { default = true }
data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

resource "aws_ecs_cluster" "nexus" {
  name = "nexus-axiom"
}

resource "aws_cloudwatch_log_group" "nexus" {
  name              = "/nexus-axiom"
  retention_in_days = 7
}

resource "aws_iam_role" "execution" {
  name = "nexus-axiom-execution"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
  managed_policy_arns = ["arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"]
}

resource "aws_ecs_task_definition" "nexus" {
  family                   = "nexus-axiom"
  network_mode             = "awsvpc"
  requires_compatibilities = ["EC2"]
  execution_role_arn       = aws_iam_role.execution.arn

  container_definitions = jsonencode([{
    name      = "nexus-axiom"
    image     = "ghcr.io/coderawesomeabhi/nexus-axiom:${var.image_tag}"
    privileged = true
    environment = [
      { name = "NEXUS_MODE", value = var.nexus_mode },
      { name = "RUST_LOG",   value = "info" }
    ]
    portMappings = [
      { containerPort = 8080, protocol = "tcp" },
      { containerPort = 9090, protocol = "tcp" }
    ]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.nexus.name
        "awslogs-region"        = var.region
        "awslogs-stream-prefix" = "nexus"
      }
    }
    mountPoints = [
      { sourceVolume = "debugfs", containerPath = "/sys/kernel/debug", readOnly = true },
      { sourceVolume = "bpffs",   containerPath = "/sys/fs/bpf",       readOnly = false }
    ]
  }])

  volume {
    name = "debugfs"
    host_path { path = "/sys/kernel/debug" }
  }
  volume {
    name = "bpffs"
    host_path { path = "/sys/fs/bpf" }
  }
}

resource "aws_security_group" "nexus" {
  name   = "nexus-axiom"
  vpc_id = data.aws_vpc.default.id

  ingress {
    from_port   = 8080
    to_port     = 9090
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_ecs_service" "nexus" {
  name                = "nexus-axiom"
  cluster             = aws_ecs_cluster.nexus.id
  task_definition     = aws_ecs_task_definition.nexus.arn
  scheduling_strategy = "DAEMON"

  network_configuration {
    subnets         = data.aws_subnets.default.ids
    security_groups = [aws_security_group.nexus.id]
  }
}

output "info" {
  value = {
    cluster = aws_ecs_cluster.nexus.name
    service = aws_ecs_service.nexus.name
    region  = var.region
  }
}
