# Architecture: Quantum Financial Digital Twin

## System Overview

QFin Twin is a distributed simulation engine that creates a living digital replica of a financial market. Thousands of heterogeneous agents trade across multiple assets through limit order books, generating emergent market behavior — bubbles, crashes, regime shifts, and contagion.

## Design Principles

1. **Performance** — Rust zero-cost abstractions. Agents process in parallel. Order books use BTreeMap for O(log n) operations.
2. **Correctness** — Price-time priority matching. Circuit breakers with configurable thresholds. Portfolio tracking with average price accounting.
3. **Extensibility** — Agent trait allows plugging in new agent types. Quantum optimization layer is optional and configurable.
4. **Observability** — Every trade, price, and portfolio is tracked. Simulation results include P&L, drawdown, and execution time.

## Component Architecture

### Agent Framework
- **Market Maker** — Avellaneda-Stoikov inventory management with volatility estimation
- **Momentum Trader** — Lookback-based signal with configurable threshold and conviction
- **Fundamental Investor** — Mean-reversion toward perceived fair value
- **Noise Trader** — Random buy/sell for market realism
- **Hedge Fund** — Multi-strategy with quantum-inspired portfolio optimization
- **Central Bank** — Liquidity injection/withdrawal, rate setting

### Market Microstructure
- **Limit Order Book** — Tick-based price levels, FIFO within level
- **Matching Engine** — Market orders match against resting limit orders
- **Circuit Breaker** — Triggers on price moves exceeding threshold, with configurable cooldown

### Quantum-Inspired Layer
- **QAOA** — Portfolio optimization using simulated quantum annealing
- **Simulated Annealing** — Regime detection and asset allocation
- **Tensor Networks** — Risk calculation acceleration (planned)

## Data Flow
```markdown
Agent.act() → Orders → OrderBook → Matching → Trades → Agent.on_trade()
↓
Price Update
↓
Circuit Breaker Check
↓
Quantum Optimization (periodic)
```

## Architectural Decision Records

### ADR-001: Rust over Python
**Decision:** Implement core engine in Rust.
**Rationale:** Agent simulation is CPU-bound. Python's GIL limits parallel agent processing. Rust's zero-cost abstractions and rayon parallelism enable thousands of agents per tick without garbage collection pauses.

### ADR-002: Tick-based discrete time
**Decision:** Discrete tick-based simulation rather than continuous time.
**Rationale:** Enables deterministic replay. Simplifies agent synchronization. Ticks can represent milliseconds or days depending on configuration.

### ADR-003: gRPC for distributed deployment
**Decision:** Use gRPC for communication between simulation nodes.
**Rationale:** Enables distributing agent computation across Kubernetes pods. Streaming market data to external consumers. Industry standard for financial infrastructure.
