use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub market_data: MarketDataConfig,
    pub cache: CacheConfig,
    pub strategies: StrategyConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub websocket_port: u16,
    pub static_port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketDataConfig {
    pub provider: String,
    pub update_interval_seconds: u64,
    pub historical_days: u32,
    pub rate_limit_delay_ms: u64,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub redis_url: Option<String>,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrategyConfig {
    pub max_concurrent_strategies: u32,
    pub initial_cash: f64,
    pub max_position_size: f64,
    pub transaction_cost: f64,
}

impl Config {
    pub async fn load() -> Result<Self> {
        // Load environment variables from .env file if it exists
        dotenvy::dotenv().ok();

        let config = Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8000".to_string())
                    .parse()?,
                websocket_port: env::var("WEBSOCKET_PORT")
                    .unwrap_or_else(|_| "8001".to_string())
                    .parse()?,
                static_port: env::var("STATIC_PORT")
                    .unwrap_or_else(|_| "8050".to_string())
                    .parse()?,
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:trading_platform.db".to_string()),
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()?,
            },
            market_data: MarketDataConfig {
                provider: env::var("MARKET_DATA_PROVIDER")
                    .unwrap_or_else(|_| "yahoo_finance".to_string()),
                update_interval_seconds: env::var("UPDATE_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                historical_days: env::var("HISTORICAL_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                rate_limit_delay_ms: env::var("RATE_LIMIT_DELAY_MS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
                max_retries: env::var("MAX_RETRIES")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                timeout_seconds: env::var("TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            cache: CacheConfig {
                enabled: env::var("CACHE_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()?,
                redis_url: env::var("REDIS_URL").ok(),
                ttl_seconds: env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()?,
            },
            strategies: StrategyConfig {
                max_concurrent_strategies: env::var("MAX_CONCURRENT_STRATEGIES")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                initial_cash: env::var("INITIAL_CASH")
                    .unwrap_or_else(|_| "10000.0".to_string())
                    .parse()?,
                max_position_size: env::var("MAX_POSITION_SIZE")
                    .unwrap_or_else(|_| "0.2".to_string())
                    .parse()?,
                transaction_cost: env::var("TRANSACTION_COST")
                    .unwrap_or_else(|_| "0.01".to_string())
                    .parse()?,
            },
        };

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
                websocket_port: 8001,
                static_port: 8050,
                cors_origins: vec!["*".to_string()],
            },
            database: DatabaseConfig {
                url: "sqlite:trading_platform.db".to_string(),
                max_connections: 10,
                min_connections: 1,
            },
            market_data: MarketDataConfig {
                provider: "yahoo_finance".to_string(),
                update_interval_seconds: 5,
                historical_days: 30,
                rate_limit_delay_ms: 1000,
                max_retries: 3,
                timeout_seconds: 10,
            },
            cache: CacheConfig {
                enabled: false,
                redis_url: None,
                ttl_seconds: 300,
            },
            strategies: StrategyConfig {
                max_concurrent_strategies: 10,
                initial_cash: 10000.0,
                max_position_size: 0.2,
                transaction_cost: 0.01,
            },
        }
    }
}