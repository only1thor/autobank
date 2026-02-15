//! Configuration management for the SpareBank 1 API client.

use crate::error::ApiError;
use crate::models::TokenData;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::debug;

/// Application configuration loaded from config file.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub financial_institution: String,
}

/// Returns the application config directory path.
pub fn app_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("autobank"))
}

/// Returns the application data directory path.
pub fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|base| base.join("autobank"))
}

/// Returns the path to the config file.
pub fn config_file_path() -> Result<PathBuf, ApiError> {
    let dir = app_config_dir().ok_or_else(|| ApiError::Config("Could not determine config directory".into()))?;

    debug!("App config dir: {}", dir.display());

    std::fs::create_dir_all(&dir)?;

    Ok(dir.join("config.toml"))
}

/// Returns the path to the token file.
pub fn token_file_path() -> Result<PathBuf, ApiError> {
    let dir = app_data_dir().ok_or_else(|| ApiError::Config("Could not determine data directory".into()))?;

    debug!("App data dir: {}", dir.display());

    std::fs::create_dir_all(&dir)?;

    Ok(dir.join("auth.json"))
}

/// Loads the application configuration from the config file.
pub fn load_config() -> Result<AppConfig, ApiError> {
    let conf_path = config_file_path()?;

    if !conf_path.exists() {
        create_config_template(&conf_path)?;
        return Err(ApiError::Config(format!(
            "Config file created at: {}. Please edit this file and add your SpareBank 1 API credentials.",
            conf_path.display()
        )));
    }

    let file = std::fs::read_to_string(&conf_path)?;
    let config: AppConfig = toml::from_str(&file)
        .map_err(|e| ApiError::Config(format!("config.toml is not in proper format: {}", e)))?;

    Ok(config)
}

/// Reads the stored access token from file.
pub fn read_token_data() -> Result<Option<TokenData>, ApiError> {
    let token_path = token_file_path()?;

    if !token_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&token_path)?;
    let token_data: TokenData = serde_json::from_str(&content)?;

    Ok(Some(token_data))
}

/// Saves token data to file.
pub fn save_token_data(token_data: &TokenData) -> Result<(), ApiError> {
    let token_path = token_file_path()?;

    let json_content = serde_json::to_string_pretty(token_data)?;
    std::fs::write(&token_path, json_content)?;

    debug!("Token data saved to {}", token_path.display());

    Ok(())
}

/// Creates a template config file.
fn create_config_template(conf_path: &PathBuf) -> Result<(), ApiError> {
    let template = r#"# Autobank Configuration File
# Add your SpareBank 1 API credentials below

client_id = "your-client-id-here"
client_secret = "your-client-secret-here"

# Your financial institution ID
# Examples: fid-smn (SpareBank 1 Midt-Norge), fid-snn (SpareBank 1 SR-Bank), etc.
financial_institution = "fid-smn"
"#;
    std::fs::write(conf_path, template)?;
    Ok(())
}
