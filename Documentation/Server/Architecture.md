# Server

The **Server** runs in AWS (or local Kubernetes). It is written in **Go**. The on-prem **Collector** remains **Rust**.

```
Collector (Rust) --HTTPS JSON--> server-ingest (Go) --> Kafka
                                                       |
                                                       v
                                         consumer-* (Go) --> PostgreSQL + Redis
                                                       |
                                                       v
                              SRX / vSRX --HTTPS--> server-query (Go)
```

## Binaries (`server/cmd/`)

| Binary | Deployment | Status |
|---|---|---|
| `ingest` | `server-ingest` | Implemented — validate JSON, produce to Kafka, `/health/*` |
| `query` | `server-query` | Routes exist; handlers return 501 until SQL/Redis |
| `consumer-session` | later | Kafka log consumer; SQL upsert not wired |
| `consumer-catalog` | later | Kafka log consumer; SQL upsert not wired |

## HTTP — ingest

| Method | Path | Caller |
|---|---|---|
| `GET` | `/health/live` | Kubernetes |
| `GET` | `/health/ready` | Kubernetes (Kafka reachable) |
| `POST` | `/ingest/v1/events` | Collector |
| `POST` | `/ingest/v1/catalog` | Collector |
| `POST` | `/ingest/v1/heartbeat` | Collector |

Contract: [server/openapi/ingest.yaml](../../server/openapi/ingest.yaml). Types: [server/pkg/types](../../server/pkg/types) ↔ [crates/common](../../crates/common).

## HTTP — query (firewall)

| Method | Path |
|---|---|
| `POST` | `/oauth_token/oauth` |
| `GET` | `/user-query/v2/users/endpoints` |
| `GET` | `/user_query/v1/ip/{ip}` |
| `GET` | `/user?ip=&id=&domain=` |

## Layout

```
server/
├── cmd/ingest/
├── cmd/query/
├── cmd/consumer-session/
├── cmd/consumer-catalog/
├── internal/api/ingest/
├── internal/api/query/
├── internal/kafka/
├── internal/config/
├── pkg/types/
├── openapi/ingest.yaml
└── Dockerfile
```

Build: [server/README.md](../../server/README.md). Deploy: [deploy/phase1](../../deploy/phase1/README.md).
