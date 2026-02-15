//! SpareBank 1 API client implementation.

use crate::auth::TokenProvider;
use crate::error::ApiError;
use crate::models::{
    AccountData, CreateTransferDTO, TransactionResponse, TransferResponse, TransferToCreditCardDTO,
};
use async_trait::async_trait;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use std::sync::Arc;
use tracing::debug;

const BASE_URL: &str = "https://api.sparebank1.no";
const ACCEPT_HEADER: &str = "application/vnd.sparebank1.v1+json; charset=utf-8";

/// Trait defining the bank API operations.
#[async_trait]
pub trait BankApiClient: Send + Sync {
    /// Fetches all accounts for the authenticated user.
    async fn get_accounts(&self) -> Result<AccountData, ApiError>;

    /// Fetches transactions for a specific account.
    async fn get_transactions(&self, account_key: &str) -> Result<TransactionResponse, ApiError>;

    /// Creates a transfer between accounts.
    async fn create_transfer(&self, transfer: CreateTransferDTO) -> Result<TransferResponse, ApiError>;

    /// Creates a transfer to a credit card.
    async fn create_credit_card_transfer(
        &self,
        transfer: TransferToCreditCardDTO,
    ) -> Result<TransferResponse, ApiError>;
}

/// SpareBank 1 API client implementation.
pub struct SpareBank1Client {
    http_client: reqwest::Client,
    base_url: String,
    token_provider: Arc<dyn TokenProvider>,
}

impl SpareBank1Client {
    /// Creates a new SpareBank 1 API client.
    pub fn new(token_provider: Arc<dyn TokenProvider>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: BASE_URL.to_string(),
            token_provider,
        }
    }

    /// Creates a new client with a custom base URL (for testing).
    pub fn with_base_url(token_provider: Arc<dyn TokenProvider>, base_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            token_provider,
        }
    }

    /// Builds headers with authentication.
    async fn build_headers(&self) -> Result<HeaderMap, ApiError> {
        let access_token = self.token_provider.get_access_token().await?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|_| ApiError::Auth("Invalid access token format".into()))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_HEADER));

        Ok(headers)
    }

    /// Makes a GET request to the API.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let headers = self.build_headers().await?;
        let url = format!("{}{}", self.base_url, path);

        debug!("GET {}", url);

        let response = self.http_client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(ApiError::Api {
                code: status.as_str().to_string(),
                message: text,
                trace_id: String::new(),
            });
        }

        serde_json::from_str(&text).map_err(ApiError::from)
    }

    /// Makes a POST request to the API.
    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let headers = self.build_headers().await?;
        let url = format!("{}{}", self.base_url, path);

        debug!("POST {}", url);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            // Try to parse as API error
            if let Ok(error_response) = serde_json::from_str::<TransferResponse>(&text) {
                if let Some(error) = error_response.errors.first() {
                    return Err(ApiError::Api {
                        code: error.code.clone(),
                        message: error.message.clone(),
                        trace_id: error.trace_id.clone(),
                    });
                }
            }
            return Err(ApiError::Api {
                code: status.as_str().to_string(),
                message: text,
                trace_id: String::new(),
            });
        }

        serde_json::from_str(&text).map_err(ApiError::from)
    }
}

#[async_trait]
impl BankApiClient for SpareBank1Client {
    async fn get_accounts(&self) -> Result<AccountData, ApiError> {
        self.get("/personal/banking/accounts?includeCreditCardAccounts=true")
            .await
    }

    async fn get_transactions(&self, account_key: &str) -> Result<TransactionResponse, ApiError> {
        let path = format!("/personal/banking/transactions?accountKey={}", account_key);
        self.get(&path).await
    }

    async fn create_transfer(&self, transfer: CreateTransferDTO) -> Result<TransferResponse, ApiError> {
        self.post("/personal/banking/transfer/debit", &transfer).await
    }

    async fn create_credit_card_transfer(
        &self,
        transfer: TransferToCreditCardDTO,
    ) -> Result<TransferResponse, ApiError> {
        self.post("/personal/banking/transfer/creditcard/transferTo", &transfer)
            .await
    }
}
