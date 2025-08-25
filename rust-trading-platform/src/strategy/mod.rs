// Strategy execution engine and data models

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::data::{MarketData, PricePoint};
use crate::error::{Result, StrategyError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Strategy {
    pub id: String,
    pub strategy_type: StrategyType,
    pub symbol: String,
    pub parameters: StrategyParameters,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    PriceDrop { threshold: f64 },
    MovingAverage { short_period: usize, long_period: usize },
    RSI { oversold: f64, overbought: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyParameters {
    pub symbol: String,
    pub additional_params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradingSignal {
    pub strategy_id: String,
    pub symbol: String,
    pub action: Action,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub explanation: String,
    pub confidence: f64,
    pub metadata: SignalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalMetadata {
    pub strategy_data: HashMap<String, f64>,
    pub market_conditions: Option<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

// Implementation methods
impl Strategy {
    pub fn new(strategy_type: StrategyType, symbol: String) -> Result<Self> {
        let parameters = StrategyParameters::from_strategy_type(&strategy_type, &symbol)?;
        
        Ok(Strategy {
            id: Uuid::new_v4().to_string(),
            strategy_type,
            symbol: symbol.clone(),
            parameters,
            created_at: Utc::now(),
            is_active: true,
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate symbol
        if self.symbol.is_empty() || self.symbol.len() > 10 {
            return Err(StrategyError::invalid_parameters("Symbol must be 1-10 characters").into());
        }

        // Validate strategy-specific parameters
        match &self.strategy_type {
            StrategyType::PriceDrop { threshold } => {
                if *threshold <= 0.0 || *threshold > 100.0 {
                    return Err(StrategyError::invalid_parameters("Price drop threshold must be between 0 and 100").into());
                }
            }
            StrategyType::MovingAverage { short_period, long_period } => {
                if *short_period == 0 || *long_period == 0 {
                    return Err(StrategyError::invalid_parameters("Moving average periods must be greater than 0").into());
                }
                if *short_period >= *long_period {
                    return Err(StrategyError::invalid_parameters("Short period must be less than long period").into());
                }
                if *long_period > 200 {
                    return Err(StrategyError::invalid_parameters("Long period should not exceed 200").into());
                }
            }
            StrategyType::RSI { oversold, overbought } => {
                if *oversold <= 0.0 || *oversold >= 100.0 {
                    return Err(StrategyError::invalid_parameters("RSI oversold level must be between 0 and 100").into());
                }
                if *overbought <= 0.0 || *overbought >= 100.0 {
                    return Err(StrategyError::invalid_parameters("RSI overbought level must be between 0 and 100").into());
                }
                if *oversold >= *overbought {
                    return Err(StrategyError::invalid_parameters("RSI oversold level must be less than overbought level").into());
                }
            }
        }

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }
}

impl StrategyParameters {
    pub fn from_strategy_type(strategy_type: &StrategyType, symbol: &str) -> Result<Self> {
        let mut additional_params = HashMap::new();

        match strategy_type {
            StrategyType::PriceDrop { threshold } => {
                additional_params.insert("threshold".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(*threshold).unwrap()));
            }
            StrategyType::MovingAverage { short_period, long_period } => {
                additional_params.insert("short_period".to_string(), serde_json::Value::Number((*short_period as u64).into()));
                additional_params.insert("long_period".to_string(), serde_json::Value::Number((*long_period as u64).into()));
            }
            StrategyType::RSI { oversold, overbought } => {
                additional_params.insert("oversold".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(*oversold).unwrap()));
                additional_params.insert("overbought".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(*overbought).unwrap()));
            }
        }

        Ok(StrategyParameters {
            symbol: symbol.to_string(),
            additional_params,
        })
    }
}

impl TradingSignal {
    pub fn new(
        strategy_id: String,
        symbol: String,
        action: Action,
        price: f64,
        explanation: String,
        confidence: f64,
        strategy_data: HashMap<String, f64>,
    ) -> Self {
        let risk_level = match confidence {
            c if c >= 0.8 => RiskLevel::Low,
            c if c >= 0.5 => RiskLevel::Medium,
            _ => RiskLevel::High,
        };

        TradingSignal {
            strategy_id,
            symbol,
            action,
            price,
            timestamp: Utc::now(),
            explanation,
            confidence,
            metadata: SignalMetadata {
                strategy_data,
                market_conditions: None,
                risk_level,
            },
        }
    }

    pub fn with_market_conditions(mut self, conditions: String) -> Self {
        self.metadata.market_conditions = Some(conditions);
        self
    }
}

impl Default for StrategyParameters {
    fn default() -> Self {
        StrategyParameters {
            symbol: "AAPL".to_string(),
            additional_params: HashMap::new(),
        }
    }
}

// Display implementations for better debugging
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Buy => write!(f, "BUY"),
            Action::Sell => write!(f, "SELL"),
            Action::Hold => write!(f, "HOLD"),
        }
    }
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::PriceDrop { threshold } => write!(f, "PriceDrop({}%)", threshold),
            StrategyType::MovingAverage { short_period, long_period } => {
                write!(f, "MovingAverage({}/{})", short_period, long_period)
            }
            StrategyType::RSI { oversold, overbought } => {
                write!(f, "RSI({}/{})", oversold, overbought)
            }
        }
    }
}

// Strategy Execution Engine

/// Trait for executing trading strategies
#[async_trait]
pub trait StrategyExecutor {
    /// Execute the strategy with current market data and historical data
    async fn execute(&self, market_data: &MarketData, historical_data: &[PricePoint]) -> Result<TradingSignal>;
    
    /// Validate strategy parameters
    fn validate_parameters(&self) -> Result<()>;
    
    /// Get the strategy being executed
    fn get_strategy(&self) -> &Strategy;
}

/// Main strategy executor that handles all strategy types
pub struct StrategyEngine {
    strategy: Strategy,
}

impl StrategyEngine {
    pub fn new(strategy: Strategy) -> Result<Self> {
        let engine = StrategyEngine { strategy };
        engine.validate_parameters()?;
        Ok(engine)
    }
}

#[async_trait]
impl StrategyExecutor for StrategyEngine {
    async fn execute(&self, market_data: &MarketData, historical_data: &[PricePoint]) -> Result<TradingSignal> {
        if !self.strategy.is_active {
            return Ok(TradingSignal::new(
                self.strategy.id.clone(),
                self.strategy.symbol.clone(),
                Action::Hold,
                market_data.price,
                "Strategy is inactive".to_string(),
                0.0,
                HashMap::new(),
            ));
        }

        match &self.strategy.strategy_type {
            StrategyType::PriceDrop { threshold } => {
                self.execute_price_drop_strategy(market_data, historical_data, *threshold).await
            }
            StrategyType::MovingAverage { short_period, long_period } => {
                self.execute_moving_average_strategy(market_data, historical_data, *short_period, *long_period).await
            }
            StrategyType::RSI { oversold, overbought } => {
                self.execute_rsi_strategy(market_data, historical_data, *oversold, *overbought).await
            }
        }
    }

    fn validate_parameters(&self) -> Result<()> {
        self.strategy.validate()
    }

    fn get_strategy(&self) -> &Strategy {
        &self.strategy
    }
}

impl StrategyEngine {
    /// Execute price drop strategy
    async fn execute_price_drop_strategy(
        &self,
        market_data: &MarketData,
        historical_data: &[PricePoint],
        threshold: f64,
    ) -> Result<TradingSignal> {
        if historical_data.is_empty() {
            return Err(StrategyError::InsufficientData.into());
        }

        // Get the most recent closing price for comparison
        let previous_close = historical_data.last().unwrap().close;
        let current_price = market_data.price;
        
        // Calculate percentage change
        let price_change_percent = ((current_price - previous_close) / previous_close) * 100.0;
        
        let mut strategy_data = HashMap::new();
        strategy_data.insert("previous_close".to_string(), previous_close);
        strategy_data.insert("current_price".to_string(), current_price);
        strategy_data.insert("price_change_percent".to_string(), price_change_percent);
        strategy_data.insert("threshold".to_string(), threshold);

        let (action, explanation, confidence) = if price_change_percent <= -threshold {
            // Price dropped by threshold or more - BUY signal
            let explanation = format!(
                "Price dropped {:.2}% (from ${:.2} to ${:.2}), exceeding threshold of {:.1}%. This indicates a potential buying opportunity.",
                price_change_percent.abs(),
                previous_close,
                current_price,
                threshold
            );
            (Action::Buy, explanation, 0.8)
        } else if price_change_percent >= threshold {
            // Price increased by threshold or more - SELL signal
            let explanation = format!(
                "Price increased {:.2}% (from ${:.2} to ${:.2}), exceeding threshold of {:.1}%. Consider taking profits.",
                price_change_percent,
                previous_close,
                current_price,
                threshold
            );
            (Action::Sell, explanation, 0.7)
        } else {
            // Price change within threshold - HOLD
            let explanation = format!(
                "Price change of {:.2}% (from ${:.2} to ${:.2}) is within threshold of {:.1}%. No action required.",
                price_change_percent,
                previous_close,
                current_price,
                threshold
            );
            (Action::Hold, explanation, 0.6)
        };

        Ok(TradingSignal::new(
            self.strategy.id.clone(),
            self.strategy.symbol.clone(),
            action,
            current_price,
            explanation,
            confidence,
            strategy_data,
        ))
    }

    /// Execute moving average crossover strategy
    async fn execute_moving_average_strategy(
        &self,
        market_data: &MarketData,
        historical_data: &[PricePoint],
        short_period: usize,
        long_period: usize,
    ) -> Result<TradingSignal> {
        if historical_data.len() < long_period {
            return Err(StrategyError::InsufficientData.into());
        }

        // Calculate moving averages
        let short_ma = self.calculate_simple_moving_average(historical_data, short_period)?;
        let long_ma = self.calculate_simple_moving_average(historical_data, long_period)?;
        
        // Get previous moving averages for crossover detection
        let prev_short_ma = if historical_data.len() > short_period {
            self.calculate_simple_moving_average(&historical_data[..historical_data.len()-1], short_period)?
        } else {
            short_ma
        };
        let prev_long_ma = if historical_data.len() > long_period {
            self.calculate_simple_moving_average(&historical_data[..historical_data.len()-1], long_period)?
        } else {
            long_ma
        };

        let current_price = market_data.price;
        
        let mut strategy_data = HashMap::new();
        strategy_data.insert("short_ma".to_string(), short_ma);
        strategy_data.insert("long_ma".to_string(), long_ma);
        strategy_data.insert("prev_short_ma".to_string(), prev_short_ma);
        strategy_data.insert("prev_long_ma".to_string(), prev_long_ma);
        strategy_data.insert("current_price".to_string(), current_price);

        let (action, explanation, confidence) = if prev_short_ma <= prev_long_ma && short_ma > long_ma {
            // Bullish crossover - BUY signal
            let explanation = format!(
                "Bullish crossover detected: {}-period MA (${:.2}) crossed above {}-period MA (${:.2}). Current price: ${:.2}. This suggests upward momentum.",
                short_period, short_ma, long_period, long_ma, current_price
            );
            (Action::Buy, explanation, 0.85)
        } else if prev_short_ma >= prev_long_ma && short_ma < long_ma {
            // Bearish crossover - SELL signal
            let explanation = format!(
                "Bearish crossover detected: {}-period MA (${:.2}) crossed below {}-period MA (${:.2}). Current price: ${:.2}. This suggests downward momentum.",
                short_period, short_ma, long_period, long_ma, current_price
            );
            (Action::Sell, explanation, 0.85)
        } else {
            // No crossover - HOLD
            let trend = if short_ma > long_ma { "bullish" } else { "bearish" };
            let explanation = format!(
                "No crossover detected. {}-period MA: ${:.2}, {}-period MA: ${:.2}. Current trend: {}. Current price: ${:.2}.",
                short_period, short_ma, long_period, long_ma, trend, current_price
            );
            (Action::Hold, explanation, 0.6)
        };

        Ok(TradingSignal::new(
            self.strategy.id.clone(),
            self.strategy.symbol.clone(),
            action,
            current_price,
            explanation,
            confidence,
            strategy_data,
        ))
    }

    /// Execute RSI strategy
    async fn execute_rsi_strategy(
        &self,
        market_data: &MarketData,
        historical_data: &[PricePoint],
        oversold: f64,
        overbought: f64,
    ) -> Result<TradingSignal> {
        const RSI_PERIOD: usize = 14;
        
        if historical_data.len() < RSI_PERIOD + 1 {
            return Err(StrategyError::InsufficientData.into());
        }

        let rsi = self.calculate_rsi(historical_data, RSI_PERIOD)?;
        let current_price = market_data.price;
        
        let mut strategy_data = HashMap::new();
        strategy_data.insert("rsi".to_string(), rsi);
        strategy_data.insert("oversold_level".to_string(), oversold);
        strategy_data.insert("overbought_level".to_string(), overbought);
        strategy_data.insert("current_price".to_string(), current_price);

        let (action, explanation, confidence) = if rsi <= oversold {
            // RSI indicates oversold condition - BUY signal
            let explanation = format!(
                "RSI at {:.1} indicates oversold condition (below {:.1}). Current price: ${:.2}. This suggests a potential buying opportunity as the stock may be undervalued.",
                rsi, oversold, current_price
            );
            (Action::Buy, explanation, 0.8)
        } else if rsi >= overbought {
            // RSI indicates overbought condition - SELL signal
            let explanation = format!(
                "RSI at {:.1} indicates overbought condition (above {:.1}). Current price: ${:.2}. This suggests taking profits as the stock may be overvalued.",
                rsi, overbought, current_price
            );
            (Action::Sell, explanation, 0.8)
        } else {
            // RSI in neutral zone - HOLD
            let zone = if rsi > 50.0 { "bullish" } else { "bearish" };
            let explanation = format!(
                "RSI at {:.1} is in neutral zone (between {:.1} and {:.1}). Current price: ${:.2}. Market sentiment: {}. No action required.",
                rsi, oversold, overbought, current_price, zone
            );
            (Action::Hold, explanation, 0.6)
        };

        Ok(TradingSignal::new(
            self.strategy.id.clone(),
            self.strategy.symbol.clone(),
            action,
            current_price,
            explanation,
            confidence,
            strategy_data,
        ))
    }

    /// Calculate Simple Moving Average
    fn calculate_simple_moving_average(&self, data: &[PricePoint], period: usize) -> Result<f64> {
        if data.len() < period {
            return Err(StrategyError::InsufficientData.into());
        }

        let sum: f64 = data.iter()
            .rev()
            .take(period)
            .map(|point| point.close)
            .sum();

        Ok(sum / period as f64)
    }

    /// Calculate RSI (Relative Strength Index)
    fn calculate_rsi(&self, data: &[PricePoint], period: usize) -> Result<f64> {
        if data.len() < period + 1 {
            return Err(StrategyError::InsufficientData.into());
        }

        // Calculate price changes
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..data.len() {
            let change = data[i].close - data[i-1].close;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        if gains.len() < period {
            return Err(StrategyError::InsufficientData.into());
        }

        // Calculate average gains and losses for the period
        let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

        // Calculate RSI
        if avg_loss == 0.0 {
            return Ok(100.0); // All gains, RSI = 100
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Ok(rsi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_price_points(prices: &[f64]) -> Vec<PricePoint> {
        prices.iter().enumerate().map(|(i, &price)| {
            PricePoint {
                timestamp: Utc::now() - chrono::Duration::days((prices.len() - i - 1) as i64),
                open: price,
                high: price * 1.02,
                low: price * 0.98,
                close: price,
                volume: 1000000,
                adjusted_close: Some(price),
            }
        }).collect()
    }

    fn create_test_market_data(symbol: &str, price: f64) -> MarketData {
        MarketData {
            symbol: symbol.to_string(),
            price,
            volume: 1000000,
            timestamp: Utc::now(),
            change: 0.0,
            change_percent: 0.0,
            market_cap: Some(1000000000),
            day_high: Some(price * 1.05),
            day_low: Some(price * 0.95),
            previous_close: Some(price),
        }
    }

    #[tokio::test]
    async fn test_price_drop_strategy_buy_signal() {
        let strategy = Strategy::new(
            StrategyType::PriceDrop { threshold: 5.0 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create historical data with price at $100
        let historical_data = create_test_price_points(&[100.0]);
        
        // Current price dropped to $94 (6% drop)
        let market_data = create_test_market_data("AAPL", 94.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Buy);
        assert!(signal.confidence > 0.7);
        assert!(signal.explanation.contains("dropped"));
        assert!(signal.explanation.contains("6.00%"));
    }

    #[tokio::test]
    async fn test_price_drop_strategy_sell_signal() {
        let strategy = Strategy::new(
            StrategyType::PriceDrop { threshold: 5.0 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create historical data with price at $100
        let historical_data = create_test_price_points(&[100.0]);
        
        // Current price increased to $106 (6% increase)
        let market_data = create_test_market_data("AAPL", 106.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Sell);
        assert!(signal.confidence > 0.6);
        assert!(signal.explanation.contains("increased"));
        assert!(signal.explanation.contains("6.00%"));
    }

    #[tokio::test]
    async fn test_price_drop_strategy_hold_signal() {
        let strategy = Strategy::new(
            StrategyType::PriceDrop { threshold: 5.0 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create historical data with price at $100
        let historical_data = create_test_price_points(&[100.0]);
        
        // Current price at $102 (2% increase, within threshold)
        let market_data = create_test_market_data("AAPL", 102.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Hold);
        assert!(signal.explanation.contains("within threshold"));
    }

    #[tokio::test]
    async fn test_moving_average_strategy_bullish_crossover() {
        let strategy = Strategy::new(
            StrategyType::MovingAverage { short_period: 2, long_period: 3 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create data where short MA crosses above long MA
        // Prices: [95, 96, 100] - short MA (2): 98, long MA (3): 97
        let historical_data = create_test_price_points(&[95.0, 96.0, 100.0]);
        let market_data = create_test_market_data("AAPL", 100.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Buy);
        assert!(signal.confidence > 0.8);
        assert!(signal.explanation.contains("Bullish crossover"));
    }

    #[tokio::test]
    async fn test_moving_average_strategy_bearish_crossover() {
        let strategy = Strategy::new(
            StrategyType::MovingAverage { short_period: 2, long_period: 3 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create data where short MA crosses below long MA
        // Prices: [105, 104, 95] - creates bearish crossover
        let historical_data = create_test_price_points(&[105.0, 104.0, 95.0]);
        let market_data = create_test_market_data("AAPL", 95.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Sell);
        assert!(signal.confidence > 0.8);
        assert!(signal.explanation.contains("Bearish crossover"));
    }

    #[tokio::test]
    async fn test_rsi_strategy_oversold_buy_signal() {
        let strategy = Strategy::new(
            StrategyType::RSI { oversold: 30.0, overbought: 70.0 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create declining price data to generate low RSI
        let prices: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64 * 2.0)).collect();
        let historical_data = create_test_price_points(&prices);
        let market_data = create_test_market_data("AAPL", 62.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Buy);
        assert!(signal.confidence > 0.7);
        assert!(signal.explanation.contains("oversold"));
    }

    #[tokio::test]
    async fn test_rsi_strategy_overbought_sell_signal() {
        let strategy = Strategy::new(
            StrategyType::RSI { oversold: 30.0, overbought: 70.0 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Create rising price data to generate high RSI
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64 * 2.0)).collect();
        let historical_data = create_test_price_points(&prices);
        let market_data = create_test_market_data("AAPL", 138.0);

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Sell);
        assert!(signal.confidence > 0.7);
        assert!(signal.explanation.contains("overbought"));
    }

    #[tokio::test]
    async fn test_inactive_strategy_returns_hold() {
        let mut strategy = Strategy::new(
            StrategyType::PriceDrop { threshold: 5.0 },
            "AAPL".to_string(),
        ).unwrap();
        
        strategy.deactivate();
        let engine = StrategyEngine::new(strategy).unwrap();
        
        let historical_data = create_test_price_points(&[100.0]);
        let market_data = create_test_market_data("AAPL", 90.0); // 10% drop

        let signal = engine.execute(&market_data, &historical_data).await.unwrap();
        
        assert_eq!(signal.action, Action::Hold);
        assert!(signal.explanation.contains("inactive"));
    }

    #[tokio::test]
    async fn test_insufficient_data_error() {
        let strategy = Strategy::new(
            StrategyType::MovingAverage { short_period: 5, long_period: 10 },
            "AAPL".to_string(),
        ).unwrap();

        let engine = StrategyEngine::new(strategy).unwrap();
        
        // Only 3 data points, but need 10 for long MA
        let historical_data = create_test_price_points(&[100.0, 101.0, 102.0]);
        let market_data = create_test_market_data("AAPL", 102.0);

        let result = engine.execute(&market_data, &historical_data).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::TradingPlatformError::Strategy(StrategyError::InsufficientData) => {
                // This is the expected error
            }
            other => panic!("Expected InsufficientData error, got: {:?}", other),
        }
    }
}