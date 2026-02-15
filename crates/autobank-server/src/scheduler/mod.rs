//! Polling scheduler for periodic transaction checks.

use crate::rules::RuleEngine;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, error, info};

/// Scheduler configuration.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub poll_interval_seconds: u64,
    pub enabled: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: 300, // 5 minutes
            enabled: true,
        }
    }
}

/// Polling scheduler for rule evaluation.
pub struct Scheduler {
    config: Arc<RwLock<SchedulerConfig>>,
    rule_engine: Arc<RuleEngine>,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new(config: SchedulerConfig, rule_engine: Arc<RuleEngine>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            rule_engine,
        }
    }

    /// Run the scheduler loop.
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) {
        info!("Scheduler started");

        loop {
            let interval = {
                let config = self.config.read().await;
                Duration::from_secs(config.poll_interval_seconds)
            };

            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Scheduler received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(interval) => {
                    if self.is_enabled().await {
                        self.poll().await;
                    } else {
                        debug!("Scheduler is disabled, skipping poll");
                    }
                }
            }
        }

        info!("Scheduler stopped");
    }

    /// Check if the scheduler is enabled.
    pub async fn is_enabled(&self) -> bool {
        self.config.read().await.enabled
    }

    /// Perform a single poll cycle.
    async fn poll(&self) {
        debug!("Starting poll cycle");

        match self.rule_engine.evaluate_all().await {
            Ok(()) => {
                debug!("Poll cycle completed successfully");
            }
            Err(e) => {
                error!("Poll cycle failed: {}", e);
            }
        }
    }

    /// Update the scheduler configuration.
    pub async fn update_config(&self, new_config: SchedulerConfig) {
        *self.config.write().await = new_config;
    }

    /// Enable the scheduler.
    pub async fn enable(&self) {
        self.config.write().await.enabled = true;
        info!("Scheduler enabled");
    }

    /// Disable the scheduler.
    pub async fn disable(&self) {
        self.config.write().await.enabled = false;
        info!("Scheduler disabled");
    }

    /// Trigger an immediate poll.
    pub async fn trigger_poll(&self) {
        info!("Manual poll triggered");
        self.poll().await;
    }
}
