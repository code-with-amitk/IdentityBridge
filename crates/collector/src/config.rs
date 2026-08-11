//! Collector configuration loaded from YAML.

use std::net::SocketAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Top-level Collector configuration file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    pub tenant_id: String,
    pub collector_id: String,
    pub http: HttpConfig,
    pub auth: AuthConfig,
    pub ad: AdConfig,
    pub server: ServerConfig,
    pub store: StoreConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Local admin bind address — HTML pages and `/api/v1/*` (HTTP, localhost only).
    #[serde(default = "default_bind")]
    pub bind: SocketAddr,
    /// Static asset root for `/static/*`.
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// HS256 secret for admin JWTs. Override via env `COLLECTOR_JWT_SECRET` in production.
    pub jwt_secret: String,
    /// Require login for local web UI (`127.0.0.1:8080`).
    #[serde(default = "default_require_local_login")]
    pub require_local_login: bool,
    /// Optional AD LDAP bind for operator login (Phase 2).
    pub ad_bind_login: Option<AdBindLoginConfig>,
    /// Login rate limit: max attempts per minute per source IP.
    #[serde(default = "default_login_rate_limit")]
    pub login_rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdBindLoginConfig {
    pub enabled: bool,
    pub bind_dn: String,
    /// Reference to Windows Credential Manager entry or env var name.
    pub password_ref: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LdapFlavor {
    #[default]
    Ad,
    Openldap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdConfig {
    pub enabled: bool,
    pub domain: String,
    /// LDAP/LDAPS URIs, e.g. `ldaps://dc1.corp.local:636` or `ldap://127.0.0.1:389` (dev).
    pub ldap_uris: Vec<String>,
    pub base_dn: String,
    pub bind_dn: String,
    /// Reference to credential store key or env var (not plaintext in production).
    pub bind_password_ref: String,
    /// Full sync interval in seconds.
    #[serde(default = "default_ldap_sync_interval_secs")]
    pub sync_interval_secs: u64,
    /// LDAP page size for paged searches.
    #[serde(default = "default_ldap_page_size")]
    pub page_size: u32,
    /// Incremental sync via uSNChanged (AD) or modifyTimestamp (OpenLDAP).
    #[serde(default = "default_use_usn_changed")]
    pub use_usn_changed: bool,
    /// `ad` for Active Directory; `openldap` for local/docker test LDAP.
    #[serde(default)]
    pub ldap_flavor: LdapFlavor,
    /// Allow plain `ldap://` without TLS (local dev / docker OpenLDAP only).
    #[serde(default)]
    pub allow_insecure_ldap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server ingest base URL, e.g. `https://server.example.com`.
    pub ingest_base_url: String,
    /// Tenant-scoped API key for ingest (or use mTLS).
    pub api_key_ref: String,
    /// Optional client certificate for mTLS.
    pub mtls_cert_path: Option<String>,
    pub mtls_key_path: Option<String>,
    /// Heartbeat interval in seconds.
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    /// `text` or `json`.
    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse config: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("config validation failed: {0}")]
    Validation(String),
}

impl CollectorConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let config: Self = serde_yaml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.tenant_id.trim().is_empty() {
            return Err(ConfigError::Validation("tenant_id must not be empty".into()));
        }
        if self.collector_id.trim().is_empty() {
            return Err(ConfigError::Validation("collector_id must not be empty".into()));
        }
        if self.http.bind.ip().is_loopback() == false {
            return Err(ConfigError::Validation(
                "http.bind must be loopback (127.0.0.1) — admin UI must not be exposed on LAN in Phase 1"
                    .into(),
            ));
        }
        if self.ad.enabled && self.ad.ldap_uris.is_empty() {
            return Err(ConfigError::Validation(
                "ad.ldap_uris must not be empty when ad.enabled is true".into(),
            ));
        }
        if self.server.ingest_base_url.trim().is_empty() {
            return Err(ConfigError::Validation(
                "server.ingest_base_url must not be empty".into(),
            ));
        }
        Ok(())
    }

    /// Redacted view for `GET /api/v1/config`.
    pub fn redacted(&self) -> Self {
        let mut c = self.clone();
        c.auth.jwt_secret = "***".into();
        c.ad.bind_password_ref = "***".into();
        c.server.api_key_ref = "***".into();
        c
    }
}

fn default_sqlite_path() -> String {
    "data/collector.db".into()
}

fn default_bind() -> SocketAddr {
    "127.0.0.1:8080".parse().expect("valid default bind")
}

fn default_static_dir() -> String {
    "web/static".into()
}

fn default_require_local_login() -> bool {
    true
}

fn default_login_rate_limit() -> u32 {
    10
}

fn default_ldap_sync_interval_secs() -> u64 {
    3600
}

fn default_ldap_page_size() -> u32 {
    1000
}

fn default_use_usn_changed() -> bool {
    true
}

fn default_heartbeat_interval_secs() -> u64 {
    60
}

fn default_log_level() -> String {
    "info".into()
}

fn default_log_format() -> String {
    "text".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_example_config() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../configs/collector.example.yaml"
        );
        let config = CollectorConfig::from_file(path).expect("example config must load");
        assert_eq!(config.http.bind.port(), 8080);
    }
}
