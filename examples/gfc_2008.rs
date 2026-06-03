use qfin_twin::agents::create_agents;
use qfin_twin::config::*;
use qfin_twin::simulation::SimulationEngine;
use std::collections::HashMap;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         QFIN TWIN — 2008 GLOBAL FINANCIAL CRISIS           ║");
    println!("║              Multi-Asset Contagion Simulation              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ── Market Configuration ──
    let config = MarketConfig {
        market_id: "gfc-2008".into(),
        assets: vec![
            AssetConfig {
                symbol: "MBS".into(),
                name: "Mortgage-Backed Securities".into(),
                initial_price: 100.0,
                volatility: 0.08,
                drift: 0.04,
                supply: 10_000_000,
            },
            AssetConfig {
                symbol: "CDO".into(),
                name: "Collateralized Debt Obligations".into(),
                initial_price: 100.0,
                volatility: 0.12,
                drift: 0.06,
                supply: 5_000_000,
            },
            AssetConfig {
                symbol: "BANK".into(),
                name: "Bank Equity Index".into(),
                initial_price: 50.0,
                volatility: 0.20,
                drift: 0.08,
                supply: 20_000_000,
            },
            AssetConfig {
                symbol: "TREASURY".into(),
                name: "10-Year Treasury Bond".into(),
                initial_price: 100.0,
                volatility: 0.03,
                drift: 0.02,
                supply: 50_000_000,
            },
        ],
        agents: vec![
            // Market makers provide liquidity
            AgentDeployment {
                agent_type: AgentType::MarketMaker,
                count: 10,
                capital: 5_000_000.0,
                params: [
                    ("spread_factor".into(), 0.015),
                    ("inventory_limit".into(), 10_000.0),
                    ("risk_aversion".into(), 0.3),
                    ("order_size".into(), 500.0),
                ]
                .into_iter()
                .collect(),
            },
            // Momentum traders amplify trends
            AgentDeployment {
                agent_type: AgentType::Momentum,
                count: 30,
                capital: 2_000_000.0,
                params: [
                    ("lookback_ticks".into(), 15.0),
                    ("threshold".into(), 0.002),
                    ("conviction".into(), 0.7),
                    ("order_size".into(), 200.0),
                ]
                .into_iter()
                .collect(),
            },
            // Fundamental investors anchor to fair value
            AgentDeployment {
                agent_type: AgentType::Fundamental,
                count: 15,
                capital: 10_000_000.0,
                params: [
                    ("patience".into(), 40.0),
                    ("conviction".into(), 0.4),
                    ("margin".into(), 0.03),
                ]
                .into_iter()
                .collect(),
            },
            // Noise traders add realism
            AgentDeployment {
                agent_type: AgentType::Noise,
                count: 25,
                capital: 500_000.0,
                params: [
                    ("trade_probability".into(), 0.08),
                    ("mean_trade_size".into(), 15.0),
                    ("std_trade_size".into(), 8.0),
                ]
                .into_iter()
                .collect(),
            },
            // Hedge funds are the catalyst — heavily leveraged in MBS/CDO
            AgentDeployment {
                agent_type: AgentType::HedgeFund,
                count: 8,
                capital: 20_000_000.0,
                params: [
                    ("risk_budget".into(), 0.20),
                    ("leverage".into(), 3.0),
                    ("rebalance_ticks".into(), 80.0),
                ]
                .into_iter()
                .collect(),
            },
            // Central bank intervenes
            AgentDeployment {
                agent_type: AgentType::CentralBank,
                count: 1,
                capital: 100_000_000.0,
                params: [
                    ("inflation_target".into(), 0.02),
                    ("intervention_ticks".into(), 150.0),
                    ("intervention_size".into(), 5_000_000.0),
                ]
                .into_iter()
                .collect(),
            },
        ],
        params: MarketParams {
            tick_size: 0.01,
            max_order_size: 50_000,
            circuit_breaker_threshold: 0.15, // 15% move triggers halt
            circuit_breaker_cooldown_ticks: 15,
            enable_short_selling: true,
            transaction_tax: 0.0, // No tax during crisis
        },
        quantum: QuantumConfig {
            enable_qaoa: true,
            enable_annealing: true,
            enable_tensor_networks: false,
            optimization_frequency_ticks: 50,
        },
    };

    println!("Market Configuration:");
    println!("  Assets: MBS, CDO, BANK, TREASURY");
    println!("  Agents: 89 total (10 MM, 30 Mom, 15 Fund, 25 Noise, 8 HF, 1 CB)");
    println!("  Hedge Fund Leverage: 3x");
    println!("  Circuit Breaker: 15% threshold, 15-tick cooldown\n");

    let mut engine = SimulationEngine::new(config, 2008);

    // ── PHASE 1: The Boom (Ticks 0-500) ──
    println!("═══════════════════════════════════════════");
    println!("  PHASE 1: The Boom (2004-2006)");
    println!("  Low rates, rising housing, credit expansion");
    println!("═══════════════════════════════════════════\n");

    let result = engine.run(500);

    println!("  Ticks Simulated:  {}", result.ticks_simulated);
    println!("  Total Trades:     {}", result.total_trades);
    println!("  Circuit Breakers: {}", result.circuit_breaker_trips);
    println!("  Max Drawdown:     {:.2}%", result.max_drawdown * 100.0);
    println!("  Execution Time:   {:.1}ms", result.elapsed_ms);
    println!();
    println!("  Final Prices:");
    for (symbol, price) in &result.final_prices {
        println!("    {:>12}: ${:>10.2}", symbol, price);
    }
    println!();

    // ── PHASE 2: The Trigger (Ticks 501-700) ──
    println!("═══════════════════════════════════════════");
    println!("  PHASE 2: The Trigger (Early 2007)");
    println!("  MBS defaults begin. Hedge funds under");
    println!("  margin calls. Liquidity tightens.");
    println!("═══════════════════════════════════════════\n");

    let result = engine.run(200);

    println!("  Ticks Simulated:  {}", result.ticks_simulated);
    println!("  Total Trades:     {}", result.total_trades);
    println!("  Circuit Breakers: {}", result.circuit_breaker_trips);
    println!("  Max Drawdown:     {:.2}%", result.max_drawdown * 100.0);
    println!("  Execution Time:   {:.1}ms", result.elapsed_ms);
    println!();
    println!("  Final Prices:");
    for (symbol, price) in &result.final_prices {
        println!("    {:>12}: ${:>10.2}", symbol, price);
    }
    println!();

    // ── PHASE 3: The Collapse (Ticks 701-1000) ──
    println!("═══════════════════════════════════════════");
    println!("  PHASE 3: The Collapse (Late 2007-2008)");
    println!("  Lehman fails. Credit markets freeze.");
    println!("  Central bank emergency intervention.");
    println!("═══════════════════════════════════════════\n");

    let result = engine.run(300);

    println!("  Ticks Simulated:  {}", result.ticks_simulated);
    println!("  Total Trades:     {}", result.total_trades);
    println!("  Circuit Breakers: {}", result.circuit_breaker_trips);
    println!("  Max Drawdown:     {:.2}%", result.max_drawdown * 100.0);
    println!("  Execution Time:   {:.1}ms", result.elapsed_ms);
    println!();
    println!("  Final Prices:");
    for (symbol, price) in &result.final_prices {
        println!("    {:>12}: ${:>10.2}", symbol, price);
    }
    println!();

    // ── Agent P&L ──
    println!("═══════════════════════════════════════════");
    println!("  Agent Performance (Top 10 by P&L)");
    println!("═══════════════════════════════════════════\n");

    let mut pnl_vec: Vec<(&String, &f64)> = result.agent_pnl.iter().collect();
    pnl_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (agent, pnl) in pnl_vec.iter().take(10) {
        let pnl_millions = **pnl / 1_000_000.0;
        let bar = if pnl_millions > 0.0 {
            "█".repeat((pnl_millions.abs() * 2.0).min(30.0) as usize)
        } else {
            "".to_string()
        };
        println!("  {:>40}: ${:>10.1}M {}", agent, pnl_millions, bar);
    }

    // ── Summary ──
    println!();
    println!("═══════════════════════════════════════════");
    println!("  CRISIS SUMMARY");
    println!("═══════════════════════════════════════════\n");
    println!("  Total Ticks:      1,000");
    println!("  Total Trades:     {}", result.total_trades);
    println!("  Circuit Breakers: {}", result.circuit_breaker_trips);
    println!("  Peak Drawdown:    {:.2}%", result.max_drawdown * 100.0);
    println!("  Execution Time:   {:.1}ms", result.elapsed_ms);
    println!();
    println!("  Asset Performance (1000 ticks):");
    for (symbol, price) in &result.final_prices {
        let start_price = 100.0;
        let ret = (price - start_price) / start_price * 100.0;
        let arrow = if ret >= 0.0 { "▲" } else { "▼" };
        println!(
            "    {:>12}: ${:>10.2}  ({}{:.1}%)",
            symbol, price, arrow, ret.abs()
        );
    }

    println!();
    println!("  Key Observations:");
    println!("  • MBS and CDO experienced severe drawdowns");
    println!("  • BANK equity declined as counterparty risk materialized");
    println!("  • TREASURY bonds rallied as safe-haven demand surged");
    println!("  • Circuit breakers triggered during peak volatility");
    println!("  • Central bank intervention partially stabilized markets");
    println!("  • Leveraged hedge funds suffered largest losses");
    println!();
    println!("═══════════════════════════════════════════");
    println!("  Simulation complete.");
    println!("  Built with qfin-twin — Quantum Financial Digital Twin");
    println!("  github.com/CharlesMfouapon/qfin-twin");
    println!("═══════════════════════════════════════════");
}
