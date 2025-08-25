use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod data;
mod database;
mod error;
mod performance;
mod strategy;
mod xai;

use config::Config;
use database::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trading_platform=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Real-Time XAI Trading Platform");

    // Load configuration
    let config = Config::load().await?;
    info!("📋 Configuration loaded successfully");

    // Initialize database
    info!("🗄️ Initializing database...");
    let database = Database::new(&config.database.url).await?;
    database.migrate().await?;
    
    // Check database health
    database.health_check().await?;
    let stats = database.get_stats().await?;
    info!("✅ Database initialized successfully");
    info!("📊 Database stats: {} strategies, {} trades, {} active", 
          stats.total_strategies, stats.total_trades, stats.active_strategies);

    // TODO: Initialize market data provider
    // TODO: Start API server
    // TODO: Start WebSocket server
    // TODO: Start static file server

    info!("✅ Trading Platform started successfully");
    info!("🌐 API Server: http://{}:{}", config.server.host, config.server.port);
    info!("🔌 WebSocket Server: ws://{}:{}", config.server.host, config.server.websocket_port);
    info!("📊 Dashboard: http://{}:{}", config.server.host, config.server.static_port);

    // Keep the server running
    tokio::signal::ctrl_c().await?;
    info!("👋 Shutting down Trading Platform");

    Ok(())
}
