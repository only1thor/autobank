//! Health check and status endpoints.

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
pub struct StatusResponse {
    status: &'static str,
    version: &'static str,
    database: &'static str,
    scheduler: &'static str,
}

/// Simple health check endpoint.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Detailed status endpoint.
pub async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        database: "connected",
        scheduler: "running",
    })
}
