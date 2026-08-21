#!/usr/bin/env bash
# Phase 0 + Phase 1 on Docker Desktop Kubernetes
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CTX="$(kubectl config current-context 2>/dev/null || true)"
if [[ "${CTX}" != "docker-desktop" ]]; then
  echo "WARNING: kubectl context is '${CTX}', expected 'docker-desktop'."
  echo "Run: kubectl config use-context docker-desktop"
  read -r -p "Continue anyway? [y/N] " ans
  [[ "${ans,,}" == "y" ]] || exit 1
fi

echo "========== PHASE 0: Platform (K8s + Kafka + nginx) =========="

echo "==> nginx Ingress Controller"
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/cloud/deploy.yaml
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=180s

echo "==> namespace"
kubectl apply -f deploy/phase0/k8s/namespace.yaml

echo "==> Kafka (Redpanda)"
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml
kubectl wait -n identity-bridge --for=condition=ready pod -l app=kafka --timeout=180s
chmod +x deploy/local/create-topics-local.sh
./deploy/local/create-topics-local.sh

echo ""
echo "========== PHASE 1: Ingestion tier =========="

kubectl apply -f deploy/phase1/ingest/serviceaccount.yaml
kubectl apply -f deploy/local/overlays/configmap.local.yaml
kubectl apply -f deploy/local/overlays/secret.local.yaml
kubectl apply -f deploy/phase1/ingest/stub-nginx-configmap.yaml
kubectl apply -f deploy/local/overlays/deployment.docker-desktop.yaml
kubectl apply -f deploy/phase1/ingest/service.yaml
kubectl apply -f deploy/phase1/ingest/pdb.yaml
kubectl apply -f deploy/local/overlays/hpa.docker-desktop.yaml
kubectl apply -f deploy/local/overlays/networkpolicy.local.yaml
kubectl apply -f deploy/local/overlays/ingest-edge-configmap.local.yaml
kubectl apply -f deploy/local/overlays/ingress.local.yaml

if ! grep -q 'ingest.local' /etc/hosts 2>/dev/null; then
  echo "127.0.0.1 ingest.local" | sudo tee -a /etc/hosts
fi

kubectl -n identity-bridge rollout status deployment/server-ingest --timeout=180s

echo ""
echo "========== DONE =========="
kubectl get pods,svc,ingress -n identity-bridge
echo "Test: curl http://ingest.local/health/ready"
