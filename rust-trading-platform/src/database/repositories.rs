// Repository implementations for database operations

use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc};

use crate::error::{Result, TradingPlatformError};
use crate::strategy::{Strategy, StrategyType, Action};
use crate::performance::{Trade, PerformanceMetrics};
use crate::data::MarketData;
use crate::database::{serialize_json, deserialize_json, datetime_to_string, string_to_datetime};

// Repository traits
#[async_trait]
pub trait StrategyRepository {
    async fn create(&self, strategy: &Strategy) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Strategy>>;
    async fn get_all(&self) -> Result<Vec<Strategy>>;
    async fn get_active(&self) -> Result<Vec<Strategy>>;
    async fn get_by_symbol(&self, symbol: &str) -> Result<Vec<Strategy>>;
    async fn update(&self, strategy: &Strategy) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn set_active(&self, id: &str, active: bool) -> Result<()>;
}

#[async_trait]
pub trait TradeRepository {
    async fn create(&self, trade: &Trade) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Trade>>;
    async fn get_by_strategy(&self, strategy_id: &str) -> Result<Vec<Trade>>;
    async fn get_by_symbol(&self, symbol: &str) -> Result<Vec<Trade>>;
    async fn get_recent(&self, limit: u32) -> Result<Vec<Trade>>;
    async fn get_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Trade>>;
    async fn update(&self, trade: &Trade) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
}

#[async_trait]
pub trait PerformanceRepository {
    async fn create_snapshot(&self, strategy_id: &str, metrics: &PerformanceMetrics) -> Result<()>;
    async fn get_latest(&self, strategy_id: &str) -> Result<Option<PerformanceMetrics>>;
    async fn get_history(&self, strategy_id: &str, limit: u32) -> Result<Vec<PerformanceMetrics>>;
    async fn delete_old_snapshots(&self, strategy_id: &str, keep_count: u32) -> Result<()>;
}

#[async_trait]
pub trait MarketDataRepository {
    async fn cache_market_data(&self, data: &MarketData) -> Result<()>;
    async fn get_cached_data(&self, symbol: &str, limit: u32) -> Result<Vec<MarketData>>;
    async fn get_latest_cached(&self, symbol: &str) -> Result<Option<MarketData>>;
    async fn cleanup_old_cache(&self, older_than: DateTime<Utc>) -> Result<u32>;
}

// Repository implementations
pub struct SqliteStrategyRepository {
    pool: SqlitePool,
}

impl SqliteStrategyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StrategyRepository for SqliteStrategyRepository {
    async fn create(&self, strategy: &Strategy) -> Result<()> {
        let strategy_type_json = serialize_json(&strategy.strategy_type)?;
        let parameters_json = serialize_json(&strategy.parameters)?;
        let created_at = datetime_to_string(strategy.created_at);

        sqlx::query(
            r#"
            INSERT INTO strategies (id, strategy_type, symbol, parameters, created_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&strategy.id)
        .bind(&strategy_type_json)
        .bind(&strategy.symbol)
        .bind(&parameters_json)
        .bind(&created_at)
        .bind(strategy.is_active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Strategy>> {
        let row = sqlx::query(
            "SELECT id, strategy_type, symbol, parameters, created_at, is_active FROM strategies WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let strategy = row_to_strategy(row)?;
                Ok(Some(strategy))
            }
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> Result<Vec<Strategy>> {
        let rows = sqlx::query(
            "SELECT id, strategy_type, symbol, parameters, created_at, is_active FROM strategies ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut strategies = Vec::new();
        for row in rows {
            strategies.push(row_to_strategy(row)?);
        }

        Ok(strategies)
    }

    async fn get_active(&self) -> Result<Vec<Strategy>> {
        let rows = sqlx::query(
            "SELECT id, strategy_type, symbol, parameters, created_at, is_active FROM strategies WHERE is_active = TRUE ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut strategies = Vec::new();
        for row in rows {
            strategies.push(row_to_strategy(row)?);
        }

        Ok(strategies)
    }

    async fn get_by_symbol(&self, symbol: &str) -> Result<Vec<Strategy>> {
        let rows = sqlx::query(
            "SELECT id, strategy_type, symbol, parameters, created_at, is_active FROM strategies WHERE symbol = ? ORDER BY created_at DESC"
        )
        .bind(symbol)
        .fetch_all(&self.pool)
        .await?;

        let mut strategies = Vec::new();
        for row in rows {
            strategies.push(row_to_strategy(row)?);
        }

        Ok(strategies)
    }

    async fn update(&self, strategy: &Strategy) -> Result<()> {
        let strategy_type_json = serialize_json(&strategy.strategy_type)?;
        let parameters_json = serialize_json(&strategy.parameters)?;

        let result = sqlx::query(
            r#"
            UPDATE strategies 
            SET strategy_type = ?, symbol = ?, parameters = ?, is_active = ?
            WHERE id = ?
            "#
        )
        .bind(&strategy_type_json)
        .bind(&strategy.symbol)
        .bind(&parameters_json)
        .bind(strategy.is_active)
        .bind(&strategy.id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TradingPlatformError::internal("Strategy not found for update"));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM strategies WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TradingPlatformError::internal("Strategy not found for deletion"));
        }

        Ok(())
    }

    async fn set_active(&self, id: &str, active: bool) -> Result<()> {
        let result = sqlx::query("UPDATE strategies SET is_active = ? WHERE id = ?")
            .bind(active)
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TradingPlatformError::internal("Strategy not found"));
        }

        Ok(())
    }
}

pub struct SqliteTradeRepository {
    pool: SqlitePool,
}

impl SqliteTradeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TradeRepository for SqliteTradeRepository {
    async fn create(&self, trade: &Trade) -> Result<()> {
        let timestamp = datetime_to_string(trade.timestamp);
        let action_str = trade.action.to_string();

        sqlx::query(
            r#"
            INSERT INTO trades (id, strategy_id, symbol, action, quantity, price, timestamp, explanation, commission, realized_pnl, trade_value)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&trade.id)
        .bind(&trade.strategy_id)
        .bind(&trade.symbol)
        .bind(&action_str)
        .bind(trade.quantity)
        .bind(trade.price)
        .bind(&timestamp)
        .bind(&trade.explanation)
        .bind(trade.commission)
        .bind(trade.realized_pnl)
        .bind(trade.trade_value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Trade>> {
        let row = sqlx::query(
            "SELECT id, strategy_id, symbol, action, quantity, price, timestamp, explanation, commission, realized_pnl, trade_value FROM trades WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let trade = row_to_trade(row)?;
                Ok(Some(trade))
            }
            None => Ok(None),
        }
    }

    async fn get_by_strategy(&self, strategy_id: &str) -> Result<Vec<Trade>> {
        let rows = sqlx::query(
            "SELECT id, strategy_id, symbol, action, quantity, price, timestamp, explanation, commission, realized_pnl, trade_value FROM trades WHERE strategy_id = ? ORDER BY timestamp DESC"
        )
        .bind(strategy_id)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            trades.push(row_to_trade(row)?);
        }

        Ok(trades)
    }

    async fn get_by_symbol(&self, symbol: &str) -> Result<Vec<Trade>> {
        let rows = sqlx::query(
            "SELECT id, strategy_id, symbol, action, quantity, price, timestamp, explanation, commission, realized_pnl, trade_value FROM trades WHERE symbol = ? ORDER BY timestamp DESC"
        )
        .bind(symbol)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            trades.push(row_to_trade(row)?);
        }

        Ok(trades)
    }

    async fn get_recent(&self, limit: u32) -> Result<Vec<Trade>> {
        let rows = sqlx::query(
            "SELECT id, strategy_id, symbol, action, quantity, price, timestamp, explanation, commission, realized_pnl, trade_value FROM trades ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            trades.push(row_to_trade(row)?);
        }

        Ok(trades)
    }

    async fn get_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Trade>> {
        let start_str = datetime_to_string(start);
        let end_str = datetime_to_string(end);

        let rows = sqlx::query(
            "SELECT id, strategy_id, symbol, action, quantity, price, timestamp, explanation, commission, realized_pnl, trade_value FROM trades WHERE timestamp BETWEEN ? AND ? ORDER BY timestamp DESC"
        )
        .bind(&start_str)
        .bind(&end_str)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            trades.push(row_to_trade(row)?);
        }

        Ok(trades)
    }

    async fn update(&self, trade: &Trade) -> Result<()> {
        let timestamp = datetime_to_string(trade.timestamp);
        let action_str = trade.action.to_string();

        let result = sqlx::query(
            r#"
            UPDATE trades 
            SET strategy_id = ?, symbol = ?, action = ?, quantity = ?, price = ?, timestamp = ?, explanation = ?, commission = ?, realized_pnl = ?, trade_value = ?
            WHERE id = ?
            "#
        )
        .bind(&trade.strategy_id)
        .bind(&trade.symbol)
        .bind(&action_str)
        .bind(trade.quantity)
        .bind(trade.price)
        .bind(&timestamp)
        .bind(&trade.explanation)
        .bind(trade.commission)
        .bind(trade.realized_pnl)
        .bind(trade.trade_value)
        .bind(&trade.id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TradingPlatformError::internal("Trade not found for update"));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM trades WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TradingPlatformError::internal("Trade not found for deletion"));
        }

        Ok(())
    }
}

// Helper functions to convert database rows to structs
fn row_to_strategy(row: sqlx::sqlite::SqliteRow) -> Result<Strategy> {
    let id: String = row.get("id");
    let strategy_type_json: String = row.get("strategy_type");
    let symbol: String = row.get("symbol");
    let parameters_json: String = row.get("parameters");
    let created_at_str: String = row.get("created_at");
    let is_active: bool = row.get("is_active");

    let strategy_type: StrategyType = deserialize_json(&strategy_type_json)?;
    let parameters = deserialize_json(&parameters_json)?;
    let created_at = string_to_datetime(&created_at_str)?;

    Ok(Strategy {
        id,
        strategy_type,
        symbol,
        parameters,
        created_at,
        is_active,
    })
}

fn row_to_trade(row: sqlx::sqlite::SqliteRow) -> Result<Trade> {
    let id: String = row.get("id");
    let strategy_id: String = row.get("strategy_id");
    let symbol: String = row.get("symbol");
    let action_str: String = row.get("action");
    let quantity: f64 = row.get("quantity");
    let price: f64 = row.get("price");
    let timestamp_str: String = row.get("timestamp");
    let explanation: Option<String> = row.get("explanation");
    let commission: Option<f64> = row.get("commission");
    let realized_pnl: Option<f64> = row.get("realized_pnl");
    let trade_value: f64 = row.get("trade_value");

    let action = match action_str.as_str() {
        "BUY" => Action::Buy,
        "SELL" => Action::Sell,
        "HOLD" => Action::Hold,
        _ => return Err(TradingPlatformError::internal(format!("Invalid action: {}", action_str))),
    };

    let timestamp = string_to_datetime(&timestamp_str)?;

    Ok(Trade {
        id,
        strategy_id,
        symbol,
        action,
        quantity,
        price,
        timestamp,
        explanation: explanation.unwrap_or_default(),
        commission: commission.unwrap_or(0.0),
        realized_pnl,
        trade_value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{StrategyType, StrategyParameters};
    use crate::strategy::Action;

    use uuid::Uuid;

    async fn create_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_strategy_repository() {
        let pool = create_test_db().await;
        let repo = SqliteStrategyRepository::new(pool);

        // Create a test strategy
        let strategy = Strategy {
            id: Uuid::new_v4().to_string(),
            strategy_type: StrategyType::PriceDrop { threshold: 5.0 },
            symbol: "AAPL".to_string(),
            parameters: StrategyParameters::default(),
            created_at: Utc::now(),
            is_active: true,
        };

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

        // Test set inactive
        repo.set_active(&strategy.id, false).await.unwrap();
        let active_strategies = repo.get_active().await.unwrap();
        assert_eq!(active_strategies.len(), 0);

        // Test delete
        repo.delete(&strategy.id).await.unwrap();
        let retrieved = repo.get_by_id(&strategy.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_trade_repository() {
        let pool = create_test_db().await;
        let trade_repo = SqliteTradeRepository::new(pool.clone());
        let strategy_repo = SqliteStrategyRepository::new(pool);

        // First create a strategy to satisfy foreign key constraint
        let strategy = Strategy {
            id: Uuid::new_v4().to_string(),
            strategy_type: StrategyType::PriceDrop { threshold: 5.0 },
            symbol: "AAPL".to_string(),
            parameters: StrategyParameters::default(),
            created_at: Utc::now(),
            is_active: true,
        };
        strategy_repo.create(&strategy).await.unwrap();

        // Create a test trade
        let trade = Trade {
            id: Uuid::new_v4().to_string(),
            strategy_id: strategy.id.clone(),
            symbol: "AAPL".to_string(),
            action: Action::Buy,
            quantity: 100.0,
            price: 150.0,
            timestamp: Utc::now(),
            explanation: "Test trade".to_string(),
            commission: 5.0,
            realized_pnl: Some(50.0),
            trade_value: 15000.0,
        };

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

        // Test get recent
        let recent_trades = trade_repo.get_recent(10).await.unwrap();
        assert_eq!(recent_trades.len(), 1);

        // Test delete
        trade_repo.delete(&trade.id).await.unwrap();
        let retrieved = trade_repo.get_by_id(&trade.id).await.unwrap();
        assert!(retrieved.is_none());
    }
}