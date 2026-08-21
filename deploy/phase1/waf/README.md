# Optional WAF on ingest ALB (Phase 1 edge)

WAF is **not required** for 50K req/s lab/staging. Enable for production edge protection.

---

## Associate WAF with ALB Ingress

1. Create a **Regional** AWS WAF Web ACL in the same region as EKS (`ap-south-1`).
2. Add rules as needed, for example:
   - **Rate-based rule** — limit requests per IP (tune for Collector egress IPs, not per-user)
   - **Geo match** — allow only countries where Collectors run (optional)
   - **AWSManagedRulesCommonRuleSet** — baseline OWASP protection
3. Copy Web ACL ARN and uncomment in [../ingest/ingress.yaml](../ingest/ingress.yaml):

```yaml
alb.ingress.kubernetes.io/wafv2-acl-arn: arn:aws:wafv2:ap-south-1:ACCOUNT_ID:regional/webacl/identity-bridge-ingest/ACL_ID
```

4. Re-apply Ingress:

```bash
kubectl apply -f deploy/phase1/ingest/ingress.yaml
```

The AWS Load Balancer Controller attaches the Web ACL to the ALB it creates.

---

## Collector IP allowlist (alternative)

If Collectors have fixed egress IPs, prefer **security group** rules on the ALB or a WAF **IP set** allowlist instead of open `0.0.0.0/0` on port 443.

---

## 50K req/s note

Rate limit WAF rules should be set above expected **aggregate** Collector traffic (e.g. 100K+ req/s ceiling with headroom), not per-firewall rates — Collectors share the ingest endpoint.
