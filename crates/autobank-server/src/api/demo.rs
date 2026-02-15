//! Demo mode API endpoints for creating test transactions.

use crate::AppState;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(demo_status))
        .route("/transactions", post(create_transaction))
        .route("/accounts", get(get_demo_accounts))
}

#[derive(Serialize)]
pub struct DemoStatusResponse {
    enabled: bool,
    message: &'static str,
}

/// Check if demo mode is enabled.
pub async fn demo_status(State(state): State<AppState>) -> Json<DemoStatusResponse> {
    Json(DemoStatusResponse {
        enabled: state.demo_mode,
        message: if state.demo_mode {
            "Demo mode is active. You can create test transactions."
        } else {
            "Demo mode is not enabled. Start server with --demo flag."
        },
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionRequest {
    pub account_key: String,
    pub description: String,
    pub amount: f64,
    #[serde(default = "default_true")]
    pub is_settled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionResponse {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub message: String,
}

/// Create a new demo transaction.
pub async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, (StatusCode, Json<CreateTransactionResponse>)> {
    // Check if demo mode is enabled
    let demo_client = state.demo_client.as_ref().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(CreateTransactionResponse {
                success: false,
                transaction_id: None,
                message: "Demo mode is not enabled. Start server with --demo flag.".to_string(),
            }),
        )
    })?;

    // Create the transaction
    let transaction = demo_client
        .create_transaction(&req.account_key, &req.description, req.amount, req.is_settled)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(CreateTransactionResponse {
                    success: false,
                    transaction_id: None,
                    message: format!("Account not found: {}", req.account_key),
                }),
            )
        })?;

    let tx_id = transaction.id.clone();
    
    // Add to the demo client
    demo_client.add_transaction(transaction).await;

    Ok(Json(CreateTransactionResponse {
        success: true,
        transaction_id: Some(tx_id),
        message: "Transaction created successfully".to_string(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoAccount {
    pub key: String,
    pub name: String,
    pub account_number: String,
    pub balance: f64,
    pub account_type: String,
}

#[derive(Serialize)]
pub struct DemoAccountsResponse {
    pub accounts: Vec<DemoAccount>,
}

/// Get demo accounts (simplified view for demo UI).
pub async fn get_demo_accounts(
    State(state): State<AppState>,
) -> Result<Json<DemoAccountsResponse>, (StatusCode, Json<CreateTransactionResponse>)> {
    let demo_client = state.demo_client.as_ref().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(CreateTransactionResponse {
                success: false,
                transaction_id: None,
                message: "Demo mode is not enabled".to_string(),
            }),
        )
    })?;

    let accounts = demo_client
        .get_accounts_list()
        .iter()
        .map(|a| DemoAccount {
            key: a.key.clone(),
            name: a.name.clone(),
            account_number: a.account_number.clone(),
            balance: a.balance,
            account_type: a.type_field.clone(),
        })
        .collect();

    Ok(Json(DemoAccountsResponse { accounts }))
}
