#!/bin/bash
# Install Nexus Axiom via Helm

set -e

echo "☸️  Installing Nexus Axiom via Helm"
echo "===================================="

# Check if helm is installed
if ! command -v helm &> /dev/null; then
    echo "❌ Helm not installed"
    echo "Install with: curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash"
    exit 1
fi

# Add Nexus Axiom Helm repo (when published)
# helm repo add nexus-axiom https://coderawesomeabhi.github.io/nexus-axiom-helm
# helm repo update

# For now, install from local chart
echo "Installing from local chart..."
helm upgrade --install nexus-axiom ./deploy/kubernetes/helm/nexus-axiom \
    --namespace nexus-axiom \
    --create-namespace \
    --set image.tag=latest \
    --wait

echo ""
echo "✅ Nexus Axiom installed!"
echo ""
echo "Check status:"
echo "  kubectl get pods -n nexus-axiom"
echo "  kubectl logs -n nexus-axiom -l app=nexus-axiom"
echo ""
echo "Apply a SecurityPolicy:"
echo "  kubectl apply -f examples/security-policy.yaml"
