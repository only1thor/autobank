//! Database migrations.

/// All database migrations in order.
pub const MIGRATIONS: &[&str] = &[
    // Migration 001: Initial schema
    r#"
-- Rules table
CREATE TABLE IF NOT EXISTS rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    trigger_account_key TEXT NOT NULL,
    conditions TEXT NOT NULL,
    actions TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Transaction tracking for deduplication
CREATE TABLE IF NOT EXISTS tracked_transactions (
    id TEXT PRIMARY KEY,
    account_key TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    first_seen_at INTEGER NOT NULL,
    last_updated_at INTEGER NOT NULL,
    settled INTEGER NOT NULL DEFAULT 0,
    raw_data TEXT NOT NULL
);

-- Rule-transaction processing record
CREATE TABLE IF NOT EXISTS rule_transaction_log (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(id),
    transaction_id TEXT NOT NULL,
    transaction_fingerprint TEXT NOT NULL,
    action_taken TEXT NOT NULL,
    processed_at INTEGER NOT NULL,
    UNIQUE(rule_id, transaction_id, transaction_fingerprint)
);

-- Rule execution history
CREATE TABLE IF NOT EXISTS rule_executions (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(id),
    transaction_id TEXT NOT NULL,
    transfer_payment_id TEXT,
    amount REAL NOT NULL,
    from_account TEXT NOT NULL,
    to_account TEXT NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    executed_at INTEGER NOT NULL
);

-- Audit log
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    actor TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    details TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tracked_transactions_account ON tracked_transactions(account_key);
CREATE INDEX IF NOT EXISTS idx_tracked_transactions_settled ON tracked_transactions(settled);
CREATE INDEX IF NOT EXISTS idx_rule_transaction_log_rule ON rule_transaction_log(rule_id);
CREATE INDEX IF NOT EXISTS idx_rule_executions_rule ON rule_executions(rule_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON audit_log(event_type);
"#,
];
