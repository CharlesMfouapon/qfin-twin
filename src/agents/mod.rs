use crate::config::AgentDeployment;

/// Create agent instances from a deployment configuration.
pub fn create_agents(deployment: &AgentDeployment) -> Vec<Box<dyn Agent>> {
    let mut agents: Vec<Box<dyn Agent>> = Vec::with_capacity(deployment.count as usize);

    for _ in 0..deployment.count {
        let agent: Box<dyn Agent> = match deployment.agent_type {
            AgentType::MarketMaker => {
                Box::new(market_maker::MarketMaker::new(deployment.capital, &deployment.params))
            }
            AgentType::Momentum => {
                Box::new(momentum::MomentumTrader::new(deployment.capital, &deployment.params))
            }
            AgentType::Fundamental => {
                Box::new(fundamental::FundamentalInvestor::new(deployment.capital, &deployment.params))
            }
            AgentType::Noise => {
                Box::new(noise::NoiseTrader::new(deployment.capital, &deployment.params))
            }
            AgentType::HedgeFund => {
                Box::new(hedge_fund::HedgeFund::new(deployment.capital, &deployment.params))
            }
            AgentType::CentralBank => {
                Box::new(central_bank::CentralBank::new(deployment.capital, &deployment.params))
            }
        };
        agents.push(agent);
    }

    agents
}
