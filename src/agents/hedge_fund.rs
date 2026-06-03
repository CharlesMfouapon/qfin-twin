use super::*;
use crate::config::AgentType;
use crate::quantum::qaoa::QAOAOptimizer;
use crate::quantum::{compute_covariance_matrix, compute_expected_returns};
use rand::Rng;
use std::collections::HashMap;

/// Hedge fund agent that runs multiple sub-strategies and uses quantum-inspired
/// optimization for portfolio allocation.
pub struct HedgeFund {
    id: AgentId,
    portfolio: Portfolio,
    risk_budget: f64,
    leverage: f64,
    rebalance_ticks: u64,
    ticks_since_rebalance: u64,
    qaoa: QAOAOptimizer,
    // Sub-strategy allocations (weights)
    momentum_weight: f64,
    fundamental_weight: f64,
    market_making_weight: f64,
}

impl HedgeFund {
    pub fn new(capital: f64, params: &HashMap<String, f64>) -> Self {
        Self {
            id: AgentId::new(),
            portfolio: Portfolio::new(capital),
            risk_budget: params.get("risk_budget").copied().unwrap_or(0.15),
            leverage: params.get("leverage").copied().unwrap_or(1.5),
            rebalance_ticks: params.get("rebalance_ticks").copied().unwrap_or(100.0) as u64,
            ticks_since_rebalance: 0,
            qaoa: QAOAOptimizer::new(4, 200, 0.005),
            momentum_weight: 0.4,
            fundamental_weight: 0.35,
            market_making_weight: 0.25,
        }
    }

    /// Generate momentum signal for a symbol.
    fn momentum_signal(prices: &[f64], lookback: usize) -> f64 {
        if prices.len() < lookback + 1 {
            return 0.0;
        }
        let recent = &prices[prices.len() - lookback..];
        (recent[recent.len() - 1] / recent[0]) - 1.0
    }

    /// Generate fundamental signal using moving average crossover.
    fn fundamental_signal(prices: &[f64]) -> f64 {
        if prices.len() < 50 {
            return 0.0;
        }
        let short_ma: f64 = prices[prices.len() - 20..].iter().sum::<f64>() / 20.0;
        let long_ma: f64 = prices[prices.len() - 50..].iter().sum::<f64>() / 50.0;
        (short_ma / long_ma) - 1.0
    }
}

impl Agent for HedgeFund {
    fn id(&self) -> AgentId { self.id }
    fn agent_type(&self) -> AgentType { AgentType::HedgeFund }
    fn portfolio(&self) -> &Portfolio { &self.portfolio }
    fn portfolio_mut(&mut self) -> &mut Portfolio { &mut self.portfolio }

    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        market_data: &MarketSnapshot,
        rng: &mut impl Rng,
    ) -> Vec<Order> {
        self.ticks_since_rebalance += 1;
        let mut orders = Vec::new();

        let total_capital = self.portfolio.total_value(market_prices) * self.leverage;

        for (symbol, &price) in market_prices {
            let historical = market_data
                .historical_prices
                .get(symbol)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            // Composite signal from sub-strategies
            let momentum_sig = Self::momentum_signal(historical, 20);
            let fundamental_sig = Self::fundamental_signal(historical);
            let composite = momentum_sig * self.momentum_weight
                + fundamental_sig * self.fundamental_weight;

            let trade_value = total_capital * composite.abs() * 0.1; // 10% of capital per signal
            let trade_size = (trade_value / price).round() as u64;

            if trade_size > 0 && composite.abs() > 0.01 {
                let side = if composite > 0.0 {
                    Side::Buy
                } else {
                    Side::Sell
                };

                orders.push(Order {
                    order_id: OrderId::new(),
                    agent_id: self.id,
                    agent_type: self.agent_type(),
                    symbol: symbol.clone(),
                    side,
                    order_type: OrderType::Limit {
                        price: if side == Side::Buy {
                            price * 0.995
                        } else {
                            price * 1.005
                        },
                    },
                    quantity: trade_size,
                    timestamp: chrono::Utc::now(),
                    tick,
                });
            }
        }

        // Periodic quantum-inspired rebalancing
        if self.ticks_since_rebalance >= self.rebalance_ticks {
            self.ticks_since_rebalance = 0;
            self.rebalance_strategy_weights(market_data, rng);
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

impl HedgeFund {
    /// Use QAOA to optimize the weights between sub-strategies.
    fn rebalance_strategy_weights(&mut self, market_data: &MarketSnapshot, rng: &mut impl Rng) {
        // Build pseudo-returns for each strategy as "assets"
        let n_strategies = 3;
        let n_periods = 50.min(
            market_data
                .historical_prices
                .values()
                .next()
                .map(|v| v.len())
                .unwrap_or(1),
        );

        if n_periods < 10 {
            return;
        }

        // Simplified: use asset price data as proxy for strategy returns
        let price_histories: Vec<Vec<f64>> = market_data
            .historical_prices
            .values()
            .take(n_strategies)
            .cloned()
            .collect();

        if price_histories.len() < n_strategies {
            return;
        }

        let returns: Vec<Vec<f64>> = price_histories
            .iter()
            .map(|p| {
                p.windows(2)
                    .map(|w| (w[1] / w[0]).ln())
                    .collect()
            })
            .collect();

        let expected_returns = compute_expected_returns(&price_histories);
        let covariance = compute_covariance_matrix(&returns);

        if expected_returns.len() < n_strategies || covariance.nrows() < n_strategies {
            return;
        }

        let result = self.qaoa.optimize(
            &expected_returns[..n_strategies],
            &covariance,
            1.0,
            0.6,
            rng,
        );

        if result.optimal_weights.len() >= 3 {
            self.momentum_weight = result.optimal_weights[0];
            self.fundamental_weight = result.optimal_weights[1];
            self.market_making_weight = result.optimal_weights[2];
        }
    }
}
