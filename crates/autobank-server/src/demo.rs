//! Demo mode implementation with mock bank client and sample data.

use async_trait::async_trait;
use sb1_api::models::{
    Account, AccountData, AccountNumber, AccountProperties, ClassificationInput,
    CreateTransferDTO, Owner, Transaction, TransactionResponse, TransferResponse,
    TransferToCreditCardDTO,
};
use sb1_api::BankApiClient;
use sb1_api::error::ApiError;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::sync::RwLock;
use tracing::info;

/// Demo bank client with mutable transaction storage.
pub struct DemoBankClient {
    accounts: Vec<Account>,
    transactions: RwLock<Vec<Transaction>>,
    next_tx_id: AtomicI64,
}

impl DemoBankClient {
    /// Create a new demo client with sample accounts and transactions.
    pub fn new() -> Self {
        let accounts = Self::create_sample_accounts();
        let transactions = Self::create_sample_transactions(&accounts);
        
        Self {
            accounts,
            transactions: RwLock::new(transactions),
            next_tx_id: AtomicI64::new(1000),
        }
    }

    fn create_demo_owner() -> Owner {
        Owner {
            name: "Demo User".to_string(),
            first_name: "Demo".to_string(),
            last_name: "User".to_string(),
            age: 30,
            customer_key: "demo-customer-key".to_string(),
            ssn_key: "demo-ssn-key".to_string(),
        }
    }

    fn create_sample_accounts() -> Vec<Account> {
        let owner = Self::create_demo_owner();
        
        vec![
            Account {
                key: "checking-1".to_string(),
                account_number: "12345678901".to_string(),
                iban: "NO9312345678901".to_string(),
                name: "Checking Account".to_string(),
                description: "Main checking account".to_string(),
                balance: 15420.50,
                available_balance: 15420.50,
                currency_code: "NOK".to_string(),
                owner: Some(owner.clone()),
                product_type: "CURRENT".to_string(),
                type_field: "ACCOUNT".to_string(),
                product_id: Some("current-account".to_string()),
                description_code: None,
                account_properties: AccountProperties {
                    is_transfer_from_enabled: true,
                    is_transfer_to_enabled: true,
                    is_payment_from_enabled: true,
                    is_allowed_in_avtale_giro: false,
                    has_access: true,
                    is_balance_preferred: true,
                    is_flexi_loan: false,
                    is_codebitor_loan: false,
                    is_security_balance: false,
                    is_aksjesparekonto: false,
                    is_savings_account: false,
                    is_bonus_account: false,
                    user_has_right_of_disposal: true,
                    user_has_right_of_access: true,
                    is_owned: true,
                    is_withdrawals_allowed: true,
                    is_blocked: false,
                    is_hidden: false,
                    is_balance_updated_immediately_on_transfer_to: true,
                    is_default_payment_account: true,
                },
                credit_card_credit_limit: None,
                credit_card_account_id: None,
            },
            Account {
                key: "savings-1".to_string(),
                account_number: "12345678902".to_string(),
                iban: "NO9312345678902".to_string(),
                name: "Savings Account".to_string(),
                description: "High-interest savings".to_string(),
                balance: 52000.00,
                available_balance: 52000.00,
                currency_code: "NOK".to_string(),
                owner: Some(owner.clone()),
                product_type: "SAVINGS".to_string(),
                type_field: "ACCOUNT".to_string(),
                product_id: Some("savings-account".to_string()),
                description_code: None,
                account_properties: AccountProperties {
                    is_transfer_from_enabled: true,
                    is_transfer_to_enabled: true,
                    is_payment_from_enabled: false,
                    is_allowed_in_avtale_giro: false,
                    has_access: true,
                    is_balance_preferred: false,
                    is_flexi_loan: false,
                    is_codebitor_loan: false,
                    is_security_balance: false,
                    is_aksjesparekonto: false,
                    is_savings_account: true,
                    is_bonus_account: false,
                    user_has_right_of_disposal: true,
                    user_has_right_of_access: true,
                    is_owned: true,
                    is_withdrawals_allowed: true,
                    is_blocked: false,
                    is_hidden: false,
                    is_balance_updated_immediately_on_transfer_to: true,
                    is_default_payment_account: false,
                },
                credit_card_credit_limit: None,
                credit_card_account_id: None,
            },
            Account {
                key: "creditcard-1".to_string(),
                account_number: "12345678903".to_string(),
                iban: "NO9312345678903".to_string(),
                name: "Credit Card".to_string(),
                description: "Visa Gold".to_string(),
                balance: -2340.00,
                available_balance: 47660.00,
                currency_code: "NOK".to_string(),
                owner: Some(owner),
                product_type: "CREDITCARD".to_string(),
                type_field: "CREDITCARD".to_string(),
                product_id: Some("visa-gold".to_string()),
                description_code: None,
                account_properties: AccountProperties::default(),
                credit_card_credit_limit: Some(50000.0),
                credit_card_account_id: Some("cc-account-123".to_string()),
            },
        ]
    }

    fn create_sample_transactions(accounts: &[Account]) -> Vec<Transaction> {
        let now = chrono::Utc::now().timestamp_millis();
        let day_ms = 86400000i64;
        
        let checking = &accounts[0];
        
        vec![
            // Recent Netflix charge
            Transaction {
                id: "tx-001".to_string(),
                non_unique_id: "tx-001".to_string(),
                description: Some("NETFLIX.COM".to_string()),
                cleaned_description: Some("Netflix".to_string()),
                account_number: AccountNumber {
                    value: checking.account_number.clone(),
                    formatted: checking.account_number.clone(),
                    unformatted: checking.account_number.clone(),
                },
                amount: -179.0,
                date: now - day_ms,
                interest_date: Some(now - day_ms),
                type_code: "PURCHASE".to_string(),
                type_text: "Purchase".to_string(),
                currency_code: "NOK".to_string(),
                can_show_details: true,
                source: "CARD".to_string(),
                is_confidential: false,
                booking_status: "BOOKED".to_string(),
                account_name: checking.name.clone(),
                account_key: checking.key.clone(),
                account_currency: "NOK".to_string(),
                is_from_currency_account: false,
                classification_input: ClassificationInput {
                    id: "tx-001".to_string(),
                    amount: -179.0,
                    type_field: "PURCHASE".to_string(),
                    text: Some("Netflix".to_string()),
                    date: now - day_ms,
                },
                remote_account_number: None,
                remote_account_name: None,
                kid_or_message: None,
            },
            // Spotify charge
            Transaction {
                id: "tx-002".to_string(),
                non_unique_id: "tx-002".to_string(),
                description: Some("SPOTIFY AB".to_string()),
                cleaned_description: Some("Spotify".to_string()),
                account_number: AccountNumber {
                    value: checking.account_number.clone(),
                    formatted: checking.account_number.clone(),
                    unformatted: checking.account_number.clone(),
                },
                amount: -119.0,
                date: now - 2 * day_ms,
                interest_date: Some(now - 2 * day_ms),
                type_code: "PURCHASE".to_string(),
                type_text: "Purchase".to_string(),
                currency_code: "NOK".to_string(),
                can_show_details: true,
                source: "CARD".to_string(),
                is_confidential: false,
                booking_status: "BOOKED".to_string(),
                account_name: checking.name.clone(),
                account_key: checking.key.clone(),
                account_currency: "NOK".to_string(),
                is_from_currency_account: false,
                classification_input: ClassificationInput {
                    id: "tx-002".to_string(),
                    amount: -119.0,
                    type_field: "PURCHASE".to_string(),
                    text: Some("Spotify".to_string()),
                    date: now - 2 * day_ms,
                },
                remote_account_number: None,
                remote_account_name: None,
                kid_or_message: None,
            },
            // Grocery store
            Transaction {
                id: "tx-003".to_string(),
                non_unique_id: "tx-003".to_string(),
                description: Some("REMA 1000 SENTRUM".to_string()),
                cleaned_description: Some("Rema 1000".to_string()),
                account_number: AccountNumber {
                    value: checking.account_number.clone(),
                    formatted: checking.account_number.clone(),
                    unformatted: checking.account_number.clone(),
                },
                amount: -342.50,
                date: now - 3 * day_ms,
                interest_date: Some(now - 3 * day_ms),
                type_code: "PURCHASE".to_string(),
                type_text: "Purchase".to_string(),
                currency_code: "NOK".to_string(),
                can_show_details: true,
                source: "CARD".to_string(),
                is_confidential: false,
                booking_status: "BOOKED".to_string(),
                account_name: checking.name.clone(),
                account_key: checking.key.clone(),
                account_currency: "NOK".to_string(),
                is_from_currency_account: false,
                classification_input: ClassificationInput {
                    id: "tx-003".to_string(),
                    amount: -342.50,
                    type_field: "PURCHASE".to_string(),
                    text: Some("Rema 1000".to_string()),
                    date: now - 3 * day_ms,
                },
                remote_account_number: None,
                remote_account_name: None,
                kid_or_message: None,
            },
            // Salary deposit
            Transaction {
                id: "tx-004".to_string(),
                non_unique_id: "tx-004".to_string(),
                description: Some("SALARY ACME CORP".to_string()),
                cleaned_description: Some("Salary".to_string()),
                account_number: AccountNumber {
                    value: checking.account_number.clone(),
                    formatted: checking.account_number.clone(),
                    unformatted: checking.account_number.clone(),
                },
                amount: 45000.0,
                date: now - 5 * day_ms,
                interest_date: Some(now - 5 * day_ms),
                type_code: "SALARY".to_string(),
                type_text: "Salary".to_string(),
                currency_code: "NOK".to_string(),
                can_show_details: true,
                source: "TRANSFER".to_string(),
                is_confidential: false,
                booking_status: "BOOKED".to_string(),
                account_name: checking.name.clone(),
                account_key: checking.key.clone(),
                account_currency: "NOK".to_string(),
                is_from_currency_account: false,
                classification_input: ClassificationInput {
                    id: "tx-004".to_string(),
                    amount: 45000.0,
                    type_field: "SALARY".to_string(),
                    text: Some("Salary".to_string()),
                    date: now - 5 * day_ms,
                },
                remote_account_number: Some("98765432100".to_string()),
                remote_account_name: Some("ACME CORP".to_string()),
                kid_or_message: Some("Salary February".to_string()),
            },
            // Pending transaction
            Transaction {
                id: "tx-005".to_string(),
                non_unique_id: "tx-005".to_string(),
                description: Some("AMAZON.COM*123ABC".to_string()),
                cleaned_description: Some("Amazon".to_string()),
                account_number: AccountNumber {
                    value: checking.account_number.clone(),
                    formatted: checking.account_number.clone(),
                    unformatted: checking.account_number.clone(),
                },
                amount: -599.0,
                date: now,
                interest_date: None,
                type_code: "PURCHASE".to_string(),
                type_text: "Purchase".to_string(),
                currency_code: "NOK".to_string(),
                can_show_details: true,
                source: "CARD".to_string(),
                is_confidential: false,
                booking_status: "PENDING".to_string(),
                account_name: checking.name.clone(),
                account_key: checking.key.clone(),
                account_currency: "NOK".to_string(),
                is_from_currency_account: false,
                classification_input: ClassificationInput {
                    id: "tx-005".to_string(),
                    amount: -599.0,
                    type_field: "PURCHASE".to_string(),
                    text: Some("Amazon".to_string()),
                    date: now,
                },
                remote_account_number: None,
                remote_account_name: None,
                kid_or_message: None,
            },
        ]
    }

    /// Add a new transaction to the demo client.
    pub async fn add_transaction(&self, tx: Transaction) {
        info!("Adding demo transaction: {} - {}", tx.id, tx.cleaned_description.as_deref().unwrap_or(""));
        self.transactions.write().await.push(tx);
    }

    /// Create a new transaction with the given parameters.
    pub fn create_transaction(
        &self,
        account_key: &str,
        description: &str,
        amount: f64,
        is_settled: bool,
    ) -> Option<Transaction> {
        let account = self.accounts.iter().find(|a| a.key == account_key)?;
        let now = chrono::Utc::now().timestamp_millis();
        let tx_id = self.next_tx_id.fetch_add(1, Ordering::SeqCst);
        
        Some(Transaction {
            id: format!("tx-{}", tx_id),
            non_unique_id: format!("tx-{}", tx_id),
            description: Some(description.to_uppercase()),
            cleaned_description: Some(description.to_string()),
            account_number: AccountNumber {
                value: account.account_number.clone(),
                formatted: account.account_number.clone(),
                unformatted: account.account_number.clone(),
            },
            amount,
            date: now,
            interest_date: if is_settled { Some(now) } else { None },
            type_code: if amount >= 0.0 { "TRANSFER".to_string() } else { "PURCHASE".to_string() },
            type_text: if amount >= 0.0 { "Transfer".to_string() } else { "Purchase".to_string() },
            currency_code: "NOK".to_string(),
            can_show_details: true,
            source: "CARD".to_string(),
            is_confidential: false,
            booking_status: if is_settled { "BOOKED".to_string() } else { "PENDING".to_string() },
            account_name: account.name.clone(),
            account_key: account.key.clone(),
            account_currency: "NOK".to_string(),
            is_from_currency_account: false,
            classification_input: ClassificationInput {
                id: format!("tx-{}", tx_id),
                amount,
                type_field: if amount >= 0.0 { "TRANSFER".to_string() } else { "PURCHASE".to_string() },
                text: Some(description.to_string()),
                date: now,
            },
            remote_account_number: None,
            remote_account_name: None,
            kid_or_message: None,
        })
    }

    /// Get all accounts (for API)
    pub fn get_accounts_list(&self) -> &[Account] {
        &self.accounts
    }
}

impl Default for DemoBankClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BankApiClient for DemoBankClient {
    async fn get_accounts(&self) -> Result<AccountData, ApiError> {
        Ok(AccountData {
            accounts: self.accounts.clone(),
            errors: vec![],
        })
    }

    async fn get_transactions(&self, account_key: &str) -> Result<TransactionResponse, ApiError> {
        let transactions = self.transactions.read().await;
        let filtered: Vec<Transaction> = transactions
            .iter()
            .filter(|tx| tx.account_key == account_key)
            .cloned()
            .collect();
        
        Ok(TransactionResponse {
            transactions: filtered,
            errors: vec![],
        })
    }

    async fn create_transfer(&self, transfer: CreateTransferDTO) -> Result<TransferResponse, ApiError> {
        info!(
            "Demo transfer: {} NOK from {} to {}",
            transfer.amount, transfer.from_account, transfer.to_account
        );
        
        Ok(TransferResponse {
            errors: vec![],
            payment_id: Some(format!("demo-payment-{}", uuid::Uuid::new_v4())),
            status: Some("COMPLETED".to_string()),
        })
    }

    async fn create_credit_card_transfer(
        &self,
        transfer: TransferToCreditCardDTO,
    ) -> Result<TransferResponse, ApiError> {
        info!(
            "Demo credit card transfer: {} NOK from {} to card {}",
            transfer.amount, transfer.from_account, transfer.credit_card_account_id
        );
        
        Ok(TransferResponse {
            errors: vec![],
            payment_id: Some(format!("demo-cc-payment-{}", uuid::Uuid::new_v4())),
            status: Some("COMPLETED".to_string()),
        })
    }
}
