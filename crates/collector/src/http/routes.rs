use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use tower_http::services::ServeDir;

use crate::config::CollectorConfig;
use crate::http::middleware::{auth_middleware, layers, rate_limit_login};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<CollectorConfig>,
}

/// Single router: HTML pages, static assets, and `/api/v1/*` on localhost.
pub fn router(state: Arc<AppState>) -> Router {
    let (trace, compression, concurrency) = layers();
    let static_dir = state.config.http.static_dir.clone();
    let config = state.config.clone();

    Router::new()
        .route("/healthz", get(healthz))
        .route("/", get(dashboard))
        .route("/login", get(login_page))
        .route("/api/v1/status", get(api_status))
        .route("/api/v1/config", get(api_config))
        .route("/api/v1/auth/login", post(api_login_stub))
        .route("/api/v1/auth/logout", post(api_logout_stub))
        .nest_service("/static", ServeDir::new(static_dir))
        .with_state(state)
        .layer(axum::middleware::from_fn_with_state(config, rate_limit_login))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(concurrency)
        .layer(compression)
        .layer(trace)
}

async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn dashboard() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Collector</title></head>
<body>
  <h1>Collector</h1>
  <p>Dashboard placeholder — see <a href="/login">login</a>.</p>
</body>
</html>"#,
    )
}

async fn login_page() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Login — Collector</title></head>
<body>
  <h1>Login</h1>
  <p>Login form placeholder (§13.7).</p>
</body>
</html>"#,
    )
}

async fn api_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "collector_id": state.config.collector_id,
        "tenant_id": state.config.tenant_id,
        "ad_enabled": state.config.ad.enabled,
        "server_url": state.config.server.ingest_base_url,
        "status": "starting",
    }))
}

async fn api_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.config.redacted())
}

async fn api_login_stub() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({ "error": "login not implemented — see §13.6" })),
    )
}

async fn api_logout_stub() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
