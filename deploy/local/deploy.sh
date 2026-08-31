#!/usr/bin/env bash
# Phase 0 + Phase 1 on kind (WSL)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "========== PHASE 0: Platform (kind + Kafka + nginx) =========="

if ! kind get clusters 2>/dev/null | grep -qx identity-bridge; then
  kind create cluster --name identity-bridge --config deploy/local/kind-config.yaml
fi

kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/kind/deploy.yaml
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=180s

kubectl apply -f deploy/phase0/k8s/namespace.yaml
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml
kubectl wait -n identity-bridge --for=condition=ready pod -l app=kafka --timeout=120s
chmod +x deploy/local/create-topics-local.sh
./deploy/local/create-topics-local.sh

echo ""
echo "========== PHASE 1: Ingestion tier =========="

chmod +x deploy/local/build-ingest-image.sh
./deploy/local/build-ingest-image.sh
kind load docker-image identity-bridge/server-ingest:latest --name identity-bridge

kubectl apply -f deploy/phase1/ingest/serviceaccount.yaml
kubectl apply -f deploy/local/overlays/configmap.local.yaml
kubectl apply -f deploy/local/overlays/secret.local.yaml
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
kubectl get pods,svc,ingress -n identity-bridge
echo "Test: curl http://ingest.local/health/ready"
