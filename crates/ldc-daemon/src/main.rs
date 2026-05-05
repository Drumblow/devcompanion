mod api;
mod config;
mod db;
mod draft;
mod error;

use anyhow::Context;
use axum::Router;
use config::AppConfig;
use db::Database;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub draft_model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ldc_daemon=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    let database = Database::connect(&config.database_path).await?;
    database.migrate().await?;

    let state = AppState {
        db: database,
        draft_model: config.draft_model,
    };

    let app = Router::new()
        .merge(api::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr)
        .await
        .context("falha ao iniciar listener HTTP")?;

    info!(%addr, "ldc-daemon iniciado");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
