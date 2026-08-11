use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use tower_http::services::ServeDir;

use crate::ad::LdapClient;
use crate::http::middleware::{auth_middleware, layers, rate_limit_login};
use crate::runtime::CollectorRuntime;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<CollectorRuntime>,
}

#[derive(Debug, Deserialize)]
pub struct CatalogUsersQuery {
    pub q: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    50
}

/// Single router: HTML pages, static assets, and `/api/v1/*` on localhost.
pub fn router(state: Arc<AppState>) -> Router {
    let (trace, compression, concurrency) = layers();
    let static_dir = state.runtime.config.http.static_dir.clone();
    let config = state.runtime.config.clone();

    Router::new()
        .route("/healthz", get(healthz))
        .route("/", get(dashboard))
        .route("/login", get(login_page))
        .route("/api/v1/status", get(api_status))
        .route("/api/v1/config", get(api_config))
        .route("/api/v1/test/ad", post(api_test_ad))
        .route("/api/v1/sync/ad", post(api_sync_ad))
        .route("/api/v1/catalog/users", get(api_catalog_users))
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
    let cfg = &state.runtime.config;
    Json(serde_json::json!({
        "collector_id": cfg.collector_id,
        "tenant_id": cfg.tenant_id,
        "ad_enabled": cfg.ad.enabled,
        "ad_domain": cfg.ad.domain,
        "ldap_flavor": cfg.ad.ldap_flavor,
        "server_url": cfg.server.ingest_base_url,
        "status": "running",
    }))
}

async fn api_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.runtime.config.redacted())
}

async fn api_test_ad(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if !state.runtime.config.ad.enabled {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "message": "ad.enabled is false" })),
        );
    }

    let client = LdapClient::new(&state.runtime.config);
    match client.test_connection().await {
        Ok(result) => {
            let status = if result.ok {
                StatusCode::OK
            } else {
                StatusCode::BAD_GATEWAY
            };
            (status, Json(serde_json::to_value(result).unwrap()))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "message": e.to_string() })),
        ),
    }
}

async fn api_sync_ad(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.runtime.ldap_sync.run_once().await {
        Ok(report) => {
            let status = if report.ok {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(serde_json::to_value(report).unwrap()))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "message": e.to_string() })),
        ),
    }
}

async fn api_catalog_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CatalogUsersQuery>,
) -> impl IntoResponse {
    let domain = &state.runtime.config.ad.domain;
    match state.runtime.store.list_catalog_users(
        domain,
        query.q.as_deref(),
        query.limit.min(500),
        query.offset,
    ) {
        Ok(users) => Json(serde_json::json!({ "users": users, "count": users.len() })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
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
