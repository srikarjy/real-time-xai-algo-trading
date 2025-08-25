// Mock market data provider for testing

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::data::{MarketData, PricePoint, HistoricalData, TimePeriod};
use crate::error::{Result, TradingPlatformError, MarketDataError};
use super::{MarketDataProvider, RateLimitInfo};

/// Mock market data provider for testing and development
pub struct MockMarketDataProvider {
    rng: Arc<Mutex<StdRng>>,
    base_prices: Arc<Mutex<HashMap<String, f64>>>,
    health_status: Arc<Mutex<bool>>,
}

impl MockMarketDataProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        let mut base_prices = HashMap::new();
        
        // Initialize with some common stock prices
        base_prices.insert("AAPL".to_string(), 150.0);
        base_prices.insert("GOOGL".to_string(), 2800.0);
        base_prices.insert("MSFT".to_string(), 300.0);
        base_prices.insert("TSLA".to_string(), 800.0);
        base_prices.insert("AMZN".to_string(), 3200.0);
        base_prices.insert("META".to_string(), 320.0);
        base_prices.insert("NVDA".to_string(), 450.0);
        base_prices.insert("NFLX".to_string(), 400.0);
        
        Self {
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
            base_prices: Arc::new(Mutex::new(base_prices)),
            health_status: Arc::new(Mutex::new(true)),
        }
    }

    /// Create a mock provider with a specific seed for deterministic testing
    pub fn new_with_seed(seed: u64) -> Self {
        let mut base_prices = HashMap::new();
        base_prices.insert("AAPL".to_string(), 150.0);
        base_prices.insert("GOOGL".to_string(), 2800.0);
        base_prices.insert("MSFT".to_string(), 300.0);
        
        Self {
            rng: Arc::new(Mutex::new(StdRng::seed_from_u64(seed))),
            base_prices: Arc::new(Mutex::new(base_prices)),
            health_status: Arc::new(Mutex::new(true)),
        }
    }

    /// Set the health status for testing error scenarios
    pub fn set_health_status(&self, healthy: bool) {
        *self.health_status.lock().unwrap() = healthy;
    }

    /// Add or update a base price for a symbol
    pub fn set_base_price(&self, symbol: &str, price: f64) {
        self.base_prices.lock().unwrap().insert(symbol.to_string(), price);
    }

    /// Generate a realistic price variation
    fn generate_price_variation(&self, base_price: f64) -> f64 {
        let mut rng = self.rng.lock().unwrap();
        
        // Generate a price variation between -5% and +5%
        let variation_percent = rng.gen_range(-0.05..0.05);
        let variation = base_price * variation_percent;
        
        (base_price + variation).max(0.01) // Ensure price is always positive
    }

    /// Generate realistic volume
    fn generate_volume(&self) -> u64 {
        let mut rng = self.rng.lock().unwrap();
        rng.gen_range(100_000..10_000_000)
    }

    /// Generate historical price points
    fn generate_historical_prices(&self, symbol: &str, base_price: f64, period: TimePeriod) -> Vec<PricePoint> {
        let mut rng = self.rng.lock().unwrap();
        let mut prices = Vec::new();
        
        let (num_points, interval) = match period {
            TimePeriod::OneDay => (24 * 4, ChronoDuration::minutes(15)), // 15-minute intervals
            TimePeriod::OneWeek => (7 * 24, ChronoDuration::hours(1)),   // Hourly intervals
            TimePeriod::OneMonth => (30, ChronoDuration::days(1)),       // Daily intervals
            TimePeriod::ThreeMonths => (90, ChronoDuration::days(1)),    // Daily intervals
            TimePeriod::SixMonths => (180, ChronoDuration::days(1)),     // Daily intervals
            TimePeriod::OneYear => (252, ChronoDuration::days(1)),       // Trading days
            TimePeriod::TwoYears => (104, ChronoDuration::weeks(1)),     // Weekly intervals
            TimePeriod::FiveYears => (60, ChronoDuration::days(30)),     // Monthly intervals
            TimePeriod::Custom { days } => {
                let points = days.min(365) as i64;
                (points, ChronoDuration::days(1))
            }
        };

        let start_time = Utc::now() - ChronoDuration::from_std(
            std::time::Duration::from_secs(interval.num_seconds() as u64 * num_points as u64)
        ).unwrap_or(ChronoDuration::days(1));

        let mut current_price = base_price;
        
        for i in 0..num_points {
            let timestamp = start_time + interval * i as i32;
            
            // Generate realistic OHLC data
            let price_change = rng.gen_range(-0.02..0.02); // ±2% change per period
            current_price *= 1.0 + price_change;
            current_price = current_price.max(0.01);
            
            let volatility = rng.gen_range(0.005..0.02); // 0.5% to 2% volatility
            let high = current_price * (1.0 + volatility);
            let low = current_price * (1.0 - volatility);
            let open = current_price * rng.gen_range(0.995..1.005);
            let close = current_price;
            let volume = self.generate_volume();
            
            if let Ok(price_point) = PricePoint::new(timestamp, open, high, low, close, volume) {
                prices.push(price_point);
            }
        }
        
        prices
    }
}

impl Default for MockMarketDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataProvider for MockMarketDataProvider {
    async fn get_current_price(&self, symbol: &str) -> Result<MarketData> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        // Check health status
        if !*self.health_status.lock().unwrap() {
            return Err(TradingPlatformError::MarketData(
                MarketDataError::ProviderUnavailable
            ));
        }

        // Get base price or return error for unknown symbols
        let base_price = {
            let prices = self.base_prices.lock().unwrap();
            match prices.get(symbol) {
                Some(&price) => price,
                None => {
                    return Err(TradingPlatformError::MarketData(
                        MarketDataError::SymbolNotFound(symbol.to_string())
                    ));
                }
            }
        };

        let current_price = self.generate_price_variation(base_price);
        let volume = self.generate_volume();
        let previous_close = base_price;

        let mut market_data = MarketData::new(symbol.to_string(), current_price, volume);
        market_data = market_data.with_change(previous_close);
        
        // Add some realistic day range
        let day_high = current_price * 1.02;
        let day_low = current_price * 0.98;
        market_data = market_data.with_day_range(day_high, day_low);

        Ok(market_data)
    }

    async fn get_historical_data(&self, symbol: &str, period: TimePeriod) -> Result<HistoricalData> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Check health status
        if !*self.health_status.lock().unwrap() {
            return Err(TradingPlatformError::MarketData(
                MarketDataError::ProviderUnavailable
            ));
        }

        // Get base price or return error for unknown symbols
        let base_price = {
            let prices = self.base_prices.lock().unwrap();
            match prices.get(symbol) {
                Some(&price) => price,
                None => {
                    return Err(TradingPlatformError::MarketData(
                        MarketDataError::SymbolNotFound(symbol.to_string())
                    ));
                }
            }
        };

        let price_points = self.generate_historical_prices(symbol, base_price, period);
        
        if price_points.is_empty() {
            return Err(TradingPlatformError::MarketData(
                MarketDataError::InsufficientHistoricalData(symbol.to_string())
            ));
        }

        let mut historical_data = HistoricalData::new(symbol.to_string(), period);
        for price_point in price_points {
            historical_data.add_price_point(price_point);
        }

        Ok(historical_data)
    }

    async fn get_multiple_prices(&self, symbols: &[String]) -> Result<HashMap<String, MarketData>> {
        let mut results = HashMap::new();
        
        for symbol in symbols {
            match self.get_current_price(symbol).await {
                Ok(market_data) => {
                    results.insert(symbol.clone(), market_data);
                }
                Err(_) => {
                    // Continue with other symbols even if one fails
                    continue;
                }
            }
        }

        if results.is_empty() {
            return Err(TradingPlatformError::MarketData(
                MarketDataError::NoDataAvailable("No data available for any symbol".to_string())
            ));
        }

        Ok(results)
    }

    async fn health_check(&self) -> Result<()> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        if *self.health_status.lock().unwrap() {
            Ok(())
        } else {
            Err(TradingPlatformError::MarketData(
                MarketDataError::ProviderUnavailable
            ))
        }
    }

    fn provider_name(&self) -> &str {
        "Mock Provider"
    }

    fn rate_limit_info(&self) -> RateLimitInfo {
        RateLimitInfo {
            requests_per_minute: 1000, // Very high limits for testing
            requests_per_hour: 60000,
            current_usage: 0,
            reset_time: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_current_price() {
        let provider = MockMarketDataProvider::new_with_seed(42);
        
        let result = provider.get_current_price("AAPL").await;
        assert!(result.is_ok());
        
        let market_data = result.unwrap();
        assert_eq!(market_data.symbol, "AAPL");
        assert!(market_data.price > 0.0);
        assert!(market_data.volume > 0);
    }

    #[tokio::test]
    async fn test_mock_provider_unknown_symbol() {
        let provider = MockMarketDataProvider::new();
        
        let result = provider.get_current_price("UNKNOWN").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            TradingPlatformError::MarketData(MarketDataError::SymbolNotFound(symbol)) => {
                assert_eq!(symbol, "UNKNOWN");
            }
            _ => panic!("Expected SymbolNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_mock_provider_historical_data() {
        let provider = MockMarketDataProvider::new_with_seed(42);
        
        let result = provider.get_historical_data("AAPL", TimePeriod::OneWeek).await;
        assert!(result.is_ok());
        
        let historical_data = result.unwrap();
        assert_eq!(historical_data.symbol, "AAPL");
        assert!(!historical_data.data_points.is_empty());
        assert_eq!(historical_data.period, TimePeriod::OneWeek);
    }

    #[tokio::test]
    async fn test_mock_provider_multiple_prices() {
        let provider = MockMarketDataProvider::new();
        let symbols = vec!["AAPL".to_string(), "GOOGL".to_string(), "MSFT".to_string()];
        
        let result = provider.get_multiple_prices(&symbols).await;
        assert!(result.is_ok());
        
        let prices = result.unwrap();
        assert_eq!(prices.len(), 3);
        assert!(prices.contains_key("AAPL"));
        assert!(prices.contains_key("GOOGL"));
        assert!(prices.contains_key("MSFT"));
    }

    #[tokio::test]
    async fn test_mock_provider_health_check() {
        let provider = MockMarketDataProvider::new();
        
        // Should be healthy by default
        let result = provider.health_check().await;
        assert!(result.is_ok());
        
        // Set unhealthy
        provider.set_health_status(false);
        let result = provider.health_check().await;
        assert!(result.is_err());
        
        // Set healthy again
        provider.set_health_status(true);
        let result = provider.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_provider_unhealthy_requests() {
        let provider = MockMarketDataProvider::new();
        provider.set_health_status(false);
        
        let result = provider.get_current_price("AAPL").await;
        assert!(result.is_err());
        
        let result = provider.get_historical_data("AAPL", TimePeriod::OneDay).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_provider_set_base_price() {
        let provider = MockMarketDataProvider::new();
        provider.set_base_price("TEST", 100.0);
        
        let prices = provider.base_prices.lock().unwrap();
        assert_eq!(prices.get("TEST"), Some(&100.0));
    }

    #[test]
    fn test_mock_provider_name() {
        let provider = MockMarketDataProvider::new();
        assert_eq!(provider.provider_name(), "Mock Provider");
    }

    #[test]
    fn test_mock_provider_rate_limit_info() {
        let provider = MockMarketDataProvider::new();
        let rate_info = provider.rate_limit_info();
        assert_eq!(rate_info.requests_per_minute, 1000);
        assert_eq!(rate_info.requests_per_hour, 60000);
    }

    #[test]
    fn test_deterministic_behavior_with_seed() {
        let provider1 = MockMarketDataProvider::new_with_seed(42);
        let provider2 = MockMarketDataProvider::new_with_seed(42);
        
        // Both providers should generate the same price variation for the same symbol
        let price1 = provider1.generate_price_variation(100.0);
        let price2 = provider2.generate_price_variation(100.0);
        
        assert_eq!(price1, price2);
    }
}