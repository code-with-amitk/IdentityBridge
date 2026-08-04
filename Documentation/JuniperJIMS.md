
JIMS
- [Collect & store Identity Information](#collect)
  - [Collector](#Collector)
  - [Rust Server](#Server)
- [Firewall queries Server](#Firewall)
  - [IP Query](#ip)
  - [User Query](#User)
  - [Batch Query](#Batch)


## Juniper Identity Management Service (JIMS)
- It is an **identity aggregation and query service** that sits between enterprise identity sources (primarily Microsoft Active Directory) and Juniper enforcement points (SRX Series, vSRX, cSRX, NFX).
- It has on-premises **Collector** for AD/event ingestion, a cloud-hosted **Identity Server** in Rust backed by **PostgreSQL on AWS**

<a name=collect></a>

## Collect & store Identity Information

<a name=Collector></a>

### Collector

**User to groupname mapping**
```bash
// AD,DC,syslog/Azure
Domain: test.com
Username: alice
UPN: alice@test.com
Groups: ["Engineering", "VPN-Users"]
```

**User to IP mapping**
```bash
// DC Security logs (Login events)
IP address: 1.2.3.4
Username: TEST\alice
Logon type: `3` (network logon) or `2` (interactive)
Workstation: `DESKTOP-ABC`
Event: login (logout = Event 4634/4647)

// syslog
alice@test.com logged in from 2.3.4.5

// machine account
3.4.5.6 | `DESKTOP-XYZ.CORP.EXAMPLE.COM` | `Workstations`, `Engineering-Devices` |
```

<a name=Server></a>

### Rust Server
Create Session record from recieved identity information and store in DB
```json
{
  "ip": "1.2.3.4",
  "username": "alice",
  "domain": "test.com",
  "device": "DESKTOP-ABC",
  "groups": ["Engineering", "VPN-Users", "Domain Users"],
  "state": "active",
  "last_seen": "2026-07-27T10:15:00Z"
}
```

**Logout event** — IP/session marked inactive; entry ages out of firewall auth table after timeout.

<a name=firewall></a>

## Firewall queries Server
- Receives **identity bindings**; Junos policy engine applies allow/deny locally.
- The SRX/vSRX loads queried entries into its **authentication table** (`ip → user → groups/roles`). Security policies then match on user or group — e.g. permit `source-identity Engineering`, deny everyone else.

<a name=ip></a>

### IP  query (on-demand lookup)
- Firewall asks for `10.1.2.99` and Server has no session, Server asks Collector to re-read DC logs or PC-probe that IP

```json
GET https://jims-server/user_query/v1/ip/10.1.2.99
Authorization: Bearer <token>

Response: single entry same shape as below, or empty if unknown (triggering Collector backfill).
```

<a name=User></a>

### User query (individual user/device lookup)
```http
GET https://<server>/user?ip=10.1.2.50&id=alice&domain=CORP.EXAMPLE.COM
Authorization: Bearer <token>

// Response returns full user + group info for that IP/user/domain triple.
```

<a name=Batch></a>

### Batch sync (periodic sync)
- Normalized data is exported as four report types: **Domains**, **Groups**, **Users**, **Devices**, each with cross-references used by batch sync.

```json
GET https://jims-server/user-query/v2/users/endpoints?begintime=<unix-ts>&entry_count=500
Authorization: Bearer <token>

// Firewall repeats until no more records, then polls every `query-interval` seconds (default 5) for changes since last `begintime`.

{
  "report_entries": [
 {
      "ip-address": "10.1.2.50",
      "user-name": "alice",
      "domain-name": "CORP.EXAMPLE.COM",
      "roles": ["Engineering", "VPN-Users"],
      "device-name": "DESKTOP-ABC",
      "timestamp": 1722070500,
      "activity": "login"
    },
    {
      "ip-address": "10.1.2.75",
      "user-name": "bob",
      "domain-name": "CORP.EXAMPLE.COM",
      "roles": ["Finance", "Domain Users"],
      "device-name": "LAPTOP-FIN-01",
      "timestamp": 1722070512,
      "activity": "login"
    }
  ]
}
```

