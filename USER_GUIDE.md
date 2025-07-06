# Real-Time XAI Trading Platform - User Guide

## 🚀 Quick Start

### 1. Installation
```bash
# Install dependencies
python install.py

# Or manually
pip install -r requirements.txt
```

### 2. Start the Platform
```bash
# Start both backend and frontend
python run_platform.py

# Or start manually
python backend/main.py  # Terminal 1
python frontend/dashboard.py  # Terminal 2
```

### 3. Access the Platform
- Open your browser to: `http://localhost:8050`
- You'll see the interactive trading dashboard

## 📊 How It Works

### Real-Time Data Flow
1. **Data Fetching**: The platform fetches live stock data every 5 seconds
2. **Strategy Execution**: Your trading rules are applied to the live data
3. **AI Explanation**: The system explains each decision in plain language
4. **Visualization**: Real-time charts show price movements and trade signals

### Strategy Types

#### 1. Price Drop Strategy
- **What it does**: Buys when price drops by a specified percentage
- **When to use**: Good for buying dips and contrarian strategies
- **Example**: Buy AAPL when price drops 5% in a day

#### 2. Moving Average Crossover
- **What it does**: Buys when short-term MA crosses above long-term MA
- **When to use**: Good for trend-following strategies
- **Example**: Buy when 10-day average crosses above 30-day average

#### 3. RSI Strategy
- **What it does**: Buys when RSI is oversold, sells when overbought
- **When to use**: Good for momentum and reversal strategies
- **Example**: Buy when RSI < 30, sell when RSI > 70

## 🎯 Creating Your First Strategy

### Step 1: Choose Strategy Type
1. Open the dashboard at `http://localhost:8050`
2. In the "Strategy Builder" panel, select a strategy type
3. Read the description to understand how it works

### Step 2: Configure Parameters
- **Stock Symbol**: Enter the stock you want to trade (e.g., AAPL, TSLA)
- **Strategy Parameters**: Set your specific rules (thresholds, periods, etc.)

### Step 3: Start the Strategy
1. Click "🚀 Start Strategy"
2. The system will create a WebSocket connection
3. You'll see real-time updates in the dashboard

## 📈 Understanding the Dashboard

### Live Market Data Panel
- **Current Price**: Real-time stock price
- **Connection Status**: Shows if data is flowing
- **Strategy Status**: Current action (BUY/SELL/HOLD)
- **AI Explanation**: Plain-language explanation of decisions

### Price Chart
- **Blue Line**: Stock price over time
- **Green Triangles**: Buy signals
- **Red Triangles**: Sell signals
- **Hover**: See exact prices and times

### Performance Metrics
- **Total Trades**: Number of trades executed
- **Total Return**: Percentage return on investment
- **Current Position**: Shares owned and average price

### Trade History
- **Recent Trades**: Last 5 trades with explanations
- **Action**: BUY/SELL decision
- **Price**: Price at which trade occurred
- **Reason**: AI explanation of why the trade was made

## 🤖 Understanding AI Explanations

### What the AI Explains
- **Why the decision was made**: What conditions triggered the action
- **Market context**: Current market conditions and trends
- **Risk factors**: Potential risks and considerations
- **Strategy logic**: How the strategy works in simple terms

### Example Explanations

#### Price Drop Strategy (BUY)
```
📉 Price Drop Detected!

The stock price has dropped by 5.2%, which is more than your threshold of 5%.

What this means:
- The stock is now cheaper than it was before
- This could be a temporary dip or a buying opportunity
- You're getting more shares for the same amount of money

Why this strategy works:
- Stocks often bounce back after significant drops
- You're buying when others might be selling (contrarian approach)
- Lower prices mean higher potential returns
```

#### Moving Average Strategy (BUY)
```
📈 Trend Reversal Detected!

The short-term average ($150.25) has crossed above the long-term average ($148.75).

What this means:
- The stock is gaining momentum
- Recent prices are higher than the long-term average
- This suggests an upward trend is starting

Think of it like: A car that was slowing down is now accelerating again.
```

## ⚠️ Risk Management

### Important Disclaimers
- **This is a simulation**: No real money is being traded
- **Past performance doesn't guarantee future results**: Historical data may not predict future performance
- **All trading involves risk**: Stock prices can go down as well as up

### Risk Management Tips
1. **Start Small**: Test strategies with small amounts
2. **Diversify**: Don't put all your money in one stock
3. **Set Stop Losses**: Limit potential losses
4. **Monitor Performance**: Regularly review strategy results
5. **Understand the Strategy**: Know what you're investing in

## 🔧 Advanced Features

### Custom Strategies
You can create custom strategies by modifying the backend code:
1. Edit `backend/main.py` to add new strategy types
2. Update `backend/xai_explainer.py` to add explanations
3. Modify `frontend/dashboard.py` to add UI elements

### Data Sources
- **Current**: Yahoo Finance (free, real-time)
- **Future**: Can be extended to other data providers
- **Historical**: 30 days of historical data for calculations

### Performance Tracking
- **Real-time P&L**: Track profits and losses
- **Trade History**: Complete record of all trades
- **Performance Metrics**: Returns, Sharpe ratio, etc.

## 🐛 Troubleshooting

### Common Issues

#### Backend Won't Start
```bash
# Check if port 8000 is available
lsof -i :8000

# Kill process if needed
kill -9 <PID>
```

#### Frontend Won't Start
```bash
# Check if port 8050 is available
lsof -i :8050

# Kill process if needed
kill -9 <PID>
```

#### No Data Loading
- Check internet connection
- Yahoo Finance API might be temporarily down
- Try refreshing the page

#### WebSocket Connection Issues
- Make sure backend is running on port 8000
- Check browser console for errors
- Try restarting both backend and frontend

### Getting Help
1. Check the console output for error messages
2. Verify all dependencies are installed: `python install.py`
3. Test the demo: `python demo.py`
4. Check the logs in the `logs/` directory

## 📚 Learning Resources

### Trading Concepts
- **Technical Analysis**: Understanding charts and indicators
- **Risk Management**: Protecting your capital
- **Market Psychology**: Understanding market behavior

### Strategy Development
- **Backtesting**: Testing strategies on historical data
- **Paper Trading**: Practicing without real money
- **Performance Analysis**: Measuring strategy effectiveness

### AI and Machine Learning
- **Explainable AI**: Understanding AI decisions
- **Feature Engineering**: Creating useful indicators
- **Model Validation**: Ensuring AI models work correctly

## 🎯 Next Steps

1. **Start Simple**: Begin with basic strategies
2. **Learn and Experiment**: Try different parameters
3. **Monitor Performance**: Track how your strategies perform
4. **Improve Gradually**: Refine strategies based on results
5. **Consider Real Trading**: When ready, apply lessons to real trading

## 📞 Support

For questions or issues:
1. Check this user guide first
2. Review the console output for error messages
3. Test with the demo script: `python demo.py`
4. Check the project documentation

---

**Remember**: This platform is for educational purposes. Always do your own research and consider consulting with financial advisors before making real investment decisions. 