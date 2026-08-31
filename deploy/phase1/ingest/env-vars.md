# Ingestion tier — environment variables

Used by Go `server-ingest` Deployment.

## Required

| Variable | Source | Example |
|---|---|---|
| `KAFKA_BROKERS` | ConfigMap `kafka-config` / key `brokers` | `b-1....amazonaws.com:9096,...` |
| `KAFKA_TOPIC_EVENTS` | ConfigMap / `topic_events` | `identity-events` |
| `KAFKA_TOPIC_CATALOG` | ConfigMap / `topic_catalog` | `identity-catalog` |
| `KAFKA_TOPIC_HEARTBEAT` | ConfigMap / `topic_heartbeat` | `identity-heartbeat` |
| `KAFKA_AUTH_MODE` | Literal in Deployment | `scram` (MSK) or `none` (local Redpanda) |
| `HTTP_BIND` | Literal in Deployment | `0.0.0.0:8080` |

## Auth (MSK)

| Variable | Source | Notes |
|---|---|---|
| `KAFKA_SCRAM_USERNAME` | Secret `server-ingest-secrets` | Required when `KAFKA_AUTH_MODE=scram` |
| `KAFKA_SCRAM_PASSWORD` | Secret `server-ingest-secrets` | Required when `KAFKA_AUTH_MODE=scram` |
| `KAFKA_TLS` | Literal | Defaults to true for `scram`; set `false` locally |

## Optional

| Variable | Default | Notes |
|---|---|---|
| `LOG_LEVEL` | `info` | `debug` for troubleshooting (`RUST_LOG` still accepted) |
| `LOG_FORMAT` | `json` | `text` or `json` |
| `KAFKA_PRODUCER_ACKS` | ConfigMap `producer_acks` | `1` or `all` |
| `KAFKA_PRODUCER_COMPRESSION` | ConfigMap `producer_compression` | `lz4`, `zstd` |
| `KAFKA_PRODUCER_LINGER_MS` | ConfigMap `producer_linger_ms` | batching delay |
| `MAX_REQUEST_BODY_BYTES` | `2097152` | 2 MiB |
| `COLLECTOR_API_KEYS` | Secret `collector_api_keys` | empty = no Bearer auth (dev) |

## Deployment snippet

```yaml
env:
  - name: KAFKA_BROKERS
    valueFrom:
      configMapKeyRef:
        name: kafka-config
        key: brokers
  - name: KAFKA_SCRAM_USERNAME
    valueFrom:
      secretKeyRef:
        name: server-ingest-secrets
        key: msk_scram_username
  - name: KAFKA_SCRAM_PASSWORD
    valueFrom:
      secretKeyRef:
        name: server-ingest-secrets
        key: msk_scram_password
  - name: KAFKA_AUTH_MODE
    value: scram
  - name: HTTP_BIND
    value: "0.0.0.0:8080"
  - name: LOG_LEVEL
    value: info
```

## IRSA alternative (no SCRAM password in Secret)

If ServiceAccount has `eks.amazonaws.com/role-arn` for MSK IAM:

- Omit `KAFKA_SCRAM_*` env vars
- Set `KAFKA_AUTH_MODE=iam` — **not implemented yet** in the Go ingest binary; use SCRAM until IAM SASL is added
- MSK cluster must have IAM access control enabled
