//! LDAP connection, bind, and paged user search.

use std::time::Instant;

use anyhow::{Context, Result};
use ldap3::adapters::PagedResults;
use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use tracing::{debug, info, warn};

use crate::config::{AdConfig, CollectorConfig};
use crate::credentials::resolve_env_secret;

use super::mapper::{map_user_entry, user_search_attrs, user_search_filter};

#[derive(Debug, serde::Serialize)]
pub struct LdapTestResult {
    pub ok: bool,
    pub uri: String,
    pub bind_dn: String,
    pub domain: String,
    pub users_found: usize,
    pub sample_users: Vec<SampleUser>,
    pub duration_ms: u128,
    pub message: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SampleUser {
    pub username: String,
    pub sid: String,
    pub groups: Vec<String>,
    pub dn: String,
}

pub struct LdapClient {
    config: AdConfig,
    tenant_id: String,
}

impl LdapClient {
    pub fn new(config: &CollectorConfig) -> Self {
        Self {
            config: config.ad.clone(),
            tenant_id: config.tenant_id.clone(),
        }
    }

    pub async fn test_connection(&self) -> Result<LdapTestResult> {
        let started = Instant::now();
        let uri = self
            .config
            .ldap_uris
            .first()
            .cloned()
            .context("ad.ldap_uris is empty")?;
        let password = resolve_env_secret(&self.config.bind_password_ref)?;

        let (entries, message) = match self.search_users(None, 5, &password).await {
            Ok(entries) => (entries, None),
            Err(e) => {
                return Ok(LdapTestResult {
                    ok: false,
                    uri: uri.clone(),
                    bind_dn: self.config.bind_dn.clone(),
                    domain: self.config.domain.clone(),
                    users_found: 0,
                    sample_users: vec![],
                    duration_ms: started.elapsed().as_millis(),
                    message: Some(e.to_string()),
                });
            }
        };

        let sample_users: Vec<SampleUser> = entries
            .iter()
            .filter_map(|e| {
                map_user_entry(e, &self.config, &self.tenant_id)
                    .ok()
                    .map(|ev| SampleUser {
                        username: ev.name.clone(),
                        sid: ev.sid.clone(),
                        groups: ev.groups.clone(),
                        dn: e.dn.clone(),
                    })
            })
            .collect();

        Ok(LdapTestResult {
            ok: true,
            uri,
            bind_dn: self.config.bind_dn.clone(),
            domain: self.config.domain.clone(),
            users_found: sample_users.len(),
            sample_users,
            duration_ms: started.elapsed().as_millis(),
            message,
        })
    }

    /// Paged user search; `max_pages` limits pages (None = all).
    pub async fn search_users(
        &self,
        usn_cursor: Option<&str>,
        max_pages: impl Into<Option<u32>>,
        password: &str,
    ) -> Result<Vec<SearchEntry>> {
        let uri = self
            .config
            .ldap_uris
            .first()
            .context("ad.ldap_uris is empty")?;

        let mut settings = LdapConnSettings::new();
        if self.config.allow_insecure_ldap {
            settings = settings.set_no_tls_verify(true);
        }

        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, uri)
            .await
            .with_context(|| format!("connect LDAP {uri}"))?;
        ldap3::drive!(conn);

        ldap.simple_bind(&self.config.bind_dn, password)
            .await
            .context("LDAP bind")?
            .success()
            .context("LDAP bind rejected")?;

        let filter = user_search_filter(&self.config, usn_cursor);
        let attrs = user_search_attrs(self.config.ldap_flavor);
        debug!(target: "collector::ad::ldap", %filter, "LDAP search");

        let max_pages = max_pages.into();
        let mut search = ldap
            .streaming_search_with(
                PagedResults::new(self.config.page_size as i32),
                &self.config.base_dn,
                Scope::Subtree,
                &filter,
                attrs,
            )
            .await
            .context("start paged LDAP search")?;

        let mut entries = Vec::new();
        let mut pages = 0u32;
        while let Some(entry) = search.next().await.context("LDAP search page")? {
            entries.push(SearchEntry::construct(entry));
            if entries.len() % (self.config.page_size as usize) == 0 {
                pages += 1;
                if max_pages.is_some_and(|m| pages >= m) {
                    break;
                }
            }
        }

        info!(
            target: "collector::ad::ldap",
            count = entries.len(),
            pages,
            "LDAP search complete"
        );

        Ok(entries)
    }

    pub async fn search_users_with_config_password(
        &self,
        usn_cursor: Option<&str>,
        max_pages: impl Into<Option<u32>>,
    ) -> Result<Vec<SearchEntry>> {
        let password = resolve_env_secret(&self.config.bind_password_ref)?;
        self.search_users(usn_cursor, max_pages, &password).await
    }
}

pub fn warn_if_ldaps_mismatch(config: &AdConfig) {
    if config.allow_insecure_ldap {
        warn!(
            target: "collector::ad::ldap",
            "ad.allow_insecure_ldap=true — for local OpenLDAP/dev only"
        );
    }
}
