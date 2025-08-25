-- Initial database schema for trading platform

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Strategies table
CREATE TABLE strategies (
    id TEXT PRIMARY KEY,
    strategy_type TEXT NOT NULL,
    symbol TEXT NOT NULL,
    parameters TEXT NOT NULL, -- JSON
    created_at TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Trades table
CREATE TABLE trades (
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
);

-- Performance snapshots table
CREATE TABLE performance_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy_id TEXT NOT NULL,
    total_return REAL NOT NULL,
    total_trades INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    metrics TEXT NOT NULL, -- JSON
    FOREIGN KEY (strategy_id) REFERENCES strategies(id) ON DELETE CASCADE
);

-- Market data cache table
CREATE TABLE market_data_cache (
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
);

-- Create indexes for better performance
CREATE INDEX idx_strategies_symbol ON strategies(symbol);
CREATE INDEX idx_strategies_active ON strategies(is_active);
CREATE INDEX idx_strategies_created_at ON strategies(created_at);

CREATE INDEX idx_trades_strategy_id ON trades(strategy_id);
CREATE INDEX idx_trades_symbol ON trades(symbol);
CREATE INDEX idx_trades_timestamp ON trades(timestamp);
CREATE INDEX idx_trades_action ON trades(action);

CREATE INDEX idx_performance_strategy_id ON performance_snapshots(strategy_id);
CREATE INDEX idx_performance_timestamp ON performance_snapshots(timestamp);

CREATE INDEX idx_market_data_symbol ON market_data_cache(symbol);
CREATE INDEX idx_market_data_timestamp ON market_data_cache(timestamp);