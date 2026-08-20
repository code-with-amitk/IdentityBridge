- [Metrics](#metrics)
- [SLO](#sloservice-level-objective)

## Metrics

| Metric | Value |
|---|---|
| Peak HTTP ingest requests/sec | **1,500,000** |
| Ingest API response | **202 Accepted** within **p99 ≤ 50 ms** (after Kafka ACK) |
| End-to-end session visibility (Redis) | **p99 ≤ 500 ms** under peak |
| Durable PostgreSQL lag under peak | **≤ 30 s** (consumer lag SLO) |
| Availability | Multi-AZ; ingest and consumer tiers scale independently |


## SLO(Service Level Objective)

| Metric | Alert threshold |
|---|---|
| Ingest p99 latency | > 100 ms |
| Kafka consumer lag (`session-writers`) | > 1M messages for 5 min |
| Redis hit rate (IP query) | < 95% |
| PG replication lag | > 10 s |
| Ingest 5xx rate | > 0.1% |