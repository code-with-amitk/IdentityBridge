#!/usr/bin/env bash
# Create Kafka topics on local Redpanda (50K partition counts).
set -euo pipefail

BROKER="${KAFKA_BROKER:-kafka.identity-bridge.svc.cluster.local:9092}"

create() {
  local topic=$1 parts=$2
  echo "Creating topic: $topic ($parts partitions)"
  kubectl exec -n identity-bridge deploy/kafka -- \
    rpk topic create "$topic" -p "$parts" --brokers "$BROKER" 2>/dev/null || \
  kubectl exec -n identity-bridge deploy/kafka -- \
    rpk topic add-partitions "$topic" -n "$parts" --brokers "$BROKER" 2>/dev/null || true
  echo "Topic: $topic ($parts partitions)"
}

kubectl wait -n identity-bridge --for=condition=ready pod -l app=kafka --timeout=120s

create identity-events 24
create identity-catalog 8
create identity-heartbeat 4

kubectl exec -n identity-bridge deploy/kafka -- rpk topic list --brokers "$BROKER"
