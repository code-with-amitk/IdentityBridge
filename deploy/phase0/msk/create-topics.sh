#!/usr/bin/env bash
# Create Identity Bridge MSK topics (Phase 0).
# Prerequisites: kafka-topics.sh on PATH, msk.env configured, network access to MSK.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/msk.env"

BOOTSTRAP="${MSK_BOOTSTRAP_BROKERS:?Set MSK_BOOTSTRAP_BROKERS in msk.env}"

create_topic() {
  local name="$1"
  local partitions="$2"
  local rf="${TOPIC_REPLICATION_FACTOR:-3}"

  echo "Creating topic ${name} (partitions=${partitions}, rf=${rf})..."
  kafka-topics.sh \
    --bootstrap-server "${BOOTSTRAP}" \
    --command-config "${SCRIPT_DIR}/client.properties" \
    --create \
    --if-not-exists \
    --topic "${name}" \
    --partitions "${partitions}" \
    --replication-factor "${rf}" \
    --config retention.ms=86400000 \
    --config compression.type=lz4
}

create_topic "${TOPIC_EVENTS}" "${TOPIC_EVENTS_PARTITIONS}"
create_topic "${TOPIC_CATALOG}" "${TOPIC_CATALOG_PARTITIONS}"
create_topic "${TOPIC_HEARTBEAT}" "${TOPIC_HEARTBEAT_PARTITIONS}"

echo "Topics:"
kafka-topics.sh \
  --bootstrap-server "${BOOTSTRAP}" \
  --command-config "${SCRIPT_DIR}/client.properties" \
  --list
