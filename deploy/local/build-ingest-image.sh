#!/usr/bin/env bash
# Build the Go server-ingest image for local Kubernetes.
set -euo pipefail   #script exits if any command fails

# ${BASH_SOURCE[0]}: An array variable built into bash that holds the path of the script currently being executed
# /../.. move 2 steps back
# cd to 2 directories up from the current directory
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "Building docker image from server/Dockerfile and providing the tag identity-bridge/server-ingest:latest"
docker build "$ROOT/server" -t identity-bridge/server-ingest:latest

echo "Built identity-bridge/server-ingest:latest"
