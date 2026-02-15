//! Rule and related types.

use serde::{Deserialize, Serialize};

/// A rule that triggers actions based on transaction conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub trigger_account_key: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Rule condition types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Match transaction description with regex pattern.
    DescriptionMatches {
        pattern: String,
        #[serde(default)]
        case_insensitive: bool,
    },

    /// Amount greater than value.
    AmountGreaterThan { value: f64 },

    /// Amount less than value.
    AmountLessThan { value: f64 },

    /// Amount between min and max (inclusive).
    AmountBetween { min: f64, max: f64 },

    /// Amount equals value within tolerance.
    AmountEquals {
        value: f64,
        #[serde(default = "default_tolerance")]
        tolerance: f64,
    },

    /// Transaction type code matches.
    TransactionType { type_code: String },

    /// Only trigger on settled transactions.
    IsSettled,

    /// Logical AND of multiple conditions.
    And { conditions: Vec<Condition> },

    /// Logical OR of multiple conditions.
    Or { conditions: Vec<Condition> },

    /// Logical NOT of a condition.
    Not { condition: Box<Condition> },
}

fn default_tolerance() -> f64 {
    0.01
}

/// Rule action types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Transfer money between accounts.
    Transfer {
        from_account: AccountRef,
        to_account: AccountRef,
        amount: AmountSpec,
        message: Option<String>,
    },
}

/// Reference to an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountRef {
    /// Reference by account key.
    ByKey { key: String },
    /// Reference by account number.
    ByNumber { number: String },
    /// The account being monitored (trigger account).
    TriggerAccount,
}

/// Specification for transfer amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AmountSpec {
    /// Fixed amount.
    Fixed { value: f64 },
    /// Same amount as the transaction.
    TransactionAmount,
    /// Absolute value of the transaction amount.
    TransactionAmountAbs,
    /// Percentage of the transaction amount.
    Percentage { of_transaction: f64 },
    /// Minimum of multiple specs.
    Min { specs: Vec<AmountSpec> },
    /// Maximum of multiple specs.
    Max { specs: Vec<AmountSpec> },
}

/// Tracked transaction for deduplication.
#[derive(Debug, Clone)]
pub struct TrackedTransaction {
    pub id: String,
    pub account_key: String,
    pub fingerprint: String,
    pub first_seen_at: i64,
    pub last_updated_at: i64,
    pub settled: bool,
    pub raw_data: String,
}

/// Log entry for rule-transaction processing.
#[derive(Debug, Clone)]
pub struct RuleTransactionLog {
    pub id: String,
    pub rule_id: String,
    pub transaction_id: String,
    pub transaction_fingerprint: String,
    pub action_taken: String,
    pub processed_at: i64,
}

/// Record of a rule execution (successful transfer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecution {
    pub id: String,
    pub rule_id: String,
    pub transaction_id: String,
    pub transfer_payment_id: Option<String>,
    pub amount: f64,
    pub from_account: String,
    pub to_account: String,
    pub status: String,
    pub error_message: Option<String>,
    pub executed_at: i64,
}

/// Decision on whether to process a transaction.
#[derive(Debug, Clone)]
pub enum ProcessingDecision {
    /// Process the transaction (new or meaningful change).
    Process,
    /// Skip processing (already handled this version).
    Skip { reason: String },
    /// Wait for more data (transaction not settled).
    Wait { reason: String },
}
