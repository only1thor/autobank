//! System API endpoints for scheduler control and status.

use crate::AppState;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::Serialize;

/// Creates the system router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/poll", post(trigger_poll))
        .route("/scheduler/enable", post(enable_scheduler))
        .route("/scheduler/disable", post(disable_scheduler))
}

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

#[derive(Serialize)]
pub struct SystemStatus {
    pub scheduler_enabled: bool,
    pub rules_count: i64,
    pub executions_count: i64,
}

#[derive(Serialize)]
pub struct PollResponse {
    pub message: String,
}

/// Get system status.
pub async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<SystemStatus>, Json<ApiError>> {
    let rules = state.db.list_rules().await.map_err(|e| Json(ApiError { error: e.to_string() }))?;
    let executions = state.db.list_executions(1000).await.map_err(|e| Json(ApiError { error: e.to_string() }))?;
    
    Ok(Json(SystemStatus {
        scheduler_enabled: state.scheduler.is_enabled().await,
        rules_count: rules.len() as i64,
        executions_count: executions.len() as i64,
    }))
}

/// Trigger an immediate poll cycle.
pub async fn trigger_poll(
    State(state): State<AppState>,
) -> Json<PollResponse> {
    state.scheduler.trigger_poll().await;
    Json(PollResponse {
        message: "Poll triggered".to_string(),
    })
}

/// Enable the scheduler.
pub async fn enable_scheduler(
    State(state): State<AppState>,
) -> Json<PollResponse> {
    state.scheduler.enable().await;
    Json(PollResponse {
        message: "Scheduler enabled".to_string(),
    })
}

/// Disable the scheduler.
pub async fn disable_scheduler(
    State(state): State<AppState>,
) -> Json<PollResponse> {
    state.scheduler.disable().await;
    Json(PollResponse {
        message: "Scheduler disabled".to_string(),
    })
}
