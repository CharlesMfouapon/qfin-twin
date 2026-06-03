use super::*;
use crate::config::AgentType;
use rand::Rng;
use rand_distr::{Distribution, LogNormal};
use std::collections::HashMap;

/// Noise trader that submits random buy/sell orders.
pub struct NoiseTrader {
    id: AgentId,
    portfolio: Portfolio,
    trade_probability: f64,
    size_distribution: LogNormal<f64>,
    max_order_value: f64,
}

impl NoiseTrader {
    pub fn new(capital: f64, params: &HashMap<String, f64>) -> Self {
        let trade_prob = params.get("trade_probability").copied().unwrap_or(0.1);
        let mean_size = params.get("mean_trade_size").copied().unwrap_or(10.0);
        let std_size = params.get("std_trade_size").copied().unwrap_or(5.0);

        Self {
            id: AgentId::new(),
            portfolio: Portfolio::new(capital),
            trade_probability: trade_prob,
            size_distribution: LogNormal::new(mean_size.ln(), std_size.ln().max(0.1)).unwrap(),
            max_order_value: capital * 0.05, // Max 5% of capital per trade
        }
    }
}

impl Agent for NoiseTrader {
    fn id(&self) -> AgentId { self.id }
    fn agent_type(&self) -> AgentType { AgentType::Noise }
    fn portfolio(&self) -> &Portfolio { &self.portfolio }
    fn portfolio_mut(&mut self) -> &mut Portfolio { &mut self.portfolio }

    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        _market_data: &MarketSnapshot,
        rng: &mut impl Rng,
    ) -> Vec<Order> {
        let mut orders = Vec::new();

        // Random decision to trade
        if rng.gen::<f64>() > self.trade_probability {
            return orders;
        }

        // Select random asset
        let symbols: Vec<&String> = market_prices.keys().collect();
        if symbols.is_empty() {
            return orders;
        }

        let symbol = symbols[rng.gen_range(0..symbols.len())].clone();
        let price = market_prices.get(symbol).copied().unwrap_or(100.0);

        // Random side
        let side = if rng.gen::<bool>() { Side::Buy } else { Side::Sell };

        // Random size from lognormal distribution
        let raw_size = self.size_distribution.sample(rng).round() as u64;
        let max_size = (self.max_order_value / price) as u64;
        let size = raw_size.min(max_size).max(1);

        // Slightly favor market orders for noise traders (immediacy)
        let order_type = if rng.gen::<f64>() < 0.7 {
            OrderType::Market
        } else {
            OrderType::Limit {
                price: if side == Side::Buy {
                    price * (1.0 + rng.gen_range(0.0..0.02))
                } else {
                    price * (1.0 - rng.gen_range(0.0..0.02))
                },
            }
        };

        orders.push(Order {
            order_id: OrderId::new(),
            agent_id: self.id,
            agent_type: self.agent_type(),
            symbol: symbol.clone(),
            side,
            order_type,
            quantity: size,
            timestamp: chrono::Utc::now(),
            tick,
        });

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
