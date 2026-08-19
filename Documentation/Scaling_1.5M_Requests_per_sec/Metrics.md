

## Metrics

| Metric | Value |
|---|---|
| Peak HTTP ingest requests/sec | **1,500,000** |
| Ingest API response | **202 Accepted** within **p99 ≤ 50 ms** (after Kafka ACK) |
| End-to-end session visibility (Redis) | **p99 ≤ 500 ms** under peak |
| Durable PostgreSQL lag under peak | **≤ 30 s** (consumer lag SLO) |
| Availability | Multi-AZ; ingest and consumer tiers scale independently |