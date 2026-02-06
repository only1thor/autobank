use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub transactions: Vec<Transaction>,
    pub errors: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub non_unique_id: String,
    pub description: Option<String>,
    pub cleaned_description: Option<String>,
    pub account_number: AccountNumber,
    pub amount: f64,
    pub date: i64,
    pub interest_date: Option<i64>,
    pub type_code: String,
    pub type_text: String,
    pub currency_code: String,
    pub can_show_details: bool,
    pub source: String,
    pub is_confidential: bool,
    pub booking_status: String,
    pub account_name: String,
    pub account_key: String,
    pub account_currency: String,
    pub is_from_currency_account: bool,
    pub classification_input: ClassificationInput,
    pub remote_account_number: Option<String>,
    pub remote_account_name: Option<String>,
    pub kid_or_message: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountNumber {
    pub value: String,
    pub formatted: String,
    pub unformatted: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationInput {
    pub id: String,
    pub amount: f64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub text: Option<String>,
    pub date: i64,
}
