// XAI explanation generation system

use crate::strategy::{Action, StrategyType, RiskLevel};
use crate::data::MarketData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplanationContext {
    pub strategy_type: StrategyType,
    pub action: Action,
    pub market_data: MarketData,
    pub strategy_data: HashMap<String, f64>,
    pub historical_context: Option<MarketContext>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Explanation {
    pub summary: String,
    pub detailed_reasoning: String,
    pub market_context: String,
    pub risk_factors: Vec<String>,
    pub confidence_level: f64,
    pub risk_level: RiskLevel,
    pub key_indicators: Vec<KeyIndicator>,
    pub alternative_scenarios: Vec<AlternativeScenario>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketContext {
    pub trend_direction: TrendDirection,
    pub volatility_level: VolatilityLevel,
    pub volume_analysis: VolumeAnalysis,
    pub support_resistance: Option<SupportResistance>,
    pub market_sentiment: MarketSentiment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyIndicator {
    pub name: String,
    pub value: f64,
    pub significance: IndicatorSignificance,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlternativeScenario {
    pub scenario_name: String,
    pub probability: f64,
    pub description: String,
    pub potential_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportResistance {
    pub support_level: f64,
    pub resistance_level: f64,
    pub current_position: PricePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    StrongUptrend,
    WeakUptrend,
    Sideways,
    WeakDowntrend,
    StrongDowntrend,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VolatilityLevel {
    VeryLow,
    Low,
    Normal,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VolumeAnalysis {
    VeryLow,
    Low,
    Normal,
    High,
    VeryHigh,
    Unusual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketSentiment {
    VeryBullish,
    Bullish,
    Neutral,
    Bearish,
    VeryBearish,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndicatorSignificance {
    Critical,
    Important,
    Moderate,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PricePosition {
    AboveResistance,
    NearResistance,
    BetweenLevels,
    NearSupport,
    BelowSupport,
}

// Implementation methods
impl ExplanationContext {
    pub fn new(
        strategy_type: StrategyType,
        action: Action,
        market_data: MarketData,
        strategy_data: HashMap<String, f64>,
    ) -> Self {
        ExplanationContext {
            strategy_type,
            action,
            market_data,
            strategy_data,
            historical_context: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_historical_context(mut self, context: MarketContext) -> Self {
        self.historical_context = Some(context);
        self
    }
}

impl Explanation {
    pub fn new(summary: String, detailed_reasoning: String) -> Self {
        Explanation {
            summary,
            detailed_reasoning,
            market_context: String::new(),
            risk_factors: Vec::new(),
            confidence_level: 0.5,
            risk_level: RiskLevel::Medium,
            key_indicators: Vec::new(),
            alternative_scenarios: Vec::new(),
            generated_at: Utc::now(),
        }
    }

    pub fn with_market_context(mut self, context: String) -> Self {
        self.market_context = context;
        self
    }

    pub fn with_risk_factors(mut self, factors: Vec<String>) -> Self {
        self.risk_factors = factors;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence_level = confidence.clamp(0.0, 1.0);
        self.risk_level = match confidence {
            c if c >= 0.8 => RiskLevel::Low,
            c if c >= 0.5 => RiskLevel::Medium,
            _ => RiskLevel::High,
        };
        self
    }

    pub fn add_key_indicator(&mut self, indicator: KeyIndicator) {
        self.key_indicators.push(indicator);
    }

    pub fn add_alternative_scenario(&mut self, scenario: AlternativeScenario) {
        self.alternative_scenarios.push(scenario);
    }

    pub fn is_high_confidence(&self) -> bool {
        self.confidence_level >= 0.8
    }

    pub fn is_low_risk(&self) -> bool {
        matches!(self.risk_level, RiskLevel::Low)
    }
}

impl MarketContext {
    pub fn new() -> Self {
        MarketContext {
            trend_direction: TrendDirection::Sideways,
            volatility_level: VolatilityLevel::Normal,
            volume_analysis: VolumeAnalysis::Normal,
            support_resistance: None,
            market_sentiment: MarketSentiment::Neutral,
        }
    }

    pub fn analyze_trend(&mut self, price_changes: &[f64]) {
        if price_changes.is_empty() {
            return;
        }

        let positive_changes = price_changes.iter().filter(|&&x| x > 0.0).count();
        let _negative_changes = price_changes.iter().filter(|&&x| x < 0.0).count();
        let total_changes = price_changes.len();

        let positive_ratio = positive_changes as f64 / total_changes as f64;
        let avg_change: f64 = price_changes.iter().sum::<f64>() / total_changes as f64;

        self.trend_direction = match (positive_ratio, avg_change) {
            (r, avg) if r > 0.7 && avg > 2.0 => TrendDirection::StrongUptrend,
            (r, avg) if r > 0.6 && avg > 0.5 => TrendDirection::WeakUptrend,
            (r, avg) if r < 0.3 && avg < -2.0 => TrendDirection::StrongDowntrend,
            (r, avg) if r < 0.4 && avg < -0.5 => TrendDirection::WeakDowntrend,
            _ => TrendDirection::Sideways,
        };
    }

    pub fn analyze_volatility(&mut self, price_changes: &[f64]) {
        if price_changes.is_empty() {
            return;
        }

        let mean = price_changes.iter().sum::<f64>() / price_changes.len() as f64;
        let variance = price_changes.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / price_changes.len() as f64;
        let std_dev = variance.sqrt();

        self.volatility_level = match std_dev {
            v if v < 0.5 => VolatilityLevel::VeryLow,
            v if v < 1.0 => VolatilityLevel::Low,
            v if v < 2.0 => VolatilityLevel::Normal,
            v if v < 4.0 => VolatilityLevel::High,
            _ => VolatilityLevel::VeryHigh,
        };
    }

    pub fn analyze_volume(&mut self, current_volume: u64, average_volume: u64) {
        let volume_ratio = current_volume as f64 / average_volume.max(1) as f64;

        self.volume_analysis = match volume_ratio {
            r if r < 0.3 => VolumeAnalysis::VeryLow,
            r if r < 0.7 => VolumeAnalysis::Low,
            r if r < 1.3 => VolumeAnalysis::Normal,
            r if r < 2.0 => VolumeAnalysis::High,
            r if r < 3.0 => VolumeAnalysis::VeryHigh,
            _ => VolumeAnalysis::Unusual,
        };
    }
}

impl KeyIndicator {
    pub fn new(name: String, value: f64, significance: IndicatorSignificance, description: String) -> Self {
        KeyIndicator {
            name,
            value,
            significance,
            description,
        }
    }

    pub fn critical(name: String, value: f64, description: String) -> Self {
        Self::new(name, value, IndicatorSignificance::Critical, description)
    }

    pub fn important(name: String, value: f64, description: String) -> Self {
        Self::new(name, value, IndicatorSignificance::Important, description)
    }
}

impl AlternativeScenario {
    pub fn new(scenario_name: String, probability: f64, description: String, potential_outcome: String) -> Self {
        AlternativeScenario {
            scenario_name,
            probability: probability.clamp(0.0, 1.0),
            description,
            potential_outcome,
        }
    }
}

impl Default for MarketContext {
    fn default() -> Self {
        Self::new()
    }
}

// Display implementations
impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::StrongUptrend => write!(f, "Strong Uptrend"),
            TrendDirection::WeakUptrend => write!(f, "Weak Uptrend"),
            TrendDirection::Sideways => write!(f, "Sideways"),
            TrendDirection::WeakDowntrend => write!(f, "Weak Downtrend"),
            TrendDirection::StrongDowntrend => write!(f, "Strong Downtrend"),
        }
    }
}

impl std::fmt::Display for VolatilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VolatilityLevel::VeryLow => write!(f, "Very Low"),
            VolatilityLevel::Low => write!(f, "Low"),
            VolatilityLevel::Normal => write!(f, "Normal"),
            VolatilityLevel::High => write!(f, "High"),
            VolatilityLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

impl std::fmt::Display for MarketSentiment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketSentiment::VeryBullish => write!(f, "Very Bullish"),
            MarketSentiment::Bullish => write!(f, "Bullish"),
            MarketSentiment::Neutral => write!(f, "Neutral"),
            MarketSentiment::Bearish => write!(f, "Bearish"),
            MarketSentiment::VeryBearish => write!(f, "Very Bearish"),
        }
    }
}