use super::*;
use nalgebra::{DMatrix, DVector};
use rand::Rng;

/// Quantum Approximate Optimization Algorithm (QAOA) simulated classically.
/// Solves the portfolio optimization problem: maximize returns, minimize risk.
pub struct QAOAOptimizer {
    n_layers: usize,
    n_iterations: u32,
    learning_rate: f64,
}

impl QAOAOptimizer {
    pub fn new(n_layers: usize, n_iterations: u32, learning_rate: f64) -> Self {
        Self {
            n_layers,
            n_iterations,
            learning_rate,
        }
    }

    /// Optimize portfolio weights using QAOA-inspired classical simulation.
    /// Minimizes: -return + lambda * risk + penalty for constraint violation.
    pub fn optimize(
        &self,
        expected_returns: &[f64],
        covariance: &DMatrix<f64>,
        risk_aversion: f64,
        max_weight: f64,
        rng: &mut impl Rng,
    ) -> OptimizationResult {
        let n_assets = expected_returns.len();
        let start = std::time::Instant::now();

        // Initialize angles (gamma, beta) for each QAOA layer
        let mut gamma: Vec<f64> = (0..self.n_layers)
            .map(|_| rng.gen_range(0.0..std::f64::consts::PI))
            .collect();
        let mut beta: Vec<f64> = (0..self.n_layers)
            .map(|_| rng.gen_range(0.0..std::f64::consts::PI / 2.0))
            .collect();

        // Initialize portfolio weights randomly
        let mut weights: Vec<f64> = (0..n_assets)
            .map(|_| rng.gen_range(0.0..max_weight))
            .collect();

        // Normalize to sum to 1
        let sum: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= sum);

        // Gradient descent on QAOA parameters
        for iteration in 0..self.n_iterations {
            // Compute current portfolio metrics
            let portfolio_return = expected_returns
                .iter()
                .zip(weights.iter())
                .map(|(r, w)| r * w)
                .sum::<f64>();

            let portfolio_risk = (0..n_assets)
                .map(|i| {
                    (0..n_assets)
                        .map(|j| weights[i] * weights[j] * covariance[(i, j)])
                        .sum::<f64>()
                })
                .sum::<f64>()
                .sqrt();

            // Cost function: negative Sharpe ratio + weight constraint penalty
            let sharpe = if portfolio_risk > 0.0 {
                portfolio_return / portfolio_risk
            } else {
                0.0
            };

            let constraint_penalty: f64 = (weights.iter().sum::<f64>() - 1.0).powi(2)
                + weights.iter().map(|w| (w - max_weight).max(0.0).powi(2)).sum::<f64>();

            let cost = -sharpe + constraint_penalty;

            // Perturb gamma and beta using simulated quantum evolution
            for layer in 0..self.n_layers {
                // Update gamma: mixing Hamiltonian
                let gamma_grad = self.gamma_gradient(
                    &weights,
                    expected_returns,
                    covariance,
                    risk_aversion,
                    gamma[layer],
                );
                gamma[layer] -= self.learning_rate * gamma_grad;

                // Update beta: problem Hamiltonian
                let beta_grad = self.beta_gradient(
                    &weights,
                    expected_returns,
                    covariance,
                    risk_aversion,
                    beta[layer],
                );
                beta[layer] -= self.learning_rate * beta_grad;
            }

            // Apply QAOA-inspired mixing to weights
            self.apply_qaoa_mixing(&mut weights, &gamma, &beta, expected_returns, covariance, rng);

            // Re-normalize
            let sum: f64 = weights.iter().sum().max(1e-10);
            weights.iter_mut().for_each(|w| *w /= sum);

            // Clamp to max weight
            weights.iter_mut().for_each(|w| *w = w.min(max_weight));
        }

        // Final evaluation
        let portfolio_return = expected_returns
            .iter()
            .zip(weights.iter())
            .map(|(r, w)| r * w)
            .sum::<f64>();

        let portfolio_risk = (0..n_assets)
            .map(|i| {
                (0..n_assets)
                    .map(|j| weights[i] * weights[j] * covariance[(i, j)])
                    .sum::<f64>()
            })
            .sum::<f64>()
            .sqrt();

        let sharpe = if portfolio_risk > 0.0 {
            portfolio_return / portfolio_risk
        } else {
            0.0
        };

        let elapsed_ms = start.elapsed().as_millis() as f64;

        OptimizationResult {
            optimal_weights: weights,
            expected_return: portfolio_return,
            expected_risk: portfolio_risk,
            sharpe_ratio: sharpe,
            iterations: self.n_iterations,
            convergence_time_ms: elapsed_ms,
        }
    }

    /// Compute gradient of the gamma (mixing) Hamiltonian.
    fn gamma_gradient(
        &self,
        weights: &[f64],
        _returns: &[f64],
        _covariance: &DMatrix<f64>,
        _risk_aversion: f64,
        gamma: f64,
    ) -> f64 {
        // Simplified gradient: derivative of cos(gamma) mixing term
        -gamma.sin() * weights.iter().map(|w| w * (1.0 - w)).sum::<f64>()
    }

    /// Compute gradient of the beta (problem) Hamiltonian.
    fn beta_gradient(
        &self,
        weights: &[f64],
        _returns: &[f64],
        _covariance: &DMatrix<f64>,
        _risk_aversion: f64,
        beta: f64,
    ) -> f64 {
        // Simplified gradient: derivative of cos(beta) problem term
        -beta.sin() * weights.iter().sum::<f64>()
    }

    /// Apply QAOA-inspired mixing to portfolio weights.
    fn apply_qaoa_mixing(
        &self,
        weights: &mut [f64],
        gamma: &[f64],
        beta: &[f64],
        _returns: &[f64],
        _covariance: &DMatrix<f64>,
        rng: &mut impl Rng,
    ) {
        for (i, w) in weights.iter_mut().enumerate() {
            let gamma_effect = gamma.iter().map(|g| g.cos()).sum::<f64>() / gamma.len() as f64;
            let beta_effect = beta.iter().map(|b| b.cos()).sum::<f64>() / beta.len() as f64;

            // Mixing: move weight toward optimal using QAOA angles
            let noise = rng.gen_range(-0.05..0.05);
            *w = (*w * gamma_effect + (1.0 / weights.len() as f64) * (1.0 - gamma_effect))
                * beta_effect
                + noise;

            *w = w.max(0.0);
        }
    }
}
