# MSK security groups — ingest tier connectivity (Phase 0)

Allow **ingest node / pod security group** → **MSK broker security group** on these ports:

| Port | Protocol | Use |
|---|---|---|
| 9092 | TCP | Plaintext (dev only — not for production) |
| 9094 | TCP | TLS |
| 9096 | TCP | SASL/SCRAM over TLS (recommended) |
| 9098 | TCP | IAM auth over TLS (alternative to SCRAM) |

## Inbound rule on MSK broker SG

```
Type: Custom TCP
Port: 9096
Source: sg-<eks-ingest-node-or-pod-sg>
Description: identity-bridge server-ingest pods → MSK SASL_SSL
```

## Outbound from EKS ingest nodes

EKS node security group typically allows all egress. If locked down, allow egress to MSK broker SG on 9096.

## Verify from cluster

```bash
kubectl run msk-debug -n identity-bridge --rm -it --restart=Never \
  --image=confluentinc/cp-kafka:7.6.0 -- bash

# Inside pod (mount client.properties or pass env):
kafka-broker-api-versions.sh --bootstrap-server "$MSK_BOOTSTRAP_BROKERS" \
  --command-config /tmp/client.properties
```

## Auth options

| Method | Phase 0 choice | Notes |
|---|---|---|
| **SCRAM-SHA-512** | Recommended | User per tier; store password in Secrets Manager |
| **IAM** | Optional | IRSA on `server-ingest` ServiceAccount; no password in K8s Secret |

Bootstrap broker string: AWS Console → MSK → cluster → **View client information** → SASL/SCRAM bootstrap servers.
