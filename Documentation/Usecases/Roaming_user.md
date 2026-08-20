- [Roaming user authorization](#roaming-user-authorization)

### Roaming user authorization

Roaming user (logs in India, travels to US):

1. User authenticates in India AD → Collector (India) sends **login session event** → Server stores `(tenant, ip_india, user, groups)`.
2. User connects in US → new IP → DC (US) **4624 event** → Collector (US) sends login for `(tenant, ip_us, user, groups)`.
3. SRX in US queries Server by **US IP** → Query tier returns current session for that IP (Redis/replica).
4. Old India session ages out via **logout event**, **idle timeout**, or **session TTL** on consumer write.

Authorization on the firewall is always **local to the IP the SRX sees** — same JIMS model. Roaming does not require a special “roaming flag”; it requires **both AD sites feeding the same `tenant_id`** so whichever site sees the logon updates the shared session store.

If India and US use **separate tenants** (split deployment), a roaming user would **not** appear on the US SRX until US AD logs the user in — standard AD behavior, not Identity Bridge magic.
