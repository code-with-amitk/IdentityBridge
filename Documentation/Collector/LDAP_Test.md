# Local LDAP test server (OpenLDAP)

Use this to test **§13.2 LDAP** without a real Active Directory domain. OpenLDAP supports bind, paged search, and user/group LDIF — not full AD attributes (`objectSid`, `uSNChanged`).

## On this page

- [Start OpenLDAP](#start-openldap)
- [Run Collector](#configure-collector)
- [Test connect and sync](#test-connect-and-sync)

---

## Start OpenLDAP

Remove any failed container first, then start clean:

```bash
cd docker/openldap
docker compose down
sudo rm -rf bootstrap/50-users.ldif
docker rm -f ib-test-ldap 2>/dev/null || true

docker compose up -d
docker compose logs -f openldap
```

### Perform Ldap search
```bash
// -w password
$ LDAPTLS_REQCERT=never ldapsearch -x -H ldap://127.0.0.1:1389 \
  -D "cn=admin,dc=example,dc=local" -w admin \
  -b "dc=example,dc=local" "(uid=alice)" uid cn mail

# alice, users, example.local
dn: uid=alice,ou=users,dc=example,dc=local
cn: Alice Example
uid: alice
mail: alice@example.local

# search result
search: 2
result: 0 Success

# numResponses: 2
# numEntries: 1
```

## Run Collector

```bash
export LDAP_BIND_PASSWORD=admin
export COLLECTOR_SERVER_API_KEY=dev

cp configs/collector.openldap-dev.yaml configs/collector.yaml
cargo run -p collector-bin
```

## Test connect and sync

```bash
# LDAP bind + sample users (5 max)
curl -s -X POST http://127.0.0.1:8080/api/v1/test/ad | jq .

# Full sync into SQLite
curl -s -X POST http://127.0.0.1:8080/api/v1/sync/ad | jq .

# Browse synced users
curl -s "http://127.0.0.1:8080/api/v1/catalog/users?limit=10" | jq .
```

Expected `test/ad` response (abbreviated):

```json
{
  "ok": true,
  "uri": "ldap://127.0.0.1:1389",
  "users_found": 3,
  "sample_users": [
    { "username": "alice", "groups": [], "dn": "uid=alice,ou=users,dc=example,dc=local" }
  ]
}
```

Groups appear when `memberOf` is present on entries (OpenLDAP `groupOfNames` uses `member` on the group — Phase 1 dev tree may show empty groups until memberof overlay is added; user sync still validates bind + search).

---

## Stop, Delete

```bash
cd docker/openldap
docker compose down
```