//! Rule management API endpoints.

use crate::AppState;
use crate::rules::Rule;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Creates the rules router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rules).post(create_rule))
        .route("/{id}", get(get_rule).put(update_rule).delete(delete_rule))
        .route("/{id}/enable", post(enable_rule))
        .route("/{id}/disable", post(disable_rule))
}

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_account_key: String,
    pub conditions: Vec<crate::rules::Condition>,
    pub actions: Vec<crate::rules::Action>,
}

#[derive(Deserialize)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_account_key: Option<String>,
    pub conditions: Option<Vec<crate::rules::Condition>>,
    pub actions: Option<Vec<crate::rules::Action>>,
}

/// List all rules.
pub async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<Rule>>, Json<ApiError>> {
    state
        .db
        .list_rules()
        .await
        .map(Json)
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}

/// Get a single rule by ID.
pub async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Rule>, Json<ApiError>> {
    state
        .db
        .get_rule(&id.to_string())
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?
        .map(Json)
        .ok_or_else(|| Json(ApiError { error: "Rule not found".to_string() }))
}

/// Create a new rule.
pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<Rule>, Json<ApiError>> {
    let now = chrono::Utc::now().timestamp();
    let rule = Rule {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        description: req.description,
        enabled: true,
        trigger_account_key: req.trigger_account_key,
        conditions: req.conditions,
        actions: req.actions,
        created_at: now,
        updated_at: now,
    };

    state
        .db
        .create_rule(&rule)
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?;

    Ok(Json(rule))
}

/// Update an existing rule.
pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRuleRequest>,
) -> Result<Json<Rule>, Json<ApiError>> {
    let mut rule = state
        .db
        .get_rule(&id.to_string())
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?
        .ok_or_else(|| Json(ApiError { error: "Rule not found".to_string() }))?;

    if let Some(name) = req.name {
        rule.name = name;
    }
    if let Some(description) = req.description {
        rule.description = Some(description);
    }
    if let Some(trigger_account_key) = req.trigger_account_key {
        rule.trigger_account_key = trigger_account_key;
    }
    if let Some(conditions) = req.conditions {
        rule.conditions = conditions;
    }
    if let Some(actions) = req.actions {
        rule.actions = actions;
    }
    rule.updated_at = chrono::Utc::now().timestamp();

    state
        .db
        .update_rule(&rule)
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?;

    Ok(Json(rule))
}

/// Delete a rule.
pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, Json<ApiError>> {
    state
        .db
        .delete_rule(&id.to_string())
        .await
        .map(|_| Json(()))
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}

/// Enable a rule.
pub async fn enable_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Rule>, Json<ApiError>> {
    state
        .db
        .set_rule_enabled(&id.to_string(), true)
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?;

    get_rule(State(state), Path(id)).await
}

/// Disable a rule.
pub async fn disable_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Rule>, Json<ApiError>> {
    state
        .db
        .set_rule_enabled(&id.to_string(), false)
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?;

    get_rule(State(state), Path(id)).await
}
