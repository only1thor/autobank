//! Audit trail system.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Audit event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication
    AuthStarted,
    AuthCompleted,
    AuthFailed,
    TokenRefreshed,

    // Rule management
    RuleCreated,
    RuleUpdated,
    RuleDeleted,
    RuleEnabled,
    RuleDisabled,

    // Rule execution
    RuleEvaluated,
    RuleMatched,
    RuleSkipped,
    TransferInitiated,
    TransferSucceeded,
    TransferFailed,

    // Scheduler
    SchedulerStarted,
    SchedulerStopped,
    PollStarted,
    PollCompleted,
    PollFailed,

    // System
    ServerStarted,
    ServerStopped,
    ConfigChanged,
    DatabaseMigrated,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_default();
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// An audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub actor: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry.
    pub fn new(event_type: AuditEventType, actor: impl Into<String>, details: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            event_type: event_type.to_string(),
            actor: actor.into(),
            resource_type: None,
            resource_id: None,
            details,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Set the resource for this entry.
    pub fn with_resource(mut self, resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set the IP address.
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }
}
