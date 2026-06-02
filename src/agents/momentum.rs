use super::*;
use crate::config::AgentType;
use rand::Rng;
use std::collections::HashMap;

/// Momentum trader that buys rising assets and sells falling ones.
pub struct MomentumTrader {
    id: AgentId,
    portfolio: Portfolio,
    lookback_ticks: usize,
    threshold: f64,    // Minimum return to trigger trade
    conviction: f64,   // How aggressively to trade (0-1)
    order_size: u64,
}

impl MomentumTrader {
    pub fn new(capital: f64, params: &HashMap<String, f64>) -> Self {
        Self {
            id: AgentId::new(),
            portfolio: Portfolio::new(capital),
            lookback_ticks: params.get("lookback_ticks").copied().unwrap_or(20.0) as usize,
            threshold: params.get("threshold").copied().unwrap_or(0.005),
            conviction: params.get("conviction").copied().unwrap_or(0.7),
            order_size: params.get("order_size").copied().unwrap_or(50.0) as u64,
        }
    }

    /// Calculate momentum signal for a symbol.
    fn momentum_signal(
        &self,
        historical_prices: &[f64],
    ) -> Option<f64> {
        if historical_prices.len() < self.lookback_ticks + 1 {
            return None;
        }

        let recent = &historical_prices[historical_prices.len() - self.lookback_ticks..];
        let start_price = recent[0];
        let end_price = recent[recent.len() - 1];

        let ret = (end_price / start_price) - 1.0;
        let abs_ret = ret.abs();

        if abs_ret > self.threshold {
            Some(ret * self.conviction)
        } else {
            None
        }
    }
}

impl Agent for MomentumTrader {
    fn id(&self) -> AgentId { self.id }
    fn agent_type(&self) -> AgentType { AgentType::Momentum }
    fn portfolio(&self) -> &Portfolio { &self.portfolio }
    fn portfolio_mut(&mut self) -> &mut Portfolio { &mut self.portfolio }

    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        market_data: &MarketSnapshot,
        _rng: &mut impl Rng,
    ) -> Vec<Order> {
        let mut orders = Vec::new();

        for (symbol, &price) in market_prices {
            if let Some(prices) = market_data.historical_prices.get(symbol) {
                if let Some(signal) = self.momentum_signal(prices) {
                    let side = if signal > 0.0 { Side::Buy } else { Side::Sell };
                    let size = (self.order_size as f64 * signal.abs()).round() as u64;

                    if size > 0 && price > 0.0 {
                        orders.push(Order {
                            order_id: OrderId::new(),
                            agent_id: self.id,
                            agent_type: self.agent_type(),
                            symbol: symbol.clone(),
                            side,
                            order_type: OrderType::Market,
                            quantity: size,
                            timestamp: chrono::Utc::now(),
                            tick,
                        });
                    }
                }
            }
        }

        orders
    }

    fn on_trade(&mut self, trade: &Trade) {
        let quantity_delta: i64 = if trade.buy_order_id == self.id.into() {
            trade.quantity as i64
        } else {
            -(trade.quantity as i64)
        };
        let cash_delta = -(quantity_delta as f64 * trade.price);
        self.portfolio.cash += cash_delta;
        self.portfolio.update_position(&trade.symbol, quantity_delta, trade.price);
    }
}
