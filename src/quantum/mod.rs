pub mod qaoa;
pub mod annealing;
pub mod tensor_networks;

use nalgebra::{DMatrix, DVector};

/// Result of a quantum-inspired optimization.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub optimal_weights: Vec<f64>,
    pub expected_return: f64,
    pub expected_risk: f64,
    pub sharpe_ratio: f64,
    pub iterations: u32,
    pub convergence_time_ms: f64,
}

/// Covariance matrix computed from historical returns.
pub fn compute_covariance_matrix(returns: &[Vec<f64>]) -> DMatrix<f64> {
    let n_assets = returns.len();
    if n_assets == 0 {
        return DMatrix::zeros(0, 
