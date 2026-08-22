# Deploy layout

Two phases only:

| Folder | Purpose |
|---|---|
| [phase0/](phase0/) | Platform — Kubernetes, Kafka, Ingress controller |
| [phase1/](phase1/) | Ingestion tier — pods, Service, HPA, Collector Ingress |
| [local/](local/) | Local overlays + one-shot scripts (Docker Desktop / kind) |

**Quick start (Docker Desktop):** `./local/deploy-docker-desktop.sh`

Docs: [Documentation/Depolyment/README.md](../Documentation/Depolyment/README.md)
