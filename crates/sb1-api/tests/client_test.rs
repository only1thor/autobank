//! Integration tests for the SpareBank 1 API client using wiremock.

use sb1_api::{BankApiClient, MockTokenProvider, SpareBank1Client};
use std::sync::Arc;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_client() -> (MockServer, SpareBank1Client) {
    let mock_server = MockServer::start().await;
    let token_provider = Arc::new(MockTokenProvider::new("test-token"));
    let client = SpareBank1Client::with_base_url(token_provider, mock_server.uri());
    (mock_server, client)
}

#[tokio::test]
async fn test_get_accounts_success() {
    let (mock_server, client) = setup_client().await;

    let response_body = r#"{
        "accounts": [
            {
                "key": "acc-1",
                "accountNumber": "12345678901",
                "iban": "NO9312345678901",
                "name": "Checking",
                "description": "",
                "balance": 5000.00,
                "availableBalance": 5000.00,
                "currencyCode": "NOK",
                "owner": null,
                "productType": "CURRENT",
                "type": "ACCOUNT",
                "productId": null,
                "descriptionCode": null,
                "accountProperties": {
                    "isTransferFromEnabled": true,
                    "isTransferToEnabled": true,
                    "isPaymentFromEnabled": true,
                    "isAllowedInAvtaleGiro": false,
                    "hasAccess": true,
                    "isBalancePreferred": false,
                    "isFlexiLoan": false,
                    "isCodebitorLoan": false,
                    "isSecurityBalance": false,
                    "isAksjesparekonto": false,
                    "isSavingsAccount": false,
                    "isBonusAccount": false,
                    "userHasRightOfDisposal": true,
                    "userHasRightOfAccess": true,
                    "isOwned": true,
                    "isWithdrawalsAllowed": true,
                    "isBlocked": false,
                    "isHidden": false,
                    "isBalanceUpdatedImmediatelyOnTransferTo": false,
                    "isDefaultPaymentAccount": false
                },
                "creditCardCreditLimit": null,
                "creditCardAccountID": null
            }
        ],
        "errors": []
    }"#;

    Mock::given(method("GET"))
        .and(path("/personal/banking/accounts"))
        .and(query_param("includeCreditCardAccounts", "true"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let result = client.get_accounts().await;
    assert!(result.is_ok());

    let accounts = result.unwrap();
    assert_eq!(accounts.accounts.len(), 1);
    assert_eq!(accounts.accounts[0].name, "Checking");
    assert_eq!(accounts.accounts[0].balance, 5000.00);
}

#[tokio::test]
async fn test_get_transactions_success() {
    let (mock_server, client) = setup_client().await;

    let response_body = r#"{
        "transactions": [
            {
                "id": "tx-1",
                "nonUniqueId": "tx-nu-1",
                "description": "Test Payment",
                "cleanedDescription": "Test Payment",
                "accountNumber": {
                    "value": "12345678901",
                    "formatted": "1234.56.78901",
                    "unformatted": "12345678901"
                },
                "amount": -100.00,
                "date": 1707753600000,
                "interestDate": null,
                "typeCode": "TRANSFER",
                "typeText": "Transfer",
                "currencyCode": "NOK",
                "canShowDetails": true,
                "source": "INTERNAL",
                "isConfidential": false,
                "bookingStatus": "BOOKED",
                "accountName": "Checking",
                "accountKey": "acc-1",
                "accountCurrency": "NOK",
                "isFromCurrencyAccount": false,
                "classificationInput": {
                    "id": "class-1",
                    "amount": -100.00,
                    "type": "EXPENSE",
                    "text": null,
                    "date": 1707753600000
                },
                "remoteAccountNumber": "98765432101",
                "remoteAccountName": "Savings",
                "kidOrMessage": "Test"
            }
        ],
        "errors": []
    }"#;

    Mock::given(method("GET"))
        .and(path("/personal/banking/transactions"))
        .and(query_param("accountKey", "acc-1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let result = client.get_transactions("acc-1").await;
    assert!(result.is_ok());

    let transactions = result.unwrap();
    assert_eq!(transactions.transactions.len(), 1);
    assert_eq!(transactions.transactions[0].id, "tx-1");
    assert_eq!(transactions.transactions[0].amount, -100.00);
}

#[tokio::test]
async fn test_create_transfer_success() {
    let (mock_server, client) = setup_client().await;

    let response_body = r#"{
        "errors": [],
        "paymentId": "pay-123",
        "status": "COMPLETED"
    }"#;

    Mock::given(method("POST"))
        .and(path("/personal/banking/transfer/debit"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let transfer = sb1_api::models::CreateTransferDTO {
        amount: "100.00".to_string(),
        due_date: None,
        message: Some("Test transfer".to_string()),
        to_account: "98765432101".to_string(),
        from_account: "12345678901".to_string(),
        currency_code: None,
    };

    let result = client.create_transfer(transfer).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.errors.is_empty());
    assert_eq!(response.payment_id, Some("pay-123".to_string()));
}

#[tokio::test]
async fn test_create_transfer_error() {
    let (mock_server, client) = setup_client().await;

    let response_body = r#"{
        "errors": [{
            "code": "INSUFFICIENT_FUNDS",
            "message": "Not enough balance",
            "traceId": "trace-123",
            "httpCode": 400,
            "resource": null,
            "localizedMessage": null
        }],
        "paymentId": null,
        "status": null
    }"#;

    Mock::given(method("POST"))
        .and(path("/personal/banking/transfer/debit"))
        .respond_with(ResponseTemplate::new(400).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let transfer = sb1_api::models::CreateTransferDTO {
        amount: "1000000.00".to_string(),
        due_date: None,
        message: None,
        to_account: "98765432101".to_string(),
        from_account: "12345678901".to_string(),
        currency_code: None,
    };

    let result = client.create_transfer(transfer).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_credit_card_transfer() {
    let (mock_server, client) = setup_client().await;

    let response_body = r#"{
        "errors": [],
        "paymentId": "pay-cc-123",
        "status": "COMPLETED"
    }"#;

    Mock::given(method("POST"))
        .and(path("/personal/banking/transfer/creditcard/transferTo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let transfer = sb1_api::models::TransferToCreditCardDTO {
        amount: "500.00".to_string(),
        due_date: None,
        from_account: "12345678901".to_string(),
        credit_card_account_id: "cc-123".to_string(),
    };

    let result = client.create_credit_card_transfer(transfer).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.payment_id, Some("pay-cc-123".to_string()));
}

#[tokio::test]
async fn test_unauthorized_request() {
    let (mock_server, client) = setup_client().await;

    Mock::given(method("GET"))
        .and(path("/personal/banking/accounts"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let result = client.get_accounts().await;
    assert!(result.is_err());
}
