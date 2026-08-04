//! Windows Service lifecycle (install / uninstall / run).

#[cfg(windows)]
mod windows_impl;

use std::sync::Arc;

use crate::config::CollectorConfig;

#[cfg(not(windows))]
use crate::logging::component;

/// Run the Collector — as a Windows Service on Windows, or as a foreground process elsewhere.
pub fn run(config: Arc<CollectorConfig>) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        windows_impl::run(config)
    }
    #[cfg(not(windows))]
    {
        run_console(config)
    }
}

#[cfg(not(windows))]
fn run_console(config: Arc<CollectorConfig>) -> anyhow::Result<()> {
    tracing::info!(
        target: component::SERVICE,
        "running in console mode (non-Windows — Windows Service wrapper skipped)"
    );

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::http::run_http_server(config))
}

#[cfg(windows)]
pub use windows_impl::{install_service, uninstall_service};

#[cfg(not(windows))]
pub fn install_service(_config_path: &str) -> anyhow::Result<()> {
    anyhow::bail!("service install is only supported on Windows")
}

#[cfg(not(windows))]
pub fn uninstall_service() -> anyhow::Result<()> {
    anyhow::bail!("service uninstall is only supported on Windows")
}
