# Active Directory — service account permissions


## Terms
### Update Sequence Number (USN)
- In Active Directory (AD), USN stands for Update Sequence Number. It is a 64-bit running number assigned by a domain controller (DC) to every single change made to its local database. As objects are created, deleted, or modified, the USN goes up. AD uses these numbers to track and sync updates cleanly

**Is USN per domain?**

- Yes, per AD domain (e.g. corp.local). Each domain controller has its own USN sequence for objects in that domain. We store the cursor per domain in SQLite (sync_cursors.domain = "corp.local").

### uSNChanged 
In Active Directory, uSNChanged (Update Sequence Number Changed) is a system-maintained attribute on every object that records the local USN assigned when the object was last modified or created. It is unique to each domain controller and is not replicated to other controllers

### Catalog 
Directory data from AD: users, groups, memberships — who exists in the directory

## Sync Data from AD store in sqlite
- The Collector creates file `data/collector.db` uses it for:
  - Bookmark where LDAP sync left off (sync_cursors)
  - Recent copy of users pulled from AD (catalog_users)
- It does not hold the full 10M-user catalog forever — that’s the Server’s job (PostgreSQL). SQLite is a local cache + sync state
- Imagine AD has:
```bash
Username	Groups	                uSNChanged
alice       Engineering, VPN-Users  500100
bob         Sales                   500050
carol       (disabled)              499900
```
After sync, `data/collector.db` might look like this conceptually:
```
Table catalog_users
domain	    username	sid	            groups_json	                    updated_at
corp.local  alice       S-1-5-21-…      ["Engineering","VPN-Users"]     2026-08-07T10:00:00Z
corp.local  bob         S-1-5-21-…      ["Sales"]                       2026-08-07T10:00:00Z

Table sync_cursors
domain	        usn_changed	        last_sync_at
corp.local      500100              2026-08-07T10:00:00Z
```

### sync_cursor
**get_usn_cursor(domain)**

Read the bookmark before sync.
```bash
First sync:  get_usn_cursor("corp.local") → None
             → full sync (all users)

Second sync: get_usn_cursor("corp.local") → Some("500100")
             → LDAP filter: uSNChanged >= 500100
             → only changed users
```

**set_usn_cursor(domain, usn)**

After a successful sync, save the new high-water mark.

```bash
Sync returns users with USNs 500100, 500105, 500110
→ set_usn_cursor("corp.local", "500110")
```

### Why SQLite instead of querying AD on every page view?
- Faster for the UI
- Doesn’t hammer the DC
- Matches design: paginated browse, not 10M rows from AD

Minimum **read-only** rights for the Collector LDAP bind account (Phase 1 catalog sync).

## Required LDAP read access

| Object / attribute | Purpose |
|---|---|
| Users under `base_dn` | Catalog sync (`sAMAccountName`, `userPrincipalName`, `objectSid`, `memberOf`, `distinguishedName`, `uSNChanged`) |
| Groups (via `memberOf` on user) | Group names for identity policy — resolved from user objects (no per-group N+1 queries in Phase 1) |

## Recommended AD groups / delegation

| Permission | Scope | Notes |
|---|---|---|
| **Read all user properties** | Domain or scoped OU | Delegation wizard → *Read all properties* on user objects |
| **List contents** | OUs containing users | Required for paged subtree search |
| **Replicating Directory Changes** (`DS-Replication-Get-Changes`) | **Not required** for LDAP uSNChanged read | Only needed for DC replication — do **not** grant unless misdocumented |
| **Domain Admin** | — | **Do not use** for Collector service account |

## uSNChanged incremental sync

- Store last `uSNChanged` in Collector SQLite (`sync_cursors`).
- Search filter: `(&(objectClass=user)(objectCategory=person)(!(userAccountControl:…:=2))(uSNChanged>=<cursor>))`.
- Disabled accounts excluded via `userAccountControl` bit 2 (ACCOUNTDISABLE).

## What this account must not have

- Write access to AD
- Password reset / unlock
- Event log read (separate account or helper for §13.3)

## Local testing without Active Directory

Use **OpenLDAP in Docker** — not a full AD emulator, but sufficient to test bind, search, paging, and catalog mapping:

- [LDAP_Test.md](LDAP_Test.md)
- Config: `configs/collector.openldap-dev.yaml`

For full AD semantics (SID, `uSNChanged`, `userAccountControl`), test against a lab domain controller or Azure AD Domain Services.
