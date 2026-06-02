use super::*;
use crate::config::AgentType;
use rand::Rng;
use std::collections::HashMap;

/// Market maker agent that provides continuous bid/ask quotes.
/// Uses an Avellaneda-Stoikov-inspired inventory management approach.
pub struct MarketMaker {
    id: AgentId,
    portfolio: Portfolio,
    spread_factor: f64,
    inventory_limit: u64,
    risk_aversion: f64,
    order_size: u64,
}

impl MarketMaker {
    pub fn new(capital: f64, params: &HashMap<String, f64>) -> Self {
        Self {
            id: AgentId::new(),
            portfolio: Portfolio::new(capital),
            spread_factor: params.get("spread_factor").copied().unwrap_or(0.02),
            inventory_limit: params.get("inventory_limit").copied().unwrap_or(1000.0) as u64,
            risk_aversion: params.get("risk_aversion").copied().unwrap_or(0.5),
            order_size: params.get("order_size").copied().unwrap_or(100.0) as u64,
        }
    }

    /// Calculate optimal bid/ask using inventory-aware pricing.
    fn calculate_quotes(
        &self,
        mid_price: f64,
        inventory_position: i64,
        volatility: f64,
    ) -> (f64, f64) {
        // Inventory penalty: skew quotes to reduce position
        let inventory_ratio = inventory_position as f64 / self.inventory_limit as f64;
        let inventory_skew = -self.risk_aversion * volatility * inventory_ratio;

        let half_spread = self.spread_factor * mid_price;
        let bid = mid_price - half_spread + inventory_skew;
        let ask = mid_price + half_spread + inventory_skew;

        (bid.max(0.01), ask.max(bid + 0.01))
    }
}

impl Agent for MarketMaker {
    fn id(&self) -> AgentId { self.id }
    fn agent_type(&self) -> AgentType { AgentType::MarketMaker }
    fn portfolio(&self) -> &Portfolio { &self.portfolio }
    fn portfolio_mut(&mut self) -> &mut Portfolio { &mut self.portfolio }

    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        market_data: &MarketSnapshot,
        rng: &mut impl Rng,
    ) -> Vec<Order> {
        let mut orders = Vec::new();

        for (symbol, &price) in market_prices {
            let position = self
                .portfolio
                .positions
                .iter()
                .find(|p| p.symbol == *symbol)
                .map(|p| p.quantity)
                .unwrap_or(0);

            // Estimate volatility from recent prices
            let volatility = estimate_volatility(
                market_data.historical_prices.get(symbol).map(|v| v.as_slice()).unwrap_or(&[]),
            );

            let (bid, ask) = self.calculate_quotes(price, position, volatility);

            // Randomize order size slightly
            let size = self.order_size + (rng.gen_range(0..20) as u64);

            orders.push(Order {
                order_id: OrderId::new(),
                agent_id: self.id,
                agent_type: self.agent_type(),
                symbol: symbol.clone(),
                side: Side::Buy,
                order_type: OrderType::Limit { price: bid },
                quantity: size,
                timestamp: chrono::Utc::now(),
                tick,
            });

            orders.push(Order {
                order_id: OrderId::new(),
                agent_id: self.id,
                agent_type: self.agent_type(),
                symbol: symbol.clone(),
                side: Side::Sell,
                order_type: OrderType::Limit { price: ask },
                quantity: size,
                timestamp: chrono::Utc::now(),
                tick,
            });
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

/// Simple volatility estimation from recent prices.
fn estimate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.02; // Default 2% volatility
    }

    let returns: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] / w[0]).ln())
        .collect();

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

    variance.sqrt().max(0.001)
}

impl From<OrderId> for uuid::Uuid {
    fn from(id: OrderId) -> Self {
        id.0
    }
}
