//! Health check and status endpoints.

use crate::AppState;
use axum::{Json, extract::State};
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
    demo_mode: bool,
}

/// Simple health check endpoint.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Detailed status endpoint.
pub async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        database: "connected",
        scheduler: "running",
        demo_mode: state.demo_mode,
    })
}
