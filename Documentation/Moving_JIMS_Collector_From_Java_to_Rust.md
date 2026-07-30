- [IdentityBridge Collector (Java 21+)](#overview)
- [Design](#Design)
  - [Maven / Gradle module layout](#modules)
  - [Core stack](#core)
  - [Libraries](#Libraries)
  - [Domain model (shared DTOs)](#domain)
- [Code path — LDAP catalog sync](#codep)
  - [Class diagram (core packages)](#classdiagram)
  - [Sequence — incremental LDAP sync](#seqdiagram)
  - [Code path — Java UI receives data](#codep1)
- [Scale](#scale)
  - [Issues with 10M AD objects (Java-specific notes)](#scale-10m)
- [Configuration file](#config)
- [Drawbacks of the Java collector + JavaFX UI approach](#drawbacks)

<a name=overview></a>

## IdentityBridge Collector (Java 21+)

- Admin UI **JavaFX**. Local desktop UI — MFC replacement
- Cloud server **Rust** + PostgreSQL (unchanged) Ingest + firewall query API

<a name=Design></a>

## Design
Collector services and JavaFX UI run in **one process** 
```
┌──────────────────── Windows Server ────────────────────┐
│  identity-bridge-collector.jar (or jpackage .exe)       │
│                                                         │
│  ┌─────────────────┐      in-process calls              │
│  │ JavaFX UI       │◄──────────────────────────────┐   │
│  └─────────────────┘                               │   │
│                                                     │   │
│  ┌──────────────────────────────────────────────────┴─┐ │
│  │ CollectorCore (Spring or plain Java services)       │ │
│  │  AdLdapConnector │ EventLogConnector │ Normalizer  │ │
│  │  SessionRepository │ RustIngestClient               │ │
│  └──────────┬───────────────────────┬─────────────────┘ │
│             │ LDAP / RPC             │ HTTPS             │
│             ▼                        ▼                   │
│      Active Directory          Rust Ingest API           │
└──────────────────────────────────────────────────────────┘
```

<a name=modules></a>

### Maven / Gradle module layout

```
identity-bridge-java/
├── collector-ui/              # JavaFX application module
│   └── src/main/java/.../ui/
├── collector-core/            # AD, session, ingest (no UI deps)
│   └── src/main/java/.../core/
├── collector-api/             # DTOs + service interfaces shared by UI and core
│   └── src/main/java/.../api/
├── collector-app/             # Main: wires UI + core (Option A)
│   └── src/main/java/.../app/CollectorApplication.java
└── collector-service/         # Optional: headless Windows Service (Option B)
    └── src/main/java/.../service/CollectorServiceMain.java
```

<a name=core></a>

### Core stack
- Java **21**
- Build: Maven or gradle(for JavaFX)
- Logging: SLF4J + Logback
- Config: Spring `@ConfigurationProperties`. yaml on disk
- Json events sent to Rust
- Local DB: SQLite JDBC or H2. `org.xerial:sqlite-jdbc`, Session cache, outbound queue, cursors 
- Thread Pool using  java.util.concurrent.ExecutorService
```
| `ad-ldap-sync` (1 thread) | Scheduled LDAP pages |
| `event-log` (1–2 threads) | Read helper pipe |
| `normalizer` (CPU cores) | Parse + merge |
| `rust-ingest` (2–4 threads) | HTTP push + retry |
| `JavaFX Application Thread` | UI updates only |
```
- AD: `com.unboundid:unboundid-ldapsdk`. Recommended — paged results, LDAPS, controls

<a name=Libraries></a>

### Libraries
- net.java.dev.jna:jna for event logging
- Netty: syslog reciever

<a name=domain></a>

### Domain model (shared DTOs)

Package: `collector-api`

```java
// Catalog sync — no IP
public record CatalogEvent(
    String eventId,
    String tenantId,
    Instant timestamp,
    String domain,
    UserCatalog user
) {}

public record UserCatalog(
    String username,
    String upn,
    String sid,
    List<String> groups
) {}

// Session — IP binding
public record SessionEvent(
    String eventId,
    String tenantId,
    Instant timestamp,
    Activity activity,  // LOGIN, LOGOUT
    String ipAddress,
    String username,
    String domain,
    String device,
    int logonType
) {}
```

<a name=codep></a>

## Code path — LDAP catalog sync

<a name=classdiagram></a>

### Class diagram (core packages)

```
AdSyncScheduler
    └── AdLdapConnector
            └── LdapClient (UnboundID)
    └── CatalogEventPublisher
            └── RustIngestClient

SessionNormalizer (lookup groups for user at login)
SessionRepository (SQLite)
```

<a name=seqdiagram></a>

### Sequence — incremental LDAP sync

```
AdSyncScheduler          AdLdapConnector       LdapClient        CatalogEventPublisher    RustIngestClient
      │                         │                   │                      │                      │
      │── tick / manual ───────►│                   │                      │                      │
      │                         │── bind LDAPS ────►│                      │                      │
      │                         │◄── OK ────────────│                      │                      │
      │                         │── paged search ──►│  (uSNChanged>=cursor)│                      │
      │                         │◄── page 1000 ─────│                      │                      │
      │                         │── map to CatalogEvent[]                   │                      │
      │                         │──────────────────────────────────────────►│                      │
      │                         │                   │                      │── POST /ingest/v1/catalog ──►
      │                         │                   │                      │◄── 202 ──────────────────────│
      │                         │── update USN cursor (SQLite)              │                      │
      │                         │── repeat pages ──►│                      │                      │
      │                         │                   │                      │                      │
      │── publish SyncCompletedEvent ──────────────────────────────────────►│ UI listens (Option A)  │
```

<a name=codep1></a>

### Code path — Java UI receives data. In-process (no HTTP between UI and core)

Uses **JavaFX properties + application events**:

```
CollectorCore
    │
    ├── SessionRepository
    │
    └── ApplicationEventPublisher (Spring) or custom EventBus
              │
              ▼
    DashboardViewModel / SessionsViewModel (JavaFX)
              │
              ▼
    TableView<SessionRecord> (JavaFX UI)
```

<a name=scale></a>

## Scale

<a name=scale-10m></a>

### Issues with 10M AD objects (Java-specific notes)

| Issue | Mitigation |
|---|---|
| Holding 10M objects in `HashMap` | **Stream pages**; catalog cache bounded or in SQLite with indexes only |
| Full GC pauses on large heaps | Keep heap **≤ 4–8 GB** on collector; do not load full catalog |
| LDAP sync blocks UI thread | All AD/Rust I/O on **background threads** / virtual threads; JavaFX `Platform.runLater` for UI updates only |
| SQLite JDBC single-writer | One writer thread for queue; readers for UI pagination |
| Event flood | Bounded `BlockingQueue` + helper push model |

<a name=config></a>

## Configuration file
`collector.yaml`:

```yaml
tenant_id: tenant-001
ad:
  domain: CORP.EXAMPLE.COM
  ldap_hosts: [dc1.corp.local, dc2.corp.local]
  ldap_port: 636
  use_ldaps: true
  bind_dn: CN=jims-svc,OU=Service Accounts,DC=corp,DC=local
  base_dn: DC=corp,DC=local
  sync_interval_minutes: 15
  page_size: 2000
event_log:
  helper_pipe: \\.\pipe\identity-bridge-events
rust:
  ingest_url: https://ingest.identity-bridge.example.com
  mtls_cert: C:\ProgramData\IdentityBridge\collector.p12
ui:
  mode: embedded  # or remote → localhost:9090
```

<a name=drawbacks></a>

## Drawbacks of the Java collector + JavaFX UI approach

| Drawback | Impact |
|---|---|
| 1. JVM required on every customer Windows Server | Install footprint (~150–300 MB runtime); ops must patch Java CVEs |
| 2. installer complexity. JavaFX native dependencies on Windows | Larger MSI than single Go `.exe`;  |
| 3. Memory footprint | JVM baseline **~256 MB–1 GB** vs Go **~20–50 MB** idle |
| 4. Startup time | JVM + Spring (if used) **3–15 s** vs Go **< 1 s** |
| 5. Windows Service packaging | procrun/winsw + JVM args — more moving parts than Go `svc` |
| 6. AD event log in pure Java | JNA path is ** brittle**; still need C# helper for JIMS parity — Java does not eliminate native code |
| 7. JavaFX ecosystem | Less common for new ops tools vs web UI; fewer developers than Go + HTML |
| 8. Remote admin / mobile | JavaFX is **desktop-only**; still need separate Flutter app hitting HTTP — duplicates UI stack |
| 9. Two UI technologies long-term | JavaFX (desktop) + Flutter (mobile) + Rust (cloud) = **3 stacks** vs Go HTML + Flutter |
| 10. 10M scale GC tuning | Requires careful heap limits and streaming; not impossible but ops-sensitive |
| 11. MFC parity cost | Recreating trees/tabs/dialogs in JavaFX is **high UI effort** vs HTML tables + htmx |
