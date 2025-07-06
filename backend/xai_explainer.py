"""
Explainable AI Module for Trading Decisions
Provides clear, layman-friendly explanations of trading strategies
"""

import pandas as pd
import numpy as np
from datetime import datetime, timedelta
from typing import Dict, List, Tuple

class XAIExplainer:
    def __init__(self):
        self.explanations = {
            "price_drop": {
                "buy": "The stock price has dropped significantly, which often indicates a good buying opportunity. This is like finding a sale at your favorite store - when prices go down, you can buy more for less money.",
                "hold": "The price hasn't dropped enough to trigger a buy signal. We're waiting for a better opportunity to enter the market.",
                "sell": "The price has recovered from the drop, so we're taking profits. This is like selling something you bought on sale when the price goes back up."
            },
            "moving_average": {
                "buy": "The short-term trend is now stronger than the long-term trend, suggesting the stock is gaining momentum. Think of it like a car accelerating - the speed is picking up.",
                "hold": "The trends are not aligned yet. We're waiting for a clear signal that the stock is moving in the right direction.",
                "sell": "The short-term trend is weakening compared to the long-term trend, suggesting we should take profits."
            },
            "rsi": {
                "buy": "The stock is oversold, meaning it has been sold too much and is likely to bounce back. This is like a rubber band that's been stretched too far - it will snap back.",
                "hold": "The stock is trading at normal levels. We're waiting for either oversold or overbought conditions.",
                "sell": "The stock is overbought, meaning it has been bought too much and might pull back. This is like a rubber band that's been compressed too much."
            }
        }
    
    def explain_decision(self, strategy_type: str, action: str, data: Dict, context: Dict = None) -> str:
        """Generate a comprehensive explanation for a trading decision"""
        
        base_explanation = self.explanations.get(strategy_type, {}).get(action.lower(), 
            "The strategy conditions have been met for this action.")
        
        # Add specific context based on strategy type
        if strategy_type == "price_drop":
            return self._explain_price_drop(action, data, base_explanation)
        elif strategy_type == "moving_average":
            return self._explain_moving_average(action, data, base_explanation)
        elif strategy_type == "rsi":
            return self._explain_rsi(action, data, base_explanation)
        
        return base_explanation
    
    def _explain_price_drop(self, action: str, data: Dict, base_explanation: str) -> str:
        """Explain price drop strategy decisions"""
        if action == "BUY":
            price_change = data.get("price_change_pct", 0)
            threshold = data.get("threshold", 5)
            
            explanation = f"""
            📉 **Price Drop Detected!**
            
            The stock price has dropped by {abs(price_change):.2f}%, which is more than your threshold of {threshold}%.
            
            **What this means:**
            - The stock is now cheaper than it was before
            - This could be a temporary dip or a buying opportunity
            - You're getting more shares for the same amount of money
            
            **Why this strategy works:**
            - Stocks often bounce back after significant drops
            - You're buying when others might be selling (contrarian approach)
            - Lower prices mean higher potential returns
            
            **Risk reminder:** Past performance doesn't guarantee future results.
            """
            
        elif action == "HOLD":
            explanation = f"""
            ⏳ **Waiting for Opportunity**
            
            The price hasn't dropped enough to trigger a buy signal yet.
            
            **Current situation:**
            - Price change: {data.get('price_change_pct', 0):.2f}%
            - Your threshold: {data.get('threshold', 5)}%
            
            **What we're watching for:**
            - A price drop of {data.get('threshold', 5)}% or more
            - This would signal a potential buying opportunity
            
            **Patience is key:** Good opportunities come to those who wait.
            """
        
        return explanation
    
    def _explain_moving_average(self, action: str, data: Dict, base_explanation: str) -> str:
        """Explain moving average strategy decisions"""
        if action == "BUY":
            short_ma = data.get("short_ma", 0)
            long_ma = data.get("long_ma", 0)
            
            explanation = f"""
            📈 **Trend Reversal Detected!**
            
            The short-term average (${short_ma:.2f}) has crossed above the long-term average (${long_ma:.2f}).
            
            **What this means:**
            - The stock is gaining momentum
            - Recent prices are higher than the long-term average
            - This suggests an upward trend is starting
            
            **Why this strategy works:**
            - Moving averages smooth out price noise
            - Crossovers often signal trend changes
            - You're buying when the trend is turning positive
            
            **Think of it like:** A car that was slowing down is now accelerating again.
            """
            
        elif action == "HOLD":
            explanation = f"""
            🔄 **Trends Not Aligned**
            
            The moving averages haven't crossed yet, so we're waiting for a clear signal.
            
            **Current averages:**
            - Short-term: ${data.get('short_ma', 0):.2f}
            - Long-term: ${data.get('long_ma', 0):.2f}
            
            **What we're waiting for:**
            - Short-term average to cross above long-term average
            - This would signal the start of an upward trend
            
            **Strategy:** We only buy when trends are clearly aligned.
            """
        
        return explanation
    
    def _explain_rsi(self, action: str, data: Dict, base_explanation: str) -> str:
        """Explain RSI strategy decisions"""
        rsi = data.get("rsi", 0)
        
        if action == "BUY":
            explanation = f"""
            🔵 **Oversold Condition Detected!**
            
            The RSI is {rsi:.1f}, which indicates the stock is oversold.
            
            **What this means:**
            - The stock has been sold too much recently
            - It's likely to bounce back from this level
            - This is often a good time to buy
            
            **Why RSI works:**
            - RSI measures how fast prices are moving
            - Values below 30 usually mean oversold
            - Stocks often recover from oversold conditions
            
            **Think of it like:** A pendulum that has swung too far in one direction will swing back.
            """
            
        elif action == "SELL":
            explanation = f"""
            🔴 **Overbought Condition Detected!**
            
            The RSI is {rsi:.1f}, which indicates the stock is overbought.
            
            **What this means:**
            - The stock has been bought too much recently
            - It might pull back from this level
            - This could be a good time to take profits
            
            **Why RSI works:**
            - RSI measures momentum and speed of price changes
            - Values above 70 usually mean overbought
            - Stocks often pull back from overbought conditions
            
            **Think of it like:** A pendulum that has swung too far will swing back the other way.
            """
            
        elif action == "HOLD":
            explanation = f"""
            ⚖️ **Normal RSI Levels**
            
            The RSI is {rsi:.1f}, which is within normal trading ranges.
            
            **What this means:**
            - The stock is not extremely overbought or oversold
            - Price movements are within normal patterns
            - No clear buy or sell signal at this time
            
            **RSI ranges:**
            - 0-30: Oversold (potential buy)
            - 30-70: Normal range (hold)
            - 70-100: Overbought (potential sell)
            
            **Strategy:** We wait for extreme conditions to make decisions.
            """
        
        return explanation
    
    def generate_market_context(self, symbol: str, current_price: float, historical_data: pd.DataFrame) -> str:
        """Generate market context for better explanations"""
        if len(historical_data) < 2:
            return "Insufficient data for market context."
        
        # Calculate basic market context
        price_change_1d = ((current_price - historical_data.iloc[-2]['Close']) / historical_data.iloc[-2]['Close']) * 100
        price_change_5d = ((current_price - historical_data.iloc[-6]['Close']) / historical_data.iloc[-6]['Close']) * 100 if len(historical_data) >= 6 else 0
        
        context = f"""
        📊 **Market Context for {symbol}**
        
        **Current Price:** ${current_price:.2f}
        **1-Day Change:** {price_change_1d:+.2f}%
        **5-Day Change:** {price_change_5d:+.2f}%
        
        **Market Sentiment:**
        """
        
        if price_change_1d > 2:
            context += "- Strong positive momentum today\n"
        elif price_change_1d < -2:
            context += "- Significant selling pressure today\n"
        else:
            context += "- Relatively stable trading today\n"
        
        if price_change_5d > 5:
            context += "- Strong upward trend over the week\n"
        elif price_change_5d < -5:
            context += "- Downward trend over the week\n"
        else:
            context += "- Sideways movement over the week\n"
        
        return context
    
    def explain_risk_factors(self, strategy_type: str, data: Dict) -> str:
        """Explain potential risks for the strategy"""
        risks = {
            "price_drop": """
            ⚠️ **Risk Factors for Price Drop Strategy:**
            
            - **Continued Decline:** The price might keep falling after you buy
            - **Market Conditions:** Bad news might cause further drops
            - **Timing Risk:** You might buy too early in a longer decline
            
            **Risk Management Tips:**
            - Set stop-loss orders to limit potential losses
            - Don't invest more than you can afford to lose
            - Consider the overall market conditions
            """,
            
            "moving_average": """
            ⚠️ **Risk Factors for Moving Average Strategy:**
            
            - **False Signals:** Crossovers can sometimes be misleading
            - **Lag:** Moving averages are based on past data
            - **Sideways Markets:** May generate many small trades
            
            **Risk Management Tips:**
            - Use additional confirmation signals
            - Be patient with the strategy
            - Consider transaction costs
            """,
            
            "rsi": """
            ⚠️ **Risk Factors for RSI Strategy:**
            
            - **Oversold Can Stay Oversold:** RSI can remain low for extended periods
            - **Overbought Can Stay Overbought:** Strong trends can keep RSI high
            - **False Signals:** RSI can give conflicting signals in trending markets
            
            **Risk Management Tips:**
            - Use RSI with other indicators
            - Don't rely solely on RSI levels
            - Consider the overall trend
            """
        }
        
        return risks.get(strategy_type, "⚠️ All trading involves risk. Past performance doesn't guarantee future results.")

# Global instance
explainer = XAIExplainer() 