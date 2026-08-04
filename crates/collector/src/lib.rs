//! On-premises Collector library.

pub mod config;
pub mod http;
pub mod logging;
pub mod service;
pub use service as ColService;

pub use config::{CollectorConfig, ConfigError};
pub use http::MiddlewareChainDoc;
pub use logging::{component, init_tracing};

/// Collector library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Start the Collector (HTTP server + background tasks).
pub async fn run(config: std::sync::Arc<CollectorConfig>) -> anyhow::Result<()> {
    http::run_http_server(config).await
}
