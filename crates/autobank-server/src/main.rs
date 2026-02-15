//! Autobank Server - Rule-based banking automation
//!
//! This server provides a REST API for managing banking automation rules,
//! executing transfers based on transaction patterns, and tracking audit logs.

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod audit;
mod db;
mod rules;
mod scheduler;

pub use api::create_router;
pub use db::Database;
pub use rules::RuleEngine;
pub use scheduler::{Scheduler, SchedulerConfig};

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub bank_client: Arc<dyn sb1_api::BankApiClient>,
    pub scheduler: Arc<Scheduler>,
    pub shutdown_tx: broadcast::Sender<()>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "autobank_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Autobank server...");

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:autobank.db".to_string());

    let db = Database::connect(&database_url).await?;
    db.run_migrations().await?;

    info!("Database initialized");

    // Initialize bank client
    let config = sb1_api::config::load_config()?;
    let token_provider = Arc::new(sb1_api::FileTokenProvider::new(config)?);
    let bank_client: Arc<dyn sb1_api::BankApiClient> =
        Arc::new(sb1_api::SpareBank1Client::new(token_provider));

    // Create rule engine
    let rule_engine = Arc::new(RuleEngine::new(db.clone(), bank_client.clone()));

    // Create scheduler
    let scheduler_config = SchedulerConfig::default();
    let scheduler = Arc::new(Scheduler::new(scheduler_config, rule_engine));

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Create app state
    let state = AppState {
        db,
        bank_client,
        scheduler: scheduler.clone(),
        shutdown_tx: shutdown_tx.clone(),
    };

    // Spawn scheduler task
    let scheduler_handle = {
        let scheduler = scheduler.clone();
        tokio::spawn(async move {
            scheduler.run(shutdown_rx).await;
        })
    };

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_tx))
        .await?;

    // Wait for scheduler to finish
    let _ = scheduler_handle.await;

    info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal(shutdown_tx: broadcast::Sender<()>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
    let _ = shutdown_tx.send(());
}
