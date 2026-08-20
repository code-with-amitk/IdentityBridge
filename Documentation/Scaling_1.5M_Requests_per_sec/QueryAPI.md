


- [Query API Tier](#query-api-tier)
- [Kubernets](#kubernets)
    - [Deployment model](#deployment-model)
    - [Initial scale](#initial-scale)
- [Data sources and read path](#data-sources-and-read-path)
- [Read replica scheduling and latency](#read-replica-scheduling-and-latency)
- [Serving many SRX / vSRX devices](#serving-many-srx--vsrx-devices)
- [vSRX in a cloud POP (5–10 customers per datacenter)](#vsrx-in-a-cloud-pop-510-customers-per-datacenter)
- [Tenants per POP](#tenants-per-pop)
- [Multi-tenancy](#multi-tenancy)
    - [One tenant, multiple AD sites (India + US)](#one-tenant-multiple-ad-sites-india--us)
- [Query endpoints](#query-endpoints)


# Query API Tier

- The **Query API tier** serves SRX and vSRX firewalls. 
- It is separate from the ingest path (Collectors → Ingestion tier → Kafka → Consumer tier).
- Firewalls call OAuth, batch sync, IP query, and user query endpoints only on this tier.

See also: [README.md](README.md) architecture diagram, [design.md §7](../../design.md) for JIMS-compatible routes.

---

## Kubernets
### Deployment model

- The Query API runs in its own Kubernetes **Deployment(ie pods)** (`server-query`). 
- It does **not** share pods with `server-ingest` or `server-consumer-*`.
- This pod connects to postgres(read replicas only)

### Initial scale
- **20–50 pods**, HPA on request rate and CPU. 
- Ingest scaling (125–300 pods at 1.5M req/sec)

---

## Data sources and read path

Query pods are **read-only** toward durable storage. They never consume Kafka and never write to the PostgreSQL primary.

```
SRX / vSRX
    │  HTTPS (OAuth, batch, IP, user)
    ▼
ALB → server-query pods
         │
         ├──► Redis (ElastiCache)     IP query hot path — sess:{tenant}:{ip}
         │
         └──► Aurora read replica(s)  Batch sync, fallback IP query, user query
```

**IP query:** Redis first. On miss, read from a replica (`sessions` table). Optional Collector refresh on persistent miss (Phase 2).

**Batch query (`/user-query/v2/users/endpoints`):** Incremental cursor on `sessions.updated_at` — index scan on replica; denormalized `groups` on session row (no join to 10M-user catalog at query time).

**User query:** Replica lookup by `(tenant_id, ip, username, domain)`.

---

## Read replica scheduling and latency

- AWS routes connections across healthy replicas (round-robin / least-connection at the storage proxy layer)
- Query pod in `ap-south-1a` prefers replica in same AZ to avoid cross-AZ latency (~1 ms)

For lowest latency:

1. Co-locate Query pods and read replicas in the same region/AZ as the POP or customer edge.
2. Keep Redis for IP query (avoids PostgreSQL on the hottest path).
3. Use replica lag monitoring — if lag &gt; 5 s, alert; batch clients already poll every ~5 s by default.

---

## Serving many SRX / vSRX devices

One Identity Bridge Server cluster serves **many firewall clients** per tenant and across tenants.

| Mechanism | Detail |
|---|---|
| **OAuth client credentials** | Each SRX/vSRX (or CSO-managed client) has `client_id` + secret → token scoped to `tenant_id` |
| **Stateless Query pods** | ALB distributes firewall HTTPS across all `server-query` replicas |
| **Per-firewall rate limits** | Token bucket per `client_id` — prevents one device from starving others |
| **Batch cursor per firewall** | `begintime` stored in PostgreSQL `batch_cursors` (or optional Redis cache) — each SRX advances its own cursor |
| **No per-firewall pod** | 1,000 firewalls do not require 1,000 pods; pods scale on aggregate QPS |

Typical load: 1,000 firewalls × 500 entries / 5 s ≈ **100K entries/sec** read burst from replicas + Redis — sized separately from 1.5M ingest req/sec.

---

## vSRX in a cloud POP (5–10 customers per datacenter)

Yes. One **regional Identity Bridge stack** (Query + ingest + Kafka + DB) in a cloud POP commonly backs **multiple customer vSRX instances** in that region.

---

## Tenants per POP

Planning guidelines for one POP region:

| Factor | Typical bound (design planning) |
|---|---|
| Small/medium tenants | **100–500** tenants per POP on shared multi-tenant stack |
| Large enterprise tenants | **1–10** heavy tenants may consume a dedicated stack or shard |
| Limiting factors | Ingest req/sec, session row count, Redis memory, replica read IOPS, OAuth token rate |

Operational policy (not product limit): set **max tenants per POP** in runbooks from load tests — e.g. cap at **200 tenants** until ingest p99 and query p99 SLOs hold, then increase.

---

## Multi-tenancy

### One tenant, multiple AD sites (India + US)

**Default model — one logical tenant, multiple Collectors, one Server region:**

```
Tenant ACME (tenant_id = acme-001)
  │
  ├── Collector (India)  ──► AD Mumbai    ──HTTPS ingest──┐
  ├── Collector (US)     ──► AD Virginia  ──HTTPS ingest──┼──► Same Identity Bridge Server (tenant_id = acme-001)
  └── Collectors share same tenant_id; events merged in Kafka → PostgreSQL
```

All Collectors for one customer use the **same `tenant_id`**. Session rows are keyed by `(tenant_id, ip_address)` globally — a user logged in Mumbai and a user logged in Virginia both appear in the same tenant session plane.

**Regional Server placement (optional):**

| Pattern | When |
|---|---|
| **Single regional Server** (e.g. US-East) | Collectors in India and US both push to one cloud region; acceptable if WAN latency to ingest &lt; SLO |
| **Regional Server per geography** | Data-residency or ingest latency requirements — **separate `tenant_id` shards** (`acme-in`, `acme-us`) or **federated query** (advanced Phase 2) |
| **Dedicated stack per large tenant** | One tenant exceeds shared POP capacity |

India and US do **not** automatically require separate Servers unless compliance or latency mandates it. Separate AD locations are handled by **multiple Collectors**, not multiple Query APIs for the same tenant.


---

## Query endpoints

| Method | Path | Primary data source |
|---|---|---|
| `POST` | `/oauth_token/oauth` | `api_clients` (replica or small cache) |
| `GET` | `/user-query/v2/users/endpoints` | Replica `sessions` (+ `batch_cursors`) |
| `GET` | `/user_query/v1/ip/{ip}` | **Redis**, then replica |
| `GET` | `/user?ip=&id=&domain=` | Replica `sessions` / catalog join if needed |