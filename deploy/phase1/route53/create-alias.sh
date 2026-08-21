#!/usr/bin/env bash
# Create Route 53 A + AAAA alias records → ALB from server-ingest Ingress.
#
# Required env:
#   HOSTED_ZONE_ID  — Route 53 hosted zone for your domain
#   INGEST_HOST     — e.g. ingest.identity-bridge.example.com
#   ALB_HOSTNAME    — from: kubectl -n identity-bridge get ingress server-ingest -o jsonpath='{.status.loadBalancer.ingress[0].hostname}'
#
# ALB hosted zone IDs (alias targets):
#   ap-south-1: ZP97RAFLXTNZK
#   us-east-1:  Z35SXDOTRQ7X7K
set -euo pipefail

: "${HOSTED_ZONE_ID:?Set HOSTED_ZONE_ID}"
: "${INGEST_HOST:?Set INGEST_HOST}"
: "${ALB_HOSTNAME:?Set ALB_HOSTNAME}"

AWS_REGION="${AWS_REGION:-ap-south-1}"

# Default ALB canonical hosted zone ID for ap-south-1; override if different region
ALB_ZONE_ID="${ALB_CANONICAL_ZONE_ID:-ZP97RAFLXTNZK}"

TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT

cat > "$TMP" <<EOF
{
  "Comment": "Identity Bridge ingest alias",
  "Changes": [
    {
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "${INGEST_HOST}",
        "Type": "A",
        "AliasTarget": {
          "HostedZoneId": "${ALB_ZONE_ID}",
          "DNSName": "${ALB_HOSTNAME}",
          "EvaluateTargetHealth": true
        }
      }
    },
    {
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "${INGEST_HOST}",
        "Type": "AAAA",
        "AliasTarget": {
          "HostedZoneId": "${ALB_ZONE_ID}",
          "DNSName": "${ALB_HOSTNAME}",
          "EvaluateTargetHealth": true
        }
      }
    }
  ]
}
EOF

aws route53 change-resource-record-sets \
  --hosted-zone-id "${HOSTED_ZONE_ID}" \
  --change-batch "file://${TMP}"

echo "Route 53 alias created: ${INGEST_HOST} → ${ALB_HOSTNAME}"
