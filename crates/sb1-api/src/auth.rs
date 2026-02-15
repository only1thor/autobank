//! OAuth authentication for SpareBank 1 API.

use crate::config::{read_token_data, save_token_data, AppConfig};
use crate::error::ApiError;
use crate::models::TokenData;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

const TOKEN_ENDPOINT: &str = "https://api.sparebank1.no/oauth/token";

/// Provider trait for obtaining access tokens.
#[async_trait::async_trait]
pub trait TokenProvider: Send + Sync {
    /// Returns a valid access token, refreshing if necessary.
    async fn get_access_token(&self) -> Result<String, ApiError>;
}

/// File-based token provider that stores tokens on disk.
pub struct FileTokenProvider {
    config: AppConfig,
    token_data: Arc<RwLock<Option<TokenData>>>,
    http_client: reqwest::Client,
}

impl FileTokenProvider {
    /// Creates a new file-based token provider.
    pub fn new(config: AppConfig) -> Result<Self, ApiError> {
        let token_data = read_token_data()?;

        Ok(Self {
            config,
            token_data: Arc::new(RwLock::new(token_data)),
            http_client: reqwest::Client::new(),
        })
    }

    /// Attempts to refresh the access token using the refresh token.
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenData, ApiError> {
        debug!("Attempting to refresh access token...");

        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .http_client
            .post(TOKEN_ENDPOINT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(ApiError::Auth(format!(
                "Token refresh failed with status {}: {}",
                status, error_body
            )));
        }

        let token_data: TokenData = response.json().await?;
        debug!("Token refreshed successfully");

        Ok(token_data)
    }

    /// Exchanges an authorization code for tokens.
    pub async fn exchange_code(&self, code: &str) -> Result<TokenData, ApiError> {
        let redirect_uri = "http://localhost:8321";

        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri),
        ];

        let response = self
            .http_client
            .post(TOKEN_ENDPOINT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(ApiError::Auth(format!(
                "Token exchange failed with status {}: {}",
                status, error_body
            )));
        }

        let token_data: TokenData = response.json().await?;
        debug!("Access token obtained successfully");

        // Save and cache the new token
        save_token_data(&token_data)?;
        *self.token_data.write().await = Some(token_data.clone());

        Ok(token_data)
    }

    /// Returns the OAuth authorization URL for the user to visit.
    pub fn get_authorization_url(&self) -> String {
        let redirect_uri = urlencoding::encode("http://localhost:8321");
        format!(
            "https://api.sparebank1.no/oauth/authorize?client_id={}&state=123&redirect_uri={}&finInst={}&response_type=code",
            self.config.client_id,
            redirect_uri,
            self.config.financial_institution
        )
    }
}

#[async_trait::async_trait]
impl TokenProvider for FileTokenProvider {
    async fn get_access_token(&self) -> Result<String, ApiError> {
        // Check if we have a token
        let token_data = self.token_data.read().await.clone();

        match token_data {
            Some(data) => {
                // Try to use existing token - if it fails, try refresh
                // For now, we just return the token and let the API call handle expiry
                // In a more robust implementation, we'd check expiry time
                Ok(data.access_token)
            }
            None => Err(ApiError::NoToken),
        }
    }
}

/// Attempts to get a valid token, refreshing if necessary.
pub async fn ensure_authenticated(provider: &FileTokenProvider) -> Result<String, ApiError> {
    let token_data = provider.token_data.read().await.clone();

    match token_data {
        Some(data) => {
            // Try refresh if we have a refresh token
            match provider.refresh_token(&data.refresh_token).await {
                Ok(new_data) => {
                    save_token_data(&new_data)?;
                    *provider.token_data.write().await = Some(new_data.clone());
                    Ok(new_data.access_token)
                }
                Err(_) => {
                    // Refresh failed, return existing token and hope it works
                    // or caller will need to re-authenticate
                    Ok(data.access_token)
                }
            }
        }
        None => Err(ApiError::NoToken),
    }
}
