//! Middleware chain for the Collector HTTP router.
//!
//! See `docs/collector/HTTP.md` for the full documented chain.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::config::CollectorConfig;
use crate::logging::component;

/// Human-readable documentation of the middleware chain (design §13.1).
pub struct MiddlewareChainDoc;

impl MiddlewareChainDoc {
    pub const CHAIN: &'static str = r"
Collector HTTP router (`127.0.0.1:8080`, HTTP — localhost only):
  1. TraceLayer           — request/response logging (tower-http)
  2. CompressionLayer      — gzip responses
  3. ConcurrencyLimitLayer — max 256 concurrent requests
  4. rate_limit_login      — bucket for POST /api/v1/auth/login (§13.6)
  5. auth_middleware       — JWT validation for protected routes (§13.6)
  6. (future) session auth for protected HTML pages (§13.7)

Phase 2: separate HTTPS listener on LAN for mobile admin — not implemented yet.
";
}

pub fn layers() -> (
    TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>,
    CompressionLayer,
    ConcurrencyLimitLayer,
) {
    (
        TraceLayer::new_for_http(),
        CompressionLayer::new(),
        ConcurrencyLimitLayer::new(256),
    )
}

/// Placeholder JWT auth — passes through; full implementation in §13.6.
pub async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    if path.starts_with("/api/v1/auth/") || path == "/healthz" {
        return Ok(next.run(request).await);
    }

    if path.starts_with("/api/v1/") {
        tracing::debug!(
            target: component::HTTP,
            path = %path,
            "auth middleware: JWT validation not yet enforced (§13.6)"
        );
    }

    Ok(next.run(request).await)
}

/// Rate limit login attempts per source IP (in-memory; full enforcement in §13.6).
pub async fn rate_limit_login(
    State(config): State<Arc<CollectorConfig>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().path() == "/api/v1/auth/login" {
        let limit = config.auth.login_rate_limit_per_minute;
        tracing::debug!(
            target: component::HTTP,
            limit,
            "login rate limit configured (enforcement in §13.6)"
        );
        let _ = Duration::from_secs(60);
    }
    Ok(next.run(request).await)
}
