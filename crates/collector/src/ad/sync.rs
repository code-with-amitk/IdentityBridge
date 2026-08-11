//! Scheduled LDAP sync and manual trigger.
//! Periodically synchronize users from Active Directory, but also allow somebody to trigger synchronization immediately

use std::sync::Arc;

use anyhow::Result;
use common::CatalogEvent;
use ldap3::SearchEntry;
use tokio::sync::Notify;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{error, info, warn};

use crate::config::CollectorConfig;
use crate::store::SqliteStore;

use super::client::{warn_if_ldaps_mismatch, LdapClient};
use super::mapper::{map_user_entry, max_usn_from_entries};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncReport {
    pub ok: bool,
    pub users_synced: usize,
    pub incremental: bool,
    pub new_cursor: Option<String>,
    pub message: Option<String>,
}

pub struct LdapSyncService {
    config: Arc<CollectorConfig>,
    store: Arc<SqliteStore>,
    ldap: LdapClient,
    manual: Notify,
}

impl LdapSyncService {
    pub fn new(config: Arc<CollectorConfig>, store: Arc<SqliteStore>) -> Self {
        warn_if_ldaps_mismatch(&config.ad);
        Self {
            ldap: LdapClient::new(&config),
            config,
            store,
            manual: Notify::new(),
        }
    }

    // Function to call from outside to trigger a sync
    pub fn trigger(&self) {
        self.manual.notify_one();
    }

    pub async fn run_scheduler(self: Arc<Self>) {
        if !self.config.ad.enabled {
            info!(target: "collector::ad::sync", "AD sync disabled in config");
            return;
        }

        let period = Duration::from_secs(self.config.ad.sync_interval_secs);
        let mut ticker = interval(period);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {

            // https://code-with-amitk.github.io/Async_Programming
            tokio::select! {
                // Ldap sync at configured interval
                _ = ticker.tick() => {
                    // Run the sync
                    if let Err(e) = self.run_once().await {
                        error!(target: "collector::ad::sync", error = %e, "scheduled LDAP sync failed");
                    }
                }
                // Manual sync triggered by somebody. if someone asks for a sync, we do it immediately
                _ = self.manual.notified() => {
                    info!(target: "collector::ad::sync", "manual LDAP sync triggered");
                    if let Err(e) = self.run_once().await {
                        error!(target: "collector::ad::sync", error = %e, "manual LDAP sync failed");
                    }
                }
            }
        }
    }

    // Do ldap sync every time the scheduler runs this function
    pub async fn run_once(&self) -> Result<SyncReport> {
        if !self.config.ad.enabled {
            return Ok(SyncReport {
                ok: false,
                users_synced: 0,
                incremental: false,
                new_cursor: None,
                message: Some("ad.enabled is false".into()),
            });
        }

        let domain = &self.config.ad.domain;

        // Get the cursor from the store
        // If we're using USN changed, we get the cursor from the store
        // If we're not using USN changed, we don't have a cursor
        let cursor = if self.config.ad.use_usn_changed {
            self.store.get_usn_cursor(domain)?
        } else { None };
        let incremental = cursor.is_some();

        // First sync (no cursor in SQLite)
        //  All enabled user accounts under base_dn (e.g. DC=corp,DC=local).
        //  Full baseline — every matching user in that subtree, page by page (default 1000 per page).
        //  Disabled accounts are excluded (AD userAccountControl bit).
        //  Groups are not listed as separate objects here; group names come from each user's memberOf.
        // Later syncs (cursor exists, use_usn_changed: true)
        let entries = self
            .ldap
            .search_users_with_config_password(cursor.as_deref(), None)
            .await?;

        let events = self.map_and_persist(&entries)?;
        let new_cursor = max_usn_from_entries(&entries, self.config.ad.ldap_flavor);

        if let Some(ref usn) = new_cursor {
            self.store.set_usn_cursor(domain, usn)?;
        } else {
            self.store.touch_sync(domain)?;
        }

        info!(
            target: "collector::ad::sync",
            users = events.len(),
            incremental,
            ?new_cursor,
            "LDAP sync finished"
        );

        // Return the sync report
        Ok(SyncReport {
            ok: true,
            users_synced: events.len(),
            incremental,
            new_cursor,
            message: None,
        })
    }

    fn map_and_persist(&self, entries: &[SearchEntry]) -> Result<Vec<CatalogEvent>> {
        let mut events = Vec::with_capacity(entries.len());

        // Iterate over the entries and map them to catalog events
        // pub struct SearchEntry {
        //    pub dn: String,
        //    pub attrs: HashMap<String, Vec<String>>,
        //    pub bin_attrs: HashMap<String, Vec<Vec<u8>>>,
        // }
        for entry in entries {
            match map_user_entry(entry, &self.config.ad, &self.config.tenant_id) {
                Ok(ev) => {
                    let upn = ev.attributes.get("upn").and_then(|v| v.as_str());
                    let dn = ev.attributes.get("dn").and_then(|v| v.as_str());
                    self.store.upsert_catalog_user(
                        &ev.domain,
                        &ev.name,
                        &ev.sid,
                        upn,
                        dn,
                        &ev.groups,
                    )?;
                    events.push(ev);
                }
                Err(e) => {
                    warn!(
                        target: "collector::ad::sync",
                        dn = %entry.dn,
                        error = %e,
                        "skip LDAP entry"
                    );
                }
            }
        }
        Ok(events)
    }
}
