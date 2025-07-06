"""
Configuration file for Real-Time XAI Trading Platform
"""

# Server Configuration
BACKEND_HOST = "0.0.0.0"
BACKEND_PORT = 8000
FRONTEND_HOST = "0.0.0.0"
FRONTEND_PORT = 8050

# Data Configuration
DEFAULT_STOCKS = ["AAPL", "TSLA", "MSFT", "GOOGL", "AMZN"]
UPDATE_INTERVAL = 5  # seconds between data updates
HISTORICAL_DAYS = 30  # days of historical data to fetch

# Trading Configuration
INITIAL_CASH = 10000  # Starting cash for simulations
MAX_POSITION_SIZE = 0.2  # Maximum 20% of portfolio in one stock
TRANSACTION_COST = 0.01  # 1% transaction cost

# Strategy Templates
STRATEGY_TEMPLATES = {
    "price_drop": {
        "name": "Price Drop Strategy",
        "description": "Buy when price drops by a specified percentage",
        "default_params": {
            "symbol": "AAPL",
            "threshold": 5
        }
    },
    "moving_average": {
        "name": "Moving Average Crossover",
        "description": "Buy when short-term MA crosses above long-term MA",
        "default_params": {
            "symbol": "AAPL",
            "short_period": 10,
            "long_period": 30
        }
    },
    "rsi": {
        "name": "RSI Strategy",
        "description": "Buy when RSI is oversold, sell when overbought",
        "default_params": {
            "symbol": "AAPL",
            "oversold": 30,
            "overbought": 70
        }
    },
    "volume_spike": {
        "name": "Volume Spike Strategy",
        "description": "Buy when volume is significantly higher than average",
        "default_params": {
            "symbol": "AAPL",
            "volume_multiplier": 2.0
        }
    }
}

# Risk Management
STOP_LOSS_PERCENTAGE = 0.05  # 5% stop loss
TAKE_PROFIT_PERCENTAGE = 0.15  # 15% take profit
MAX_DAILY_TRADES = 10  # Maximum trades per day

# UI Configuration
CHART_HEIGHT = 400
CHART_WIDTH = "100%"
THEME = "bootstrap"  # bootstrap, darkly, cosmo, etc.

# Logging Configuration
LOG_LEVEL = "INFO"
LOG_FILE = "trading_platform.log"

# API Configuration
YAHOO_FINANCE_TIMEOUT = 10  # seconds
MAX_RETRIES = 3
RATE_LIMIT_DELAY = 1  # seconds between API calls 