use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

impl OrderId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Unique identifier for an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Order side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
}

/// An order submitted by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: OrderId,
    pub agent_id: AgentId,
    pub agent_type: crate::config::AgentType,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
    pub timestamp: DateTime<Utc>,
    pub tick: u64,
}

/// A trade resulting from matched orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: Uuid,
    pub symbol: String,
    pub price: f64,
    pub quantity: u64,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub buyer_type: crate::config::AgentType,
    pub seller_type: crate::config::AgentType,
    pub timestamp: DateTime<Utc>,
    pub tick: u64,
}

/// Agent portfolio state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub cash: f64,
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: i64, // Negative for short positions
    pub average_price: f64,
}

impl Portfolio {
    pub fn new(cash: f64) -> Self {
        Self {
            cash,
            positions: Vec::new(),
        }
    }

    pub fn total_value(&self, prices: &std::collections::HashMap<String, f64>) -> f64 {
        let position_value: f64 = self
            .positions
            .iter()
            .map(|p| {
                let price = prices.get(&p.symbol).copied().unwrap_or(0.0);
                p.quantity as f64 * price
            })
            .sum();
        self.cash + position_value
    }

    pub fn update_position(&mut self, symbol: &str, quantity_delta: i64, price: f64) {
        if let Some(pos) = self.positions.iter_mut().find(|p| p.symbol == symbol) {
            let new_quantity = pos.quantity + quantity_delta;
            if new_quantity == 0 {
                self.positions.retain(|p| p.symbol != symbol);
            } else {
                // Update average price for the remaining position
                let old_value = pos.quantity as f64 * pos.average_price;
                let trade_value = quantity_delta as f64 * price;
                pos.quantity = new_quantity;
                pos.average_price = (old_value + trade_value) / new_quantity as f64;
            }
        } else if quantity_delta != 0 {
            self.positions.push(Position {
                symbol: symbol.to_string(),
                quantity: quantity_delta,
                average_price: price,
            });
        }
    }
}
