use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradingPlatformError {
    #[error("Strategy error: {0}")]
    Strategy(#[from] StrategyError),
    
    #[error("Market data error: {0}")]
    MarketData(#[from] MarketDataError),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseFloatError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("Invalid strategy parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Strategy not found: {0}")]
    NotFound(String),
    
    #[error("Strategy execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Insufficient historical data")]
    InsufficientData,
    
    #[error("Strategy already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Strategy is not active: {0}")]
    NotActive(String),
}

#[derive(Debug, Error)]
pub enum MarketDataError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("API rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Data provider unavailable")]
    ProviderUnavailable,
    
    #[error("Invalid data format")]
    InvalidFormat,
    
    #[error("No data available for symbol: {0}")]
    NoDataAvailable(String),
    
    #[error("Historical data insufficient for symbol: {0}")]
    InsufficientHistoricalData(String),
}

pub type Result<T> = std::result::Result<T, TradingPlatformError>;

impl From<anyhow::Error> for TradingPlatformError {
    fn from(err: anyhow::Error) -> Self {
        TradingPlatformError::Internal(err.to_string())
    }
}

// Helper functions for error handling
impl TradingPlatformError {
    pub fn config<T: std::fmt::Display>(msg: T) -> Self {
        TradingPlatformError::Config(msg.to_string())
    }
    
    pub fn internal<T: std::fmt::Display>(msg: T) -> Self {
        TradingPlatformError::Internal(msg.to_string())
    }
}

impl StrategyError {
    pub fn invalid_parameters<T: std::fmt::Display>(msg: T) -> Self {
        StrategyError::InvalidParameters(msg.to_string())
    }
    
    pub fn execution_failed<T: std::fmt::Display>(msg: T) -> Self {
        StrategyError::ExecutionFailed(msg.to_string())
    }
}

impl MarketDataError {
    pub fn symbol_not_found<T: std::fmt::Display>(symbol: T) -> Self {
        MarketDataError::SymbolNotFound(symbol.to_string())
    }
    
    pub fn no_data_available<T: std::fmt::Display>(symbol: T) -> Self {
        MarketDataError::NoDataAvailable(symbol.to_string())
    }
}