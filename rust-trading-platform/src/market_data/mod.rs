// Market data provider implementations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;

use crate::data::{MarketData, HistoricalData, TimePeriod};
use crate::error::{Result, TradingPlatformError};

pub mod yahoo_finance;
pub mod mock_provider;

pub use yahoo_finance::YahooFinanceProvider;
pub use mock_provider::MockMarketDataProvider;

/// Trait for market data providers
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Get current market data for a symbol
    async fn get_current_price(&self, symbol: &str) -> Result<MarketData>;
    
    /// Get historical price data for a symbol
    async fn get_historical_data(&self, symbol: &str, period: TimePeriod) -> Result<HistoricalData>;
    
    /// Get current prices for multiple symbols
    async fn get_multiple_prices(&self, symbols: &[String]) -> Result<HashMap<String, MarketData>>;
    
    /// Check if the provider is healthy and responsive
    async fn health_check(&self) -> Result<()>;
    
    /// Get provider-specific information
    fn provider_name(&self) -> &str;
    
    /// Get rate limit information
    fn rate_limit_info(&self) -> RateLimitInfo;
}

/// Rate limit information for a provider
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub current_usage: u32,
    pub reset_time: Option<DateTime<Utc>>,
}

impl Default for RateLimitInfo {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 2000,
            current_usage: 0,
            reset_time: None,
        }
    }
}

/// Configuration for market data providers
#[derive(Debug, Clone)]
pub struct MarketDataConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub rate_limit_delay_ms: u64,
    pub cache_ttl_seconds: u64,
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self {
            provider: "yahoo_finance".to_string(),
            api_key: None,
            base_url: "https://query1.finance.yahoo.com".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            rate_limit_delay_ms: 100,
            cache_ttl_seconds: 60,
        }
    }
}

/// Factory for creating market data providers
pub struct MarketDataProviderFactory;

impl MarketDataProviderFactory {
    /// Create a market data provider based on configuration
    pub fn create_provider(config: &MarketDataConfig) -> Result<Box<dyn MarketDataProvider>> {
        match config.provider.as_str() {
            "yahoo_finance" => {
                let provider = YahooFinanceProvider::new(config.clone())?;
                Ok(Box::new(provider))
            }
            "mock" => {
                let provider = MockMarketDataProvider::new();
                Ok(Box::new(provider))
            }
            _ => Err(TradingPlatformError::Config(format!(
                "Unknown market data provider: {}",
                config.provider
            ))),
        }
    }
}

/// Retry logic with exponential backoff
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }
        
        let delay_ms = (self.base_delay.as_millis() as f64) 
            * self.backoff_multiplier.powi((attempt - 1) as i32);
        
        let delay = Duration::from_millis(delay_ms as u64);
        std::cmp::min(delay, self.max_delay)
    }
    
    /// Execute a function with retry logic
    pub async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    if attempt < self.max_retries {
                        let delay = self.calculate_delay(attempt + 1);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            TradingPlatformError::internal("Retry policy failed without error")
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.calculate_delay(0), Duration::from_millis(0));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(1000));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(2000));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(4000));
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_retries: 10,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        };
        
        // Should cap at max_delay
        assert_eq!(policy.calculate_delay(10), Duration::from_secs(5));
    }

    #[test]
    fn test_market_data_config_default() {
        let config = MarketDataConfig::default();
        assert_eq!(config.provider, "yahoo_finance");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_rate_limit_info_default() {
        let info = RateLimitInfo::default();
        assert_eq!(info.requests_per_minute, 60);
        assert_eq!(info.requests_per_hour, 2000);
        assert_eq!(info.current_usage, 0);
    }

    #[tokio::test]
    async fn test_retry_policy_success_on_first_try() {
        let policy = RetryPolicy::default();
        let mut call_count = 0;
        
        let result = policy.execute_with_retry(|| {
            call_count += 1;
            async { Ok::<i32, TradingPlatformError>(42) }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count, 1);
    }

    #[tokio::test]
    async fn test_retry_policy_success_after_retries() {
        let policy = RetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };
        
        let mut call_count = 0;
        
        let result = policy.execute_with_retry(|| {
            call_count += 1;
            async move {
                if call_count < 3 {
                    Err(TradingPlatformError::internal("Temporary failure"))
                } else {
                    Ok::<i32, TradingPlatformError>(42)
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count, 3);
    }

    #[tokio::test]
    async fn test_retry_policy_failure_after_max_retries() {
        let policy = RetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };
        
        let mut call_count = 0;
        
        let result = policy.execute_with_retry(|| {
            call_count += 1;
            async { Err::<i32, TradingPlatformError>(TradingPlatformError::internal("Always fails")) }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(call_count, 3); // Initial attempt + 2 retries
    }
}