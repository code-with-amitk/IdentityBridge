//! Collector binary — `collector.exe` on Windows.

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use collector::{init_tracing, ColService, CollectorConfig, VERSION};
use tracing::info;

#[derive(Parser)]
#[command(name = "collector", version = VERSION, about = "Identity Bridge Collector")]
struct Cli {
    /// Path to collector YAML config.
    #[arg(short, long, default_value = "configs/collector.yaml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in foreground (default on non-Windows).
    Run,
    /// Run as Windows Service (internal — use `install` first).
    #[cfg(windows)]
    Service,
    /// Install Windows Service.
    Install,
    /// Uninstall Windows Service.
    Uninstall,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => {
            let config_path = match cli.config.to_str() {
                Some(v) => v,
                None => "configs/collector.yaml",
            };
            ColService::install_service(config_path)?;
            return Ok(());
        }
        Some(Commands::Uninstall) => {
            ColService::uninstall_service()?;
            return Ok(());
        }
        #[cfg(windows)]
        Some(Commands::Service) => {
            let config = Arc::new(CollectorConfig::from_file(&cli.config)?);
            init_tracing(&config.logging)?;
            return ColService::run(config);
        }
        Some(Commands::Run) | None => {}
    }

    let config = Arc::new(CollectorConfig::from_file(&cli.config)?);

    // Parse log level(ex: info) and format(eg: text) from the config file
    init_tracing(&config.logging)?;

    info!(
        target: collector::component::SERVICE,
        collector_id = %config.collector_id,
        tenant_id = %config.tenant_id,
        bind = %config.http.bind,
        "Collector starting"
    );

    info!(
        target: collector::component::HTTP,
        doc = collector::MiddlewareChainDoc::CHAIN,
        "HTTP middleware chain"
    );

    // Call run function from crates/collector/src/lib.rs
    collector::run(config).await
}
