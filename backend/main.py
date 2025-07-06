from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
import asyncio
import json
import yfinance as yf
from datetime import datetime, timedelta
import pandas as pd
from typing import Dict, List
import logging
from xai_explainer import explainer

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="Real-Time XAI Trading Platform")

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Store active connections
active_connections: List[WebSocket] = []

# Store user strategies
user_strategies: Dict[str, Dict] = {}

# Store simulation data
simulation_data: Dict[str, Dict] = {}

class TradingSimulator:
    def __init__(self):
        self.positions = {}
        self.cash = 10000  # Starting cash
        self.trades = []
        self.current_prices = {}
        
    def execute_strategy(self, symbol: str, strategy: Dict, current_price: float, historical_data: pd.DataFrame):
        """Execute user strategy and return explanation"""
        action = "HOLD"
        explanation = "No action taken - conditions not met"
        strategy_data = {}
        
        if strategy["type"] == "price_drop":
            # Buy when price drops by specified percentage
            if len(historical_data) >= 2:
                prev_price = historical_data.iloc[-2]['Close']
                price_change_pct = ((current_price - prev_price) / prev_price) * 100
                
                strategy_data = {
                    "price_change_pct": price_change_pct,
                    "threshold": strategy["threshold"]
                }
                
                if price_change_pct <= -strategy["threshold"]:
                    action = "BUY"
                    
        elif strategy["type"] == "moving_average":
            # Buy when short MA crosses above long MA
            if len(historical_data) >= strategy["long_period"]:
                short_ma = historical_data['Close'].rolling(window=strategy["short_period"]).mean().iloc[-1]
                long_ma = historical_data['Close'].rolling(window=strategy["long_period"]).mean().iloc[-1]
                
                strategy_data = {
                    "short_ma": short_ma,
                    "long_ma": long_ma,
                    "short_period": strategy["short_period"],
                    "long_period": strategy["long_period"]
                }
                
                if short_ma > long_ma:
                    action = "BUY"
                    
        elif strategy["type"] == "rsi":
            # RSI strategy
            if len(historical_data) >= 14:
                rsi = self.calculate_rsi(historical_data['Close'])
                strategy_data = {
                    "rsi": rsi,
                    "oversold": strategy["oversold"],
                    "overbought": strategy["overbought"]
                }
                
                if rsi < strategy["oversold"]:
                    action = "BUY"
                elif rsi > strategy["overbought"]:
                    action = "SELL"
        
        # Generate explanation using XAI explainer
        explanation = explainer.explain_decision(strategy["type"], action, strategy_data)
        
        return action, explanation
    
    def calculate_rsi(self, prices: pd.Series, period: int = 14) -> float:
        """Calculate RSI indicator"""
        delta = prices.diff()
        gain = (delta.where(delta > 0, 0)).rolling(window=period).mean()
        loss = (-delta.where(delta < 0, 0)).rolling(window=period).mean()
        rs = gain / loss
        rsi = 100 - (100 / (1 + rs))
        return rsi.iloc[-1]

# Global simulator instance
simulator = TradingSimulator()

@app.get("/")
async def root():
    return {"message": "Real-Time XAI Trading Platform API"}

@app.get("/stocks/{symbol}")
async def get_stock_data(symbol: str):
    """Get current stock data"""
    try:
        stock = yf.Ticker(symbol)
        info = stock.info
        current_price = info.get('regularMarketPrice', 0)
        
        return {
            "symbol": symbol,
            "price": current_price,
            "name": info.get('longName', symbol),
            "change": info.get('regularMarketChange', 0),
            "change_percent": info.get('regularMarketChangePercent', 0),
            "volume": info.get('volume', 0),
            "timestamp": datetime.now().isoformat()
        }
    except Exception as e:
        logger.error(f"Error fetching data for {symbol}: {e}")
        return {"error": f"Could not fetch data for {symbol}"}

@app.post("/strategy")
async def create_strategy(strategy: Dict):
    """Create a new trading strategy"""
    strategy_id = f"strategy_{len(user_strategies) + 1}"
    user_strategies[strategy_id] = strategy
    simulation_data[strategy_id] = {
        "trades": [],
        "performance": {"total_return": 0, "trades_count": 0},
        "current_position": {"symbol": strategy.get("symbol", "AAPL"), "shares": 0, "avg_price": 0}
    }
    
    return {"strategy_id": strategy_id, "message": "Strategy created successfully"}

@app.get("/strategies")
async def get_strategies():
    """Get all user strategies"""
    return user_strategies

@app.websocket("/ws/{strategy_id}")
async def websocket_endpoint(websocket: WebSocket, strategy_id: str):
    await websocket.accept()
    active_connections.append(websocket)
    
    try:
        if strategy_id not in user_strategies:
            await websocket.send_text(json.dumps({"error": "Strategy not found"}))
            return
            
        strategy = user_strategies[strategy_id]
        symbol = strategy.get("symbol", "AAPL")
        
        # Send initial data
        stock_data = await get_stock_data(symbol)
        await websocket.send_text(json.dumps({
            "type": "initial_data",
            "data": stock_data
        }))
        
        # Start real-time monitoring
        while True:
            try:
                # Fetch real-time data
                stock = yf.Ticker(symbol)
                current_price = stock.info.get('regularMarketPrice', 0)
                
                # Get historical data for strategy execution
                hist_data = stock.history(period="30d")
                
                # Execute strategy
                action, explanation = simulator.execute_strategy(
                    symbol, strategy, current_price, hist_data
                )
                
                # Update simulation data
                if action != "HOLD":
                    simulation_data[strategy_id]["trades"].append({
                        "timestamp": datetime.now().isoformat(),
                        "action": action,
                        "price": current_price,
                        "explanation": explanation
                    })
                
                # Send real-time update
                await websocket.send_text(json.dumps({
                    "type": "update",
                    "symbol": symbol,
                    "price": current_price,
                    "action": action,
                    "explanation": explanation,
                    "timestamp": datetime.now().isoformat(),
                    "simulation_data": simulation_data[strategy_id]
                }))
                
                # Wait 5 seconds before next update
                await asyncio.sleep(5)
                
            except Exception as e:
                logger.error(f"Error in real-time monitoring: {e}")
                await websocket.send_text(json.dumps({
                    "type": "error",
                    "message": str(e)
                }))
                await asyncio.sleep(10)
                
    except WebSocketDisconnect:
        active_connections.remove(websocket)
        logger.info("WebSocket disconnected")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000) 