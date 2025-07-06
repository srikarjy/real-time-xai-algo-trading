import dash
from dash import dcc, html, Input, Output, State, callback_context
import dash_bootstrap_components as dbc
import plotly.graph_objs as go
import plotly.express as px
from plotly.subplots import make_subplots
import pandas as pd
import requests
import json
import asyncio
import websockets
import threading
import time
from datetime import datetime, timedelta
import numpy as np

# Initialize Dash app
app = dash.Dash(__name__, external_stylesheets=[dbc.themes.BOOTSTRAP])
app.title = "Real-Time XAI Trading Platform"

# Global variables for real-time data
current_data = {}
strategy_data = {}
websocket_connection = None
is_connected = False

# Strategy templates
STRATEGY_TEMPLATES = {
    "price_drop": {
        "name": "Price Drop Strategy",
        "description": "Buy when price drops by a specified percentage",
        "fields": [
            {"name": "symbol", "type": "text", "label": "Stock Symbol", "default": "AAPL"},
            {"name": "threshold", "type": "number", "label": "Price Drop %", "default": 5}
        ]
    },
    "moving_average": {
        "name": "Moving Average Crossover",
        "description": "Buy when short-term MA crosses above long-term MA",
        "fields": [
            {"name": "symbol", "type": "text", "label": "Stock Symbol", "default": "AAPL"},
            {"name": "short_period", "type": "number", "label": "Short Period", "default": 10},
            {"name": "long_period", "type": "number", "label": "Long Period", "default": 30}
        ]
    },
    "rsi": {
        "name": "RSI Strategy",
        "description": "Buy when RSI is oversold, sell when overbought",
        "fields": [
            {"name": "symbol", "type": "text", "label": "Stock Symbol", "default": "AAPL"},
            {"name": "oversold", "type": "number", "label": "Oversold Level", "default": 30},
            {"name": "overbought", "type": "number", "label": "Overbought Level", "default": 70}
        ]
    }
}

# Layout
app.layout = dbc.Container([
    dbc.Row([
        dbc.Col([
            html.H1("📈 Real-Time XAI Trading Platform", className="text-center mb-4"),
            html.P("Build your trading strategy and see it work with live market data", 
                   className="text-center text-muted")
        ])
    ]),
    
    dbc.Row([
        # Strategy Builder Panel
        dbc.Col([
            dbc.Card([
                dbc.CardHeader("🎯 Strategy Builder"),
                dbc.CardBody([
                    dbc.Select(
                        id="strategy-type",
                        options=[
                            {"label": template["name"], "value": key}
                            for key, template in STRATEGY_TEMPLATES.items()
                        ],
                        value="price_drop",
                        className="mb-3"
                    ),
                    
                    html.Div(id="strategy-description", className="mb-3"),
                    
                    html.Div(id="strategy-fields"),
                    
                    dbc.Button("🚀 Start Strategy", id="start-strategy", 
                              color="success", className="w-100 mt-3")
                ])
            ], className="mb-4")
        ], md=4),
        
        # Real-time Data Panel
        dbc.Col([
            dbc.Card([
                dbc.CardHeader([
                    html.Span("📊 Live Market Data", id="connection-status")
                ]),
                dbc.CardBody([
                    html.Div(id="current-price-display"),
                    html.Div(id="strategy-status"),
                    html.Div(id="ai-explanation")
                ])
            ], className="mb-4")
        ], md=8)
    ]),
    
    dbc.Row([
        # Price Chart
        dbc.Col([
            dbc.Card([
                dbc.CardHeader("📈 Price Chart"),
                dbc.CardBody([
                    dcc.Graph(id="price-chart", style={"height": "400px"})
                ])
            ])
        ], md=8),
        
        # Performance Metrics
        dbc.Col([
            dbc.Card([
                dbc.CardHeader("📊 Performance"),
                dbc.CardBody([
                    html.Div(id="performance-metrics")
                ])
            ])
        ], md=4)
    ]),
    
    dbc.Row([
        # Trade History
        dbc.Col([
            dbc.Card([
                dbc.CardHeader("📋 Trade History"),
                dbc.CardBody([
                    html.Div(id="trade-history")
                ])
            ])
        ])
    ]),
    
    # Hidden div for storing data
    dcc.Store(id="strategy-data"),
    dcc.Store(id="websocket-data"),
    
    # Interval for updates
    dcc.Interval(id="update-interval", interval=5000, n_intervals=0)
], fluid=True)

# Callbacks
@app.callback(
    [Output("strategy-description", "children"),
     Output("strategy-fields", "children")],
    [Input("strategy-type", "value")]
)
def update_strategy_fields(strategy_type):
    if not strategy_type:
        return "", ""
    
    template = STRATEGY_TEMPLATES[strategy_type]
    
    description = html.P(template["description"], className="text-muted")
    
    fields = []
    for field in template["fields"]:
        if field["type"] == "text":
            fields.append(
                dbc.FormGroup([
                    dbc.Label(field["label"]),
                    dbc.Input(
                        id=f"field-{field['name']}",
                        type="text",
                        value=field["default"],
                        placeholder=field["label"]
                    )
                ], className="mb-3")
            )
        elif field["type"] == "number":
            fields.append(
                dbc.FormGroup([
                    dbc.Label(field["label"]),
                    dbc.Input(
                        id=f"field-{field['name']}",
                        type="number",
                        value=field["default"],
                        placeholder=field["label"]
                    )
                ], className="mb-3")
            )
    
    return description, fields

@app.callback(
    [Output("strategy-data", "data"),
     Output("start-strategy", "children")],
    [Input("start-strategy", "n_clicks")],
    [State("strategy-type", "value")] + 
    [State(f"field-{field['name']}", "value") 
     for template in STRATEGY_TEMPLATES.values() 
     for field in template["fields"]]
)
def create_strategy(n_clicks, strategy_type, *field_values):
    if not n_clicks:
        return {}, "🚀 Start Strategy"
    
    # Get field names for the selected strategy
    field_names = [field["name"] for field in STRATEGY_TEMPLATES[strategy_type]["fields"]]
    
    # Create strategy object
    strategy = {
        "type": strategy_type,
        **dict(zip(field_names, field_values))
    }
    
    # Send strategy to backend
    try:
        response = requests.post("http://localhost:8000/strategy", json=strategy)
        if response.status_code == 200:
            strategy_id = response.json()["strategy_id"]
            strategy["id"] = strategy_id
            
            # Start WebSocket connection
            start_websocket_connection(strategy_id)
            
            return strategy, "✅ Strategy Active"
        else:
            return {}, "❌ Error Starting Strategy"
    except Exception as e:
        print(f"Error creating strategy: {e}")
        return {}, "❌ Connection Error"

def start_websocket_connection(strategy_id):
    """Start WebSocket connection in a separate thread"""
    def websocket_thread():
        global websocket_connection, is_connected
        
        try:
            import asyncio
            import websockets
            
            async def connect():
                global websocket_connection, is_connected
                uri = f"ws://localhost:8000/ws/{strategy_id}"
                
                try:
                    websocket_connection = await websockets.connect(uri)
                    is_connected = True
                    
                    while True:
                        try:
                            message = await websocket_connection.recv()
                            data = json.loads(message)
                            
                            # Update global data
                            global current_data, strategy_data
                            if data["type"] == "update":
                                current_data = data
                                strategy_data = data.get("simulation_data", {})
                            
                        except websockets.exceptions.ConnectionClosed:
                            break
                        except Exception as e:
                            print(f"WebSocket error: {e}")
                            break
                            
                except Exception as e:
                    print(f"WebSocket connection error: {e}")
                    is_connected = False
            
            # Run the async function
            asyncio.run(connect())
            
        except Exception as e:
            print(f"Thread error: {e}")
    
    # Start thread
    thread = threading.Thread(target=websocket_thread)
    thread.daemon = True
    thread.start()

@app.callback(
    [Output("connection-status", "children"),
     Output("current-price-display", "children"),
     Output("strategy-status", "children"),
     Output("ai-explanation", "children")],
    [Input("update-interval", "n_intervals"),
     Input("websocket-data", "data")]
)
def update_real_time_data(n_intervals, websocket_data):
    global current_data, is_connected
    
    # Connection status
    if is_connected:
        status = html.Span("🟢 Connected", className="text-success")
    else:
        status = html.Span("🔴 Disconnected", className="text-danger")
    
    # Current price display
    if current_data:
        price_info = [
            html.H3(f"${current_data.get('price', 0):.2f}", className="text-primary"),
            html.P(f"Symbol: {current_data.get('symbol', 'N/A')}"),
            html.P(f"Last Update: {current_data.get('timestamp', 'N/A')}")
        ]
    else:
        price_info = [html.P("Waiting for data...")]
    
    # Strategy status
    if current_data and current_data.get('action') != 'HOLD':
        action = current_data.get('action', 'HOLD')
        action_color = "success" if action == "BUY" else "warning"
        strategy_status = [
            html.H5(f"Action: {action}", className=f"text-{action_color}"),
            html.P(f"Price: ${current_data.get('price', 0):.2f}")
        ]
    else:
        strategy_status = [html.P("Strategy monitoring...")]
    
    # AI explanation
    if current_data and current_data.get('explanation'):
        explanation = [
            html.H6("🤖 AI Explanation:"),
            html.P(current_data.get('explanation'), className="text-muted")
        ]
    else:
        explanation = [html.P("Waiting for AI insights...")]
    
    return status, price_info, strategy_status, explanation

@app.callback(
    Output("price-chart", "figure"),
    [Input("update-interval", "n_intervals")]
)
def update_price_chart(n_intervals):
    global current_data, strategy_data
    
    # Create sample data for demonstration
    dates = pd.date_range(start=datetime.now() - timedelta(days=30), 
                         end=datetime.now(), freq='D')
    prices = np.random.normal(150, 10, len(dates))
    
    # Add current price if available
    if current_data and current_data.get('price'):
        prices[-1] = current_data['price']
    
    fig = go.Figure()
    
    # Price line
    fig.add_trace(go.Scatter(
        x=dates,
        y=prices,
        mode='lines',
        name='Stock Price',
        line=dict(color='blue', width=2)
    ))
    
    # Add trade markers if available
    if strategy_data and strategy_data.get('trades'):
        trades = strategy_data['trades']
        buy_times = []
        buy_prices = []
        sell_times = []
        sell_prices = []
        
        for trade in trades:
            trade_time = datetime.fromisoformat(trade['timestamp'].replace('Z', '+00:00'))
            if trade['action'] == 'BUY':
                buy_times.append(trade_time)
                buy_prices.append(trade['price'])
            elif trade['action'] == 'SELL':
                sell_times.append(trade_time)
                sell_prices.append(trade['price'])
        
        if buy_times:
            fig.add_trace(go.Scatter(
                x=buy_times,
                y=buy_prices,
                mode='markers',
                name='Buy Signals',
                marker=dict(color='green', size=10, symbol='triangle-up')
            ))
        
        if sell_times:
            fig.add_trace(go.Scatter(
                x=sell_times,
                y=sell_prices,
                mode='markers',
                name='Sell Signals',
                marker=dict(color='red', size=10, symbol='triangle-down')
            ))
    
    fig.update_layout(
        title="Real-Time Stock Price",
        xaxis_title="Date",
        yaxis_title="Price ($)",
        hovermode='x unified'
    )
    
    return fig

@app.callback(
    Output("performance-metrics", "children"),
    [Input("update-interval", "n_intervals")]
)
def update_performance_metrics(n_intervals):
    global strategy_data
    
    if not strategy_data:
        return [html.P("No performance data available")]
    
    trades = strategy_data.get('trades', [])
    performance = strategy_data.get('performance', {})
    
    metrics = [
        html.H6("📊 Performance Metrics"),
        html.P(f"Total Trades: {len(trades)}"),
        html.P(f"Total Return: {performance.get('total_return', 0):.2f}%"),
        html.P(f"Current Position: {strategy_data.get('current_position', {}).get('shares', 0)} shares")
    ]
    
    return metrics

@app.callback(
    Output("trade-history", "children"),
    [Input("update-interval", "n_intervals")]
)
def update_trade_history(n_intervals):
    global strategy_data
    
    if not strategy_data or not strategy_data.get('trades'):
        return [html.P("No trades yet")]
    
    trades = strategy_data['trades']
    
    trade_cards = []
    for trade in trades[-5:]:  # Show last 5 trades
        trade_time = datetime.fromisoformat(trade['timestamp'].replace('Z', '+00:00'))
        action_color = "success" if trade['action'] == 'BUY' else "warning"
        
        card = dbc.Card([
            dbc.CardBody([
                html.H6(f"{trade['action']} {trade.get('symbol', 'N/A')}", 
                       className=f"text-{action_color}"),
                html.P(f"Price: ${trade['price']:.2f}"),
                html.P(f"Time: {trade_time.strftime('%H:%M:%S')}"),
                html.P(f"Reason: {trade['explanation']}", className="text-muted small")
            ])
        ], className="mb-2")
        
        trade_cards.append(card)
    
    return trade_cards

if __name__ == "__main__":
    app.run(debug=True, host="0.0.0.0", port=8050) 