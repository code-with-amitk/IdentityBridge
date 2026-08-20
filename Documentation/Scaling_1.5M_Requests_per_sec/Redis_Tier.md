- [Redis Cluster](#redis-cluster)
- [Capacity](#capacity)

# Redis Cluster
Redis is **not optional** at 1.5M ingest req/sec target for IP query SLO.

## Capacity
```
1 shard = 150k ops/sec
2 shards = 300k
10 shards = 1.5M
```