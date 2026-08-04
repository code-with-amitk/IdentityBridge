//! Structured logging with component tags (`target` = module path).

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::LoggingConfig;

/// Initialize global tracing from config and `RUST_LOG`.
pub fn init_tracing(logging: &LoggingConfig) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("collector={},common={},server={}", logging.level, logging.level, logging.level))
    });

    match logging.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json().with_target(true))
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(false)
                        .compact(),
                )
                .init();
        }
    }

    tracing::info!(
        target: "collector::logging",
        level = %logging.level,
        format = %logging.format,
        "tracing initialized"
    );

    Ok(())
}

/// Log with an explicit component tag (use as `target` in tracing macros).
pub mod component {
    pub const HTTP: &str = "collector::http";
    pub const SERVICE: &str = "collector::service";
    pub const CONFIG: &str = "collector::config";
}
