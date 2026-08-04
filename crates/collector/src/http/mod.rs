//! Axum HTTP server — local web UI and admin REST API on one listener.

mod middleware;
mod routes;

use std::sync::Arc;

use tokio::net::TcpListener;

use crate::config::CollectorConfig;
use crate::logging::component;

pub use middleware::MiddlewareChainDoc;
pub use routes::AppState;

/// Run the local HTTP server until shutdown.
///
/// Serves HTML pages and `/api/v1/*` on `http.web_bind` (default `127.0.0.1:8080`).
/// A separate HTTPS listener for remote/mobile admin is **Phase 2** — see `docs/collector/HTTP.md`.
pub async fn run_http_server(config: Arc<CollectorConfig>) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        config: config.clone(),
    });
    let router = routes::router(state);
    let addr = config.http.bind;

    tracing::info!(
        target: component::HTTP,
        %addr,
        "starting Collector HTTP listener (localhost — HTML + /api/v1/*)"
    );

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!(target: component::HTTP, "shutdown signal received");
}
