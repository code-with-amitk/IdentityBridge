use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Directory catalog change from LDAP sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEvent {
    pub event_id: Uuid,
    pub tenant_id: String,
    pub object_type: CatalogObjectType,
    pub sid: String,
    pub domain: String,
    pub name: String,
    pub groups: Vec<String>,
    pub attributes: serde_json::Value,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogObjectType {
    User,
    Group,
    Device,
}

/// Login/logout or session update from event log or syslog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub event_id: Uuid,
    pub tenant_id: String,
    pub event_type: SessionEventType,
    pub domain: String,
    pub username: String,
    pub ip_address: String,
    pub device: Option<String>,
    pub logon_type: Option<u32>,
    pub groups: Vec<String>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionEventType {
    Login,
    Logout,
    Refresh,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Active,
    LoggedOut,
    Pending,
    Expired,
}
