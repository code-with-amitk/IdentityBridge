# Identity Bridge

**Identity Bridge** follows the same concept as **[Juniper Identity Management Service (JIMS)](Docs/JuniperJIMS.md)** — a centralized identity collector and server that maps users, groups, and IP addresses from Active Directory (and other identity sources) so firewalls can enforce identity-based policy.

Compared to JIMS (C++ collector + MFC admin UI on Windows), Identity Bridge is built as:

| Component | Technology | Role |
|---|---|---|
| **Collector** | **Go** | Runs on customer premises (Windows Server); collects identity from AD and pushes to the server |
| **Server** | **Rust** | Cloud-hosted; stores identity in PostgreSQL and serves SRX / vSRX (batch, IP, user query) |
| **Admin UI** | **HTML** (browser) | Talks to the Go collector over local HTTP — replaces the JIMS MFC desktop UI |

**Terminology:** *JIMS Collector* and *Identity Bridge Collector* refer to the same role and are used interchangeably in this project.

## Documentation

- [Juniper JIMS overview](Docs/JuniperJIMS.md)
- **Earlier approach:** 
    - [Java-based collector (reference)](Docs/JIMS_Collector_Java.md)
