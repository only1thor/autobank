//! Model serialization/deserialization tests.

use sb1_api::models::*;

const ACCOUNT_JSON: &str = r#"{
    "key": "acc-123",
    "accountNumber": "12345678901",
    "iban": "NO9312345678901",
    "name": "My Checking Account",
    "description": "Main account",
    "balance": 15000.50,
    "availableBalance": 15000.50,
    "currencyCode": "NOK",
    "owner": {
        "name": "John Doe",
        "firstName": "John",
        "lastName": "Doe",
        "age": 30,
        "customerKey": "cust-123",
        "ssnKey": "ssn-123"
    },
    "productType": "CURRENT_ACCOUNT",
    "type": "ACCOUNT",
    "productId": "prod-123",
    "descriptionCode": "desc-001",
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
        "isDefaultPaymentAccount": true
    },
    "creditCardCreditLimit": null,
    "creditCardAccountID": null
}"#;

const TRANSACTION_JSON: &str = r#"{
    "id": "tx-12345",
    "nonUniqueId": "tx-nu-12345",
    "description": "NETFLIX.COM",
    "cleanedDescription": "Netflix Subscription",
    "accountNumber": {
        "value": "12345678901",
        "formatted": "1234.56.78901",
        "unformatted": "12345678901"
    },
    "amount": -149.00,
    "date": 1707753600000,
    "interestDate": 1707753600000,
    "typeCode": "VISA",
    "typeText": "Card payment",
    "currencyCode": "NOK",
    "canShowDetails": true,
    "source": "VISA",
    "isConfidential": false,
    "bookingStatus": "BOOKED",
    "accountName": "My Checking Account",
    "accountKey": "acc-123",
    "accountCurrency": "NOK",
    "isFromCurrencyAccount": false,
    "classificationInput": {
        "id": "class-123",
        "amount": -149.00,
        "type": "EXPENSE",
        "text": "Entertainment",
        "date": 1707753600000
    },
    "remoteAccountNumber": null,
    "remoteAccountName": "Netflix",
    "kidOrMessage": null
}"#;

const TRANSFER_RESPONSE_JSON: &str = r#"{
    "errors": [],
    "paymentId": "pay-12345",
    "status": "COMPLETED"
}"#;

const TRANSFER_ERROR_RESPONSE_JSON: &str = r#"{
    "errors": [{
        "code": "INSUFFICIENT_FUNDS",
        "message": "Not enough balance",
        "traceId": "trace-123",
        "httpCode": 400,
        "resource": "/transfer/debit",
        "localizedMessage": {
            "locale": "no",
            "message": "Ikke nok penger"
        }
    }],
    "paymentId": null,
    "status": null
}"#;

#[test]
fn test_deserialize_account() {
    let account: Account = serde_json::from_str(ACCOUNT_JSON).expect("Failed to parse account");

    assert_eq!(account.key, "acc-123");
    assert_eq!(account.account_number, "12345678901");
    assert_eq!(account.name, "My Checking Account");
    assert_eq!(account.balance, 15000.50);
    assert_eq!(account.type_field, "ACCOUNT");
    assert!(account.account_properties.is_transfer_from_enabled);
    assert!(account.account_properties.is_default_payment_account);

    let owner = account.owner.expect("Owner should be present");
    assert_eq!(owner.name, "John Doe");
    assert_eq!(owner.age, 30);
}

#[test]
fn test_deserialize_transaction() {
    let tx: Transaction = serde_json::from_str(TRANSACTION_JSON).expect("Failed to parse transaction");

    assert_eq!(tx.id, "tx-12345");
    assert_eq!(tx.description, Some("NETFLIX.COM".to_string()));
    assert_eq!(tx.cleaned_description, Some("Netflix Subscription".to_string()));
    assert_eq!(tx.amount, -149.00);
    assert_eq!(tx.type_code, "VISA");
    assert_eq!(tx.booking_status, "BOOKED");
    assert_eq!(tx.account_number.formatted, "1234.56.78901");
}

#[test]
fn test_deserialize_transfer_response_success() {
    let response: TransferResponse =
        serde_json::from_str(TRANSFER_RESPONSE_JSON).expect("Failed to parse transfer response");

    assert!(response.errors.is_empty());
    assert_eq!(response.payment_id, Some("pay-12345".to_string()));
    assert_eq!(response.status, Some("COMPLETED".to_string()));
}

#[test]
fn test_deserialize_transfer_response_error() {
    let response: TransferResponse =
        serde_json::from_str(TRANSFER_ERROR_RESPONSE_JSON).expect("Failed to parse error response");

    assert_eq!(response.errors.len(), 1);
    assert!(response.payment_id.is_none());

    let error = &response.errors[0];
    assert_eq!(error.code, "INSUFFICIENT_FUNDS");
    assert_eq!(error.http_code, 400);
    assert!(error.localized_message.is_some());
}

#[test]
fn test_serialize_create_transfer_dto() {
    let dto = CreateTransferDTO {
        amount: "100.50".to_string(),
        due_date: None,
        message: Some("Test payment".to_string()),
        to_account: "98765432101".to_string(),
        from_account: "12345678901".to_string(),
        currency_code: None,
    };

    let json = serde_json::to_string(&dto).expect("Failed to serialize");

    // Verify camelCase
    assert!(json.contains("\"toAccount\""));
    assert!(json.contains("\"fromAccount\""));
    // Skip serializing None values
    assert!(!json.contains("dueDate"));
    assert!(!json.contains("currencyCode"));
}

#[test]
fn test_serialize_credit_card_transfer_dto() {
    let dto = TransferToCreditCardDTO {
        amount: "500.00".to_string(),
        due_date: Some("2024-02-15".to_string()),
        from_account: "12345678901".to_string(),
        credit_card_account_id: "cc-123".to_string(),
    };

    let json = serde_json::to_string(&dto).expect("Failed to serialize");

    assert!(json.contains("\"creditCardAccountId\""));
    assert!(json.contains("\"dueDate\""));
}

#[test]
fn test_account_data_with_errors() {
    let json = r#"{
        "accounts": [],
        "errors": [{"code": "SOME_ERROR", "message": "Something went wrong"}]
    }"#;

    let data: AccountData = serde_json::from_str(json).expect("Failed to parse");
    assert!(data.accounts.is_empty());
    assert_eq!(data.errors.len(), 1);
}

#[test]
fn test_transaction_response_empty() {
    let json = r#"{
        "transactions": [],
        "errors": []
    }"#;

    let response: TransactionResponse = serde_json::from_str(json).expect("Failed to parse");
    assert!(response.transactions.is_empty());
    assert!(response.errors.is_empty());
}
