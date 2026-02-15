//! Mock implementations for testing.

use crate::auth::TokenProvider;
use crate::client::BankApiClient;
use crate::error::ApiError;
use crate::models::{
    AccountData, CreateTransferDTO, TransactionResponse, TransferResponse, TransferToCreditCardDTO,
};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use tokio::sync::RwLock;

/// Mock token provider for testing.
pub struct MockTokenProvider {
    token: String,
}

impl MockTokenProvider {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl TokenProvider for MockTokenProvider {
    async fn get_access_token(&self) -> Result<String, ApiError> {
        Ok(self.token.clone())
    }
}

/// Mock bank client for testing rule evaluation and transfer logic.
pub struct MockBankClient {
    accounts: RwLock<AccountData>,
    transactions: RwLock<HashMap<String, TransactionResponse>>,
    transfer_results: RwLock<VecDeque<Result<TransferResponse, ApiError>>>,
    transfer_history: RwLock<Vec<TransferRecord>>,
}

/// Record of a transfer attempt.
#[derive(Debug, Clone)]
pub enum TransferRecord {
    Regular(CreateTransferDTO),
    CreditCard(TransferToCreditCardDTO),
}

impl MockBankClient {
    /// Creates a new mock client with empty data.
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(AccountData::default()),
            transactions: RwLock::new(HashMap::new()),
            transfer_results: RwLock::new(VecDeque::new()),
            transfer_history: RwLock::new(Vec::new()),
        }
    }

    /// Sets the accounts to return.
    pub async fn set_accounts(&self, accounts: AccountData) {
        *self.accounts.write().await = accounts;
    }

    /// Sets transactions for a specific account.
    pub async fn set_transactions(&self, account_key: impl Into<String>, transactions: TransactionResponse) {
        self.transactions
            .write()
            .await
            .insert(account_key.into(), transactions);
    }

    /// Queues a transfer result to be returned on the next transfer call.
    pub async fn queue_transfer_result(&self, result: Result<TransferResponse, ApiError>) {
        self.transfer_results.write().await.push_back(result);
    }

    /// Returns all transfer attempts made.
    pub async fn get_transfer_history(&self) -> Vec<TransferRecord> {
        self.transfer_history.read().await.clone()
    }

    /// Clears transfer history.
    pub async fn clear_transfer_history(&self) {
        self.transfer_history.write().await.clear();
    }
}

impl Default for MockBankClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BankApiClient for MockBankClient {
    async fn get_accounts(&self) -> Result<AccountData, ApiError> {
        Ok(self.accounts.read().await.clone())
    }

    async fn get_transactions(&self, account_key: &str) -> Result<TransactionResponse, ApiError> {
        let transactions = self.transactions.read().await;
        transactions
            .get(account_key)
            .cloned()
            .ok_or_else(|| ApiError::Api {
                code: "NOT_FOUND".to_string(),
                message: format!("No transactions for account {}", account_key),
                trace_id: String::new(),
            })
    }

    async fn create_transfer(&self, transfer: CreateTransferDTO) -> Result<TransferResponse, ApiError> {
        // Record the transfer attempt
        self.transfer_history
            .write()
            .await
            .push(TransferRecord::Regular(transfer));

        // Return queued result or default success
        self.transfer_results
            .write()
            .await
            .pop_front()
            .unwrap_or_else(|| {
                Ok(TransferResponse {
                    errors: vec![],
                    payment_id: Some("mock-payment-id".to_string()),
                    status: Some("COMPLETED".to_string()),
                })
            })
    }

    async fn create_credit_card_transfer(
        &self,
        transfer: TransferToCreditCardDTO,
    ) -> Result<TransferResponse, ApiError> {
        // Record the transfer attempt
        self.transfer_history
            .write()
            .await
            .push(TransferRecord::CreditCard(transfer));

        // Return queued result or default success
        self.transfer_results
            .write()
            .await
            .pop_front()
            .unwrap_or_else(|| {
                Ok(TransferResponse {
                    errors: vec![],
                    payment_id: Some("mock-payment-id".to_string()),
                    status: Some("COMPLETED".to_string()),
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Account, AccountProperties};

    fn create_test_account(key: &str, name: &str, balance: f64) -> Account {
        Account {
            key: key.to_string(),
            account_number: format!("1234567890{}", key),
            iban: format!("NO{}", key),
            name: name.to_string(),
            description: String::new(),
            balance,
            available_balance: balance,
            currency_code: "NOK".to_string(),
            owner: None,
            product_type: "CURRENT".to_string(),
            type_field: "ACCOUNT".to_string(),
            product_id: None,
            description_code: None,
            account_properties: AccountProperties::default(),
            credit_card_credit_limit: None,
            credit_card_account_id: None,
        }
    }

    #[tokio::test]
    async fn test_mock_client_accounts() {
        let client = MockBankClient::new();

        let accounts = AccountData {
            accounts: vec![
                create_test_account("1", "Checking", 1000.0),
                create_test_account("2", "Savings", 5000.0),
            ],
            errors: vec![],
        };

        client.set_accounts(accounts.clone()).await;

        let result = client.get_accounts().await.unwrap();
        assert_eq!(result.accounts.len(), 2);
        assert_eq!(result.accounts[0].name, "Checking");
        assert_eq!(result.accounts[1].balance, 5000.0);
    }

    #[tokio::test]
    async fn test_mock_client_transfers() {
        let client = MockBankClient::new();

        let transfer = CreateTransferDTO {
            amount: "100".to_string(),
            due_date: None,
            message: Some("Test transfer".to_string()),
            to_account: "2".to_string(),
            from_account: "1".to_string(),
            currency_code: None,
        };

        let result = client.create_transfer(transfer).await.unwrap();
        assert!(result.errors.is_empty());
        assert!(result.payment_id.is_some());

        let history = client.get_transfer_history().await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_client_queued_error() {
        let client = MockBankClient::new();

        // Queue an error
        client
            .queue_transfer_result(Err(ApiError::Api {
                code: "INSUFFICIENT_FUNDS".to_string(),
                message: "Not enough money".to_string(),
                trace_id: "trace-123".to_string(),
            }))
            .await;

        let transfer = CreateTransferDTO {
            amount: "100".to_string(),
            due_date: None,
            message: None,
            to_account: "2".to_string(),
            from_account: "1".to_string(),
            currency_code: None,
        };

        let result = client.create_transfer(transfer).await;
        assert!(result.is_err());
    }
}
