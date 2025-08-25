// Yahoo Finance API implementation

use async_trait::async_trait;
use chrono::{Utc, TimeZone};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::data::{MarketData, PricePoint, HistoricalData, TimePeriod};
use crate::error::{Result, TradingPlatformError, MarketDataError};
use super::{MarketDataProvider, MarketDataConfig, RateLimitInfo, RetryPolicy};

/// Yahoo Finance API provider
pub struct YahooFinanceProvider {
    client: Client,
    config: MarketDataConfig,
    retry_policy: RetryPolicy,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl YahooFinanceProvider {
    /// Create a new Yahoo Finance provider
    pub fn new(config: MarketDataConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| TradingPlatformError::internal(format!("Failed to create HTTP client: {}", e)))?;

        let retry_policy = RetryPolicy {
            max_retries: config.max_retries,
            base_delay: Duration::from_millis(config.retry_delay_ms),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
            config.rate_limit_delay_ms,
        )));

        Ok(Self {
            client,
            config,
            retry_policy,
            rate_limiter,
        })
    }

    /// Get quote data from Yahoo Finance
    async fn fetch_quote(&self, symbol: &str) -> Result<YahooQuoteResponse> {
        {
            let mut limiter = self.rate_limiter.lock().await;
            limiter.wait_if_needed().await;
        }

        let url = format!(
            "{}/v8/finance/chart/{}",
            self.config.base_url,
            symbol
        );

        debug!("Fetching quote for symbol: {} from URL: {}", symbol, url);

        let response = self.retry_policy.execute_with_retry(|| {
            let client = &self.client;
            let url = url.clone();
            async move {
                let response = client.get(&url).send().await
                    .map_err(|e| TradingPlatformError::MarketData(
                        MarketDataError::ProviderUnavailable
                    ))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    error!("Yahoo Finance API error: {} - {}", status, error_text);
                    
                    return match status.as_u16() {
                        404 => Err(TradingPlatformError::MarketData(
                            MarketDataError::SymbolNotFound(symbol.to_string())
                        )),
                        429 => Err(TradingPlatformError::MarketData(
                            MarketDataError::RateLimitExceeded
                        )),
                        _ => Err(TradingPlatformError::MarketData(
                            MarketDataError::ProviderUnavailable
                        )),
                    };
                }

                Ok(response)
            }
        }).await?;

        let quote_response: YahooQuoteResponse = response.json().await
            .map_err(|e| {
                error!("Failed to parse Yahoo Finance response: {}", e);
                TradingPlatformError::MarketData(MarketDataError::InvalidFormat)
            })?;

        debug!("Successfully fetched quote for symbol: {}", symbol);
        Ok(quote_response)
    }

    /// Convert Yahoo Finance response to MarketData
    fn convert_to_market_data(&self, symbol: &str, response: &YahooQuoteResponse) -> Result<MarketData> {
        let chart = response.chart.result.first()
            .ok_or_else(|| TradingPlatformError::MarketData(
                MarketDataError::NoDataAvailable(symbol.to_string())
            ))?;

        let meta = &chart.meta;
        let current_price = meta.regular_market_price
            .ok_or_else(|| TradingPlatformError::MarketData(
                MarketDataError::NoDataAvailable(symbol.to_string())
            ))?;

        let previous_close = meta.previous_close.unwrap_or(current_price);
        let volume = meta.regular_market_volume.unwrap_or(0) as u64;

        let mut market_data = MarketData::new(symbol.to_string(), current_price, volume);
        
        if let (Some(high), Some(low)) = (meta.regular_market_day_high, meta.regular_market_day_low) {
            market_data = market_data.with_day_range(high, low);
        }
        
        market_data = market_data.with_change(previous_close);

        // Set additional fields if available
        if let Some(market_cap) = meta.market_cap {
            market_data.market_cap = Some(market_cap as u64);
        }

        Ok(market_data)
    }

    /// Get historical data from Yahoo Finance
    async fn fetch_historical_data(&self, symbol: &str, period: &TimePeriod) -> Result<YahooHistoricalResponse> {
        {
            let mut limiter = self.rate_limiter.lock().await;
            limiter.wait_if_needed().await;
        }

        let (period_str, interval) = match period {
            TimePeriod::OneDay => ("1d", "1m"),
            TimePeriod::OneWeek => ("7d", "5m"),
            TimePeriod::OneMonth => ("1mo", "1h"),
            TimePeriod::ThreeMonths => ("3mo", "1d"),
            TimePeriod::SixMonths => ("6mo", "1d"),
            TimePeriod::OneYear => ("1y", "1d"),
            TimePeriod::TwoYears => ("2y", "1wk"),
            TimePeriod::FiveYears => ("5y", "1mo"),
            TimePeriod::Custom { days } => {
                if *days <= 7 {
                    ("7d", "1h")
                } else if *days <= 30 {
                    ("1mo", "1d")
                } else {
                    ("1y", "1d")
                }
            }
        };

        let url = format!(
            "{}/v8/finance/chart/{}?period1=0&period2=9999999999&interval={}&range={}",
            self.config.base_url,
            symbol,
            interval,
            period_str
        );

        debug!("Fetching historical data for symbol: {} from URL: {}", symbol, url);

        let response = self.retry_policy.execute_with_retry(|| {
            let client = &self.client;
            let url = url.clone();
            async move {
                let response = client.get(&url).send().await
                    .map_err(|e| TradingPlatformError::MarketData(
                        MarketDataError::ProviderUnavailable
                    ))?;

                if !response.status().is_success() {
                    let status = response.status();
                    return match status.as_u16() {
                        404 => Err(TradingPlatformError::MarketData(
                            MarketDataError::SymbolNotFound(symbol.to_string())
                        )),
                        429 => Err(TradingPlatformError::MarketData(
                            MarketDataError::RateLimitExceeded
                        )),
                        _ => Err(TradingPlatformError::MarketData(
                            MarketDataError::ProviderUnavailable
                        )),
                    };
                }

                Ok(response)
            }
        }).await?;

        let historical_response: YahooHistoricalResponse = response.json().await
            .map_err(|e| {
                error!("Failed to parse Yahoo Finance historical response: {}", e);
                TradingPlatformError::MarketData(MarketDataError::InvalidFormat)
            })?;

        debug!("Successfully fetched historical data for symbol: {}", symbol);
        Ok(historical_response)
    }

    /// Convert Yahoo Finance historical response to HistoricalData
    fn convert_to_historical_data(&self, symbol: &str, response: &YahooHistoricalResponse, period: &TimePeriod) -> Result<HistoricalData> {
        let chart = response.chart.result.first()
            .ok_or_else(|| TradingPlatformError::MarketData(
                MarketDataError::NoDataAvailable(symbol.to_string())
            ))?;

        let timestamps = &chart.timestamp;
        let indicators = &chart.indicators.quote.first()
            .ok_or_else(|| TradingPlatformError::MarketData(
                MarketDataError::NoDataAvailable(symbol.to_string())
            ))?;

        let opens = &indicators.open;
        let highs = &indicators.high;
        let lows = &indicators.low;
        let closes = &indicators.close;
        let volumes = &indicators.volume;

        let mut historical_data = HistoricalData::new(symbol.to_string(), *period);

        for (i, &timestamp) in timestamps.iter().enumerate() {
            if let (Some(&open), Some(&high), Some(&low), Some(&close), Some(&volume)) = (
                opens.get(i).and_then(|v| v.as_ref()),
                highs.get(i).and_then(|v| v.as_ref()),
                lows.get(i).and_then(|v| v.as_ref()),
                closes.get(i).and_then(|v| v.as_ref()),
                volumes.get(i).and_then(|v| v.as_ref()),
            ) {
                let datetime = Utc.timestamp_opt(timestamp as i64, 0).single()
                    .ok_or_else(|| TradingPlatformError::internal("Invalid timestamp"))?;

                if let Ok(price_point) = PricePoint::new(datetime, open, high, low, close, volume) {
                    historical_data.add_price_point(price_point);
                }
            }
        }

        if historical_data.data_points.is_empty() {
            return Err(TradingPlatformError::MarketData(
                MarketDataError::InsufficientHistoricalData(symbol.to_string())
            ));
        }

        Ok(historical_data)
    }
}

#[async_trait]
impl MarketDataProvider for YahooFinanceProvider {
    async fn get_current_price(&self, symbol: &str) -> Result<MarketData> {
        info!("Getting current price for symbol: {}", symbol);
        
        let response = self.fetch_quote(symbol).await?;
        let market_data = self.convert_to_market_data(symbol, &response)?;
        
        info!("Successfully retrieved current price for {}: ${:.2}", symbol, market_data.price);
        Ok(market_data)
    }

    async fn get_historical_data(&self, symbol: &str, period: TimePeriod) -> Result<HistoricalData> {
        info!("Getting historical data for symbol: {} (period: {:?})", symbol, period);
        
        let response = self.fetch_historical_data(symbol, &period).await?;
        let historical_data = self.convert_to_historical_data(symbol, &response, &period)?;
        
        info!("Successfully retrieved {} historical data points for {}", 
              historical_data.data_points.len(), symbol);
        Ok(historical_data)
    }

    async fn get_multiple_prices(&self, symbols: &[String]) -> Result<HashMap<String, MarketData>> {
        info!("Getting current prices for {} symbols", symbols.len());
        
        let mut results = HashMap::new();
        let mut errors = Vec::new();

        // Process symbols in batches to respect rate limits
        for symbol in symbols {
            match self.get_current_price(symbol).await {
                Ok(market_data) => {
                    results.insert(symbol.clone(), market_data);
                }
                Err(e) => {
                    warn!("Failed to get price for symbol {}: {}", symbol, e);
                    errors.push((symbol.clone(), e));
                }
            }
            
            // Small delay between requests to avoid rate limiting
            sleep(Duration::from_millis(self.config.rate_limit_delay_ms)).await;
        }

        if results.is_empty() && !errors.is_empty() {
            return Err(errors.into_iter().next().unwrap().1);
        }

        info!("Successfully retrieved prices for {}/{} symbols", results.len(), symbols.len());
        Ok(results)
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing health check for Yahoo Finance provider");
        
        // Try to fetch a well-known symbol (Apple) to test connectivity
        match self.get_current_price("AAPL").await {
            Ok(_) => {
                debug!("Yahoo Finance provider health check passed");
                Ok(())
            }
            Err(e) => {
                error!("Yahoo Finance provider health check failed: {}", e);
                Err(e)
            }
        }
    }

    fn provider_name(&self) -> &str {
        "Yahoo Finance"
    }

    fn rate_limit_info(&self) -> RateLimitInfo {
        RateLimitInfo {
            requests_per_minute: 60,
            requests_per_hour: 2000,
            current_usage: 0, // Would need to track this in a real implementation
            reset_time: None,
        }
    }
}

/// Rate limiter to prevent API abuse
struct RateLimiter {
    delay_ms: u64,
    last_request: Option<SystemTime>,
}

impl RateLimiter {
    fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            last_request: None,
        }
    }

    async fn wait_if_needed(&mut self) {
        if let Some(last_request) = self.last_request {
            let elapsed = last_request.elapsed().unwrap_or(Duration::from_secs(0));
            let required_delay = Duration::from_millis(self.delay_ms);
            
            if elapsed < required_delay {
                let wait_time = required_delay - elapsed;
                sleep(wait_time).await;
            }
        }
        
        self.last_request = Some(SystemTime::now());
    }
}

// Yahoo Finance API response structures
#[derive(Debug, Deserialize)]
struct YahooQuoteResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooHistoricalResponse {
    chart: YahooHistoricalChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Vec<YahooQuoteResult>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YahooHistoricalChart {
    result: Vec<YahooHistoricalResult>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YahooQuoteResult {
    meta: YahooMeta,
    timestamp: Vec<u64>,
    indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
struct YahooHistoricalResult {
    meta: YahooMeta,
    timestamp: Vec<u64>,
    indicators: YahooHistoricalIndicators,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooMeta {
    currency: String,
    symbol: String,
    exchange_name: String,
    instrument_type: String,
    first_trade_date: Option<u64>,
    regular_market_time: Option<u64>,
    gmtoffset: Option<i32>,
    timezone: String,
    exchange_timezone_name: String,
    regular_market_price: Option<f64>,
    chart_previous_close: Option<f64>,
    previous_close: Option<f64>,
    scale: Option<u32>,
    price_hint: Option<u32>,
    current_trading_period: Option<serde_json::Value>,
    trading_periods: Option<Vec<Vec<serde_json::Value>>>,
    data_granularity: String,
    range: String,
    valid_ranges: Vec<String>,
    regular_market_day_high: Option<f64>,
    regular_market_day_low: Option<f64>,
    regular_market_volume: Option<u64>,
    market_cap: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooHistoricalIndicators {
    quote: Vec<YahooHistoricalQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    open: Vec<Option<f64>>,
    high: Vec<Option<f64>>,
    low: Vec<Option<f64>>,
    close: Vec<Option<f64>>,
    volume: Vec<Option<u64>>,
}

#[derive(Debug, Deserialize)]
struct YahooHistoricalQuote {
    open: Vec<Option<f64>>,
    high: Vec<Option<f64>>,
    low: Vec<Option<f64>>,
    close: Vec<Option<f64>>,
    volume: Vec<Option<u64>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yahoo_finance_provider_creation() {
        let config = MarketDataConfig::default();
        let provider = YahooFinanceProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let config = MarketDataConfig::default();
        let provider = YahooFinanceProvider::new(config).unwrap();
        assert_eq!(provider.provider_name(), "Yahoo Finance");
    }

    #[test]
    fn test_rate_limit_info() {
        let config = MarketDataConfig::default();
        let provider = YahooFinanceProvider::new(config).unwrap();
        let rate_info = provider.rate_limit_info();
        assert_eq!(rate_info.requests_per_minute, 60);
        assert_eq!(rate_info.requests_per_hour, 2000);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(100); // 100ms delay
        
        let start = SystemTime::now();
        limiter.wait_if_needed().await; // First call should not wait
        let first_elapsed = start.elapsed().unwrap();
        
        limiter.wait_if_needed().await; // Second call should wait
        let second_elapsed = start.elapsed().unwrap();
        
        // Second call should take at least 100ms more than first
        assert!(second_elapsed.as_millis() >= first_elapsed.as_millis() + 100);
    }

    // Integration tests would go here, but they require network access
    // and should be run separately from unit tests
}