

# Collector
## Tasks of Collector
- Event log connector, Syslog receiver
- **Session normalizer** Merge catalog + login events; session FSM 
- **Ingest client** Push catalog/session events to Server Ingest API
- **HTML admin UI** Browser pages (Dashboard, Sessions, AD config, …)
- **Auth** Operator login for admin UI and API 

### 1. LDAP sync flow

> At **10M+ objects**, use **incremental sync only** in steady state.

```
1. Scheduler tick (default 15 min) or POST /api/v1/sync/ad
2. ldap.Bind(serviceAccount) over LDAPS
3. If cursor exists: search (uSNChanged>=cursor) — else paged full baseline (off-peak)
4. Paged search: (&(objectClass=user)(!(userAccountControl:1.2.840.113556.1.4.803:=2)))
5. Stream each page → CatalogEvent batch → Rust /ingest/v1/catalog (no full RAM buffer)
6. Update sync cursor in SQLite after each successful page
7. Admin UI: GET /api/v1/catalog/users?page=&q= (search/pagination only — never full 10M)
```

### 2. Event log sync

> At scale, use **real-time tail / WEF push**, not full log scans.

```
1. Helper tails DC Security log (EvtSubscribe / WEF) — push to Rust `tokio::mpsc` channel
2. Filter Event ID 4624 (login), 4634/4647 (logout)
3. Extract: IpAddress, TargetUserName, WorkstationName, LogonType
4. Dedup → SessionEvent → normalizer → bounded SQLite (active sessions)
5. Micro-batch POST Rust /ingest/v1/events (50–200 ms or 500 events)
6. Expose via GET /api/v1/sessions (paginated)
```

### 3. Session normalization (`session::normalizer`)
```
Catalog (alice → [Engineering, VPN-Users])
    +
Login event (10.1.2.50 → alice)
    ↓
SessionRecord {
  ip, username, domain, device, groups[], state, last_seen, pushed_to_rust
}
```

## Scale Design

### Event Logs
- Event logs are the **throughput hot path**. Design for **push/stream**, not “poll entire Security log every N seconds.”
- Windows Event Forwarding (WEF) or helper **`EvtSubscribe`** — avoid full log scans 
- Flush to Rust every **50–200 ms** OR **500 events**, whichever first
- **Dedup** Key: `(dc, record_id)` or `(ip, username, timestamp, event_id)` — drop duplicates within **60s** window
- 
```
DC Security Log (4624/4634)
        │  WEF subscription OR helper tail (real-time)
        ▼
┌──────────────────┐    bounded     ┌──────────────────┐    micro-batch    ┌─────────────┐
│ Event log helper │─── channel ──►│ Rust normalizer  │─── 100–500 ev / 50ms ──►│ Server ingest │
│ (per DC)         │    (100K cap)  │ + dedup          │    gzip JSON           │ client        │
└──────────────────┘                └────────┬─────────┘                        └─────────────┘
                                           ▼
                                     Session SQLite
                                     (active sessions only,
                                      TTL + max rows cap)
```


## Collector Challenges
### Event log collection with Pure Rust 
MS-RPC to Windows event log is difficult. Phase 1 options:

1. **Windows helper** (small C# or C++ service) writes events to a local named pipe / file; Rust reads them.
2. **WEF subscription** forwarding to Rust syslog listener.
3. **LDAP-only MVP** — catalog sync first; IP sessions from syslog (Phase 1b).


## Collector → Server push design
- Auth mTLS client cert **or** Bearer collector API key + `X-Tenant-Id`
- Format JSON array of events converted to sqlite.gzip
- Idempotency `event_id` UUID per event
- Retry Exponential backoff; persist failed batches in SQLite queue
