// Performance tracking and metrics

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::strategy::Action;
use crate::error::{Result, TradingPlatformError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMetrics {
    pub strategy_id: String,
    pub total_return: f64,
    pub total_return_percent: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub current_position: Position,
    pub max_drawdown: f64,
    pub max_drawdown_percent: f64,
    pub sharpe_ratio: Option<f64>,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub profit_factor: f64,
    pub initial_capital: f64,
    pub current_capital: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub shares: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub current_value: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
    pub cost_basis: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trade {
    pub id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub action: Action,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub explanation: String,
    pub commission: f64,
    pub realized_pnl: Option<f64>,
    pub trade_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Portfolio {
    pub id: String,
    pub strategy_id: String,
    pub initial_capital: f64,
    pub current_capital: f64,
    pub positions: HashMap<String, Position>,
    pub trade_history: Vec<Trade>,
    pub performance_snapshots: Vec<PerformanceSnapshot>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_value: f64,
    pub cash_balance: f64,
    pub positions_value: f64,
    pub total_return: f64,
    pub daily_return: f64,
    pub drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskMetrics {
    pub strategy_id: String,
    pub value_at_risk_95: f64,
    pub value_at_risk_99: f64,
    pub expected_shortfall: f64,
    pub beta: Option<f64>,
    pub alpha: Option<f64>,
    pub volatility: f64,
    pub correlation_to_market: Option<f64>,
    pub calculated_at: DateTime<Utc>,
}

// Implementation methods
impl PerformanceMetrics {
    pub fn new(strategy_id: String, initial_capital: f64) -> Self {
        PerformanceMetrics {
            strategy_id: strategy_id.clone(),
            total_return: 0.0,
            total_return_percent: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            current_position: Position::empty("".to_string()),
            max_drawdown: 0.0,
            max_drawdown_percent: 0.0,
            sharpe_ratio: None,
            win_rate: 0.0,
            average_win: 0.0,
            average_loss: 0.0,
            profit_factor: 0.0,
            initial_capital,
            current_capital: initial_capital,
            last_updated: Utc::now(),
        }
    }

    pub fn update_from_trades(&mut self, trades: &[Trade]) {
        self.total_trades = trades.len() as u32;
        
        let mut total_pnl = 0.0;
        let mut wins = 0;
        let mut losses = 0;
        let mut win_sum = 0.0;
        let mut loss_sum = 0.0;

        for trade in trades {
            if let Some(pnl) = trade.realized_pnl {
                total_pnl += pnl;
                if pnl > 0.0 {
                    wins += 1;
                    win_sum += pnl;
                } else if pnl < 0.0 {
                    losses += 1;
                    loss_sum += pnl.abs();
                }
            }
        }

        self.winning_trades = wins;
        self.losing_trades = losses;
        self.total_return = total_pnl;
        self.total_return_percent = if self.initial_capital > 0.0 {
            (total_pnl / self.initial_capital) * 100.0
        } else {
            0.0
        };
        self.current_capital = self.initial_capital + total_pnl;

        // Calculate derived metrics
        if self.total_trades > 0 {
            self.win_rate = (self.winning_trades as f64 / self.total_trades as f64) * 100.0;
        }

        if wins > 0 {
            self.average_win = win_sum / wins as f64;
        }

        if losses > 0 {
            self.average_loss = loss_sum / losses as f64;
        }

        if self.average_loss > 0.0 {
            self.profit_factor = self.average_win / self.average_loss;
        }

        self.last_updated = Utc::now();
    }

    pub fn calculate_sharpe_ratio(&mut self, returns: &[f64], risk_free_rate: f64) {
        if returns.len() < 2 {
            self.sharpe_ratio = None;
            return;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            self.sharpe_ratio = Some((mean_return - risk_free_rate) / std_dev);
        } else {
            self.sharpe_ratio = None;
        }
    }

    pub fn update_drawdown(&mut self, current_value: f64, peak_value: f64) {
        let drawdown = peak_value - current_value;
        let drawdown_percent = if peak_value > 0.0 {
            (drawdown / peak_value) * 100.0
        } else {
            0.0
        };

        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown;
        }

        if drawdown_percent > self.max_drawdown_percent {
            self.max_drawdown_percent = drawdown_percent;
        }
    }
}

impl Position {
    pub fn empty(symbol: String) -> Self {
        Position {
            symbol,
            shares: 0.0,
            average_price: 0.0,
            current_price: 0.0,
            current_value: 0.0,
            unrealized_pnl: 0.0,
            unrealized_pnl_percent: 0.0,
            cost_basis: 0.0,
            last_updated: Utc::now(),
        }
    }

    pub fn new(symbol: String, shares: f64, price: f64) -> Self {
        let cost_basis = shares * price;
        Position {
            symbol,
            shares,
            average_price: price,
            current_price: price,
            current_value: cost_basis,
            unrealized_pnl: 0.0,
            unrealized_pnl_percent: 0.0,
            cost_basis,
            last_updated: Utc::now(),
        }
    }

    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
        self.current_value = self.shares * new_price;
        self.unrealized_pnl = self.current_value - self.cost_basis;
        self.unrealized_pnl_percent = if self.cost_basis > 0.0 {
            (self.unrealized_pnl / self.cost_basis) * 100.0
        } else {
            0.0
        };
        self.last_updated = Utc::now();
    }

    pub fn add_shares(&mut self, additional_shares: f64, price: f64) {
        let additional_cost = additional_shares * price;
        let total_cost = self.cost_basis + additional_cost;
        let total_shares = self.shares + additional_shares;

        if total_shares > 0.0 {
            self.average_price = total_cost / total_shares;
        }

        self.shares = total_shares;
        self.cost_basis = total_cost;
        self.update_price(price);
    }

    pub fn remove_shares(&mut self, shares_to_remove: f64, price: f64) -> Result<f64> {
        if shares_to_remove > self.shares {
            return Err(TradingPlatformError::internal("Cannot remove more shares than owned"));
        }

        let realized_pnl = shares_to_remove * (price - self.average_price);
        self.shares -= shares_to_remove;
        self.cost_basis = self.shares * self.average_price;
        self.update_price(price);

        Ok(realized_pnl)
    }

    pub fn is_empty(&self) -> bool {
        self.shares == 0.0
    }
}

impl Trade {
    pub fn new(
        strategy_id: String,
        symbol: String,
        action: Action,
        quantity: f64,
        price: f64,
        explanation: String,
        commission: f64,
    ) -> Self {
        let trade_value = quantity * price;
        
        Trade {
            id: Uuid::new_v4().to_string(),
            strategy_id,
            symbol,
            action,
            quantity,
            price,
            timestamp: Utc::now(),
            explanation,
            commission,
            realized_pnl: None,
            trade_value,
        }
    }

    pub fn with_realized_pnl(mut self, pnl: f64) -> Self {
        self.realized_pnl = Some(pnl);
        self
    }

    pub fn net_value(&self) -> f64 {
        match self.action {
            Action::Buy => -(self.trade_value + self.commission),
            Action::Sell => self.trade_value - self.commission,
            Action::Hold => 0.0,
        }
    }
}

impl Portfolio {
    pub fn new(strategy_id: String, initial_capital: f64) -> Self {
        Portfolio {
            id: Uuid::new_v4().to_string(),
            strategy_id,
            initial_capital,
            current_capital: initial_capital,
            positions: HashMap::new(),
            trade_history: Vec::new(),
            performance_snapshots: Vec::new(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    pub fn execute_trade(&mut self, trade: Trade) -> Result<()> {
        let symbol = trade.symbol.clone();
        
        match trade.action {
            Action::Buy => {
                if self.current_capital < trade.trade_value + trade.commission {
                    return Err(TradingPlatformError::internal("Insufficient capital for trade"));
                }

                self.current_capital -= trade.trade_value + trade.commission;
                
                let position = self.positions.entry(symbol.clone()).or_insert_with(|| Position::empty(symbol));
                position.add_shares(trade.quantity, trade.price);
            }
            Action::Sell => {
                if let Some(position) = self.positions.get_mut(&symbol) {
                    let realized_pnl = position.remove_shares(trade.quantity, trade.price)?;
                    self.current_capital += trade.trade_value - trade.commission;
                    
                    let mut updated_trade = trade;
                    updated_trade.realized_pnl = Some(realized_pnl);
                    self.trade_history.push(updated_trade);
                    
                    if position.is_empty() {
                        self.positions.remove(&symbol);
                    }
                } else {
                    return Err(TradingPlatformError::internal("Cannot sell shares not owned"));
                }
                return Ok(());
            }
            Action::Hold => {
                // No action needed for hold
            }
        }

        self.trade_history.push(trade);
        self.last_updated = Utc::now();
        Ok(())
    }

    pub fn update_position_prices(&mut self, prices: &HashMap<String, f64>) {
        for (symbol, position) in &mut self.positions {
            if let Some(&new_price) = prices.get(symbol) {
                position.update_price(new_price);
            }
        }
        self.last_updated = Utc::now();
    }

    pub fn total_value(&self) -> f64 {
        let positions_value: f64 = self.positions.values().map(|p| p.current_value).sum();
        self.current_capital + positions_value
    }

    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }

    pub fn create_snapshot(&mut self) {
        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            total_value: self.total_value(),
            cash_balance: self.current_capital,
            positions_value: self.positions.values().map(|p| p.current_value).sum(),
            total_return: self.total_value() - self.initial_capital,
            daily_return: 0.0, // Would be calculated based on previous snapshot
            drawdown: 0.0, // Would be calculated based on peak value
        };

        self.performance_snapshots.push(snapshot);
        self.last_updated = Utc::now();
    }
}

// Display implementations are in strategy module