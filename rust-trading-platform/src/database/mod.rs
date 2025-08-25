// Database connection management and repository implementations

use sqlx::SqlitePool;
use chrono::{DateTime, Utc};
use crate::error::{Result, TradingPlatformError};

pub mod migrations;
pub mod repositories;

pub use repositories::*;

/// Database connection pool manager
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection with the given URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        Ok(Database { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let strategies_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM strategies")
            .fetch_one(&self.pool)
            .await?;

        let trades_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trades")
            .fetch_one(&self.pool)
            .await?;

        let active_strategies_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM strategies WHERE is_active = TRUE"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            total_strategies: strategies_count as u32,
            total_trades: trades_count as u32,
            active_strategies: active_strategies_count as u32,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_strategies: u32,
    pub total_trades: u32,
    pub active_strategies: u32,
}

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

impl DatabaseConfig {
    pub fn new(url: String) -> Self {
        DatabaseConfig {
            url,
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
        }
    }

    pub async fn create_pool(&self) -> Result<SqlitePool> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(self.connect_timeout))
            .idle_timeout(std::time::Duration::from_secs(self.idle_timeout))
            .connect(&self.url)
            .await?;

        Ok(pool)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig::new("sqlite:trading_platform.db".to_string())
    }
}

// Helper functions for database operations
pub fn serialize_json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(TradingPlatformError::from)
}

pub fn deserialize_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str(json).map_err(TradingPlatformError::from)
}

pub fn datetime_to_string(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

pub fn string_to_datetime(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| TradingPlatformError::internal(format!("Failed to parse datetime: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_database_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.url, "sqlite:trading_platform.db");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
    }

    #[test]
    fn test_datetime_serialization() {
        let now = Utc::now();
        let serialized = datetime_to_string(now);
        let deserialized = string_to_datetime(&serialized).unwrap();
        
        // Allow for small differences due to precision
        let diff = (now.timestamp_millis() - deserialized.timestamp_millis()).abs();
        assert!(diff < 1000); // Less than 1 second difference
    }

    #[test]
    fn test_json_serialization() {
        use std::collections::HashMap;
        
        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        data.insert("key2".to_string(), "value2".to_string());
        
        let serialized = serialize_json(&data).unwrap();
        let deserialized: HashMap<String, String> = deserialize_json(&serialized).unwrap();
        
        assert_eq!(data, deserialized);
    }
}