//! REST API handlers and routes.

mod accounts;
mod audit;
mod executions;
mod health;
mod rules;
mod system;

use crate::AppState;
use axum::{Router, routing::get};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Creates the main application router with all routes.
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/status", get(health::status))
        .nest("/api/accounts", accounts::router())
        .nest("/api/rules", rules::router())
        .route("/api/rules/{rule_id}/executions", get(executions::get_rule_executions))
        .nest("/api/executions", executions::router())
        .nest("/api/audit", audit::router())
        .nest("/api/system", system::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
