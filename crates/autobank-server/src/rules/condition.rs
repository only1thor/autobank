//! Condition evaluation logic.

use super::types::{AmountSpec, Condition};
use regex::Regex;
use sb1_api::models::Transaction;

impl Condition {
    /// Evaluate this condition against a transaction.
    pub fn evaluate(&self, tx: &Transaction) -> bool {
        match self {
            Condition::DescriptionMatches { pattern, case_insensitive } => {
                let description = tx
                    .cleaned_description
                    .as_deref()
                    .or(tx.description.as_deref())
                    .unwrap_or("");

                let regex_pattern = if *case_insensitive {
                    format!("(?i){}", pattern)
                } else {
                    pattern.clone()
                };

                Regex::new(&regex_pattern)
                    .map(|re| re.is_match(description))
                    .unwrap_or(false)
            }

            Condition::AmountGreaterThan { value } => tx.amount > *value,

            Condition::AmountLessThan { value } => tx.amount < *value,

            Condition::AmountBetween { min, max } => tx.amount >= *min && tx.amount <= *max,

            Condition::AmountEquals { value, tolerance } => (tx.amount - value).abs() <= *tolerance,

            Condition::TransactionType { type_code } => tx.type_code == *type_code,

            Condition::IsSettled => tx.booking_status == "BOOKED",

            Condition::And { conditions } => conditions.iter().all(|c| c.evaluate(tx)),

            Condition::Or { conditions } => conditions.iter().any(|c| c.evaluate(tx)),

            Condition::Not { condition } => !condition.evaluate(tx),
        }
    }
}

impl AmountSpec {
    /// Calculate the amount for a transfer based on the transaction.
    pub fn calculate(&self, tx: &Transaction) -> f64 {
        match self {
            AmountSpec::Fixed { value } => *value,

            AmountSpec::TransactionAmount => tx.amount,

            AmountSpec::TransactionAmountAbs => tx.amount.abs(),

            AmountSpec::Percentage { of_transaction } => tx.amount.abs() * (of_transaction / 100.0),

            AmountSpec::Min { specs } => specs
                .iter()
                .map(|s| s.calculate(tx))
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),

            AmountSpec::Max { specs } => specs
                .iter()
                .map(|s| s.calculate(tx))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sb1_api::models::{AccountNumber, ClassificationInput};

    fn create_test_transaction(amount: f64, description: &str, booking_status: &str) -> Transaction {
        Transaction {
            id: "tx-1".to_string(),
            non_unique_id: "tx-nu-1".to_string(),
            description: Some(description.to_string()),
            cleaned_description: Some(description.to_string()),
            account_number: AccountNumber {
                value: "12345678901".to_string(),
                formatted: "1234.56.78901".to_string(),
                unformatted: "12345678901".to_string(),
            },
            amount,
            date: 1707753600000,
            interest_date: None,
            type_code: "VISA".to_string(),
            type_text: "Card payment".to_string(),
            currency_code: "NOK".to_string(),
            can_show_details: true,
            source: "VISA".to_string(),
            is_confidential: false,
            booking_status: booking_status.to_string(),
            account_name: "Checking".to_string(),
            account_key: "acc-1".to_string(),
            account_currency: "NOK".to_string(),
            is_from_currency_account: false,
            classification_input: ClassificationInput {
                id: "class-1".to_string(),
                amount,
                type_field: "EXPENSE".to_string(),
                text: None,
                date: 1707753600000,
            },
            remote_account_number: None,
            remote_account_name: Some("Netflix".to_string()),
            kid_or_message: None,
        }
    }

    #[test]
    fn test_description_matches() {
        let tx = create_test_transaction(-149.0, "NETFLIX.COM payment", "BOOKED");

        let condition = Condition::DescriptionMatches {
            pattern: "netflix".to_string(),
            case_insensitive: true,
        };
        assert!(condition.evaluate(&tx));

        let condition_case_sensitive = Condition::DescriptionMatches {
            pattern: "netflix".to_string(),
            case_insensitive: false,
        };
        assert!(!condition_case_sensitive.evaluate(&tx));
    }

    #[test]
    fn test_amount_conditions() {
        let tx = create_test_transaction(-149.0, "Test", "BOOKED");

        assert!(Condition::AmountLessThan { value: 0.0 }.evaluate(&tx));
        assert!(Condition::AmountGreaterThan { value: -200.0 }.evaluate(&tx));
        assert!(Condition::AmountBetween { min: -200.0, max: -100.0 }.evaluate(&tx));
        assert!(Condition::AmountEquals { value: -149.0, tolerance: 0.01 }.evaluate(&tx));
    }

    #[test]
    fn test_is_settled() {
        let booked_tx = create_test_transaction(-100.0, "Test", "BOOKED");
        let pending_tx = create_test_transaction(-100.0, "Test", "PENDING");

        assert!(Condition::IsSettled.evaluate(&booked_tx));
        assert!(!Condition::IsSettled.evaluate(&pending_tx));
    }

    #[test]
    fn test_logical_operators() {
        let tx = create_test_transaction(-149.0, "Netflix", "BOOKED");

        let and_condition = Condition::And {
            conditions: vec![
                Condition::AmountLessThan { value: 0.0 },
                Condition::IsSettled,
            ],
        };
        assert!(and_condition.evaluate(&tx));

        let or_condition = Condition::Or {
            conditions: vec![
                Condition::AmountGreaterThan { value: 1000.0 },
                Condition::IsSettled,
            ],
        };
        assert!(or_condition.evaluate(&tx));

        let not_condition = Condition::Not {
            condition: Box::new(Condition::AmountGreaterThan { value: 0.0 }),
        };
        assert!(not_condition.evaluate(&tx));
    }

    #[test]
    fn test_amount_spec_calculation() {
        let tx = create_test_transaction(-149.0, "Test", "BOOKED");

        assert_eq!(AmountSpec::Fixed { value: 100.0 }.calculate(&tx), 100.0);
        assert_eq!(AmountSpec::TransactionAmount.calculate(&tx), -149.0);
        assert_eq!(AmountSpec::TransactionAmountAbs.calculate(&tx), 149.0);
        assert!((AmountSpec::Percentage { of_transaction: 10.0 }.calculate(&tx) - 14.9).abs() < 0.01);
    }
}
