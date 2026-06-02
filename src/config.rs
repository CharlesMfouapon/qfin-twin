use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete market configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub market_id: String,
    pub assets: Vec<AssetConfig>,
    pub agents: Vec<AgentDeployment>,
    pub params: MarketParams,
    pub quantum: QuantumConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub symbol: String,
    pub name: String,
    pub initial_price: f64,
    pub volatility: f64,
    pub drift: f64,
    pub supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDeployment {
    pub agent_type: AgentType,
    pub count: u32,
    pub capital: f64,
    pub params: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    MarketMaker,
    Momentum,
    Fundamental,
    Noise,
    HedgeFund,
    CentralBank,
}

impl AgentType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "market_maker" => Some(Self::MarketMaker),
            "momentum" => Some(Self::Momentum),
            "fundamental" => Some(Self::Fundamental),
            "noise" => Some(Self::Noise),
            "hedge_fund" => Some(Self::HedgeFund),
            "central_bank" => Some(Self::CentralBank),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketParams {
    pub tick_size: f64,
    pub max_order_size: u64,
    pub circuit_breaker_threshold: f64,
    pub circuit_breaker_cooldown_ticks: u64,
    pub enable_short_selling: bool,
    pub transaction_tax: f64,
}

impl Default for MarketParams {
    fn default() -> Self {
        Self {
            tick_size: 0.01,
            max_order_size: 10_000,
            circuit_breaker_threshold: 0.10, // 10% move triggers halt
            circuit_breaker_cooldown_ticks: 20,
            enable_short_selling: true,
            transaction_tax: 0.001, // 0.1%
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumConfig {
    pub enable_qaoa: bool,
    pub enable_annealing: bool,
    pub enable_tensor_networks: bool,
    pub optimization_frequency_ticks: u64,
}

impl Default for QuantumConfig {
    fn default() -> Self {
        Self {
            enable_qaoa: true,
            enable_annealing: true,
            enable_tensor_networks: false,
            optimization_frequency_ticks: 100,
        }
    }
}

/// Event types that can be injected into the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    RateHike,
    RateCut,
    BankFailure,
    GeopoliticalShock,
    FlashCrash,
    LiquidityInjection,
    RegulatoryChange,
}

impl EventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rate_hike" => Some(Self::RateHike),
            "rate_cut" => Some(Self::RateCut),
            "bank_failure" => Some(Self::BankFailure),
            "geopolitical" => Some(Self::GeopoliticalShock),
            "flash_crash" => Some(Self::FlashCrash),
            "liquidity_injection" => Some(Self::LiquidityInjection),
            "regulatory_change" => Some(Self::RegulatoryChange),
            _ => None,
        }
    }
}
