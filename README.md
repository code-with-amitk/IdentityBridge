# Identity Bridge

**Identity Bridge** follows the same concept as **[Juniper Identity Management Service (JIMS)](Documentation/JuniperJIMS.md)** — a centralized identity collector and server that maps users, groups, and IP addresses from Active Directory (and other identity sources) so firewalls can enforce identity-based policy.

Compared to JIMS (C++ collector + MFC admin UI on Windows), Identity Bridge is built as:

| Component | Technology | Role |
|---|---|---|
| **Collector** | **Rust** | Runs on customer premises (Windows Server); collects identity from AD and pushes to the server |
| **Server** | **Go** | Cloud-hosted; stores identity in PostgreSQL and serves SRX / vSRX (batch, IP, user query) |
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
    - [Go module](server/README.md)
- Deployment
  - [Kubernets, nginx, kafka, Ingestion Tier](./Documentation/Depolyment/README.md)
  - [How Container are Creted](./Documentation/Depolyment/Cointainer_Creation.md)

![Architecture Diagram](https://www.plantuml.com/plantuml/svg///www.plantuml.com/plantuml/png/bLJ1RkCs4BtpAmQvj4NXn9HRe0wAOgFi8atZfbhPG7hWXIjD7CBIK2MfROnYWPxw0UqVxfUKf1dPyIP6ejDmvhr7wd4uRwoJnlLjmngyTU0q6BMySr0hWvLJcfXTAUgaaIrqibh99SxTka48PKdB1Xdx2jxTEpGa6wXqssb2SfOD03WwrqtZLzhm8v0MplcJnMnJp7QBRw_dWahTEuJl9x7kCxk0YqP_yFlESW_3fT8K5n5vCSlP8otfP8Naq1p1NmpyjrCyAoxXuaOGBHNT2rhCvk21hI8PJ8Xw7d9niyzxDyjFpjA6ipnMzo6NDP9JRfqRLIyHvcW-Re-mZwKbsh0ZB1GQ-7eoucFeT6s_UJZ2bhVFNVrW1Na1WzO-U70rov-35xe6xlzKhvI6iYUdMXHq9MzN4WgZwxv24pKAVaWYUv54Tb1YF7wHoFKWycc8Xg2MJdwGraj6gg1We211XJHE1KkjZyTGS7-Qu-FZOZgDggk0_bBkKOEIxLiqJr_9ZJOhDFRvQza08DajwFC__q8n7XNJ3VsmkGE1UmPFb8DcJWPAvYbPeanUPbs9jGfDLC3x7iqwSat87JJeRRwnvWJERkPmrXjjvEQjvel1b_sjVfNMeJafZ6gbMGkoTzBKtg3lleL5djW7gOcI9In_2NYImDapu19b3ILsZQgaJHmUEMNv2agMAFmLd4dYPDkREcXwl20_3oBjlG-U0itBtsFTcdksaVmKnhNlQDfCTylEEOn6FuMn2gTmfwqZsIBpovZsrLEuB5Y2XPdg4xbqrHtMyClyjsjsdZrNyQbOYeFJ5tzUB8evYwkO3uoOP0aOfjLLYWk-3RDjUCs2G8pZo_KA2zsfofuAKcdGZl2Z3woWIZhPwEMmPf4IoUdPUdhryBoNHlSTOwaIKR7tTp22_ZJA8JWjg8sDnJQGQ67u7MdNL7byjtvKuOzM6MCU7EAoz9n0sIRuDlGu3_1nl62FZ8rpQtGhQBU7mfDNj-cqhR9M41z__lk_oHWiDD8XLMiM9NP_weyUpUufqnu6RQV9eY3efbFu4VWOMii6VcocDqXrtpR_0m00)