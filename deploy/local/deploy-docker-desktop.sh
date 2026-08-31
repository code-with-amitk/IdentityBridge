#!/usr/bin/env bash
# Phase 0 + Phase 1 on Docker Desktop Kubernetes
set -euo pipefail

#ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
#cd "$ROOT"
cd "/home"
NGINX_INGRESS_CONTROLLER_YAML="https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/cloud/deploy.yaml"

CTX="$(kubectl config current-context 2>/dev/null || true)"
if [[ "${CTX}" != "docker-desktop" ]]; then
  echo "WARNING: kubectl context is '${CTX}', expected 'docker-desktop'."
  echo "Run: kubectl config use-context docker-desktop"
  read -r -p "Continue anyway? [y/N] " ans
  [[ "${ans,,}" == "y" ]] || exit 1
fi

echo "========== PHASE 0: Platform (K8s + Kafka + nginx) =========="

echo "====== 1. Create nginx ingress controller(API Gateway) ======="
kubectl apply -f $NGINX_INGRESS_CONTROLLER_YAML
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=180s

echo "====== 2. Create namespace=identity-bridge ======="
kubectl apply -f deploy/phase0/k8s/namespace.yaml

echo "====== 3. Create Kafka(Redpanda) Pod ======="
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml
kubectl wait -n identity-bridge --for=condition=ready pod -l app=kafka --timeout=180s

echo "====== 4. Create Kafka(Redpanda) Topics ======="
chmod +x deploy/local/create-topics-local.sh
./deploy/local/create-topics-local.sh

echo ""
echo "========== PHASE 1: Ingestion tier =========="

chmod +x deploy/local/build-ingest-image.sh
./deploy/local/build-ingest-image.sh

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

echo "Checking deployment status..."
kubectl -n identity-bridge rollout status deployment/server-ingest --timeout=180s

echo ""
echo "Deployment status: Completed"

kubectl get pods,svc,ingress -n identity-bridge
echo "Test: curl http://ingest.local/health/ready"
