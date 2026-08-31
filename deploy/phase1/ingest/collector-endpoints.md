# Collector → Server ingest URLs

Use these in **Collector** config (`server.ingest_base_url` or per-path URLs).  
Hostname must match [ingress.yaml](ingress.yaml) and ACM certificate.

**Deployment target:** 50,000 requests/sec (8 `server-ingest` pods, HPA 4–25).

---

## Base URL

```
https://ingest.identity-bridge.example.com
```

Replace `ingest.identity-bridge.example.com` with your Route 53 record (see [ingest-edge-configmap.yaml](ingest-edge-configmap.yaml)).

---

## Ingest paths

| Purpose | Method | Path | Called by |
|---|---|---|---|
| Session / login events | `POST` | `/ingest/v1/events` | Collector |
| Catalog deltas | `POST` | `/ingest/v1/catalog` | Collector |
| Collector liveness | `POST` | `/ingest/v1/heartbeat` | Collector |

Full URLs:

```
https://ingest.identity-bridge.example.com/ingest/v1/events
https://ingest.identity-bridge.example.com/ingest/v1/catalog
https://ingest.identity-bridge.example.com/ingest/v1/heartbeat
```

---

## Collector YAML example

```yaml
server:
  ingest_base_url: "https://ingest.identity-bridge.example.com"
  api_key_ref: "COLLECTOR_INGEST_API_KEY"
```

Paths are appended by the ingest client: `/ingest/v1/events`, etc.

---

## TLS and auth

| Layer | Detail |
|---|---|
| TLS | Terminated at **ALB** (ACM certificate) |
| Auth | Bearer token (`COLLECTOR_API_KEYS`) or mTLS (later). Empty API keys = open (local/dev). |

---

## Health (ALB target group)

ALB checks pod target directly:

```
GET http://<pod-ip>:8080/health/ready
```

Same path used by Kubernetes readiness probe.

---

## Phase 3 stub behavior

Until the Collector ingest client ships, you can POST JSON directly. Go `server-ingest` validates the body and produces to Kafka.
