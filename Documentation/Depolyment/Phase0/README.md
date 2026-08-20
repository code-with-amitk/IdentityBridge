

# Prepare Cluster

## Done
- 1 EKS cluster config (`identity-bridge`)
- 2 **node groups** inside that cluster: `system`, `ingest`
- Topics, security-group
- Optional [Karpenter](https://code-with-amitk.github.io/System_Design/Concepts/Kubernets/Terms.html) manifest
- 1 EKS Cluster per [AWS region](https://github.com/code-with-amitk/Code-examples/blob/master/System-Design/Concepts/AWS/Terms/README.adoc#geographic-region-aws-region), not 1 cluster per Tier

## Runbook & config
- Runbook: [deploy/phase0/README.md](../../../deploy/phase0/README.md)  
- EKS config: [deploy/phase0/eks/eksctl-cluster.yaml](../../../deploy/phase0/eks/eksctl-cluster.yaml)

## One cluster or many?

```
┌─────────────────────────────────────────────────────────────────────────┐
│  AWS Region (e.g. ap-south-1) — ONE VPC                                 │
│                                                                         │
│  ┌──────────────────────── EKS cluster: identity-bridge ──────────────┐ │
│  │  namespace: identity-bridge                                       │ │
│  │    ├── server-ingest pods      (ingestion tier)   ← Phase 1–3     │ │
│  │    ├── server-consumer-* pods  (consumer tier)    ← later phase   │ │
│  │    └── server-query pods       (query tier)       ← later phase   │ │
│  │                                                                   │ │
│  │  node group: system   (ALB controller, autoscaler, …)            │ │
│  │  node group: ingest   (server-ingest only)                        │ │
│  │  node group: consumer (optional, later)                           │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌──────── Amazon MSK (Kafka) ────────┐  ← NOT on EKS; managed service │
│  │  brokers in same VPC               │                                │
│  └────────────────────────────────────┘                                │
│                                                                         │
│  Aurora, Redis, ALB — also outside EKS (managed AWS services)           │
└─────────────────────────────────────────────────────────────────────────┘
```

## EKS cluster `identity-bridge` — node groups

### `system`

- Cluster Autoscaler, AWS Load Balancer Controller, CoreDNS, etc.
- Scaling: 2–5 nodes (starts at 2)

### `ingest`

- AZs: `ap-south-1a`, `ap-south-1b`, `ap-south-1c`
- Instance types: `m7g.xlarge`, `c7g.xlarge`
- Scaling: node group `minSize: 3`, `maxSize: 80` (Cluster Autoscaler)
- Ingest **pods**: HPA `minReplicas: 50`, `maxReplicas: 300` (Phase 2)

Rough capacity in one region: ~**3–4 ingest pods per m7g.xlarge** → 80 nodes × ~3 ≈ **240 pods** at node max; increase `maxSize` or instance size for 300 pods.
