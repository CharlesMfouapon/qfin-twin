use crate::types::*;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};

/// Price level in the order book.
#[derive(Debug, Clone)]
struct PriceLevel {
    orders: VecDeque<Order>,
    total_quantity: u64,
}

impl PriceLevel {
    fn new() -> Self {
        Self {
            orders: VecDeque::new(),
            total_quantity: 0,
        }
    }

    fn add_order(&mut self, order: Order) {
        self.total_quantity += order.quantity;
        self.orders.push_back(order);
    }

    fn remove_order(&mut self, order_id: OrderId) {
        if let Some(pos) = self.orders.iter().position(|o| o.order_id == order_id) {
            let order = self.orders.remove(pos).unwrap();
            self.total_quantity -= order.quantity;
        }
    }

    fn take_quantity(&mut self, quantity: u64) -> Vec<Order> {
        let mut taken = Vec::new();
        let mut remaining = quantity;

        while remaining > 0 && !self.orders.is_empty() {
            let order = self.orders.pop_front().unwrap();
            let fill_qty = remaining.min(order.quantity);
            remaining -= fill_qty;
            self.total_quantity -= fill_qty;

            if fill_qty < order.quantity {
                let mut residual = order.clone();
                residual.quantity = order.quantity - fill_qty;
                self.total_quantity += residual.quantity;
                self.orders.push_front(residual);
            }

            taken.push(Order {
                quantity: fill_qty,
                ..order
            });
        }

        taken
    }
}

/// A single order book for one asset.
pub struct OrderBook {
    symbol: String,
    bids: BTreeMap<u64, PriceLevel>, // Price levels in tick units, highest first
    asks: BTreeMap<u64, PriceLevel>, // Price levels in tick units, lowest first
    tick_size: f64,
}

impl OrderBook {
    pub fn new(symbol: String, tick_size: f64) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            tick_size,
        }
    }

    /// Convert a price to tick units.
    fn price_to_ticks(&self, price: f64) -> u64 {
        (price / self.tick_size).round() as u64
    }

    /// Convert tick units back to price.
    fn ticks_to_price(&self, ticks: u64) -> f64 {
        ticks as f64 * self.tick_size
    }

    /// Add an order to the book.
    pub fn add_order(&mut self, order: Order) {
        let price_ticks = match &order.order_type {
            OrderType::Limit { price } => self.price_to_ticks(*price),
            OrderType::Market => {
                // Market orders match immediately, not stored
                return;
            }
        };

        let book = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        book.entry(price_ticks)
            .or_insert_with(PriceLevel::new)
            .add_order(order);
    }

    /// Match a market order against the book and return resulting trades.
    pub fn match_market_order(
        &mut self,
        order: &Order,
    ) -> (Vec<Trade>, Option<Order>) {
        let contra_book = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let mut trades = Vec::new();
        let mut remaining_quantity = order.quantity;

        // Iterate through price levels from best to worst
        let price_levels: Vec<u64> = match order.side {
            Side::Buy => contra_book.keys().copied().collect(),
            Side::Sell => contra_book.keys().rev().copied().collect(),
        };

        for price_ticks in price_levels {
            if remaining_quantity == 0 {
                break;
            }

            if let Some(level) = contra_book.get_mut(&price_ticks) {
                let fills = level.take_quantity(remaining_quantity);
                let fill_total: u64 = fills.iter().map(|f| f.quantity).sum();
                remaining_quantity -= fill_total;

                let match_price = self.ticks_to_price(price_ticks);

                for fill in &fills {
                    let (buy_order_id, sell_order_id, buyer_type, seller_type) = match order.side {
                        Side::Buy => (order.order_id, fill.order_id, order.agent_type, fill.agent_type),
                        Side::Sell => (fill.order_id, order.order_id, fill.agent_type, order.agent_type),
                    };

                    trades.push(Trade {
                        trade_id: uuid::Uuid::new_v4(),
                        symbol: self.symbol.clone(),
                        price: match_price,
                        quantity: fill.quantity,
                        buy_order_id,
                        sell_order_id,
                        buyer_type,
                        seller_type,
                        timestamp: chrono::Utc::now(),
                        tick: order.tick,
                    });
                }

                // Remove empty price levels
                if level.orders.is_empty() {
                    contra_book.remove(&price_ticks);
                }
            }
        }

        let remainder = if remaining_quantity > 0 {
            Some(Order {
                quantity: remaining_quantity,
                ..order.clone()
            })
        } else {
            None
        };

        (trades, remainder)
    }

    /// Get the best bid price.
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|t| self.ticks_to_price(*t))
    }

    /// Get the best ask price.
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|t| self.ticks_to_price(*t))
    }

    /// Get the mid price.
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Get total volume at all price levels.
    pub fn total_bid_volume(&self) -> u64 {
        self.bids.values().map(|l| l.total_quantity).sum()
    }

    pub fn total_ask_volume(&self) -> u64 {
        self.asks.values().map(|l| l.total_quantity).sum()
    }

    /// Get the spread.
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
}
