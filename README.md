# Identity Bridge

**Identity Bridge** follows the same concept as **[Juniper Identity Management Service (JIMS)](Documentation/JuniperJIMS.md)** — a centralized identity collector and server that maps users, groups, and IP addresses from Active Directory (and other identity sources) so firewalls can enforce identity-based policy.

Compared to JIMS (C++ collector + MFC admin UI on Windows), Identity Bridge is built as:

| Component | Technology | Role |
|---|---|---|
| **Collector** | **Rust** | Runs on customer premises (Windows Server); collects identity from AD and pushes to the server |
| **Server** | **Rust** | Cloud-hosted; stores identity in PostgreSQL and serves SRX / vSRX (batch, IP, user query) |
| **Admin UI** | **HTML** (browser) | Local HTTP on the Collector host — replaces the JIMS MFC desktop UI |

**Terminology:** **Collector** and **Server** are the two main components. *JIMS Collector* is the legacy name for the same on-prem role.

## Quick start (Collector)

```bash
cp configs/collector.example.yaml configs/collector.yaml
cargo run -p collector-bin
```

Open `http://127.0.0.1:8080/` on the Collector host (RDP/console).

See [Documentation/Collector/Running.md](Documentation/Collector/Running.md) for CLI flow, WSL vs Windows steps, and Windows Service install.

## Documentation

- [Juniper JIMS overview](Documentation/JuniperJIMS.md)
- **Earlier approach:** 
    - [Java-based collector (reference)](Documentation/Moving_JIMS_Collector_From_Java_to_Rust.md)
- Collector
    - [Architecture](Documentation/Collector/Architecture.md)
    - [Running](Documentation/Collector/Running.md)
- Server
    - [Architecture](Documentation/Server/Architecture.md)
