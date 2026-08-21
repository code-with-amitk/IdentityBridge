# Ingestion tier — environment variables

Used by `server-ingest` Deployment (Phase 2). Config objects defined in Phase 1.

## Required

| Variable | Source | Example |
|---|---|---|
| `KAFKA_BROKERS` | ConfigMap `kafka-config` / key `brokers` | `b-1....amazonaws.com:9096,...` |
| `KAFKA_TOPIC_EVENTS` | ConfigMap / `topic_events` | `identity-events` |
| `KAFKA_TOPIC_CATALOG` | ConfigMap / `topic_catalog` | `identity-catalog` |
| `KAFKA_TOPIC_HEARTBEAT` | ConfigMap / `topic_heartbeat` | `identity-heartbeat` |
| `KAFKA_SCRAM_USERNAME` | Secret `server-ingest-secrets` | `identity-bridge-ingest` |
| `KAFKA_SCRAM_PASSWORD` | Secret `server-ingest-secrets` | *(from Secrets Manager)* |
| `HTTP_BIND` | Literal in Deployment | `0.0.0.0:8080` |

## Optional

| Variable | Default | Notes |
|---|---|---|
| `RUST_LOG` | `info` | `debug` for troubleshooting |
| `KAFKA_PRODUCER_ACKS` | ConfigMap `producer_acks` | `1` or `all` |
| `KAFKA_PRODUCER_COMPRESSION` | ConfigMap `producer_compression` | `lz4`, `zstd` |
| `KAFKA_PRODUCER_LINGER_MS` | ConfigMap `producer_linger_ms` | batching delay |
| `KAFKA_PRODUCER_IDEMPOTENCE` | ConfigMap `producer_enable_idempotence` | `true` |
| `MAX_REQUEST_BODY_BYTES` | `2097152` | 2 MiB — Rust phase |
| `COLLECTOR_API_KEYS` | Secret `collector_api_keys` | empty until auth phase |

## Deployment snippet (Phase 2 reference)

```yaml
env:
  - name: KAFKA_BROKERS
    valueFrom:
      configMapKeyRef:
        name: kafka-config
        key: brokers
  - name: KAFKA_TOPIC_EVENTS
    valueFrom:
      configMapKeyRef:
        name: kafka-config
        key: topic_events
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
  - name: HTTP_BIND
    value: "0.0.0.0:8080"
  - name: RUST_LOG
    value: info
```

## IRSA alternative (no SCRAM password in Secret)

If ServiceAccount has `eks.amazonaws.com/role-arn` for MSK IAM:

- Omit `KAFKA_SCRAM_*` env vars
- Set `KAFKA_AUTH_MODE=iam` in Rust app (Phase 5)
- MSK cluster must have IAM access control enabled
