// Database migration utilities

use sqlx::{SqlitePool, Row};
use crate::error::Result;

/// Initialize the database with all required tables
pub async fn initialize_database(pool: &SqlitePool) -> Result<()> {
    // Enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;

    // Create strategies table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS strategies (
            id TEXT PRIMARY KEY,
            strategy_type TEXT NOT NULL,
            symbol TEXT NOT NULL,
            parameters TEXT NOT NULL,
            created_at TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create trades table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trades (
            id TEXT PRIMARY KEY,
            strategy_id TEXT NOT NULL,
            symbol TEXT NOT NULL,
            action TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            timestamp TEXT NOT NULL,
            explanation TEXT,
            commission REAL DEFAULT 0.0,
            realized_pnl REAL,
            trade_value REAL NOT NULL,
            FOREIGN KEY (strategy_id) REFERENCES strategies(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create performance snapshots table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS performance_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id TEXT NOT NULL,
            total_return REAL NOT NULL,
            total_trades INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            metrics TEXT NOT NULL,
            FOREIGN KEY (strategy_id) REFERENCES strategies(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create market data cache table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS market_data_cache (
            symbol TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            price REAL NOT NULL,
            volume INTEGER,
            change_percent REAL,
            market_cap INTEGER,
            day_high REAL,
            day_low REAL,
            previous_close REAL,
            PRIMARY KEY (symbol, timestamp)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes for better performance
    create_indexes(pool).await?;

    Ok(())
}

/// Create database indexes for better query performance
async fn create_indexes(pool: &SqlitePool) -> Result<()> {
    // Index on strategies
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_strategies_symbol ON strategies(symbol)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_strategies_active ON strategies(is_active)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_strategies_created_at ON strategies(created_at)")
        .execute(pool)
        .await?;

    // Index on trades
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_strategy_id ON trades(strategy_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_action ON trades(action)")
        .execute(pool)
        .await?;

    // Index on performance snapshots
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_performance_strategy_id ON performance_snapshots(strategy_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_performance_timestamp ON performance_snapshots(timestamp)")
        .execute(pool)
        .await?;

    // Index on market data cache
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_market_data_symbol ON market_data_cache(symbol)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_market_data_timestamp ON market_data_cache(timestamp)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Check if database schema is up to date
pub async fn check_schema_version(pool: &SqlitePool) -> Result<bool> {
    // Check if all required tables exist
    let tables = vec!["strategies", "trades", "performance_snapshots", "market_data_cache"];
    
    for table in tables {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;

        if count == 0 {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Get database schema information
pub async fn get_schema_info(pool: &SqlitePool) -> Result<Vec<TableInfo>> {
    let rows = sqlx::query(
        "SELECT name, sql FROM sqlite_master WHERE type='table' ORDER BY name"
    )
    .fetch_all(pool)
    .await?;

    let mut tables = Vec::new();
    for row in rows {
        let name: String = row.get("name");
        let sql: Option<String> = row.get("sql");
        
        // Get row count for each table
        let count_query = format!("SELECT COUNT(*) FROM {}", name);
        let row_count: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        tables.push(TableInfo {
            name,
            sql,
            row_count: row_count as u32,
        });
    }

    Ok(tables)
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub sql: Option<String>,
    pub row_count: u32,
}

/// Clean up old data based on retention policies
pub async fn cleanup_old_data(pool: &SqlitePool, days_to_keep: u32) -> Result<u32> {
    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_to_keep as i64);
    let cutoff_str = crate::database::datetime_to_string(cutoff_date);

    // Clean up old market data cache entries
    let result = sqlx::query("DELETE FROM market_data_cache WHERE timestamp < ?")
        .bind(&cutoff_str)
        .execute(pool)
        .await?;

    // Clean up old performance snapshots (keep at least one per strategy)
    sqlx::query(
        r#"
        DELETE FROM performance_snapshots 
        WHERE timestamp < ? 
        AND id NOT IN (
            SELECT MAX(id) 
            FROM performance_snapshots 
            GROUP BY strategy_id
        )
        "#
    )
    .bind(&cutoff_str)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;


    async fn create_test_db() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_initialize_database() {
        let pool = create_test_db().await;
        
        initialize_database(&pool).await.unwrap();
        
        // Check that all tables were created
        let schema_valid = check_schema_version(&pool).await.unwrap();
        assert!(schema_valid);
    }

    #[tokio::test]
    async fn test_schema_info() {
        let pool = create_test_db().await;
        initialize_database(&pool).await.unwrap();
        
        let tables = get_schema_info(&pool).await.unwrap();
        assert!(tables.len() >= 4); // At least our 4 main tables
        
        let table_names: Vec<String> = tables.iter().map(|t| t.name.clone()).collect();
        assert!(table_names.contains(&"strategies".to_string()));
        assert!(table_names.contains(&"trades".to_string()));
        assert!(table_names.contains(&"performance_snapshots".to_string()));
        assert!(table_names.contains(&"market_data_cache".to_string()));
    }

    #[tokio::test]
    async fn test_cleanup_old_data() {
        let pool = create_test_db().await;
        initialize_database(&pool).await.unwrap();
        
        // Insert some test data
        let old_timestamp = crate::database::datetime_to_string(
            chrono::Utc::now() - chrono::Duration::days(100)
        );
        
        sqlx::query(
            "INSERT INTO market_data_cache (symbol, timestamp, price, volume, change_percent) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("TEST")
        .bind(&old_timestamp)
        .bind(100.0)
        .bind(1000)
        .bind(1.5)
        .execute(&pool)
        .await
        .unwrap();

        // Clean up data older than 30 days
        let deleted_count = cleanup_old_data(&pool, 30).await.unwrap();
        assert_eq!(deleted_count, 1);
    }
}