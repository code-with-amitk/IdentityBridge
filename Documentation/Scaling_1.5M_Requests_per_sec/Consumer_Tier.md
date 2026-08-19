Contents


# Consumer Tier

The **consumer tier** reads from Kafka and writes to **SQL(Aurora)** and **Redis** (hot session index). It scales independently from the ingestion tier.

## Sequence diagram


```mermaid
sequenceDiagram
    autonumber

    participant K as Kafka<br/>identity-events
    box Consumer Pod
    participant C as Kafka Consumer<br/>Tokio task
    participant B as Batch accumulator<br/>5000 rows / 200ms
    participant R as Redis pipeline<br/>sess:{tenant}:{ip}
    participant P as PgBouncer
    participant PG as PostgreSQL<br/>sessions staging
    end

    K->>C: Poll messages<br/>(consumer group: session-writers)
    C->>B: Append records
    Note over B: Flush when<br/>5000 rows OR 200ms

    B->>R: SET sess:tenant:ip (pipeline)
    R-->>B: OK

    B->>P: COPY staging_sessions
    P->>PG: Bulk insert
    PG-->>P: OK
    P-->>B: OK

    B->>PG: MERGE staging → sessions
    PG-->>B: OK

    B->>C: Commit offset
    C->>K: offset commit
```

## Consumer groups
```
    Topic           |   Initial Pods
    ----------------|--------------
`catalog-writers`   |   64
`identity-events`   |   16
```
* Max pods per topic ≈ partition count (128 session partitions → max ~128 useful session consumers).
* With **128 partitions** and **64 session pods**, each pod owns ~**2 partitions** on average.

Kafka rebalances on pod scale-up/down. Use **cooperative-sticky** assignor to minimize churn.


## Scaling
```
if (`kafka_consumer_group_lag` > 10K avg per pod for 5 min) {
    HPA scale up consumer deployment
}

// P99 means 99 out of 100 requests finish in expected time
if (PG `write_latency` p99 > 100 ms) {
    Increase batch size or add PgBouncer pool; do not add consumers blindly
}

if (Redis CPU > 70%)
{
    Scale ElastiCache shards
}
```