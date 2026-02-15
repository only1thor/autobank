//! Rule engine for evaluating and executing rules.

use super::types::{AccountRef, Action, AmountSpec, ProcessingDecision, Rule, RuleExecution, RuleTransactionLog, TrackedTransaction};
use crate::db::Database;
use sb1_api::models::{Account, CreateTransferDTO, Transaction};
use sb1_api::BankApiClient;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Transaction fingerprint for change detection.
pub struct TransactionFingerprint {
    pub transaction_id: String,
    pub fingerprint: String,
}

impl TransactionFingerprint {
    /// Create a fingerprint from a transaction.
    pub fn from_transaction(tx: &Transaction) -> Self {
        let content = format!(
            "{}|{}|{}|{}|{}",
            tx.id,
            tx.cleaned_description.as_deref().unwrap_or(""),
            tx.amount,
            tx.type_code,
            tx.booking_status
        );

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let fingerprint = hex::encode(hasher.finalize());

        Self {
            transaction_id: tx.id.clone(),
            fingerprint,
        }
    }
}

/// Rule engine for evaluating and executing rules.
pub struct RuleEngine {
    db: Database,
    bank_client: Arc<dyn BankApiClient>,
}

impl RuleEngine {
    /// Create a new rule engine.
    pub fn new(db: Database, bank_client: Arc<dyn BankApiClient>) -> Self {
        Self { db, bank_client }
    }

    /// Evaluate all enabled rules against recent transactions.
    pub async fn evaluate_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rules_by_account = self.db.get_enabled_rules_by_account().await?;

        for (account_key, rules) in rules_by_account {
            debug!("Processing {} rules for account {}", rules.len(), account_key);

            let transactions = match self.bank_client.get_transactions(&account_key).await {
                Ok(response) => response.transactions,
                Err(e) => {
                    error!("Failed to fetch transactions for account {}: {}", account_key, e);
                    continue;
                }
            };

            for tx in transactions {
                let fingerprint = TransactionFingerprint::from_transaction(&tx);
                let decision = self.check_processing_decision(&tx, &fingerprint).await?;

                match decision {
                    ProcessingDecision::Skip { reason } => {
                        debug!("Skipping transaction {}: {}", tx.id, reason);
                        continue;
                    }
                    ProcessingDecision::Wait { reason } => {
                        debug!("Waiting on transaction {}: {}", tx.id, reason);
                        continue;
                    }
                    ProcessingDecision::Process => {
                        self.update_tracked_transaction(&tx, &fingerprint).await?;

                        for rule in &rules {
                            if let Err(e) = self.evaluate_and_execute(rule, &tx, &fingerprint).await {
                                error!("Error evaluating rule {} for transaction {}: {}", rule.id, tx.id, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a transaction should be processed.
    async fn check_processing_decision(
        &self,
        tx: &Transaction,
        fingerprint: &TransactionFingerprint,
    ) -> Result<ProcessingDecision, Box<dyn std::error::Error + Send + Sync>> {
        let tracked = self.db.get_tracked_transaction(&tx.id).await?;

        match tracked {
            None => {
                // New transaction
                Ok(ProcessingDecision::Process)
            }
            Some(existing) => {
                if existing.fingerprint == fingerprint.fingerprint {
                    // Same version, already processed
                    Ok(ProcessingDecision::Skip {
                        reason: "Already processed this version".to_string(),
                    })
                } else {
                    // Transaction changed, re-evaluate
                    Ok(ProcessingDecision::Process)
                }
            }
        }
    }

    /// Update the tracked transaction record.
    async fn update_tracked_transaction(
        &self,
        tx: &Transaction,
        fingerprint: &TransactionFingerprint,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp();
        let raw_data = serde_json::to_string(tx)?;

        let tracked = TrackedTransaction {
            id: tx.id.clone(),
            account_key: tx.account_key.clone(),
            fingerprint: fingerprint.fingerprint.clone(),
            first_seen_at: now,
            last_updated_at: now,
            settled: tx.booking_status == "BOOKED",
            raw_data,
        };

        self.db.upsert_tracked_transaction(&tracked).await?;
        Ok(())
    }

    /// Evaluate a rule against a transaction and execute if matched.
    async fn evaluate_and_execute(
        &self,
        rule: &Rule,
        tx: &Transaction,
        fingerprint: &TransactionFingerprint,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if already processed
        if self.db.has_processed(&rule.id, &tx.id, &fingerprint.fingerprint).await? {
            debug!("Rule {} already processed transaction {} with this fingerprint", rule.id, tx.id);
            return Ok(());
        }

        // Evaluate conditions
        let all_match = rule.conditions.iter().all(|c| c.evaluate(tx));

        let now = chrono::Utc::now().timestamp();

        if !all_match {
            // Record skip
            let log = RuleTransactionLog {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                transaction_id: tx.id.clone(),
                transaction_fingerprint: fingerprint.fingerprint.clone(),
                action_taken: "skipped".to_string(),
                processed_at: now,
            };
            self.db.record_processing(&log).await?;
            return Ok(());
        }

        info!("Rule '{}' matched transaction {}", rule.name, tx.id);

        // Execute actions
        for action in &rule.actions {
            self.execute_action(rule, tx, action, fingerprint).await?;
        }

        Ok(())
    }

    /// Execute a single action.
    async fn execute_action(
        &self,
        rule: &Rule,
        tx: &Transaction,
        action: &Action,
        fingerprint: &TransactionFingerprint,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match action {
            Action::Transfer {
                from_account,
                to_account,
                amount,
                message,
            } => {
                self.execute_transfer(rule, tx, from_account, to_account, amount, message.clone(), fingerprint).await
            }
        }
    }

    /// Execute a transfer action.
    async fn execute_transfer(
        &self,
        rule: &Rule,
        tx: &Transaction,
        from_account: &AccountRef,
        to_account: &AccountRef,
        amount_spec: &AmountSpec,
        message: Option<String>,
        fingerprint: &TransactionFingerprint,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let accounts = self.bank_client.get_accounts().await?.accounts;
        let now = chrono::Utc::now().timestamp();

        let from_acc = self.resolve_account_ref(from_account, &rule.trigger_account_key, &accounts)?;
        let to_acc = self.resolve_account_ref(to_account, &rule.trigger_account_key, &accounts)?;
        let amount = amount_spec.calculate(tx);

        info!(
            "Executing transfer: {} -> {}, amount: {:.2}",
            from_acc.account_number, to_acc.account_number, amount
        );

        let transfer = CreateTransferDTO {
            amount: format!("{:.2}", amount),
            due_date: None,
            message,
            to_account: to_acc.account_number.clone(),
            from_account: from_acc.account_number.clone(),
            currency_code: None,
        };

        let result = self.bank_client.create_transfer(transfer).await;

        let (status, payment_id, error_msg) = match result {
            Ok(response) if response.errors.is_empty() => {
                ("success".to_string(), response.payment_id, None)
            }
            Ok(response) => {
                let err = response.errors.first().map(|e| e.message.clone()).unwrap_or_default();
                ("failed".to_string(), None, Some(err))
            }
            Err(e) => ("failed".to_string(), None, Some(e.to_string())),
        };

        // Record execution
        let execution = RuleExecution {
            id: Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            transaction_id: tx.id.clone(),
            transfer_payment_id: payment_id,
            amount,
            from_account: from_acc.account_number.clone(),
            to_account: to_acc.account_number.clone(),
            status: status.clone(),
            error_message: error_msg.clone(),
            executed_at: now,
        };
        self.db.record_execution(&execution).await?;

        // Record processing
        let log = RuleTransactionLog {
            id: Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            transaction_id: tx.id.clone(),
            transaction_fingerprint: fingerprint.fingerprint.clone(),
            action_taken: format!("executed:{}", status),
            processed_at: now,
        };
        self.db.record_processing(&log).await?;

        if let Some(err) = error_msg {
            warn!("Transfer failed: {}", err);
        }

        Ok(())
    }

    /// Resolve an account reference to an actual account.
    fn resolve_account_ref<'a>(
        &self,
        account_ref: &AccountRef,
        trigger_account_key: &str,
        accounts: &'a [Account],
    ) -> Result<&'a Account, Box<dyn std::error::Error + Send + Sync>> {
        match account_ref {
            AccountRef::TriggerAccount => accounts
                .iter()
                .find(|a| a.key == trigger_account_key)
                .ok_or_else(|| format!("Trigger account {} not found", trigger_account_key).into()),

            AccountRef::ByKey { key } => accounts
                .iter()
                .find(|a| a.key == *key)
                .ok_or_else(|| format!("Account with key {} not found", key).into()),

            AccountRef::ByNumber { number } => accounts
                .iter()
                .find(|a| a.account_number == *number)
                .ok_or_else(|| format!("Account with number {} not found", number).into()),
        }
    }
}
