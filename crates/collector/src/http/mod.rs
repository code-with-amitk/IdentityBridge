//! Axum HTTP server — local web UI and admin REST API on one listener.

mod middleware;
mod routes;

use std::sync::Arc;

use tokio::net::TcpListener;

use crate::logging::component;
use crate::runtime::CollectorRuntime;

pub use middleware::MiddlewareChainDoc;
pub use routes::AppState;

/// Run the local HTTP server until shutdown.
pub async fn run_http_server(runtime: Arc<CollectorRuntime>) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        runtime: runtime.clone(),
    });
    let router = routes::router(state);
    let addr = runtime.config.http.bind;

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
