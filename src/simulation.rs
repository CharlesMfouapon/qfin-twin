use crate::agents::Agent;
use crate::config::*;
use crate::market::order_book::OrderBook;
use crate::quantum::qaoa::QAOAOptimizer;
use crate::types::*;
use dashmap::DashMap;
use parking_lot::RwLock;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;

/// The main simulation engine.
pub struct SimulationEngine {
    config: MarketConfig,
    agents: Vec<Box<dyn Agent>>,
    order_books: HashMap<String, OrderBook>,
    prices: HashMap<String, f64>,
    historical_prices: HashMap<String, Vec<f64>>,
    trades: Vec<Trade>,
    tick: u64,
    circuit_breaker_active: bool,
    circuit_breaker_ticks_remaining: u64,
    rng: StdRng,
    qaoa: QAOAOptimizer,
}

impl SimulationEngine {
    pub fn new(config: MarketConfig, seed: u64) -> Self {
        let qaoa = QAOAOptimizer::new(3, 100, 0.01);
        let mut engine = Self {
            config,
            agents: Vec::new(),
            order_books: HashMap::new(),
            prices: HashMap::new(),
            historical_prices: HashMap::new(),
            trades: Vec::new(),
            tick: 0,
            circuit_breaker_active: false,
            circuit_breaker_ticks_remaining: 0,
            rng: StdRng::seed_from_u64(seed),
            qaoa,
        };

        engine.initialize();
        engine
    }

    fn initialize(&mut self) {
        // Initialize order books
        for asset in &self.config.assets {
            self.order_books.insert(
                asset.symbol.clone(),
                OrderBook::new(asset.symbol.clone(), self.config.params.tick_size),
            );
            self.prices.insert(asset.symbol.clone(), asset.initial_price);
            self.historical_prices
                .insert(asset.symbol.clone(), vec![asset.initial_price]);
        }
    }

    /// Run the simulation for a specified number of ticks.
    pub fn run(&mut self, ticks: u64) -> SimulationResult {
        let start = std::time::Instant::now();

        for _ in 0..ticks {
            self.step();
        }

        let elapsed_ms = start.elapsed().as_millis() as f64;

        SimulationResult {
            ticks_simulated: ticks,
            final_prices: self.prices.clone(),
            total_trades: self.trades.len() as u64,
            agent_pnl: self.compute_agent_pnl(),
            circuit_breaker_trips: 0,
            max_drawdown: self.compute_max_drawdown(),
            elapsed_ms,
        }
    }

    fn step(&mut self) {
        self.tick += 1;

        // Handle circuit breaker
        if self.circuit_breaker_active {
            self.circuit_breaker_ticks_remaining -= 1;
            if self.circuit_breaker_ticks_remaining == 0 {
                self.circuit_breaker_active = false;
            }
            return; // No trading during halt
        }

        // Collect market snapshot
        let snapshot = self.build_market_snapshot();

        // Get agent actions
        let all_orders: Vec<Order> = self
            .agents
            .iter_mut()
            .flat_map(|agent| agent.act(self.tick, &self.prices, &snapshot, &mut self.rng))
            .collect();

        // Process orders through order books
        let mut new_trades = Vec::new();
        for order in &all_orders {
            if let Some(book) = self.order_books.get_mut(&order.symbol) {
                match &order.order_type {
                    OrderType::Market => {
                        let (trades, _) = book.match_market_order(order);
                        new_trades.extend(trades);
                    }
                    OrderType::Limit { .. } => {
                        book.add_order(order.clone());
                    }
                }
            }
        }

        // Update prices based on last trade or mid price
        for (symbol, book) in &self.order_books {
            let new_price = new_trades
                .iter()
                .rev()
                .find(|t| t.symbol == *symbol)
                .map(|t| t.price)
                .or_else(|| book.mid_price())
                .unwrap_or(*self.prices.get(symbol).unwrap_or(&100.0));

            self.prices.insert(symbol.clone(), new_price);
            self.historical_prices
                .entry(symbol.clone())
                .or_default()
                .push(new_price);

            // Check circuit breaker
            if !self.circuit_breaker_active {
                let price_history = self.historical_prices.get(symbol).unwrap();
                if price_history.len() >= 2 {
                    let prev = price_history[price_history.len() - 2];
                    let curr = price_history[price_history.len() - 1];
                    let change = (curr - prev).abs() / prev;
                    if change > self.config.params.circuit_breaker_threshold {
                        self.circuit_breaker_active = true;
                        self.circuit_breaker_ticks_remaining =
                            self.config.params.circuit_breaker_cooldown_ticks;
                    }
                }
            }
        }

        // Notify agents of trades
        for trade in &new_trades {
            for agent in &mut self.agents {
                agent.on_trade(trade);
            }
        }

        self.trades.extend(new_trades);

        // Quantum-inspired portfolio optimization for hedge fund agents
        if self.tick % self.config.quantum.optimization_frequency_ticks == 0
            && self.config.quantum.enable_qaoa
        {
            self.run_quantum_optimization();
        }
    }

    fn build_market_snapshot(&self) -> crate::agents::MarketSnapshot {
        crate::agents::MarketSnapshot {
            prices: self.prices.clone(),
            bids: self
                .order_books
                .iter()
                .map(|(s, b)| (s.clone(), b.best_bid()))
                .collect(),
            asks: self
                .order_books
                .iter()
                .map(|(s, b)| (s.clone(), b.best_ask()))
                .collect(),
            volumes: self
                .order_books
                .iter()
                .map(|(s, b)| (s.clone(), b.total_bid_volume() + b.total_ask_volume()))
                .collect(),
            vix_equivalent: 0.0,
            tick: self.tick,
            historical_prices: self.historical_prices.clone(),
            recent_trades: self.trades.iter().rev().take(50).cloned().collect(),
        }
    }

    fn run_quantum_optimization(&mut self) {
        // Collect returns data for all assets
        let price_histories: Vec<Vec<f64>> = self.config.assets.iter().map(|a| {
            self.historical_prices.get(&a.symbol).cloned().unwrap_or_default()
        }).collect();

        let returns: Vec<Vec<f64>> = price_histories.iter().map(|prices| {
            if prices.len() < 2 { return vec![0.0]; }
            prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect()
        }).collect();

        let expected_returns = crate::quantum::compute_expected_returns(&price_histories);
        let covariance = crate::quantum::compute_covariance_matrix(&returns);

        if expected_returns.is_empty() || covariance.nrows() == 0 {
            return;
        }

        let _result = self.qaoa.optimize(
            &expected_returns,
            &covariance,
            1.0,
            0.4,
            &mut self.rng,
        );
    }

    fn compute_agent_pnl(&self) -> HashMap<String, f64> {
        self.agents
            .iter()
            .map(|a| {
                let pnl = a.portfolio().total_value(&self.prices);
                (format!("{:?}-{}", a.agent_type(), a.id().0), pnl)
            })
            .collect()
    }

    fn compute_max_drawdown(&self) -> f64 {
        // Compute max drawdown from the first asset's price history
        self.historical_prices
            .values()
            .next()
            .map(|prices| {
                let mut peak = prices[0];
                let mut max_dd = 0.0;
                for &price in prices {
                    if price > peak {
                        peak = price;
                    }
                    let dd = (peak - price) / peak;
                    if dd > max_dd {
                        max_dd = dd;
                    }
                }
                max_dd
            })
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub ticks_simulated: u64,
    pub final_prices: HashMap<String, f64>,
    pub total_trades: u64,
    pub agent_pnl: HashMap<String, f64>,
    pub circuit_breaker_trips: u64,
    pub max_drawdown: f64,
    pub elapsed_ms: f64,
}
