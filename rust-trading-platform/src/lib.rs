pub mod api;
pub mod config;
pub mod data;
pub mod database;
pub mod error;
pub mod market_data;
pub mod performance;
pub mod strategy;
pub mod xai;

pub use config::Config;
pub use error::{Result, TradingPlatformError};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::*;
    use crate::data::*;
    use crate::performance::*;
    use crate::xai::*;
    use crate::database::*;
    use crate::database::repositories::{StrategyRepository, TradeRepository, SqliteStrategyRepository, SqliteTradeRepository};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_config_loading() {
        let config = Config::default();
        assert_eq!(config.server.port, 8000);
        assert_eq!(config.server.websocket_port, 8001);
        assert_eq!(config.server.static_port, 8050);
        assert_eq!(config.database.url, "sqlite:trading_platform.db");
        assert_eq!(config.strategies.initial_cash, 10000.0);
    }

    #[test]
    fn test_strategy_creation_and_validation() {
        // Test valid price drop strategy
        let price_drop = StrategyType::PriceDrop { threshold: 5.0 };
        let strategy = Strategy::new(price_drop, "AAPL".to_string()).unwrap();
        assert!(strategy.validate().is_ok());
        assert_eq!(strategy.symbol, "AAPL");
        assert!(strategy.is_active);

        // Test invalid strategy parameters
        let invalid_price_drop = StrategyType::PriceDrop { threshold: -5.0 };
        let invalid_strategy = Strategy::new(invalid_price_drop, "AAPL".to_string()).unwrap();
        assert!(invalid_strategy.validate().is_err());

        // Test moving average validation
        let invalid_ma = StrategyType::MovingAverage { short_period: 30, long_period: 10 };
        let invalid_ma_strategy = Strategy::new(invalid_ma, "AAPL".to_string()).unwrap();
        assert!(invalid_ma_strategy.validate().is_err());

        // Test RSI validation
        let invalid_rsi = StrategyType::RSI { oversold: 80.0, overbought: 20.0 };
        let invalid_rsi_strategy = Strategy::new(invalid_rsi, "AAPL".to_string()).unwrap();
        assert!(invalid_rsi_strategy.validate().is_err());
    }

    #[test]
    fn test_trading_signal_creation() {
        let mut strategy_data = HashMap::new();
        strategy_data.insert("price_change".to_string(), -5.2);
        strategy_data.insert("threshold".to_string(), 5.0);

        let signal = TradingSignal::new(
            "strategy-123".to_string(),
            "AAPL".to_string(),
            Action::Buy,
            150.0,
            "Price dropped below threshold".to_string(),
            0.85,
            strategy_data,
        );

        assert_eq!(signal.action, Action::Buy);
        assert_eq!(signal.price, 150.0);
        assert_eq!(signal.confidence, 0.85);
        assert_eq!(signal.metadata.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_market_data_validation() {
        let mut market_data = MarketData::new("AAPL".to_string(), 150.0, 1000000);
        market_data = market_data.with_change(145.0).with_day_range(152.0, 148.0);
        
        assert!(market_data.validate().is_ok());
        assert_eq!(market_data.change, 5.0);
        assert!((market_data.change_percent - 3.45).abs() < 0.01);

        // Test invalid data
        let invalid_data = MarketData::new("".to_string(), -10.0, 0);
        assert!(invalid_data.validate().is_err());
    }

    #[test]
    fn test_price_point_creation() {
        let timestamp = Utc::now();
        let price_point = PricePoint::new(timestamp, 100.0, 105.0, 98.0, 103.0, 50000).unwrap();
        
        assert_eq!(price_point.typical_price(), 102.0);
        assert_eq!(price_point.price_range(), 7.0);
        assert_eq!(price_point.body_size(), 3.0);
        assert!(price_point.is_bullish());
        assert!(!price_point.is_bearish());

        // Test invalid price point
        let invalid_point = PricePoint::new(timestamp, 100.0, 95.0, 98.0, 103.0, 50000);
        assert!(invalid_point.is_err());
    }

    #[test]
    fn test_historical_data_operations() {
        let mut historical = HistoricalData::new("AAPL".to_string(), TimePeriod::OneWeek);
        
        let timestamp = Utc::now();
        let price_point = PricePoint::new(timestamp, 100.0, 105.0, 98.0, 103.0, 50000).unwrap();
        historical.add_price_point(price_point);

        assert_eq!(historical.data_points.len(), 1);
        assert!(historical.get_latest().is_some());
        assert_eq!(historical.get_closing_prices(), vec![103.0]);
        assert_eq!(historical.get_volumes(), vec![50000]);

        // Test moving average calculation
        for i in 1..=10 {
            let point = PricePoint::new(
                timestamp,
                100.0 + i as f64,
                105.0 + i as f64,
                98.0 + i as f64,
                103.0 + i as f64,
                50000,
            ).unwrap();
            historical.add_price_point(point);
        }

        let ma_5 = historical.calculate_simple_moving_average(5);
        assert!(ma_5.is_some());
        assert!(ma_5.unwrap() > 103.0);
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new("strategy-123".to_string(), 10000.0);
        
        // Create some sample trades
        let mut trades = Vec::new();
        trades.push(Trade::new(
            "strategy-123".to_string(),
            "AAPL".to_string(),
            Action::Buy,
            10.0,
            100.0,
            "Buy signal".to_string(),
            5.0,
        ).with_realized_pnl(50.0));
        
        trades.push(Trade::new(
            "strategy-123".to_string(),
            "AAPL".to_string(),
            Action::Sell,
            10.0,
            105.0,
            "Sell signal".to_string(),
            5.0,
        ).with_realized_pnl(-20.0));

        metrics.update_from_trades(&trades);
        
        assert_eq!(metrics.total_trades, 2);
        assert_eq!(metrics.winning_trades, 1);
        assert_eq!(metrics.losing_trades, 1);
        assert_eq!(metrics.total_return, 30.0);
        assert_eq!(metrics.win_rate, 50.0);
    }

    #[test]
    fn test_position_management() {
        let mut position = Position::new("AAPL".to_string(), 100.0, 50.0);
        assert_eq!(position.cost_basis, 5000.0);
        assert_eq!(position.average_price, 50.0);

        // Update price
        position.update_price(55.0);
        assert_eq!(position.unrealized_pnl, 500.0);
        assert_eq!(position.unrealized_pnl_percent, 10.0);

        // Add more shares
        position.add_shares(50.0, 60.0);
        assert_eq!(position.shares, 150.0);
        assert!((position.average_price - 53.33).abs() < 0.01);

        // Remove shares
        let realized_pnl = position.remove_shares(50.0, 65.0).unwrap();
        assert!(realized_pnl > 0.0);
        assert_eq!(position.shares, 100.0);
    }

    #[test]
    fn test_portfolio_operations() {
        let mut portfolio = Portfolio::new("strategy-123".to_string(), 10000.0);
        
        let buy_trade = Trade::new(
            "strategy-123".to_string(),
            "AAPL".to_string(),
            Action::Buy,
            10.0,
            100.0,
            "Buy signal".to_string(),
            5.0,
        );

        portfolio.execute_trade(buy_trade).unwrap();
        assert_eq!(portfolio.current_capital, 8995.0); // 10000 - 1000 - 5
        assert_eq!(portfolio.positions.len(), 1);

        // Update position prices
        let mut prices = HashMap::new();
        prices.insert("AAPL".to_string(), 110.0);
        portfolio.update_position_prices(&prices);

        assert_eq!(portfolio.total_value(), 10095.0); // 8995 + 1100
        assert_eq!(portfolio.total_unrealized_pnl(), 100.0);
    }

    #[test]
    fn test_explanation_context() {
        let strategy_type = StrategyType::PriceDrop { threshold: 5.0 };
        let market_data = MarketData::new("AAPL".to_string(), 150.0, 1000000);
        let mut strategy_data = HashMap::new();
        strategy_data.insert("price_change".to_string(), -5.2);

        let context = ExplanationContext::new(
            strategy_type,
            Action::Buy,
            market_data,
            strategy_data,
        );

        assert_eq!(context.action, Action::Buy);
        assert_eq!(context.market_data.symbol, "AAPL");
        assert!(context.historical_context.is_none());
    }

    #[test]
    fn test_market_context_analysis() {
        let mut context = MarketContext::new();
        
        // Test trend analysis
        let price_changes = vec![1.0, 2.0, -0.5, 1.5, 2.5, 1.0, 3.0];
        context.analyze_trend(&price_changes);
        assert!(matches!(context.trend_direction, TrendDirection::WeakUptrend | TrendDirection::StrongUptrend));

        // Test volatility analysis
        context.analyze_volatility(&price_changes);
        assert!(matches!(context.volatility_level, VolatilityLevel::Low | VolatilityLevel::Normal));

        // Test volume analysis
        context.analyze_volume(2000000, 1000000);
        assert!(matches!(context.volume_analysis, VolumeAnalysis::High | VolumeAnalysis::VeryHigh));
    }

    #[test]
    fn test_explanation_building() {
        let mut explanation = Explanation::new(
            "Buy signal generated".to_string(),
            "Price dropped below threshold".to_string(),
        );

        explanation = explanation
            .with_confidence(0.85)
            .with_market_context("Strong downward movement".to_string())
            .with_risk_factors(vec!["Market volatility".to_string()]);

        explanation.add_key_indicator(KeyIndicator::critical(
            "Price Change".to_string(),
            -5.2,
            "Significant price drop".to_string(),
        ));

        explanation.add_alternative_scenario(AlternativeScenario::new(
            "Continued decline".to_string(),
            0.3,
            "Price may continue falling".to_string(),
            "Additional losses".to_string(),
        ));

        assert!(explanation.is_high_confidence());
        assert!(explanation.is_low_risk());
        assert_eq!(explanation.key_indicators.len(), 1);
        assert_eq!(explanation.alternative_scenarios.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let strategy_type = StrategyType::PriceDrop { threshold: 5.0 };
        let strategy = Strategy::new(strategy_type, "AAPL".to_string()).unwrap();
        
        // Test JSON serialization
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: Strategy = serde_json::from_str(&json).unwrap();
        assert_eq!(strategy, deserialized);

        // Test market data serialization
        let market_data = MarketData::new("AAPL".to_string(), 150.0, 1000000);
        let json = serde_json::to_string(&market_data).unwrap();
        let deserialized: MarketData = serde_json::from_str(&json).unwrap();
        assert_eq!(market_data, deserialized);
    }

    async fn create_test_database() -> Database {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_database_initialization() {
        let db = create_test_database().await;
        
        // Test health check
        assert!(db.health_check().await.is_ok());
        
        // Test stats
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_strategies, 0);
        assert_eq!(stats.total_trades, 0);
        assert_eq!(stats.active_strategies, 0);
    }

    #[tokio::test]
    async fn test_strategy_repository_operations() {
        let db = create_test_database().await;
        let repo = SqliteStrategyRepository::new(db.pool().clone());

        // Create test strategy
        let strategy_type = StrategyType::PriceDrop { threshold: 5.0 };
        let strategy = Strategy::new(strategy_type, "AAPL".to_string()).unwrap();

        // Test create
        repo.create(&strategy).await.unwrap();

        // Test get by id
        let retrieved = repo.get_by_id(&strategy.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, strategy.id);
        assert_eq!(retrieved.symbol, strategy.symbol);

        // Test get all
        let all_strategies = repo.get_all().await.unwrap();
        assert_eq!(all_strategies.len(), 1);

        // Test get active
        let active_strategies = repo.get_active().await.unwrap();
        assert_eq!(active_strategies.len(), 1);

        // Test get by symbol
        let symbol_strategies = repo.get_by_symbol("AAPL").await.unwrap();
        assert_eq!(symbol_strategies.len(), 1);

        // Test update
        let mut updated_strategy = strategy.clone();
        updated_strategy.is_active = false;
        repo.update(&updated_strategy).await.unwrap();

        let retrieved = repo.get_by_id(&strategy.id).await.unwrap().unwrap();
        assert!(!retrieved.is_active);

        // Test set active
        repo.set_active(&strategy.id, true).await.unwrap();
        let retrieved = repo.get_by_id(&strategy.id).await.unwrap().unwrap();
        assert!(retrieved.is_active);

        // Test delete
        repo.delete(&strategy.id).await.unwrap();
        let retrieved = repo.get_by_id(&strategy.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_trade_repository_operations() {
        let db = create_test_database().await;
        let trade_repo = SqliteTradeRepository::new(db.pool().clone());
        let strategy_repo = SqliteStrategyRepository::new(db.pool().clone());

        // First create a strategy to satisfy foreign key constraint
        let strategy_type = StrategyType::PriceDrop { threshold: 5.0 };
        let strategy = Strategy::new(strategy_type, "AAPL".to_string()).unwrap();
        strategy_repo.create(&strategy).await.unwrap();

        // Create test trade
        let trade = Trade::new(
            strategy.id.clone(),
            "AAPL".to_string(),
            Action::Buy,
            100.0,
            150.0,
            "Test trade".to_string(),
            5.0,
        ).with_realized_pnl(50.0);

        // Test create
        trade_repo.create(&trade).await.unwrap();

        // Test get by id
        let retrieved = trade_repo.get_by_id(&trade.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, trade.id);
        assert_eq!(retrieved.symbol, trade.symbol);
        assert_eq!(retrieved.action, trade.action);

        // Test get by strategy
        let strategy_trades = trade_repo.get_by_strategy(&trade.strategy_id).await.unwrap();
        assert_eq!(strategy_trades.len(), 1);

        // Test get by symbol
        let symbol_trades = trade_repo.get_by_symbol("AAPL").await.unwrap();
        assert_eq!(symbol_trades.len(), 1);

        // Test get recent
        let recent_trades = trade_repo.get_recent(10).await.unwrap();
        assert_eq!(recent_trades.len(), 1);

        // Test get by date range
        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let range_trades = trade_repo.get_by_date_range(start, end).await.unwrap();
        assert_eq!(range_trades.len(), 1);

        // Test update
        let mut updated_trade = trade.clone();
        updated_trade.explanation = "Updated explanation".to_string();
        trade_repo.update(&updated_trade).await.unwrap();

        let retrieved = trade_repo.get_by_id(&trade.id).await.unwrap().unwrap();
        assert_eq!(retrieved.explanation, "Updated explanation");

        // Test delete
        trade_repo.delete(&trade.id).await.unwrap();
        let retrieved = trade_repo.get_by_id(&trade.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_database_migrations() {
        // Test database creation and migration
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();

        // Verify schema is correct
        let schema_valid = crate::database::migrations::check_schema_version(db.pool()).await.unwrap();
        assert!(schema_valid);

        // Get schema info
        let tables = crate::database::migrations::get_schema_info(db.pool()).await.unwrap();
        assert!(tables.len() >= 4);

        let table_names: Vec<String> = tables.iter().map(|t| t.name.clone()).collect();
        assert!(table_names.contains(&"strategies".to_string()));
        assert!(table_names.contains(&"trades".to_string()));
        assert!(table_names.contains(&"performance_snapshots".to_string()));
        assert!(table_names.contains(&"market_data_cache".to_string()));
    }

    #[tokio::test]
    async fn test_database_error_handling() {
        let db = create_test_database().await;
        let repo = SqliteStrategyRepository::new(db.pool().clone());

        // Test get non-existent strategy
        let result = repo.get_by_id("non-existent-id").await.unwrap();
        assert!(result.is_none());

        // Test delete non-existent strategy
        let result = repo.delete("non-existent-id").await;
        assert!(result.is_err());

        // Test update non-existent strategy
        let strategy_type = StrategyType::PriceDrop { threshold: 5.0 };
        let strategy = Strategy::new(strategy_type, "AAPL".to_string()).unwrap();
        let result = repo.update(&strategy).await;
        assert!(result.is_err());
    }

    // Market Data Provider Tests
    #[tokio::test]
    async fn test_mock_market_data_provider() {
        use crate::market_data::{MockMarketDataProvider, MarketDataProvider};
        use crate::data::TimePeriod;

        let provider = MockMarketDataProvider::new_with_seed(42);

        // Test current price
        let market_data = provider.get_current_price("AAPL").await.unwrap();
        assert_eq!(market_data.symbol, "AAPL");
        assert!(market_data.price > 0.0);
        assert!(market_data.volume > 0);

        // Test historical data
        let historical_data = provider.get_historical_data("AAPL", TimePeriod::OneWeek).await.unwrap();
        assert_eq!(historical_data.symbol, "AAPL");
        assert!(!historical_data.data_points.is_empty());
        assert_eq!(historical_data.period, TimePeriod::OneWeek);

        // Test multiple prices
        let symbols = vec!["AAPL".to_string(), "GOOGL".to_string()];
        let prices = provider.get_multiple_prices(&symbols).await.unwrap();
        assert_eq!(prices.len(), 2);
        assert!(prices.contains_key("AAPL"));
        assert!(prices.contains_key("GOOGL"));

        // Test health check
        assert!(provider.health_check().await.is_ok());
        assert_eq!(provider.provider_name(), "Mock Provider");
    }

    #[tokio::test]
    async fn test_market_data_provider_factory() {
        use crate::market_data::{MarketDataProviderFactory, MarketDataConfig};

        // Test mock provider creation
        let mut config = MarketDataConfig::default();
        config.provider = "mock".to_string();
        
        let provider = MarketDataProviderFactory::create_provider(&config).unwrap();
        assert_eq!(provider.provider_name(), "Mock Provider");

        // Test Yahoo Finance provider creation
        config.provider = "yahoo_finance".to_string();
        let provider = MarketDataProviderFactory::create_provider(&config).unwrap();
        assert_eq!(provider.provider_name(), "Yahoo Finance");

        // Test unknown provider
        config.provider = "unknown".to_string();
        let result = MarketDataProviderFactory::create_provider(&config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_market_data_provider_error_handling() {
        use crate::market_data::{MockMarketDataProvider, MarketDataProvider};
        use crate::error::{TradingPlatformError, MarketDataError};

        let provider = MockMarketDataProvider::new();

        // Test unknown symbol
        let result = provider.get_current_price("UNKNOWN").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TradingPlatformError::MarketData(MarketDataError::SymbolNotFound(symbol)) => {
                assert_eq!(symbol, "UNKNOWN");
            }
            _ => panic!("Expected SymbolNotFound error"),
        }

        // Test provider unavailable
        provider.set_health_status(false);
        let result = provider.get_current_price("AAPL").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TradingPlatformError::MarketData(MarketDataError::ProviderUnavailable) => {}
            _ => panic!("Expected ProviderUnavailable error"),
        }
    }

    #[tokio::test]
    async fn test_retry_policy() {
        use crate::market_data::RetryPolicy;
        use std::time::Duration;

        let policy = RetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };

        let mut call_count = 0;
        
        // Test successful retry
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

    #[test]
    fn test_market_data_config() {
        use crate::market_data::MarketDataConfig;

        let config = MarketDataConfig::default();
        assert_eq!(config.provider, "yahoo_finance");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_url, "https://query1.finance.yahoo.com");
    }

    #[test]
    fn test_rate_limit_info() {
        use crate::market_data::RateLimitInfo;

        let info = RateLimitInfo::default();
        assert_eq!(info.requests_per_minute, 60);
        assert_eq!(info.requests_per_hour, 2000);
        assert_eq!(info.current_usage, 0);
        assert!(info.reset_time.is_none());
    }
}