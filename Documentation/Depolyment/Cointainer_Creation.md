## How

Ingest pods run the Go binary built from `server/Dockerfile`. Ingress-nginx and Kafka still use public images.

```
curl http://ingest.local/health/ready
        │
        ▼
ingress-nginx  (official controller image, port 80)
        │
        ▼
Service server-ingest :8080
        │
        ▼
2× pods  image=identity-bridge/server-ingest:latest
         (Go cmd/ingest)
```

Build locally:

```
./deploy/local/build-ingest-image.sh
```

## Nginx container (Ingress Controller only)

```
$ deploy-docker-desktop.sh
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.11.3/deploy/static/provider/cloud/deploy.yaml
```

## Ingest container

```
deploy/local/overlays/deployment.docker-desktop.yaml
containers:
  - name: server-ingest
    image: identity-bridge/server-ingest:latest
    imagePullPolicy: IfNotPresent
    ports:
      - name: http
        containerPort: 8080
```
