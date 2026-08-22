# Identity Bridge

**Identity Bridge** follows the same concept as **[Juniper Identity Management Service (JIMS)](Documentation/JuniperJIMS.md)** — a centralized identity collector and server that maps users, groups, and IP addresses from Active Directory (and other identity sources) so firewalls can enforce identity-based policy.

Compared to JIMS (C++ collector + MFC admin UI on Windows), Identity Bridge is built as:

| Component | Technology | Role |
|---|---|---|
| **Collector** | **Rust** | Runs on customer premises (Windows Server); collects identity from AD and pushes to the server |
| **Server** | **Rust** | Cloud-hosted; stores identity in PostgreSQL and serves SRX / vSRX (batch, IP, user query) |
| **Admin UI** | **HTML** (browser) | Local HTTP on the Collector host — replaces the JIMS MFC desktop UI |

**Terminology:** **Collector** and **Server** are the two main components. *JIMS Collector* is the legacy name for the same on-prem role.

## Documentation

- [Juniper JIMS overview](Documentation/JuniperJIMS.md)
- [Scaling_1.5M_Requests_per_sec](Documentation/Scaling_1.5M_Requests_per_sec/README.md)
- Earlier approach: [Java-based collector (reference)](Documentation/Moving_JIMS_Collector_From_Java_to_Rust.md)
- Server & Collector
  - Collector
    - [Architecture](Documentation/Collector/Architecture.md)
    - [Build and install on windows](Documentation/Collector/Start_Collector.md)
  - Server
    - [Architecture](Documentation/Server/Architecture.md)
- Deployment
  - [Kubernets, nginx, kafka, Ingestion Tier](./Documentation/Depolyment/README.md)
  - [How Container are Creted](./Documentation/Depolyment/Cointainer_Creation.md)
