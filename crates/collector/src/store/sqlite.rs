//! Local SQLite — sync cursors and recent catalog rows (not full 10M catalog).

use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create store directory {}", parent.display()))?;
        }
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("open sqlite {}", path.as_ref().display()))?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().expect("sqlite lock");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sync_cursors (
                domain       TEXT PRIMARY KEY,
                usn_changed  TEXT,
                last_sync_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS catalog_users (
                domain       TEXT NOT NULL,
                username     TEXT NOT NULL,
                sid          TEXT NOT NULL,
                upn          TEXT,
                dn           TEXT,
                groups_json  TEXT NOT NULL,
                updated_at   TEXT NOT NULL,
                PRIMARY KEY (domain, username)
            );

            CREATE INDEX IF NOT EXISTS idx_catalog_users_domain_updated
                ON catalog_users(domain, updated_at DESC);
            "#,
        )?;
        Ok(())
    }

    pub fn get_usn_cursor(&self, domain: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().expect("sqlite lock");
        let mut stmt = conn.prepare("SELECT usn_changed FROM sync_cursors WHERE domain = ?1")?;
        let mut rows = stmt.query(params![domain])?;
        if let Some(row) = rows.next()? {
            let usn: Option<String> = row.get(0)?;
            Ok(usn)
        } else {
            Ok(None)
        }
    }

    // update usn_changed value in the sync_cursors table
    // Every domain has a unique usn_changed value
    // The usn_changed value is used to determine the last time the domain was synced.
    pub fn set_usn_cursor(&self, domain: &str, usn_changed: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().expect("sqlite lock");
        conn.execute(
            r#"
            INSERT INTO sync_cursors (domain, usn_changed, last_sync_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(domain) DO UPDATE SET
                usn_changed = excluded.usn_changed,
                last_sync_at = excluded.last_sync_at
            "#,
            params![domain, usn_changed, now],
        )?;
        Ok(())
    }

    pub fn touch_sync(&self, domain: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().expect("sqlite lock");
        conn.execute(
            r#"
            INSERT INTO sync_cursors (domain, usn_changed, last_sync_at)
            VALUES (?1, NULL, ?2)
            ON CONFLICT(domain) DO UPDATE SET last_sync_at = excluded.last_sync_at
            "#,
            params![domain, now],
        )?;
        Ok(())
    }

    // Insert information recieved from AD (domain, username, SID, UPN, DN, and groups)
    // into the catalog_users table
    pub fn upsert_catalog_user(
        &self,
        domain: &str,
        username: &str,
        sid: &str,
        upn: Option<&str>,
        dn: Option<&str>,
        groups: &[String],
    ) -> Result<()> {
        let groups_json = serde_json::to_string(groups)?;
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().expect("sqlite lock");
        conn.execute(
            r#"
            INSERT INTO catalog_users (domain, username, sid, upn, dn, groups_json, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(domain, username) DO UPDATE SET
                sid = excluded.sid,
                upn = excluded.upn,
                dn = excluded.dn,
                groups_json = excluded.groups_json,
                updated_at = excluded.updated_at
            "#,
            params![domain, username, sid, upn, dn, groups_json, now],
        )?;
        Ok(())
    }

    pub fn list_catalog_users(
        &self,
        domain: &str,
        prefix: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CatalogUserRow>> {
        let conn = self.conn.lock().expect("sqlite lock");
        if let Some(p) = prefix {
            let pattern = format!("{p}%");
            let mut stmt = conn.prepare(
                r#"
                SELECT username, sid, upn, dn, groups_json, updated_at
                FROM catalog_users
                WHERE domain = ?1 AND username LIKE ?2
                ORDER BY username
                LIMIT ?3 OFFSET ?4
                "#,
            )?;
            let mut rows = stmt.query(params![domain, pattern, limit, offset])?;
            return Self::read_user_rows(&mut rows);
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT username, sid, upn, dn, groups_json, updated_at
            FROM catalog_users
            WHERE domain = ?1
            ORDER BY username
            LIMIT ?2 OFFSET ?3
            "#,
        )?;
        let mut rows = stmt.query(params![domain, limit, offset])?;
        Self::read_user_rows(&mut rows)
    }

    fn read_user_rows(rows: &mut rusqlite::Rows<'_>) -> Result<Vec<CatalogUserRow>> {
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let groups_json: String = row.get(4)?;
            let groups: Vec<String> = serde_json::from_str(&groups_json).unwrap_or_default();
            out.push(CatalogUserRow {
                username: row.get(0)?,
                sid: row.get(1)?,
                upn: row.get(2)?,
                dn: row.get(3)?,
                groups,
                updated_at: row.get(5)?,
            });
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CatalogUserRow {
    pub username: String,
    pub sid: String,
    pub upn: Option<String>,
    pub dn: Option<String>,
    pub groups: Vec<String>,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = SqliteStore::open(&path).unwrap();
        assert!(store.get_usn_cursor("corp.local").unwrap().is_none());
        store.set_usn_cursor("corp.local", "999").unwrap();
        assert_eq!(
            store.get_usn_cursor("corp.local").unwrap().as_deref(),
            Some("999")
        );
    }
}
