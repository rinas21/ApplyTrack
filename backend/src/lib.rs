pub mod analytics;
pub mod applications;
pub mod auth;
pub mod common;
pub mod companies;
pub mod config;
pub mod database;
pub mod email;
pub mod extension;
pub mod jobs;
pub mod middleware;
pub mod notifications;
pub mod reminders;
pub mod subscriptions;
pub mod users;

use std::sync::Arc;

use axum::{
    http::{header, Method},
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use crate::{auth::jwt::JwtVerifier, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub jwt: JwtVerifier,
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(state.config.cors_origin.parse::<axum::http::HeaderValue>().unwrap_or_else(|_| axum::http::HeaderValue::from_static("http://localhost:4200")))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT, header::HeaderName::from_static("x-dev-sub"), header::HeaderName::from_static("x-dev-email")]);

    let api = Router::new()
        .merge(users::handlers::router())
        .nest("/companies", companies::handlers::router())
        .nest("/jobs", jobs::handlers::router())
        .nest("/applications", applications::handlers::router())
        .nest("/analytics", analytics::handlers::router())
        .nest("/reminders", reminders::handlers::router())
        .nest("/email", email::handlers::router())
        .nest("/extension", extension::handlers::router())
        .nest("/settings", users::handlers::settings_router());

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/health", get(health))
        .route("/api-docs/openapi.json", get(|| async { Json(openapi()) }))
        .nest("/api/v1", api)
        .layer(cors)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    #[derive(OpenApi)]
    #[openapi(paths(users::handlers::me), components(schemas(users::handlers::MeResponse, common::error::ErrorBody)))]
    struct ApiDoc;
    ApiDoc::openapi()
}
