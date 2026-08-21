#!/usr/bin/env bash
# Remove local Identity Bridge install (kind OR Docker Desktop Kubernetes).
set -euo pipefail

echo "==> Current kubectl context: $(kubectl config current-context 2>/dev/null || echo none)"

# --- kind cluster (from deploy/local/deploy.sh) ---
if command -v kind >/dev/null 2>&1 && kind get clusters 2>/dev/null | grep -qx identity-bridge; then
  echo "==> Deleting kind cluster 'identity-bridge'"
  kind delete cluster --name identity-bridge
else
  echo "    No kind cluster 'identity-bridge' found"
fi

# --- Resources on active cluster (Docker Desktop or leftover) ---
if kubectl cluster-info >/dev/null 2>&1; then
  CTX="$(kubectl config current-context)"
  echo "==> Removing app namespaces on context: ${CTX}"

  kubectl delete namespace identity-bridge --ignore-not-found --timeout=120s || true

  read -r -p "Also remove ingress-nginx controller namespace? [y/N] " ans
  if [[ "${ans,,}" == "y" ]]; then
    kubectl delete namespace ingress-nginx --ignore-not-found --timeout=120s || true
  fi
fi

# --- /etc/hosts ---
if grep -q 'ingest.local' /etc/hosts 2>/dev/null; then
  echo "==> Removing ingest.local from /etc/hosts (may prompt for sudo)"
  sudo sed -i '/ingest.local/d' /etc/hosts
fi

echo ""
echo "Done. To use Docker Desktop Kubernetes:"
echo "  1. Docker Desktop → Settings → Kubernetes → Enable Kubernetes"
echo "  2. ./deploy/local/deploy-docker-desktop.sh"
