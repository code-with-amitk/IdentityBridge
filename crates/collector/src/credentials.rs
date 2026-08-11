//! Resolve secrets from environment variables (Phase 1).

use anyhow::{Context, Result};

/// Read bind password / API key from an env var named in config (`bind_password_ref`).
pub fn resolve_env_secret(ref_name: &str) -> Result<String> {
    std::env::var(ref_name).with_context(|| {
        format!("environment variable `{ref_name}` is not set (see config bind_password_ref)")
    })
}
