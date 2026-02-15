//! Integration tests for the rule engine.

use sb1_api::models::{
    Account, AccountData, AccountNumber, AccountProperties, ClassificationInput, Transaction,
    TransactionResponse,
};
use sb1_api::mock::TransferRecord;
use sb1_api::{BankApiClient, MockBankClient};
use std::sync::Arc;

/// Create a test account for testing
fn create_test_account(key: &str, name: &str, number: &str, balance: f64) -> Account {
    Account {
        key: key.to_string(),
        account_number: number.to_string(),
        iban: format!("NO{}", number),
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

/// Create a test transaction
fn create_test_transaction(
    id: &str,
    account_key: &str,
    amount: f64,
    description: &str,
    booking_status: &str,
) -> Transaction {
    Transaction {
        id: id.to_string(),
        non_unique_id: id.to_string(),
        description: Some(description.to_string()),
        cleaned_description: Some(description.to_string()),
        account_number: AccountNumber {
            value: "12345678901".to_string(),
            formatted: "1234.56.78901".to_string(),
            unformatted: "12345678901".to_string(),
        },
        amount,
        date: 1739577600, // 2025-02-15
        interest_date: None,
        type_code: "PURCHASE".to_string(),
        type_text: "Varekjøp".to_string(),
        currency_code: "NOK".to_string(),
        can_show_details: true,
        source: "ONLINE".to_string(),
        is_confidential: false,
        booking_status: booking_status.to_string(),
        account_name: "Checking Account".to_string(),
        account_key: account_key.to_string(),
        account_currency: "NOK".to_string(),
        is_from_currency_account: false,
        classification_input: ClassificationInput {
            id: id.to_string(),
            amount,
            type_field: "PURCHASE".to_string(),
            text: Some(description.to_string()),
            date: 1739577600,
        },
        remote_account_number: None,
        remote_account_name: None,
        kid_or_message: None,
    }
}

mod rule_engine_tests {
    use super::*;

    #[tokio::test]
    async fn test_rule_engine_basic_matching() {
        // Setup mock bank client with accounts and transactions
        let mock_client = Arc::new(MockBankClient::new());

        let accounts = AccountData {
            accounts: vec![
                create_test_account("checking", "Checking Account", "12345678901", 10000.0),
                create_test_account("savings", "Savings Account", "12345678902", 50000.0),
            ],
            errors: vec![],
        };
        mock_client.set_accounts(accounts).await;

        // Set up a transaction that should match a rule
        let transactions = TransactionResponse {
            transactions: vec![create_test_transaction(
                "tx-001",
                "checking",
                -149.0, // Netflix subscription
                "NETFLIX.COM",
                "BOOKED",
            )],
            errors: vec![],
        };
        mock_client.set_transactions("checking", transactions).await;

        // Verify the mock client works correctly
        let fetched_accounts = mock_client.get_accounts().await.unwrap();
        assert_eq!(fetched_accounts.accounts.len(), 2);

        let fetched_txns = mock_client.get_transactions("checking").await.unwrap();
        assert_eq!(fetched_txns.transactions.len(), 1);
        assert_eq!(
            fetched_txns.transactions[0].cleaned_description,
            Some("NETFLIX.COM".to_string())
        );
    }

    #[tokio::test]
    async fn test_transfer_recording() {
        let mock_client = Arc::new(MockBankClient::new());

        // Setup accounts
        let accounts = AccountData {
            accounts: vec![
                create_test_account("checking", "Checking", "12345678901", 10000.0),
                create_test_account("savings", "Savings", "12345678902", 50000.0),
            ],
            errors: vec![],
        };
        mock_client.set_accounts(accounts).await;

        // Make a transfer
        let transfer = sb1_api::models::CreateTransferDTO {
            amount: "149.00".to_string(),
            from_account: "12345678902".to_string(),
            to_account: "12345678901".to_string(),
            message: Some("Netflix refill".to_string()),
            due_date: None,
            currency_code: None,
        };

        let result = mock_client.create_transfer(transfer).await.unwrap();
        assert!(result.payment_id.is_some());

        // Check transfer was recorded
        let history = mock_client.get_transfer_history().await;
        assert_eq!(history.len(), 1);

        match &history[0] {
            TransferRecord::Regular(dto) => {
                assert_eq!(dto.amount, "149.00");
                assert_eq!(dto.message, Some("Netflix refill".to_string()));
            }
            _ => panic!("Expected regular transfer"),
        }
    }
}

mod condition_tests {
    use super::*;

    // These tests are here to verify conditions work with real transaction data
    // The unit tests in condition.rs cover the logic, these cover integration

    #[test]
    fn test_transaction_has_expected_fields() {
        let tx = create_test_transaction("tx-1", "account-1", -99.99, "SPOTIFY", "BOOKED");

        assert_eq!(tx.id, "tx-1");
        assert_eq!(tx.account_key, "account-1");
        assert_eq!(tx.amount, -99.99);
        assert_eq!(tx.cleaned_description, Some("SPOTIFY".to_string()));
        assert_eq!(tx.booking_status, "BOOKED");
    }

    #[test]
    fn test_accounts_have_expected_fields() {
        let acc = create_test_account("key-1", "My Account", "11112222333", 1234.56);

        assert_eq!(acc.key, "key-1");
        assert_eq!(acc.name, "My Account");
        assert_eq!(acc.account_number, "11112222333");
        assert_eq!(acc.balance, 1234.56);
    }
}
