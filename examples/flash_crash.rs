use qfin_twin::config::*;
use qfin_twin::simulation::SimulationEngine;

fn main() {
    let config = MarketConfig {
        market_id: "flash-crash-demo".into(),
        assets: vec![
            AssetConfig {
                symbol: "TECH".into(),
                name: "Tech Index".into(),
                initial_price: 100.0,
                volatility: 0.25,
                drift: 0.05,
                supply: 1_000_000,
            },
            AssetConfig {
                symbol: "BOND".into(),
                name: "Government Bond".into(),
                initial_price: 100.0,
                volatility: 0.05,
                drift: 0.02,
                supply: 10_000_000,
            },
        ],
        agents: vec![
            AgentDeployment {
                agent_type: AgentType::MarketMaker,
                count: 5,
                capital: 1_000_000.0,
                params: [
                    ("spread_factor".into(), 0.01),
                    ("inventory_limit".into(), 5000.0),
                    ("risk_aversion".into(), 0.5),
                ].into_iter().collect(),
            },
            AgentDeployment {
                agent_type: AgentType::Momentum,
                count: 20,
                capital: 100_000.0,
                params: [
                    ("lookback_ticks".into(), 10.0),
                    ("threshold".into(), 0.003),
                    ("conviction".into(), 0.8),
                ].into_iter().collect(),
            },
            AgentDeployment {
                agent_type: AgentType::Noise,
                count: 10,
                capital: 50_000.0,
                params: Default::default(),
            },
        ],
        params: MarketParams::default(),
        quantum: QuantumConfig::default(),
    };

    let mut engine = SimulationEngine::new(config, 42);

    println!("Running flash crash simulation...\n");
    println!("Market: TECH + BOND");
    println!("Agents: 5 Market Makers, 20 Momentum Traders, 10 Noise Traders\n");

    // Phase 1: Normal trading
    println!("Phase 1: Normal trading (500 ticks)...");
    let result = engine.run(500);
    println!("  Trades: {}", result.total_trades);
    println!("  TECH price: ${:.2}", result.final_prices.get("TECH").unwrap_or(&0.0));
    println!("  BOND price: ${:.2}", result.final_prices.get("BOND").unwrap_or(&0.0));
    println!("  Time: {:.1}ms\n", result.elapsed_ms);

    // In a full implementation, we'd inject a shock event here
    // and observe the cascade

    println!("Simulation complete.");
}
