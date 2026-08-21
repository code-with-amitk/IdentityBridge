
Contents
- [Kubernetes, Ingestion Tier Install on Docker Desktop](#kubernetes-ingestion-tier-install-on-docker-desktop)
  - [Phase 0 — Kubernets, nginx, kafka](#phase-0--kubernets-nginx-kafka)
  - [Phase 1 - Ingestion Tier](#phase-1---ingestion-tier)
- [Verify deployment](#verify-deployment)
- [Architecture (one cluster)](#architecture-one-cluster)


## Kubernetes, Ingestion Tier Install on Docker Desktop

```bash
cd ~/IdentityBridge
kubectl config use-context docker-desktop
./deploy/local/deploy-docker-desktop.sh
curl http://ingest.local/health/ready
```

Detail: [deploy/local/README-docker-desktop.md](../../deploy/local/README-docker-desktop.md)

**Phase 0 Setup bringup** Kubernetes + Kafka + Ingress (nginx local / ALB AWS)

**Phase 1 Ingestion tier bringup** ConfigMap, Deployment, Service, HPA, Ingress rules

### Phase 0 — Kubernets, nginx, kafka
```
kubectl config use-context docker-desktop // Point kubectl at Docker Desktop's built-in cluster
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/cloud/deploy.yaml`        // Install **nginx Ingress Controller** — acts as HTTP load balancer on localhost:80
kubectl wait -n ingress-nginx --for=condition=ready pod -l app.kubernetes.io/component=controller --timeout=180s    // Wait until nginx is ready to accept traffic
kubectl apply -f deploy/phase0/k8s/namespace.yaml   // Create `identity-bridge` namespace 
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml`   // Deploy **Redpanda** (Kafka-compatible broker) inside the cluster
kubectl wait -n identity-bridge --for=condition=ready pod -l app=kafka --timeout=180s    // Wait until Kafka broker pod is ready
./deploy/local/create-topics-local.sh   // Create topics: `identity-events` (24p), `identity-catalog` (8p), `identity-heartbeat` (4p)
```

### Phase 1 - Ingestion Tier

| Command | What it does |
|---|---|
| `kubectl apply -f deploy/phase1/ingest/serviceaccount.yaml` | ServiceAccount for ingest pods |
| `kubectl apply -f deploy/local/overlays/configmap.local.yaml` | Kafka bootstrap `kafka.identity-bridge.svc.cluster.local:9092` + topic names |
| `kubectl apply -f deploy/local/overlays/secret.local.yaml` | Placeholder secrets (no MSK SCRAM locally) |
| `kubectl apply -f deploy/phase1/ingest/stub-nginx-configmap.yaml` | nginx stub serving `/health/live` and `/health/ready` |
| `kubectl apply -f deploy/local/overlays/deployment.docker-desktop.yaml` | **2** ingest pods, no `nodeSelector` (fits single-node Docker Desktop) |
| `kubectl apply -f deploy/phase1/ingest/service.yaml` | ClusterIP Service on port 8080 |
| `kubectl apply -f deploy/phase1/ingest/pdb.yaml` | PodDisruptionBudget — keep 80% available during drains |
| `kubectl apply -f deploy/local/overlays/hpa.docker-desktop.yaml` | HPA min 1, max 4 (lighter than AWS 4–25) |
| `kubectl apply -f deploy/local/overlays/networkpolicy.local.yaml` | Allow egress to in-cluster Kafka :9092 + DNS |
| `kubectl apply -f deploy/local/overlays/ingest-edge-configmap.local.yaml` | Public hostname `ingest.local` for Collector config |
| `kubectl apply -f deploy/local/overlays/ingress.local.yaml` | Route `ingest.local` → `server-ingest` via nginx |
| `echo "127.0.0.1 ingest.local" \| sudo tee -a /etc/hosts` | Resolve hostname to localhost |
| `kubectl -n identity-bridge rollout status deployment/server-ingest` | Wait until all ingest pods pass readiness |


## Verify deployment

```bash
kubectl get nodes
kubectl get pods,svc,ingress,hpa -n identity-bridge
kubectl get pods -n identity-bridge -l app=server-ingest -o wide
curl http://ingest.local/health/ready          # local
kubectl describe ingress server-ingest -n identity-bridge   # AWS ALB address
```


## Architecture (one cluster)

```mermaid
Your WSL shell / browser
        │
        │  http://ingest.local  (127.0.0.1 in /etc/hosts)
        ▼
┌─────────────────────────────────────────────────────────┐
│  Docker Desktop Kubernetes (single node: docker-desktop) │
│                                                          │
│  namespace: ingress-nginx                                │
│    nginx Ingress Controller  ← local “load balancer”     │
│    (LoadBalancer → localhost:80)                       │
│         │                                                │
│         ▼                                                │
│  namespace: identity-bridge                              │
│    Ingress server-ingest  (host: ingest.local)           │
│         │                                                │
│         ▼                                                │
│    Service server-ingest :8080                           │
│         │                                                │
│         ├──► server-ingest pod (nginx stub)              │
│         └──► server-ingest pod                           │
│                                                          │
│    kafka pod (Redpanda :9092)  ← Kafka API             │
│      topics: identity-events, identity-catalog, ...      │
└─────────────────────────────────────────────────────────┘
```
