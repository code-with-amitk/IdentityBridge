# Phase 0 — Platform (Kubernetes + Kafka + Ingress)

Bring up the **cluster foundation** before any Identity Bridge app pods.

| Component | AWS (staging/prod) | Local Docker Desktop |
|---|---|---|
| Kubernetes | EKS (`eksctl-cluster.yaml`) | Docker Desktop K8s |
| Kafka | Amazon MSK + topics script | Redpanda in cluster (`kafka-in-cluster.yaml`) |
| Ingress controller | AWS Load Balancer Controller → ALB | **nginx** Ingress Controller |

**Not in Phase 0:** ingest Deployment, ConfigMap for app, Aurora, Redis.

**Next:** [Phase 1 — Ingestion tier](../phase1/README.md)

Local quick path: [deploy/local/README-docker-desktop.md](../local/README-docker-desktop.md)

---

## Phase 0 checklist

- [ ] Kubernetes cluster running (`kubectl get nodes`)
- [ ] Namespace `identity-bridge` created
- [ ] Kafka broker reachable + topics created
- [ ] Ingress controller running (nginx local / ALB controller AWS)

---

## A. Local — Docker Desktop (recommended for dev)

### Prerequisites

Docker Desktop → **Settings → Kubernetes → Enable Kubernetes** → Apply & Restart

```bash
kubectl config use-context docker-desktop
kubectl get nodes
```

### Commands

```bash
cd ~/IdentityBridge

# 0.1 nginx Ingress Controller (local load balancer)
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/cloud/deploy.yaml
kubectl wait -n ingress-nginx --for=condition=ready pod -l app.kubernetes.io/component=controller --timeout=180s
```
Installs nginx — receives HTTP on **localhost:80** (Docker Desktop maps LoadBalancer to 127.0.0.1).

```bash
# 0.2 App namespace
kubectl apply -f deploy/phase0/k8s/namespace.yaml
```
Creates `identity-bridge` namespace.

```bash
# 0.3 Kafka (Redpanda, in-cluster)
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml
kubectl wait -n identity-bridge --for=condition=ready pod -l app=kafka --timeout=180s
```
Single broker at `kafka.identity-bridge.svc.cluster.local:9092`.

```bash
# 0.4 Kafka topics (50K partition counts)
chmod +x deploy/local/create-topics-local.sh
./deploy/local/create-topics-local.sh
```

**Or one shot:** `./deploy/local/deploy-docker-desktop.sh` (Phase 0 + Phase 1)

---

## B. Local — kind (alternative)

```bash
kind create cluster --name identity-bridge --config deploy/local/kind-config.yaml
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/kind/deploy.yaml
kubectl apply -f deploy/phase0/k8s/namespace.yaml
kubectl apply -f deploy/phase0/kafka-in-cluster.yaml
./deploy/local/create-topics-local.sh
```

**Or:** `./deploy/local/deploy.sh`

---

## C. AWS — EKS + MSK

See [Runbook_README.md](Runbook_README.md) for full AWS commands.

Summary:

```bash
eksctl create cluster -f deploy/phase0/eks/eksctl-cluster.yaml
kubectl apply -f deploy/phase0/k8s/namespace.yaml
# Install Cluster Autoscaler (Runbook §3)
# Create MSK cluster + security groups (Runbook §4)
./deploy/phase0/msk/create-topics.sh   # after msk.env configured
# Install AWS Load Balancer Controller (Phase 1 README § AWS ALB — needed before ALB Ingress)
```

---

## Verify Phase 0

```bash
kubectl get nodes
kubectl get ns identity-bridge
kubectl get pods -n identity-bridge -l app=kafka
kubectl get pods -n ingress-nginx
kubectl exec -n identity-bridge deploy/kafka -- rpk topic list   # local only
```

---

## Directory layout

```
deploy/phase0/
├── README.md                 ← this file
├── Runbook_README.md         ← AWS EKS + MSK detail
├── k8s/namespace.yaml
├── kafka-in-cluster.yaml     ← local Kafka (Redpanda)
├── eks/eksctl-cluster.yaml   ← AWS EKS
├── msk/                      ← AWS MSK topics + SG docs
└── karpenter/                ← optional AWS node pool
```
