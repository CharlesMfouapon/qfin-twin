use super::*;
use crate::config::AgentType;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;

/// Fundamental investor that trades toward perceived fair value using mean-reversion logic.
pub struct FundamentalInvestor {
    id: AgentId,
    portfolio: Portfolio,
    patience: u64,
    conviction: f64,
    margin: f64,
    fair_value_estimates: HashMap<String, f64>,
    ticks_since_last_eval: u64,
}

impl FundamentalInvestor {
    pub fn new(capital: f64, params: &HashMap<String, f64>) -> Self {
        Self {
            id: AgentId::new(),
            portfolio: Portfolio::new(capital),
            patience: params.get("patience").copied().unwrap_or(50.0) as u64,
            conviction: params.get("conviction").copied().unwrap_or(0.3),
            margin: params.get("margin").copied().unwrap_or(0.05),
            fair_value_estimates: HashMap::new(),
            ticks_since_last_eval: 0,
        }
    }

    /// Estimate fair value using a noisy P/E mean-reversion model.
    fn estimate_fair_value(&self, current_price: f64, historical_prices: &[f64], rng: &mut impl Rng) -> f64 {
        if historical_prices.len() < 20 {
            return current_price;
        }

        // Compute long-term moving average
        let ma: f64 = historical_prices.iter().sum::<f64>() / historical_prices.len() as f64;
        
        // Add noise to represent valuation uncertainty
        let normal = Normal::new(0.0, 0.03).unwrap();
        let noise = normal.sample(rng);
        
        // Mean reversion: fair value is anchored to long-term average with noise
        ma * (1.0 + noise)
    }
}

impl Agent for FundamentalInvestor {
    fn id(&self) -> AgentId { self.id }
    fn agent_type(&self) -> AgentType { AgentType::Fundamental }
    fn portfolio(&self) -> &Portfolio { &self.portfolio }
    fn portfolio_mut(&mut self) -> &mut Portfolio { &mut self.portfolio }

    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        market_data: &MarketSnapshot,
        rng: &mut impl Rng,
    ) -> Vec<Order> {
        self.ticks_since_last_eval += 1;
        let mut orders = Vec::new();

        // Only re-evaluate periodically
        if self.ticks_since_last_eval < self.patience {
            return orders;
        }
        self.ticks_since_last_eval = 0;

        for (symbol, &price) in market_prices {
            let historical = market_data
                .historical_prices
                .get(symbol)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let fair_value = self.estimate_fair_value(price, historical, rng);
            self.fair_value_estimates.insert(symbol.clone(), fair_value);

            let mispricing = (fair_value - price) / price;

            if mispricing.abs() > self.margin {
                let side = if mispricing > 0.0 {
                    Side::Buy
                } else {
                    Side::Sell
                };

                let position = self
                    .portfolio
                    .positions
                    .iter()
                    .find(|p| p.symbol == *symbol)
                    .map(|p| p.quantity)
                    .unwrap_or(0);

                let max_trade = (self.portfolio.cash * self.conviction / price) as u64;
                let trade_size = (max_trade as f64 * mispricing.abs()).round() as u64;

                if trade_size > 0 {
                    orders.push(Order {
                        order_id: OrderId::new(),
                        agent_id: self.id,
                        agent_type: self.agent_type(),
                        symbol: symbol.clone(),
                        side,
                        order_type: OrderType::Limit {
                            price: if side == Side::Buy {
                                price * (1.0 - self.margin)
                            } else {
                                price * (1.0 + self.margin)
                            },
                        },
                        quantity: trade_size,
                        timestamp: chrono::Utc::now(),
                        tick,
                    });
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
