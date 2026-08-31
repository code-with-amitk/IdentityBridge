# Phase 1 — Ingestion tier bring-up

Everything to run **`server-ingest`** pods and expose them to Collectors.

**Prerequisite:** [Phase 0](../phase0/README.md) complete (K8s + Kafka + Ingress controller).

**Sizing:** 50K req/s — 8 pods steady, HPA 4–25 (AWS). Docker Desktop uses [local overlays](../local/overlays/) (2 pods).

---

## What Phase 1 includes

| Was (old) | Now |
|---|---|
| Phase 1 config objects | ConfigMap, Secret, ServiceAccount |
| Phase 2 manifests | Deployment, Service, HPA, PDB, NetworkPolicy, Go ingest |
| Phase 3 edge | Ingress (ALB or nginx), edge ConfigMap, Collector URLs |

All manifests: [`ingest/`](ingest/)

---

## A. Local — Docker Desktop

```bash
cd ~/IdentityBridge
kubectl config use-context docker-desktop
./deploy/local/deploy-docker-desktop.sh
```

Or manual Phase 1 only (after Phase 0):

```bash
kubectl apply -f deploy/phase1/ingest/serviceaccount.yaml
kubectl apply -f deploy/local/overlays/configmap.local.yaml
kubectl apply -f deploy/local/overlays/secret.local.yaml
./deploy/local/build-ingest-image.sh
kubectl apply -f deploy/local/overlays/deployment.docker-desktop.yaml
kubectl apply -f deploy/phase1/ingest/service.yaml
kubectl apply -f deploy/phase1/ingest/pdb.yaml
kubectl apply -f deploy/local/overlays/hpa.docker-desktop.yaml
kubectl apply -f deploy/local/overlays/networkpolicy.local.yaml
kubectl apply -f deploy/local/overlays/ingest-edge-configmap.local.yaml
kubectl apply -f deploy/local/overlays/ingress.local.yaml
echo "127.0.0.1 ingest.local" | sudo tee -a /etc/hosts
kubectl -n identity-bridge rollout status deployment/server-ingest
curl http://ingest.local/health/ready
```

---

## B. AWS — EKS (50K req/s)

### B.1 Install AWS Load Balancer Controller (one-time, part of Phase 0 on AWS)

See [Runbook_README.md](../phase0/Runbook_README.md) or AWS docs:

### B.2 Apply Phase 1 manifests

```bash
kubectl apply -f deploy/phase1/ingest/serviceaccount.yaml
kubectl apply -f deploy/phase1/ingest/configmap.yaml      # edit brokers first
kubectl apply -f deploy/phase1/ingest/secret.yaml         # from secret.yaml.example
# Build/push Go image to your registry, then:
kubectl apply -f deploy/phase1/ingest/deployment.yaml
kubectl apply -f deploy/phase1/ingest/service.yaml
kubectl apply -f deploy/phase1/ingest/pdb.yaml
kubectl apply -f deploy/phase1/ingest/hpa.yaml
kubectl apply -f deploy/phase1/ingest/networkpolicy.yaml
kubectl apply -f deploy/phase1/ingest/ingest-edge-configmap.yaml
kubectl apply -f deploy/phase1/ingest/ingress.yaml        # edit ACM cert + host
kubectl -n identity-bridge rollout status deployment/server-ingest
```

### B.3 Route 53 (optional)

```bash
./deploy/phase1/route53/create-alias.sh
```

---

## Verify Phase 1

```bash
kubectl get pods,svc,ingress,hpa -n identity-bridge
kubectl get pods -n identity-bridge -l app=server-ingest --no-headers | wc -l
kubectl describe ingress server-ingest -n identity-bridge
curl http://ingest.local/health/ready    # local
```

Collector URLs: [ingest/collector-endpoints.md](ingest/collector-endpoints.md)

---

## Manifest reference

| File | Purpose |
|---|---|
| `configmap.yaml` | Kafka brokers + topic names (AWS MSK) |
| `secret.yaml.example` | MSK SCRAM credentials template |
| `serviceaccount.yaml` | Pod identity (+ optional IRSA) |
| `stub-nginx-configmap.yaml` | Unused nginx stub (rollback only) |
| `deployment.yaml` | 8 replicas, Go `server-ingest` image, nodeSelector ingest (AWS) |
| `service.yaml` | ClusterIP :8080 |
| `hpa.yaml` | min 4, max 25, CPU 60% |
| `pdb.yaml` | minAvailable 80% |
| `networkpolicy.yaml` | Egress to MSK + DNS (AWS VPC CIDR) |
| `ingress.yaml` | ALB Ingress (AWS) |
| `ingest-edge-configmap.yaml` | Public hostname + Collector URLs |

Env vars: [ingest/env-vars.md](ingest/env-vars.md)
