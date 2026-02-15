//! Database repository implementation.

use crate::audit::AuditEntry;
use crate::rules::{Rule, RuleExecution, RuleTransactionLog, TrackedTransaction};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Database connection pool and operations.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the database.
    pub async fn connect(url: &str) -> Result<Self, DbError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;

        Ok(Self { pool })
    }

    /// Run all migrations.
    pub async fn run_migrations(&self) -> Result<(), DbError> {
        for (i, migration) in super::MIGRATIONS.iter().enumerate() {
            info!("Running migration {}", i + 1);
            sqlx::raw_sql(migration).execute(&self.pool).await?;
        }
        Ok(())
    }

    // --- Rules ---

    /// List all rules.
    pub async fn list_rules(&self) -> Result<Vec<Rule>, DbError> {
        let rows = sqlx::query_as::<_, RuleRow>(
            "SELECT id, name, description, enabled, trigger_account_key, conditions, actions, created_at, updated_at FROM rules ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get a rule by ID.
    pub async fn get_rule(&self, id: &str) -> Result<Option<Rule>, DbError> {
        let row = sqlx::query_as::<_, RuleRow>(
            "SELECT id, name, description, enabled, trigger_account_key, conditions, actions, created_at, updated_at FROM rules WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Get all enabled rules grouped by trigger account.
    pub async fn get_enabled_rules_by_account(&self) -> Result<std::collections::HashMap<String, Vec<Rule>>, DbError> {
        let rules = sqlx::query_as::<_, RuleRow>(
            "SELECT id, name, description, enabled, trigger_account_key, conditions, actions, created_at, updated_at FROM rules WHERE enabled = 1"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map: std::collections::HashMap<String, Vec<Rule>> = std::collections::HashMap::new();
        for row in rules {
            let rule: Rule = row.try_into()?;
            map.entry(rule.trigger_account_key.clone())
                .or_default()
                .push(rule);
        }
        Ok(map)
    }

    /// Create a new rule.
    pub async fn create_rule(&self, rule: &Rule) -> Result<(), DbError> {
        let conditions = serde_json::to_string(&rule.conditions)?;
        let actions = serde_json::to_string(&rule.actions)?;

        sqlx::query(
            "INSERT INTO rules (id, name, description, enabled, trigger_account_key, conditions, actions, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(rule.enabled)
        .bind(&rule.trigger_account_key)
        .bind(&conditions)
        .bind(&actions)
        .bind(rule.created_at)
        .bind(rule.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update a rule.
    pub async fn update_rule(&self, rule: &Rule) -> Result<(), DbError> {
        let conditions = serde_json::to_string(&rule.conditions)?;
        let actions = serde_json::to_string(&rule.actions)?;

        sqlx::query(
            "UPDATE rules SET name = ?, description = ?, enabled = ?, trigger_account_key = ?, conditions = ?, actions = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(rule.enabled)
        .bind(&rule.trigger_account_key)
        .bind(&conditions)
        .bind(&actions)
        .bind(rule.updated_at)
        .bind(&rule.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a rule.
    pub async fn delete_rule(&self, id: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM rules WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Set rule enabled status.
    pub async fn set_rule_enabled(&self, id: &str, enabled: bool) -> Result<(), DbError> {
        sqlx::query("UPDATE rules SET enabled = ?, updated_at = ? WHERE id = ?")
            .bind(enabled)
            .bind(chrono::Utc::now().timestamp())
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Tracked Transactions ---

    /// Get a tracked transaction by ID.
    pub async fn get_tracked_transaction(&self, id: &str) -> Result<Option<TrackedTransaction>, DbError> {
        let row = sqlx::query_as::<_, TrackedTransactionRow>(
            "SELECT id, account_key, fingerprint, first_seen_at, last_updated_at, settled, raw_data FROM tracked_transactions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// Upsert a tracked transaction.
    pub async fn upsert_tracked_transaction(&self, tx: &TrackedTransaction) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO tracked_transactions (id, account_key, fingerprint, first_seen_at, last_updated_at, settled, raw_data) 
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET fingerprint = excluded.fingerprint, last_updated_at = excluded.last_updated_at, settled = excluded.settled, raw_data = excluded.raw_data"
        )
        .bind(&tx.id)
        .bind(&tx.account_key)
        .bind(&tx.fingerprint)
        .bind(tx.first_seen_at)
        .bind(tx.last_updated_at)
        .bind(tx.settled)
        .bind(&tx.raw_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // --- Rule Transaction Log ---

    /// Check if a rule+transaction+fingerprint has been processed.
    pub async fn has_processed(&self, rule_id: &str, tx_id: &str, fingerprint: &str) -> Result<bool, DbError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM rule_transaction_log WHERE rule_id = ? AND transaction_id = ? AND transaction_fingerprint = ?"
        )
        .bind(rule_id)
        .bind(tx_id)
        .bind(fingerprint)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }

    /// Record a rule processing event.
    pub async fn record_processing(&self, log: &RuleTransactionLog) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO rule_transaction_log (id, rule_id, transaction_id, transaction_fingerprint, action_taken, processed_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&log.id)
        .bind(&log.rule_id)
        .bind(&log.transaction_id)
        .bind(&log.transaction_fingerprint)
        .bind(&log.action_taken)
        .bind(log.processed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // --- Rule Executions ---

    /// Record a rule execution.
    pub async fn record_execution(&self, exec: &RuleExecution) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO rule_executions (id, rule_id, transaction_id, transfer_payment_id, amount, from_account, to_account, status, error_message, executed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&exec.id)
        .bind(&exec.rule_id)
        .bind(&exec.transaction_id)
        .bind(&exec.transfer_payment_id)
        .bind(exec.amount)
        .bind(&exec.from_account)
        .bind(&exec.to_account)
        .bind(&exec.status)
        .bind(&exec.error_message)
        .bind(exec.executed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get executions for a rule.
    pub async fn get_rule_executions(&self, rule_id: &str) -> Result<Vec<RuleExecution>, DbError> {
        let rows = sqlx::query_as::<_, RuleExecutionRow>(
            "SELECT id, rule_id, transaction_id, transfer_payment_id, amount, from_account, to_account, status, error_message, executed_at FROM rule_executions WHERE rule_id = ? ORDER BY executed_at DESC"
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get all recent executions.
    pub async fn list_executions(&self, limit: i64) -> Result<Vec<RuleExecution>, DbError> {
        let rows = sqlx::query_as::<_, RuleExecutionRow>(
            "SELECT id, rule_id, transaction_id, transfer_payment_id, amount, from_account, to_account, status, error_message, executed_at FROM rule_executions ORDER BY executed_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get a single execution by ID.
    pub async fn get_execution(&self, id: &str) -> Result<Option<RuleExecution>, DbError> {
        let row = sqlx::query_as::<_, RuleExecutionRow>(
            "SELECT id, rule_id, transaction_id, transfer_payment_id, amount, from_account, to_account, status, error_message, executed_at FROM rule_executions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    // --- Audit Log ---

    /// Log an audit entry.
    pub async fn log_audit(&self, entry: &AuditEntry) -> Result<(), DbError> {
        let details = serde_json::to_string(&entry.details)?;

        sqlx::query(
            "INSERT INTO audit_log (id, timestamp, event_type, actor, resource_type, resource_id, details, ip_address, user_agent) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&entry.id)
        .bind(entry.timestamp)
        .bind(&entry.event_type)
        .bind(&entry.actor)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(&details)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Query audit log entries.
    pub async fn query_audit(&self, limit: i64) -> Result<Vec<AuditEntry>, DbError> {
        let rows = sqlx::query_as::<_, AuditEntryRow>(
            "SELECT id, timestamp, event_type, actor, resource_type, resource_id, details, ip_address, user_agent FROM audit_log ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get a single audit entry by ID.
    pub async fn get_audit_entry(&self, id: &str) -> Result<Option<AuditEntry>, DbError> {
        let row = sqlx::query_as::<_, AuditEntryRow>(
            "SELECT id, timestamp, event_type, actor, resource_type, resource_id, details, ip_address, user_agent FROM audit_log WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }
}

// --- Row types for SQLx ---

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: String,
    name: String,
    description: Option<String>,
    enabled: bool,
    trigger_account_key: String,
    conditions: String,
    actions: String,
    created_at: i64,
    updated_at: i64,
}

impl TryFrom<RuleRow> for Rule {
    type Error = DbError;

    fn try_from(row: RuleRow) -> Result<Self, Self::Error> {
        Ok(Rule {
            id: row.id,
            name: row.name,
            description: row.description,
            enabled: row.enabled,
            trigger_account_key: row.trigger_account_key,
            conditions: serde_json::from_str(&row.conditions)?,
            actions: serde_json::from_str(&row.actions)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TrackedTransactionRow {
    id: String,
    account_key: String,
    fingerprint: String,
    first_seen_at: i64,
    last_updated_at: i64,
    settled: bool,
    raw_data: String,
}

impl From<TrackedTransactionRow> for TrackedTransaction {
    fn from(row: TrackedTransactionRow) -> Self {
        TrackedTransaction {
            id: row.id,
            account_key: row.account_key,
            fingerprint: row.fingerprint,
            first_seen_at: row.first_seen_at,
            last_updated_at: row.last_updated_at,
            settled: row.settled,
            raw_data: row.raw_data,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleExecutionRow {
    id: String,
    rule_id: String,
    transaction_id: String,
    transfer_payment_id: Option<String>,
    amount: f64,
    from_account: String,
    to_account: String,
    status: String,
    error_message: Option<String>,
    executed_at: i64,
}

impl From<RuleExecutionRow> for RuleExecution {
    fn from(row: RuleExecutionRow) -> Self {
        RuleExecution {
            id: row.id,
            rule_id: row.rule_id,
            transaction_id: row.transaction_id,
            transfer_payment_id: row.transfer_payment_id,
            amount: row.amount,
            from_account: row.from_account,
            to_account: row.to_account,
            status: row.status,
            error_message: row.error_message,
            executed_at: row.executed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AuditEntryRow {
    id: String,
    timestamp: i64,
    event_type: String,
    actor: String,
    resource_type: Option<String>,
    resource_id: Option<String>,
    details: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl TryFrom<AuditEntryRow> for AuditEntry {
    type Error = DbError;

    fn try_from(row: AuditEntryRow) -> Result<Self, Self::Error> {
        Ok(AuditEntry {
            id: row.id,
            timestamp: row.timestamp,
            event_type: row.event_type,
            actor: row.actor,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            details: serde_json::from_str(&row.details)?,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
        })
    }
}
