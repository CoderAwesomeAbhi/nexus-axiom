# Kubernetes Deployment Guide

## Quick Start

### Using kubectl

```bash
# Install CRD
kubectl apply -f manifests/crd.yaml

# Deploy DaemonSet
kubectl apply -f manifests/daemonset.yaml

# Apply security policy
kubectl apply -f manifests/example-policy.yaml
```

### Using Helm

```bash
# Install
helm install nexus-axiom ./helm

# Upgrade
helm upgrade nexus-axiom ./helm

# Uninstall
helm uninstall nexus-axiom
```

## Configuration

### Security Policy

Create a `SecurityPolicy` resource:

```yaml
apiVersion: nexus-axiom.io/v1
kind: SecurityPolicy
metadata:
  name: my-policy
  namespace: production
spec:
  mode: enforce
  blockWX: true
  blockMprotect: true
  blockPtrace: true
  
  allowlist:
  - process: "nginx"
    namespace: "production"
  
  protectedPaths:
  - "/etc/passwd"
  - "/etc/shadow"
  
  networkPolicy:
    blockedPorts: [22, 23]
    rateLimitPPS: 1000
```

## Monitoring

### Prometheus Metrics

Metrics are exposed on port 9090:

```bash
kubectl port-forward -n kube-system ds/nexus-axiom 9090:9090
curl http://localhost:9090/metrics
```

### Dashboard

Web dashboard on port 8080:

```bash
kubectl port-forward -n kube-system ds/nexus-axiom 8080:8080
# Open http://localhost:8080 in browser
```

## Requirements

- Kubernetes 1.20+
- Linux kernel 5.8+ with eBPF LSM support
- Nodes must have `lsm=bpf` in kernel boot parameters

## Troubleshooting

### Check if eBPF LSM is enabled

```bash
kubectl exec -n kube-system ds/nexus-axiom -- cat /sys/kernel/security/lsm
# Should contain "bpf"
```

### View logs

```bash
kubectl logs -n kube-system ds/nexus-axiom -f
```

### Check policy status

```bash
kubectl get securitypolicies -A
kubectl describe securitypolicy default-security-policy
```
