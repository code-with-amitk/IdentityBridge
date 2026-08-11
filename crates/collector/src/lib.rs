//! On-premises Collector library.

pub mod ad;
pub mod config;
pub mod credentials;
pub mod http;
pub mod logging;
pub mod runtime;
pub mod service;
pub mod store;

pub use service as ColService;

use std::sync::Arc;

pub use config::{CollectorConfig, ConfigError, LdapFlavor};
pub use http::MiddlewareChainDoc;
pub use logging::{component, init_tracing};
pub use runtime::CollectorRuntime;

/// Collector library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Start the Collector (HTTP server + background tasks).
pub async fn run(config: Arc<CollectorConfig>) -> anyhow::Result<()> {
    //? operator is used for error propagation.
    //? after a function call, it unwraps the result if successful, 
    // or immediately returns the error to the calling function if it fails.
    let runtime = Arc::new(CollectorRuntime::new(config)?);

    // Run loop{} and read from AD and fill the data into the catalog_users table
    runtime.spawn_background_tasks();

    // Run the HTTP server and listen for requests on axum
    http::run_http_server(runtime).await
}
