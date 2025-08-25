// Market data provider and management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{Result, MarketDataError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
    pub change: f64,
    pub change_percent: f64,
    pub market_cap: Option<u64>,
    pub day_high: Option<f64>,
    pub day_low: Option<f64>,
    pub previous_close: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adjusted_close: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoricalData {
    pub symbol: String,
    pub data_points: Vec<PricePoint>,
    pub period: TimePeriod,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimePeriod {
    OneDay,
    OneWeek,
    OneMonth,
    ThreeMonths,
    SixMonths,
    OneYear,
    TwoYears,
    FiveYears,
    Custom { days: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketIndicators {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub moving_averages: HashMap<usize, f64>, // period -> value
    pub rsi: Option<f64>,
    pub bollinger_bands: Option<BollingerBands>,
    pub volume_average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BollingerBands {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketSummary {
    pub symbol: String,
    pub current_price: f64,
    pub daily_change: f64,
    pub daily_change_percent: f64,
    pub volume: u64,
    pub market_status: MarketStatus,
    pub last_trade_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketStatus {
    Open,
    Closed,
    PreMarket,
    AfterHours,
    Holiday,
}

// Implementation methods
impl MarketData {
    pub fn new(symbol: String, price: f64, volume: u64) -> Self {
        MarketData {
            symbol,
            price,
            volume,
            timestamp: Utc::now(),
            change: 0.0,
            change_percent: 0.0,
            market_cap: None,
            day_high: None,
            day_low: None,
            previous_close: None,
        }
    }

    pub fn with_change(mut self, previous_close: f64) -> Self {
        self.change = self.price - previous_close;
        self.change_percent = if previous_close != 0.0 {
            (self.change / previous_close) * 100.0
        } else {
            0.0
        };
        self.previous_close = Some(previous_close);
        self
    }

    pub fn with_day_range(mut self, high: f64, low: f64) -> Self {
        self.day_high = Some(high);
        self.day_low = Some(low);
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.symbol.is_empty() {
            return Err(MarketDataError::InvalidFormat.into());
        }

        if self.price <= 0.0 {
            return Err(MarketDataError::InvalidFormat.into());
        }

        if let (Some(high), Some(low)) = (self.day_high, self.day_low) {
            if high < low {
                return Err(MarketDataError::InvalidFormat.into());
            }
            if self.price > high || self.price < low {
                return Err(MarketDataError::InvalidFormat.into());
            }
        }

        Ok(())
    }

    pub fn is_significant_change(&self, threshold: f64) -> bool {
        self.change_percent.abs() >= threshold
    }
}

impl PricePoint {
    pub fn new(timestamp: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64, volume: u64) -> Result<Self> {
        if high < low || open < 0.0 || close < 0.0 || high < 0.0 || low < 0.0 {
            return Err(MarketDataError::InvalidFormat.into());
        }

        if open > high || open < low || close > high || close < low {
            return Err(MarketDataError::InvalidFormat.into());
        }

        Ok(PricePoint {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            adjusted_close: None,
        })
    }

    pub fn with_adjusted_close(mut self, adjusted_close: f64) -> Self {
        self.adjusted_close = Some(adjusted_close);
        self
    }

    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    pub fn price_range(&self) -> f64 {
        self.high - self.low
    }

    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}

impl HistoricalData {
    pub fn new(symbol: String, period: TimePeriod) -> Self {
        HistoricalData {
            symbol,
            data_points: Vec::new(),
            period,
            last_updated: Utc::now(),
        }
    }

    pub fn add_price_point(&mut self, price_point: PricePoint) {
        self.data_points.push(price_point);
        self.data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        self.last_updated = Utc::now();
    }

    pub fn get_latest(&self) -> Option<&PricePoint> {
        self.data_points.last()
    }

    pub fn get_closing_prices(&self) -> Vec<f64> {
        self.data_points.iter().map(|p| p.close).collect()
    }

    pub fn get_volumes(&self) -> Vec<u64> {
        self.data_points.iter().map(|p| p.volume).collect()
    }

    pub fn calculate_simple_moving_average(&self, period: usize) -> Option<f64> {
        if self.data_points.len() < period {
            return None;
        }

        let sum: f64 = self.data_points
            .iter()
            .rev()
            .take(period)
            .map(|p| p.close)
            .sum();

        Some(sum / period as f64)
    }

    pub fn is_stale(&self, max_age_minutes: i64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.last_updated);
        age.num_minutes() > max_age_minutes
    }
}

impl TimePeriod {
    pub fn to_days(&self) -> u32 {
        match self {
            TimePeriod::OneDay => 1,
            TimePeriod::OneWeek => 7,
            TimePeriod::OneMonth => 30,
            TimePeriod::ThreeMonths => 90,
            TimePeriod::SixMonths => 180,
            TimePeriod::OneYear => 365,
            TimePeriod::TwoYears => 730,
            TimePeriod::FiveYears => 1825,
            TimePeriod::Custom { days } => *days,
        }
    }
}

impl MarketIndicators {
    pub fn new(symbol: String) -> Self {
        MarketIndicators {
            symbol,
            timestamp: Utc::now(),
            moving_averages: HashMap::new(),
            rsi: None,
            bollinger_bands: None,
            volume_average: None,
        }
    }

    pub fn add_moving_average(&mut self, period: usize, value: f64) {
        self.moving_averages.insert(period, value);
    }

    pub fn get_moving_average(&self, period: usize) -> Option<f64> {
        self.moving_averages.get(&period).copied()
    }
}

impl Default for MarketStatus {
    fn default() -> Self {
        MarketStatus::Closed
    }
}

// Display implementations
impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatus::Open => write!(f, "OPEN"),
            MarketStatus::Closed => write!(f, "CLOSED"),
            MarketStatus::PreMarket => write!(f, "PRE_MARKET"),
            MarketStatus::AfterHours => write!(f, "AFTER_HOURS"),
            MarketStatus::Holiday => write!(f, "HOLIDAY"),
        }
    }
}

impl std::fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimePeriod::OneDay => write!(f, "1D"),
            TimePeriod::OneWeek => write!(f, "1W"),
            TimePeriod::OneMonth => write!(f, "1M"),
            TimePeriod::ThreeMonths => write!(f, "3M"),
            TimePeriod::SixMonths => write!(f, "6M"),
            TimePeriod::OneYear => write!(f, "1Y"),
            TimePeriod::TwoYears => write!(f, "2Y"),
            TimePeriod::FiveYears => write!(f, "5Y"),
            TimePeriod::Custom { days } => write!(f, "{}D", days),
        }
    }
}