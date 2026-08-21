# Local deploy — Docker Desktop Kubernetes

Use the **built-in Kubernetes** in Docker Desktop (not kind). Works from **WSL2** when Docker Desktop WSL integration is enabled.

---

## Prerequisites

1. **Docker Desktop** installed (Windows/Mac) with WSL2 backend  
2. **Settings → Kubernetes → Enable Kubernetes** → Apply & Restart  
3. Wait until Kubernetes status is **Running**  
4. `kubectl` available in WSL:

```bash
$ kubectl config get-contexts
CURRENT   NAME             CLUSTER          AUTHINFO         NAMESPACE
*         docker-desktop   docker-desktop   docker-desktop

$ kubectl config use-context docker-desktop
Switched to context "docker-desktop".

$ kubectl get nodes
NAME             STATUS   ROLES           AGE     VERSION
docker-desktop   Ready    control-plane   5m14s   v1.32.2
```

## Step 1 — One-shot deploy

```bash
cd ~/IdentityBridge
chmod +x deploy/local/deploy-docker-desktop.sh deploy/local/create-topics-local.sh
kubectl config use-context docker-desktop
./deploy/local/deploy-docker-desktop.sh
```

---

## Step 2 — What each step does (manual equivalent)

**Phase 0** = platform. **Phase 1** = ingestion tier.

```bash
# Use Docker Desktop cluster
kubectl config use-context docker-desktop
```
Switches kubectl to Docker Desktop’s API server (runs inside Docker Desktop VM).

```bash
kubectl apply -f .../ingress-nginx/.../provider/cloud/deploy.yaml
```
**Phase 0** — Installs **nginx Ingress Controller** (local load balancer, replaces AWS ALB).

```bash
kubectl apply -f deploy/phase0/k8s/namespace.yaml
```
**Phase 0** — Creates namespace `identity-bridge`.

```bash
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml
```
**Phase 0** — Deploys **Redpanda** (Kafka-compatible broker) on port **9092**.

```bash
./deploy/local/create-topics-local.sh
```
**Phase 0** — Creates topics: `identity-events` (24p), `identity-catalog` (8p), `identity-heartbeat` (4p).

```bash
kubectl apply -f deploy/local/overlays/configmap.local.yaml
kubectl apply -f deploy/local/overlays/secret.local.yaml
kubectl apply -f deploy/phase1/ingest/serviceaccount.yaml
kubectl apply -f deploy/phase1/ingest/stub-nginx-configmap.yaml
```
**Phase 1** — Kafka broker address, secrets, service account, health stub.

```bash
kubectl apply -f deploy/local/overlays/deployment.docker-desktop.yaml
kubectl apply -f deploy/phase1/ingest/service.yaml
kubectl apply -f deploy/phase1/ingest/pdb.yaml
kubectl apply -f deploy/local/overlays/hpa.docker-desktop.yaml
kubectl apply -f deploy/local/overlays/networkpolicy.local.yaml
```
**Phase 1** — 2 ingest pods, Service, PDB, HPA 1–4, NetworkPolicy.

```bash
kubectl apply -f deploy/local/overlays/ingress.local.yaml
echo "127.0.0.1 ingest.local" | sudo tee -a /etc/hosts
```
**Phase 1** — Routes `http://ingest.local/*` → `server-ingest` Service.

---

## Step 3 — Verify

```bash
# Node (single node: docker-desktop)
kubectl get nodes

# App pods — expect kafka + 2 server-ingest
kubectl get pods -n identity-bridge -o wide

# Pod count
kubectl get pods -n identity-bridge -l app=server-ingest --no-headers | wc -l

# Ingress + backend
kubectl get ingress -n identity-bridge
kubectl describe ingress server-ingest -n identity-bridge
kubectl get endpoints -n identity-bridge server-ingest

# nginx = local LB (NOT AWS ELB/ALB)
kubectl get svc -n ingress-nginx ingress-nginx-controller
# EXTERNAL-IP often 127.0.0.1 on Docker Desktop

# HTTP test
curl -v http://ingest.local/health/ready
curl -v http://ingest.local/
```

### Is it connected to ALB/ELB?

**No — locally there is no AWS ALB.** Traffic path:

```
curl → 127.0.0.1:80 → nginx Ingress Controller → Ingress server-ingest → Service → pods
```

On AWS (Phase 1 ALB Ingress), the same Ingress object would be handled by **AWS Load Balancer Controller** and show an `*.elb.amazonaws.com` address.

---

## Kafka interfaces

| Use | Address |
|---|---|
| From ingest pods (ConfigMap `brokers`) | `kafka.identity-bridge.svc.cluster.local:9092` |
| Topics | `identity-events`, `identity-catalog`, `identity-heartbeat` |
| Test produce | `kubectl exec -n identity-bridge deploy/kafka -- rpk topic produce identity-events -k 't:ip' <<<'{"test":1}'` |
| List topics | `kubectl exec -n identity-bridge deploy/kafka -- rpk topic list` |

Collector URLs (when Rust app exists):

```
http://ingest.local/ingest/v1/events
http://ingest.local/ingest/v1/catalog
http://ingest.local/ingest/v1/heartbeat
```

---

## Tear down (Docker Desktop only)

```bash
./deploy/local/revert-local.sh
# Answer 'y' to remove ingress-nginx if you want a clean slate
```

Keeps Docker Desktop Kubernetes enabled; only removes our namespaces.

---

## kind vs Docker Desktop

| | kind (`deploy.sh`) | Docker Desktop (`deploy-docker-desktop.sh`) |
|---|---|---|
| Cluster | Separate kind container | Docker Desktop built-in K8s |
| Ingress manifest | `provider/kind` | `provider/cloud` |
| Ingest replicas | 8 (HPA 4–25) | **2** (HPA 1–4) |
| nodeSelector ingest | Required (kind labels node) | **Removed** (single node) |
| Port 80 | kind extraPortMappings | Docker Desktop LoadBalancer → localhost |
