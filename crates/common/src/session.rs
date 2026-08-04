use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::SessionState;

/// Normalized session row after catalog merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub ip_address: String,
    pub username: String,
    pub domain: String,
    pub device: Option<String>,
    pub groups: Vec<String>,
    pub state: SessionState,
    pub last_seen: DateTime<Utc>,
    pub pushed_to_server: bool,
}
