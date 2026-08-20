
- [Will PostgreSQL become a bottleneck?](#will-postgresql-become-a-bottleneck)
- [Aurora(SQL)](#aurorasql)
  - [Tables](#tables)
    - [Session table design (write-optimized)](#session-table-design-write-optimized)


# Will PostgreSQL become a bottleneck?
- Yes — if every ingest request maps to a synchronous row write on the primary.
- No — if the architecture treats PostgreSQL as a bulk, eventually-consistent store behind Kafka and Redis.

# Aurora(SQL)
- At **1.5M HTTP requests/sec** with **100 records/request**:
```
1500000x100 = 150 M records/sec → far beyond single-node PostgreSQL write capacity
```
- We will use AWS Aurora for same

## Tables
### Session table design (write-optimized)

```sql
CREATE TABLE sessions (
    tenant_id     uuid NOT NULL,
    ip_address    inet NOT NULL,
    username      text NOT NULL,
    domain        text,
    groups_json   jsonb,
    state         text NOT NULL,  -- active | inactive
    last_seen     timestamptz NOT NULL,
    updated_at    timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, ip_address)
) PARTITION BY HASH (tenant_id);

-- 32 or 64 hash partitions
CREATE INDEX sessions_active_updated
    ON sessions (tenant_id, updated_at)
    WHERE state = 'active';
```

