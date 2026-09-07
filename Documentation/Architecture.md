

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
│  ┌──────────────────────── Go Identity Server ───────────────────────────────────┐  │
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

Monorepo: **Collector** is Rust; **Server** is Go. JSON types are duplicated on purpose (`crates/common` ↔ `server/pkg/types`) so the Collector does not depend on the server binary.

```
identity-bridge/
├── Cargo.toml                 # collector workspace
├── crates/
│   ├── common/                # Collector DTOs (JSON contract with Go server)
│   ├── collector/             # on-prem collector library
│   └── collector-bin/         # Windows Service main
├── server/                    # Go AWS identity server
│   ├── cmd/ingest/
│   ├── cmd/query/
│   ├── cmd/consumer-session/
│   ├── cmd/consumer-catalog/
│   └── pkg/types/
├── configs/
│   └── collector.example.yaml
└── web/static/
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