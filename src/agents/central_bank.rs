use super::*;
use crate::config::AgentType;
use rand::Rng;
use std::collections::HashMap;

/// Central bank agent that implements monetary policy through market operations.
/// Intervenes to stabilize inflation and output around targets.
pub struct CentralBank {
    id: AgentId,
    portfolio: Portfolio,
    inflation_target: f64,
    intervention_ticks: u64,
    ticks_since_intervention: u64,
    intervention_size: f64,
    price_history: HashMap<String, Vec<f64>>,
}

impl CentralBank {
    pub fn new(capital: f64, params: &HashMap<String, f64>) -> Self {
        Self {
            id: AgentId::new(),
            portfolio: Portfolio::new(capital),
            inflation_target: params.get("inflation_target").copied().unwrap_or(0.02),
            intervention_ticks: params.get("intervention_ticks").copied().unwrap_or(100.0) as u64,
            ticks_since_intervention: 0,
            intervention_size: params.get("intervention_size").copied().unwrap_or(1_000_000.0),
            price_history: HashMap::new(),
        }
    }

    /// Compute annualized inflation from price history.
    fn compute_inflation(&self, symbol: &str) -> Option<f64> {
        let prices = self.price_history.get(symbol)?;
        if prices.len() < 100 {
            return None;
        }

        let current = prices[prices.len() - 1];
        let year_ago = prices[prices.len().saturating_sub(100)];

        if year_ago > 0.0 {
            Some((current / year_ago) - 1.0)
        } else {
            None
        }
    }

    /// Taylor rule: determine target interest rate based on inflation and output.
    fn taylor_rule(&self, inflation: f64, output_gap: f64) -> f64 {
        let neutral_rate = 0.02; // 2% neutral rate
        let inflation_weight = 1.5;
        let output_weight = 0.5;

        neutral_rate + inflation + inflation_weight * (inflation - self.inflation_target)
            + output_weight * output_gap
    }
}

impl Agent for CentralBank {
    fn id(&self) -> AgentId { self.id }
    fn agent_type(&self) -> AgentType { AgentType::CentralBank }
    fn portfolio(&self) -> &Portfolio { &self.portfolio }
    fn portfolio_mut(&mut self) -> &mut Portfolio { &mut self.portfolio }

    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        market_data: &MarketSnapshot,
        _rng: &mut impl Rng,
    ) -> Vec<Order> {
        // Update price history
        for (symbol, &price) in market_prices {
            self.price_history
                .entry(symbol.clone())
                .or_default()
                .push(price);
        }

        self.ticks_since_intervention += 1;

        if self.ticks_since_intervention < self.intervention_ticks {
            return vec![];
        }

        self.ticks_since_intervention = 0;
        let mut orders = Vec::new();

        // Evaluate economic conditions and intervene
        for (symbol, &price) in market_prices {
            if let Some(inflation) = self.compute_inflation(symbol) {
                let output_gap = 0.0; // Simplified: assume output at potential
                let target_rate = self.taylor_rule(inflation, output_gap);
                let current_rate = 0.03; // Simplified proxy for current rate

                // If inflation is above target, tighten (sell assets)
                // If inflation is below target, loosen (buy assets)
                let intervention_direction = target_rate - current_rate;

                if intervention_direction.abs() > 0.005 {
                    let side = if intervention_direction > 0.0 {
                        Side::Sell // Tighten: reduce liquidity
                    } else {
                        Side::Buy // Loosen: inject liquidity
                    };

                    let size = (self.intervention_size / price) as u64;

                    if size > 0 {
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
