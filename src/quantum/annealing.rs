use rand::Rng;
use std::f64::consts::E;

/// Simulated quantum annealing for market regime detection.
/// Uses a temperature schedule inspired by quantum tunneling to escape local optima.
pub struct QuantumAnnealer {
    initial_temperature: f64,
    cooling_rate: f64,
    n_iterations: u32,
    n_regimes: usize,
}

impl QuantumAnnealer {
    pub fn new(n_regimes: usize) -> Self {
        Self {
            initial_temperature: 10.0,
            cooling_rate: 0.95,
            n_iterations: 1000,
            n_regimes,
        }
    }

    /// Detect market regimes from a time series of returns.
    /// Returns regime assignments for each observation and the transition matrix.
    pub fn detect_regimes(&self, returns: &[f64], rng: &mut impl Rng) -> RegimeResult {
        let n = returns.len();
        if n < 10 {
            return RegimeResult {
                regimes: vec![0; n],
                transition_matrix: vec![vec![0.0; self.n_regimes]; self.n_regimes],
                regime_means: vec![0.0; self.n_regimes],
                regime_variances: vec![0.0; self.n_regimes],
                energy: 0.0,
            };
        }

        // Initialize random regime assignments
        let mut regimes: Vec<usize> = (0..n)
            .map(|_| rng.gen_range(0..self.n_regimes))
            .collect();

        let mut temperature = self.initial_temperature;
        let mut current_energy = self.compute_energy(&regimes, returns);
        let mut best_regimes = regimes.clone();
        let mut best_energy = current_energy;

        for _ in 0..self.n_iterations {
            // Propose a change: flip one observation's regime
            let idx = rng.gen_range(0..n);
            let old_regime = regimes[idx];
            let new_regime = rng.gen_range(0..self.n_regimes);
            regimes[idx] = new_regime;

            let new_energy = self.compute_energy(&regimes, returns);
            let delta = new_energy - current_energy;

            // Quantum-inspired acceptance: uses tunneling probability
            let acceptance = if delta < 0.0 {
                1.0
            } else {
                // Quantum tunneling factor: e^(-delta / (T * h))
                // where h simulates the reduced Planck constant effect
                let h_bar = 0.1; // Tunneling amplitude
                (-delta / (temperature * h_bar)).exp()
            };

            if rng.gen::<f64>() < acceptance {
                current_energy = new_energy;
                if current_energy < best_energy {
                    best_energy = current_energy;
                    best_regimes = regimes.clone();
                }
            } else {
                // Reject: revert
                regimes[idx] = old_regime;
            }

            // Cool the system (but with quantum fluctuations)
            temperature *= self.cooling_rate;
            if temperature < 0.001 {
                // Reheat: quantum fluctuation
                temperature = self.initial_temperature * rng.gen_range(0.1..0.3);
            }
        }

        // Compute regime statistics
        let (means, variances) = self.compute_regime_stats(&best_regimes, returns);
        let transition = self.compute_transition_matrix(&best_regimes);

        RegimeResult {
            regimes: best_regimes,
            transition_matrix: transition,
            regime_means: means,
            regime_variances: variances,
            energy: best_energy,
        }
    }

    /// Energy function: within-regime variance + transition penalty.
    fn compute_energy(&self, regimes: &[usize], returns: &[f64]) -> f64 {
        let (means, variances) = self.compute_regime_stats(regimes, returns);
        let mut energy = 0.0;

        // Within-regime variance
        for (i, &variance) in variances.iter().enumerate() {
            energy += variance * 10.0;
        }

        // Transition penalty: penalize frequent regime switches
        for window in regimes.windows(2) {
            if window[0] != window[1] {
                energy += 0.5;
            }
        }

        energy
    }

    fn compute_regime_stats(&self, regimes: &[usize], returns: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let mut sums = vec![0.0; self.n_regimes];
        let mut counts = vec![0usize; self.n_regimes];

        for (&r, &ret) in regimes.iter().zip(returns.iter()) {
            sums[r] += ret;
            counts[r] += 1;
        }

        let means: Vec<f64> = sums
            .iter()
            .zip(counts.iter())
            .map(|(&s, &c)| if c > 0 { s / c as f64 } else { 0.0 })
            .collect();

        let mut sq_diffs = vec![0.0; self.n_regimes];
        for (&r, &ret) in regimes.iter().zip(returns.iter()) {
            let diff = ret - means[r];
            sq_diffs[r] += diff * diff;
        }

        let variances: Vec<f64> = sq_diffs
            .iter()
            .zip(counts.iter())
            .map(|(&sd, &c)| if c > 1 { sd / (c - 1) as f64 } else { 0.0 })
            .collect();

        (means, variances)
    }

    fn compute_transition_matrix(&self, regimes: &[usize]) -> Vec<Vec<f64>> {
        let mut counts = vec![vec![0usize; self.n_regimes]; self.n_regimes];

        for window in regimes.windows(2) {
            counts[window[0]][window[1]] += 1;
        }

        counts
            .iter()
            .map(|row| {
                let total: usize = row.iter().sum();
                if total > 0 {
                    row.iter().map(|&c| c as f64 / total as f64).collect()
                } else {
                    vec![1.0 / self.n_regimes as f64; self.n_regimes]
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct RegimeResult {
    pub regimes: Vec<usize>,
    pub transition_matrix: Vec<Vec<f64>>,
    pub regime_means: Vec<f64>,
    pub regime_variances: Vec<f64>,
    pub energy: f64,
}
