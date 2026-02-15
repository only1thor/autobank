//! Rule execution history API endpoints.

use crate::AppState;
use crate::rules::RuleExecution;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Creates the executions router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_executions))
        .route("/{id}", get(get_execution))
}

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

#[derive(Deserialize)]
pub struct ListExecutionsQuery {
    /// Maximum number of executions to return (default: 100)
    pub limit: Option<i64>,
}

/// List recent executions across all rules.
pub async fn list_executions(
    State(state): State<AppState>,
    Query(query): Query<ListExecutionsQuery>,
) -> Result<Json<Vec<RuleExecution>>, Json<ApiError>> {
    let limit = query.limit.unwrap_or(100);
    state
        .db
        .list_executions(limit)
        .await
        .map(Json)
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}

/// Get a single execution by ID.
pub async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RuleExecution>, Json<ApiError>> {
    state
        .db
        .get_execution(&id.to_string())
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?
        .map(Json)
        .ok_or_else(|| Json(ApiError { error: "Execution not found".to_string() }))
}

/// Get executions for a specific rule.
pub async fn get_rule_executions(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<Vec<RuleExecution>>, Json<ApiError>> {
    state
        .db
        .get_rule_executions(&rule_id.to_string())
        .await
        .map(Json)
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}
