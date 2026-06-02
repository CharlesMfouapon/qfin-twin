pub mod market_maker;
pub mod momentum;
pub mod fundamental;
pub mod noise;
pub mod hedge_fund;
pub mod central_bank;

use crate::config::AgentType;
use crate::types::*;
use std::collections::HashMap;

/// Trait that all trading agents must implement.
pub trait Agent: Send + Sync {
    fn id(&self) -> AgentId;
    fn agent_type(&self) -> AgentType;
    fn portfolio(&self) -> &Portfolio;
    fn portfolio_mut(&mut self) -> &mut Portfolio;

    /// Called each simulation tick. Returns orders to submit.
    fn act(
        &mut self,
        tick: u64,
        market_prices: &HashMap<String, f64>,
        market_data: &MarketSnapshot,
        rng: &mut impl rand::Rng,
    ) -> Vec<Order>;

    /// Called when one of this agent's orders is filled.
    fn on_trade(&mut self, trade: &Trade);
}

/// Snapshot of market data available to agents each tick.
#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    pub prices: HashMap<String, f64>,
    pub bids: HashMap<String, Option<f64>>,
    pub asks: HashMap<String, Option<f64>>,
    pub volumes: HashMap<String, u64>,
    pub vix_equivalent: f64,
    pub tick: u64,
    pub historical_prices: HashMap<String, Vec<f64>>,
    pub recent_trades: Vec<Trade>,
}
