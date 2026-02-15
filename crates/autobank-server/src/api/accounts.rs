//! Account-related API endpoints.

use crate::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use sb1_api::models::{AccountData, TransactionResponse};
use serde::Serialize;

/// Creates the accounts router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_accounts))
        .route("/{key}", get(get_account))
        .route("/{key}/transactions", get(get_transactions))
}

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

/// List all accounts.
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<AccountData>, Json<ApiError>> {
    state
        .bank_client
        .get_accounts()
        .await
        .map(Json)
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}

/// Get a single account by key.
pub async fn get_account(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<sb1_api::models::Account>, Json<ApiError>> {
    let accounts = state
        .bank_client
        .get_accounts()
        .await
        .map_err(|e| Json(ApiError { error: e.to_string() }))?;

    accounts
        .accounts
        .into_iter()
        .find(|a| a.key == key)
        .map(Json)
        .ok_or_else(|| Json(ApiError { error: "Account not found".to_string() }))
}

/// Get transactions for an account.
pub async fn get_transactions(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<TransactionResponse>, Json<ApiError>> {
    state
        .bank_client
        .get_transactions(&key)
        .await
        .map(Json)
        .map_err(|e| Json(ApiError { error: e.to_string() }))
}
