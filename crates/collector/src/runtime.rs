//! Collector runtime — HTTP server + background LDAP sync.

use std::sync::Arc;

use anyhow::Result;
use tokio::task::JoinHandle;

use crate::ad::LdapSyncService;
use crate::config::CollectorConfig;
use crate::store::SqliteStore;

pub struct CollectorRuntime {
    pub config: Arc<CollectorConfig>,
    pub store: Arc<SqliteStore>,
    pub ldap_sync: Arc<LdapSyncService>,
}

impl CollectorRuntime {
    pub fn new(config: Arc<CollectorConfig>) -> Result<Self> {
        let store = Arc::new(SqliteStore::open(&config.store.sqlite_path)?);
        let ldap_sync = Arc::new(LdapSyncService::new(config.clone(), store.clone()));
        Ok(Self {
            config,
            store,
            ldap_sync,
        })
    }

    pub fn spawn_background_tasks(self: &Arc<Self>) -> JoinHandle<()> {
        let sync = self.ldap_sync.clone();
        tokio::spawn(async move {
            sync.run_scheduler().await;
        })
    }
}
