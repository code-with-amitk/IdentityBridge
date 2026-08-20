- [Kafka Tier](#kafka-tier)
- [Topics](#topics)
- [cluster (starting point)](#cluster-starting-point)
- [Partitioning](#partitioning)
  - [How it Help](#how-it-help)
  - [Partition keys (per topic)](#partition-keys-per-topic)
    - [Session events (`identity-events`)](#session-events-identity-events)
    - [Catalog deltas (`identity-catalog`)](#catalog-deltas-identity-catalog)
    - [Heartbeat (`identity-heartbeat`)](#heartbeat-identity-heartbeat)
    - [Partition count vs consumer pods](#partition-count-vs-consumer-pods)
- [Message Envelope](#message-envelope)



# Kafka Tier

## Topics
```
            Topic |             Purpose                    | Initial partitions     | Replication 
`identity-events` | Session / login / logout micro-batches | **128** (scale to 256) | 3
`identity-catalog`| User / group / membership deltas       | **32** (scale to 64)   | 3 
`identity-heartbeat` | Collector liveness (optional)       | **8**                  | 3
```
**Rule:** `partitions ≥ max consumer pods` for the topic’s consumer group so each pod can own at least one partition.

## cluster (starting point)

| Component | Initial spec |
|---|---|
| Brokers | **12× kafka.m5.4xlarge** (or **kafka.m7g.4xlarge**) across 3 AZs |
| Storage | **2 TB gp3** per broker |
| Throughput target | ~**24 GB/s** ingress peak (1.5M × 16 KB) — validate with MSK sizing calculator |
| Retention | **24 h** events; **72 h** catalog (compliance / replay) |

## Partitioning
### How it Help

1. **Ordering of requests:** Login before logout for same `(tenant, ip)`
2. **Parallel processing of requests:** More partitions → more consumer pods in parallel
3. **DB locality:** Optional: co-locate tenant shards (advanced)

### Partition keys (per topic)

#### Session events (`identity-events`)
```
partition_key = Hash("{tenant_id}:{ip_address}")
```
- All events for 1 IP in 1 tenant are strictly ordered
- **Hotspot Risk:** Very large tenants with concentrated IP ranges — monitor partition byte rate
- If `ip_address` missing. Key = Hash(`{tenant_id}:{username_hash}`)

#### Catalog deltas (`identity-catalog`)
```
partition_key = "{tenant_id}:{object_sid}"
```

#### Heartbeat (`identity-heartbeat`)
```
partition_key = "{tenant_id}:{collector_id}"
```
- Low volume; used for ops visibility only.

#### Partition count vs consumer pods
```
            Topic | Partitions
`identity-events` | **128**
`identity-catalog` | **32**
`identity-heartbeat` | **8**
```

## Message Envelope

```json
{
  "batch_id": "uuid",
  "tenant_id": "tenant-001",
  "collector_id": "collector-dc1-01",
  "received_at": "2026-08-19T08:00:00Z",
  "record_type": "session",
  "records": [ "..." ]
}
```