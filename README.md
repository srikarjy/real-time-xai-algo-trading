<<<<<<< HEAD
# real-time-xai-algo-trading
=======
# Real-Time XAI Trading Algorithm Platform

A real-time trading simulation platform that allows users to input their own strategies and see how they perform against live stock data with explainable AI insights.

## Features

- **Real-time Stock Data**: Live price feeds from major exchanges
- **Strategy Builder**: Simple interface to create trading strategies
- **Live Simulation**: See your strategy in action with real-time data
- **Explainable AI**: Understand why the AI makes certain decisions
- **Performance Comparison**: Compare your strategy vs market performance
- **Interactive Dashboard**: Real-time charts and metrics

## Quick Start

1. Install dependencies:
```bash
pip install -r requirements.txt
```

2. Start the backend server:
```bash
python backend/main.py
```

3. Start the frontend dashboard:
```bash
python frontend/dashboard.py
```

4. Open your browser to `http://localhost:8050`

## How It Works

1. **Strategy Input**: Users define simple trading rules (e.g., "Buy when price drops 5%")
2. **Real-time Monitoring**: The system fetches live stock data every few seconds
3. **Strategy Execution**: Your rules are applied to the live data
4. **AI Explanation**: The system explains each decision in plain language
5. **Performance Tracking**: See how your strategy performs vs buy-and-hold

## Example Strategies

- **Moving Average Crossover**: Buy when short MA crosses above long MA
- **RSI Strategy**: Buy when RSI < 30, sell when RSI > 70
- **Price Drop Strategy**: Buy when price drops more than 5% in a day
- **Volume Strategy**: Buy when volume is 2x average volume

## Architecture

- **Backend**: FastAPI with WebSocket support for real-time data
- **Frontend**: Dash/Plotly for interactive visualizations
- **Data**: yfinance for real-time stock data
- **AI**: Simple rule-based system with explainable decisions 
>>>>>>> 47e282f (Initial commit: Real-Time XAI Trading Platform)
