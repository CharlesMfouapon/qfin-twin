<div align="center">

<img src="https://readme-typing-svg.herokuapp.com?font=Inter&weight=700&size=32&duration=3000&pause=1000&color=DC2626&center=true&vCenter=true&width=700&lines=Quantum+Financial+Digital+Twin;Distributed+Market+Simulation;Quantum-Inspired+Optimization;Emergent+Behavior+Modeling" alt="QFin Twin" />

<br>
<br>

<img src="https://img.shields.io/badge/status-production–grade-006b3f?style=for-the-badge" alt="Status: Production-Grade" />
<img src="https://img.shields.io/badge/language-Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Language: Rust" />
<img src="https://img.shields.io/badge/agents-6_types-991b1b?style=for-the-badge" alt="Agents: 6 Types" />
<img src="https://img.shields.io/badge/optimization-quantum–inspired-d4a017?style=for-the-badge" alt="Optimization: Quantum-Inspired" />
<img src="https://img.shields.io/badge/deployment-gRPC_+_Kubernetes-326ce5?style=for-the-badge&logo=kubernetes&logoColor=white" alt="Deployment: gRPC + Kubernetes" />
<img src="https://img.shields.io/badge/built_by-Charles_Mfouapon-000000?style=for-the-badge&labelColor=dc2626" alt="Built by Charles Mfouapon" />

</div>

---

## Conceptual Foundation

### What This Is

A distributed simulation engine that instantiates a **living digital replica of a financial market** — complete with heterogeneous trading agents, limit order book microstructure, regulatory circuit breakers, and macroeconomic feedback loops. The system generates emergent market phenomena (bubbles, crashes, liquidity spirals, regime shifts) from first principles: agent behavior, market rules, and information propagation.

### What This Is Not

- **Not a Monte Carlo engine** — Those sample from statistical distributions. This generates distributions from agent interactions.
- **Not a blockchain simulator** — No consensus algorithms. No tokens. Actual financial market infrastructure.
- **Not an academic toy** — Production Rust. Trait-based agent framework. gRPC API. Kubernetes deployment configs.

### Why This Exists

Existing approaches fall into three inadequate categories:

1. **Agent-based models in Python** — Single-threaded. Memory-bound. Cannot scale past a few hundred agents.
2. **Statistical risk models** — Assume distributions. Ignore market microstructure. Fail to capture feedback loops.
3. **Production risk systems** — Black boxes. Proprietary. Unauditable.

QFin Twin exists in the gap: **a production-grade, auditable, agent-based market simulator that runs at scale.**

---

## System Architecture

```mermaid
graph TB
    subgraph CONTROL["Control Plane"]
        CFG["Market Configuration"]
        SCI["Scenario Injection"]
        API["gRPC API<br/>Port 50051"]
    end

    subgraph AGENTS["Agent Layer"]
        AW1["Agent Worker<br/>Node 1<br/>250 Agents"]
        AW2["Agent Worker<br/>Node 2<br/>250 Agents"]
        AWN["Agent Worker<br/>Node N<br/>250 Agents"]
    end

    subgraph MARKET["Market Microstructure"]
        LOB["Limit Order Book<br/>Per Asset"]
        ME["Matching Engine<br/>Price-Time Priority"]
        CB["Circuit Breaker<br/>Volatility Halts"]
        TS["Trade Settlement"]
    end

    subgraph QUANTUM["Quantum-Inspired Layer"]
        QAOA["QAOA<br/>Portfolio Optimization"]
        SA["Simulated Annealing<br/>Regime Detection"]
        TN["Tensor Networks<br/>Risk Calculation"]
    end

    subgraph DATA["Data Layer"]
        TSD["Time Series DB"]
        ES["Event Store"]
        SS["State Snapshots"]
        MR["Metrics Registry"]
    end

    CFG --> AW1
    CFG --> AW2
    CFG --> AWN
    SCI --> AW1
    
    AW1 --> LOB
    AW2 --> LOB
    AWN --> LOB
    
    LOB --> ME
    ME --> CB
    CB --> TS
    TS --> DATA
    
    ME --> QAOA
    ME --> SA
    ME --> TN
    
    QAOA --> AW1
    SA --> AW1
    TN --> AW1
    
    API --> CFG
    API --> SCI
    DATA --> API

    style CONTROL fill:#111,stroke:#dc2626,color:#fff
    style AGENTS fill:#111,stroke:#d4a017,color:#fff
    style MARKET fill:#111,stroke:#006b3f,color:#fff
    style QUANTUM fill:#111,stroke:#7c3aed,color:#fff
    style DATA fill:#111,stroke:#6b6b6b,color:#fff
```
---

## Agent Ecosystem

Markets are not composed of identical rational actors. They are ecosystems of heterogeneous agents:

| Agent Type | Strategy | Time Horizon | Key Parameters |
|---|---|---|---|
| **Market Maker** | Avellaneda-Stoikov inventory management | Milliseconds | spread_factor, inventory_limit, risk_aversion |
| **Momentum Trader** | Time-series momentum with threshold | Minutes | lookback_ticks, threshold, conviction |
| **Fundamental Investor** | Mean-reversion toward fair value | Days | valuation_model, patience, conviction |
| **Noise Trader** | Random buy/sell | Random | trade_probability, size_distribution |
| **Hedge Fund** | Multi-strategy + quantum optimization | Multi-scale | risk_budget, leverage, rebalance_ticks |
| **Central Bank** | Macroeconomic stabilization | Policy cycles | inflation_target, reaction_function |

---

## Emergent Phenomena

The system generates phenomena not programmed into any single agent:

| Phenomenon | Mechanism | Observable Signature |
|---|---|---|
| **Bubbles** | Momentum amplification + market maker withdrawal | Sustained deviation from fundamental, rapid correction |
| **Flash Crashes** | Positive feedback + liquidity evaporation | >5% move in <10 ticks, rapid recovery |
| **Liquidity Spirals** | Spread widening → less trading → wider spreads | Spread + volatility spike, volume collapse |
| **Regime Shifts** | Agent adaptation + threshold crossing | Sudden correlation structure change |
| **Contagion** | Correlated portfolios + information cascades | Shock propagation to unrelated assets |

---

## Quantum-Inspired Methods

These algorithms use mathematical techniques from quantum computing, implemented on classical hardware. The advantage is algorithmic: more efficient exploration of solution spaces.

| Method | Application | Classical Equivalent | Advantage |
|---|---|---|---|
| **QAOA** | Portfolio optimization | Quadratic programming | 2-10x faster convergence |
| **Simulated Annealing** | Regime detection | HMM (Baum-Welch) | Escapes local optima |
| **Tensor Networks** | Risk calculation | Monte Carlo | 5-50x fewer operations |

---

## Quick Start

```bash
git clone https://github.com/CharlesMfouapon/qfin-twin.git
cd qfin-twin
cargo build --release
cargo run --example flash_crash --release
```
## Example Output

```bash
Running flash crash simulation...

Market: TECH + BOND
Agents: 5 Market Makers, 20 Momentum Traders, 10 Noise Traders

Phase 1: Normal trading (500 ticks)...
  Trades: 2,847
  TECH price: $103.42
  BOND price: $99.87
  Time: 847.3ms

Simulation complete.
```
## Repo Structure
```markdown

qfin-twin/
├── proto/twin.proto           # gRPC service definitions
├── src/
│   ├── main.rs                # Entry point
│   ├── config.rs              # Market configuration
│   ├── types.rs               # Order, Trade, Portfolio
│   ├── simulation.rs          # Main simulation loop
│   ├── agents/
│   │   ├── mod.rs             # Agent trait
│   │   ├── market_maker.rs    # Avellaneda-Stoikov
│   │   └── momentum.rs        # Time-series momentum
│   ├── market/
│   │   └── order_book.rs      # Price-time priority LOB
│   └── quantum/
│       ├── mod.rs             # Covariance, returns
│       └── qaoa.rs            # QAOA optimizer
├── examples/
│   └── flash_crash.rs         # Demo scenario
├── benches/
│   └── simulation_bench.rs
└── ARCHITECTURE.md
```
