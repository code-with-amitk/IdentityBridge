# Phase 0 — Shared platform prerequisites

```bash
export AWS_REGION=ap-south-1
export AWS_PROFILE=your-profile   # optional
```

---

## 1. EKS cluster and kubectl access

```bash
# Create cluster + system + ingest node groups (~20 min)
eksctl create cluster -f deploy/phase0/eks/eksctl-cluster.yaml

# Point kubectl at the new cluster
aws eks update-kubeconfig --region ap-south-1 --name identity-bridge

# Verify
kubectl cluster-info
kubectl get nodes
kubectl get nodes -l workload=ingest
```

**Expected:** ≥ 3 ingest nodes (one per AZ), taint `workload=ingest:NoSchedule`.

Alternative node provisioning: [karpenter/nodepool-ingest.yaml](karpenter/nodepool-ingest.yaml) if Karpenter is already installed.

---

## 2. Namespace

```bash
kubectl apply -f deploy/phase0/k8s/namespace.yaml
kubectl get namespace identity-bridge --show-labels
```

---

## 3. Cluster Autoscaler

Ingest node group is tagged for Cluster Autoscaler (`k8s.io/cluster-autoscaler/*` in eksctl config).

```bash
helm repo add autoscaler https://kubernetes.github.io/autoscaler
helm upgrade --install cluster-autoscaler autoscaler/cluster-autoscaler \
  --namespace kube-system \
  --set autoDiscovery.clusterName=identity-bridge \
  --set awsRegion=ap-south-1 \
  --set rbac.serviceAccount.create=true \
  --set extraArgs.balance-similar-node-groups=true \
  --set extraArgs.skip-nodes-with-system-pods=false
```

Autoscaler scales the **ingest** node group toward HPA max (300 ingest pods). Size `maxSize: 80` on ingest node group in eksctl config — tune from pod density ( ~3–4 ingest pods per m7g.xlarge).

---

## 4. Amazon MSK

### 4.1 Create cluster (AWS Console or CLI)

Minimum for ingest phase:

| Setting | Value |
|---|---|
| Cluster name | `identity-bridge-msk` |
| Kafka version | 3.6+ |
| Brokers | 3× `kafka.m7g.large` (dev/staging) — scale per [Kafka_Tier.md](../../Documentation/Scaling_1.5M_Requests_per_sec/Kafka_Tier.md) for prod |
| Authentication | SASL/SCRAM enabled |
| Encryption in transit | TLS |
| VPC | Same VPC as EKS |

Document bootstrap brokers in `msk/msk.env` (from `msk.env.example`).

### 4.2 Security groups

Follow [msk/security-groups.md](msk/security-groups.md). Add MSK broker inbound from EKS node SG on port **9096**.

### 4.3 SCRAM user

```bash
aws kafka batch-associate-scram-secret \
  --cluster-arn "arn:aws:kafka:${AWS_REGION}:ACCOUNT:cluster/identity-bridge-msk/UUID" \
  --secret-arn-list "arn:aws:secretsmanager:${AWS_REGION}:ACCOUNT:secret:msk-scram-ingest"
```

Store username/password in AWS Secrets Manager; copy [msk/client.properties.example](msk/client.properties.example) → `client.properties` locally (gitignored).

### 4.4 Create topics

```bash
cp deploy/phase0/msk/msk.env.example deploy/phase0/msk/msk.env
cp deploy/phase0/msk/client.properties.example deploy/phase0/msk/client.properties
# Edit both files, then:
chmod +x deploy/phase0/msk/create-topics.sh
./deploy/phase0/msk/create-topics.sh
```

Topics created:

| Topic | Partitions |
|---|---|
| `identity-events` | 128 |
| `identity-catalog` | 32 |
| `identity-heartbeat` | 8 |

---

## 5. Phase 0 verification checklist

```bash
kubectl config current-context          # identity-bridge
kubectl get nodes -l workload=ingest    # ≥ 3 Ready
kubectl get ns identity-bridge          # Active, labels present
./deploy/phase0/msk/create-topics.sh    # topics listed (after MSK up)
```
