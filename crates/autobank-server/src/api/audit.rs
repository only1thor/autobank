//! Audit log API endpoints.

use crate::AppState;
use crate::audit::AuditEntry;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Creates the audit router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_audit_entries))
        .route("/{id}", get(get_audit_entry))
}

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

#[derive(Deserialize)]
pub struct ListAuditQuery {
    /// Maximum number of entries to return (default: 100)
    pub limit: Option<i64>,
    /// Filter by event type
    pub event_type: Option<String>,
}

/// List recent audit log entries.
pub async fn list_audit_entries(
    State(state): State<AppState>,
    Query(query): Query<ListAuditQuery>,
) -> Result<Json<Vec<AuditEntry>>, Json<ApiError>> {
    let limit = query.limit.unwrap_or(100);
    // TODO: Add event_type filtering when needed
    state
        .db
        .query_audit(limit)
        .await
        .map(Json)
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}

/// Get a single audit entry by ID.
pub async fn get_audit_entry(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditEntry>, Json<ApiError>> {
    state
        .db
        .get_audit_entry(&id.to_string())
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?
        .map(Json)
        .ok_or_else(|| Json(ApiError { error: "Audit entry not found".to_string() }))
}
