

## Architecture
```
┌──────────────────────── Customer datacenter (on-premises) ────────────────────────┐
│                                                                                   │
│  ┌──────────────────────── Active Directory (separate infrastructure) ──────────┐ │
│  │  Domain Controllers          LDAP/LDAPS (636)    Security event logs         │ │
│  │  Users · Groups · Devices    MS-RPC / WEF        (4624 / 4634)               │ │
│  └───────────────────────────────▲───────────────────────────▲──────────────────┘ │
│                                  │ network                   │ network            │
│                                  │ LDAP/LDAPS                │ event log / syslog │
│                                  │                           │                    │
│  ┌───────────────────────────────┴───────────────────────────┴──────────────────┐ │
│  │  Dedicated Windows Server — Collector (Rust Windows Service: collector.exe)  │ │
│  │                                                                              │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │ │
│  │  │ AD LDAP      │  │ Event log    │  │ Session      │  │ Ingest client   │   │ │
│  │  │ connector    │─►│ connector    │─►│ normalizer   │─►│ (HTTPS outbound)│   │ │
│  │  │ (ldap3)      │  │ (+ helper)   │  │ + SQLite     │  └────────┬────────┘   │ │
│  │  └──────────────┘  └──────────────┘  └──────┬───────┘           │            │ │
│  └──────────────────────────────────────────────────────────────────────────────┘ │
│                                                                        │          │
│  Browser (RDP to collector server) ──► localhost:8080                  │          │
│  Flutter (corp Wi‑Fi / VPN) ──────────► collector:8443                 │          │
└────────────────────────────────────────────────────────────────────────┼──────────┘
                                                                         │sqlite.zip
                                                                         |
                                                         HTTPS ingest    │
                                                         /ingest/v1/*    ▼
┌──────────────────────────── AWS Cloud ─────────────────────────────────────────────────┐
│                                                                                        │
│  ┌──────────────────────── Rust Identity Server ────────────────────────────────────┐  │
│  │                                                                                  │  │
│  │  ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────────────┐   │  │
│  │  │ Ingest API      │      │ Query API       │      │ Session engine + cache  │   │  │
│  │  │ POST /ingest/*  │◄─────│ OAuth batch/IP/ │─────►│ (optional Redis)        │   │  │
│  │  │ ◄── Collector   │      │ user query      │      └───────────┬─────────────┘   │  │
│  │  └────────┬────────┘      └────────▲────────┘                  │                 │  │
│  │           │                         │                          ▼                 │  │
│  │           │                         │               ┌─────────────────────┐      │  │
│  │           └─────────────────────────┼──────────────►│ PostgreSQL (RDS)    │      │  │
│  │                                     │               └─────────────────────┘      │  │
│  └─────────────────────────────────────┼────────────────────────────────────────────┘  │
│                                        │ HTTPS OAuth2 + batch / IP / user query        │
│  ┌─────────────────────────────────────┴───────────────────────────────────────────┐   │
│  │  SRX / vSRX (enforcement point)                                                 │   │
│  │  Queries Server Query API only — builds local auth table — applies policy       │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                        │
│  (On-prem SRX may also query the same Server Query API over VPN / Direct Connect)      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

## Workspace structure

Monorepo with **collector** and **server** crates sharing types:

```
identity-bridge/
├── Cargo.toml                 # workspace
├── crates/
│   ├── ib-common/             # shared DTOs: CatalogEvent, SessionEvent, SessionRecord
│   ├── ib-collector/          # on-prem collector library
│   │   ├── src/
│   │   │   ├── ad/
│   │   │   │   ├── ldap.rs    # LDAP/LDAPS paged sync (ldap3)
│   │   │   │   └── eventlog.rs
│   │   │   ├── session/
│   │   │   │   ├── normalizer.rs
│   │   │   │   └── fsm.rs
│   │   │   ├── ingest/
│   │   │   │   └── client.rs  # POST /ingest/v1/* to server
│   │   │   ├── store/
│   │   │   │   └── sqlite.rs  # rusqlite
│   │   │   ├── api/           # /api/v1/* for Flutter + htmx
│   │   │   ├── web/           # axum HTML routes + askama/maud templates
│   │   │   └── auth/
│   │   └── templates/         # HTML (see UI-design.md)
│   ├── ib-server/             # AWS identity server (ingest + query)
│   │   ├── src/
│   │   │   ├── api_ingest/
│   │   │   ├── api_query/
│   │   │   ├── core/
│   │   │   └── db/
│   └── ib-collector-bin/      # Windows Service main
│       └── src/main.rs
├── configs/
│   └── collector.example.yaml
└── web/static/                # CSS, JS for collector admin UI
```

**Collector libraries (recommended):**

| Concern | Crate |
|---|---|
| HTTP server | `axum` + `tower` + `tower-http` |
| LDAP | `ldap3` |
| JWT | `jsonwebtoken` |
| SQLite | `rusqlite` |
| Windows Service | `windows-service` |
| Config | `config` or `serde_yaml` |
| Templates | `askama` or `maud` |
| Async runtime | `tokio` |
| Logging / metrics | `tracing`, `tracing-subscriber` |