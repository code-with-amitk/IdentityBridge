
JIMS
- [Collect & store Identity Information](#collect)
-- [Collector](#Collector)
-- [Rust Server](#Server)
- [Firewall queries Server](#Firewall)
-- [IP Query](#ip)
-- [User Query](#User)
-- [Batch Query](#Batch)


## Juniper Identity Management Service (JIMS)
- It is an **identity aggregation and query service** that sits between enterprise identity sources (primarily Microsoft Active Directory) and Juniper enforcement points (SRX Series, vSRX, cSRX, NFX).
- It has on-premises **Collector** for AD/event ingestion, a cloud-hosted **Identity Server** in Rust backed by **PostgreSQL on AWS**

<a name=collect></a>

## Collect & store Identity Information

<a name=Collector></a>

### Collector
Collects below information from AD,DC,syslog & sends to server

**username ↔ groups** // AD/Azure sync (no IP yet) 
```c
Domain: `CORP.EXAMPLE.COM`
Username: `alice`
UPN: `alice@corp.example.com`
Groups: `Engineering`, `VPN-Users`, `Domain Users`
```

**IP ↔ username**

- Src1: DC Security Log(Login event)
```json
IP address: `10.1.2.50`
Username `CORP\alice`
Logon type: `3` (network logon) or `2` (interactive)
Workstation: `DESKTOP-ABC`
Event: login (logout = Event 4634/4647)
```

- Src2: Syslog from VPN concentrator
```c
alice@corp.example.com logged in from 203.0.113.45
```

- Src3: Device-only session (machine account)
```
`10.1.2.75` | `DESKTOP-XYZ.CORP.EXAMPLE.COM` | `Workstations`, `Engineering-Devices` |
```

<a name=Server></a>

### Rust Server (Aggregate + DB Store)
Create Session record from recieved identity information and store in DB
```json
{
  "ip": "10.1.2.50",
  "username": "alice",
  "domain": "CORP.EXAMPLE.COM",
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

