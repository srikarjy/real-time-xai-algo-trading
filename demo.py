#!/usr/bin/env python3
"""
Demo script for Real-Time XAI Trading Platform
Shows how to create and test trading strategies
"""

import requests
import json
import time
from datetime import datetime

# Backend API URL
BASE_URL = "http://localhost:8000"

def test_backend_connection():
    """Test if the backend is running"""
    try:
        response = requests.get(f"{BASE_URL}/")
        if response.status_code == 200:
            print("✅ Backend is running!")
            return True
        else:
            print("❌ Backend is not responding correctly")
            return False
    except requests.exceptions.ConnectionError:
        print("❌ Cannot connect to backend. Make sure it's running on port 8000")
        return False

def get_stock_data(symbol):
    """Get current stock data"""
    try:
        response = requests.get(f"{BASE_URL}/stocks/{symbol}")
        if response.status_code == 200:
            data = response.json()
            print(f"📊 {symbol} Current Price: ${data['price']:.2f}")
            print(f"   Change: {data['change']:+.2f} ({data['change_percent']:+.2f}%)")
            return data
        else:
            print(f"❌ Error getting data for {symbol}")
            return None
    except Exception as e:
        print(f"❌ Error: {e}")
        return None

def create_strategy(strategy_data):
    """Create a new trading strategy"""
    try:
        response = requests.post(f"{BASE_URL}/strategy", json=strategy_data)
        if response.status_code == 200:
            result = response.json()
            print(f"✅ Strategy created: {result['strategy_id']}")
            return result['strategy_id']
        else:
            print(f"❌ Error creating strategy: {response.text}")
            return None
    except Exception as e:
        print(f"❌ Error: {e}")
        return None

def demo_price_drop_strategy():
    """Demo the price drop strategy"""
    print("\n🎯 Demo: Price Drop Strategy")
    print("=" * 40)
    
    strategy = {
        "type": "price_drop",
        "symbol": "AAPL",
        "threshold": 3  # Buy when price drops 3%
    }
    
    strategy_id = create_strategy(strategy)
    if strategy_id:
        print(f"Strategy will buy AAPL when price drops by 3% or more")
        print(f"WebSocket URL: ws://localhost:8000/ws/{strategy_id}")
        print("Open the dashboard at http://localhost:8050 to see it in action!")

def demo_moving_average_strategy():
    """Demo the moving average strategy"""
    print("\n🎯 Demo: Moving Average Strategy")
    print("=" * 40)
    
    strategy = {
        "type": "moving_average",
        "symbol": "TSLA",
        "short_period": 10,
        "long_period": 30
    }
    
    strategy_id = create_strategy(strategy)
    if strategy_id:
        print(f"Strategy will buy TSLA when 10-day MA crosses above 30-day MA")
        print(f"WebSocket URL: ws://localhost:8000/ws/{strategy_id}")
        print("Open the dashboard at http://localhost:8050 to see it in action!")

def demo_rsi_strategy():
    """Demo the RSI strategy"""
    print("\n🎯 Demo: RSI Strategy")
    print("=" * 40)
    
    strategy = {
        "type": "rsi",
        "symbol": "MSFT",
        "oversold": 30,
        "overbought": 70
    }
    
    strategy_id = create_strategy(strategy)
    if strategy_id:
        print(f"Strategy will buy MSFT when RSI < 30, sell when RSI > 70")
        print(f"WebSocket URL: ws://localhost:8000/ws/{strategy_id}")
        print("Open the dashboard at http://localhost:8050 to see it in action!")

def main():
    print("📈 Real-Time XAI Trading Platform Demo")
    print("=" * 50)
    
    # Test backend connection
    if not test_backend_connection():
        print("\n💡 To start the platform:")
        print("1. Install dependencies: pip install -r requirements.txt")
        print("2. Start backend: python backend/main.py")
        print("3. Start frontend: python frontend/dashboard.py")
        print("4. Run this demo again")
        return
    
    # Get some sample stock data
    print("\n📊 Sample Stock Data:")
    get_stock_data("AAPL")
    get_stock_data("TSLA")
    get_stock_data("MSFT")
    
    # Demo different strategies
    demo_price_drop_strategy()
    demo_moving_average_strategy()
    demo_rsi_strategy()
    
    print("\n🎉 Demo Complete!")
    print("\nNext Steps:")
    print("1. Open http://localhost:8050 in your browser")
    print("2. Create your own strategies using the dashboard")
    print("3. Watch real-time data and AI explanations")
    print("4. Compare your strategy performance with the market")

if __name__ == "__main__":
    main() 