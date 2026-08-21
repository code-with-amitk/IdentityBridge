# Local deploy — kind (WSL, without Docker Desktop Kubernetes)

Use **kind** when you are **not** using Docker Desktop’s built-in Kubernetes.

For Docker Desktop K8s, use [README-docker-desktop.md](README-docker-desktop.md) instead.

---

## Prerequisites

```bash
docker version
kind version
kubectl version --client
```

---

## Deploy

```bash
cd ~/IdentityBridge
chmod +x deploy/local/deploy.sh deploy/local/create-topics-local.sh
./deploy/local/deploy.sh
```

Creates kind cluster `identity-bridge`, then **Phase 0** (nginx + Kafka) and **Phase 1** (8 ingest pods, HPA 4–25).

---

## Revert

```bash
./deploy/local/revert-local.sh
```

Or only kind:

```bash
kind delete cluster --name identity-bridge
sudo sed -i '/ingest.local/d' /etc/hosts
```

---

## Verify

```bash
kubectl get pods -n identity-bridge
curl http://ingest.local/health/ready
```

See [README.md](README.md) for architecture diagram.
