//! SpareBank 1 API client library
//!
//! This crate provides an async client for interacting with SpareBank 1's banking API,
//! including account management, transaction retrieval, and transfers.
//!
//! # Example
//!
//! ```ignore
//! use sb1_api::{BankApiClient, SpareBank1Client, config::load_config, auth::FileTokenProvider};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = load_config()?;
//!     let token_provider = Arc::new(FileTokenProvider::new(config)?);
//!     let client = SpareBank1Client::new(token_provider);
//!     
//!     let accounts = client.get_accounts().await?;
//!     println!("Found {} accounts", accounts.accounts.len());
//!     
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod mock;
pub mod models;

pub use auth::{FileTokenProvider, TokenProvider};
pub use client::{BankApiClient, SpareBank1Client};
pub use error::ApiError;
pub use mock::{MockBankClient, MockTokenProvider};
