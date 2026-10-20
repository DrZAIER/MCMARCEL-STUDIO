use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Gamma, Normal};

/// Deterministic, per-draw random stream.
///
/// The stream is derived from the run seed and draw index only, so scenarios
/// inside a draw share the same sampled epidemiological and economic values.
/// This preserves paired comparison while keeping every draw reproducible.
pub struct DrawRng {
    rng: ChaCha8Rng,
}

impl DrawRng {
    pub fn new(seed: u64, draw: u32) -> Self {
        let mixed = splitmix64(seed ^ (draw as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        Self { rng: ChaCha8Rng::seed_from_u64(mixed) }
    }

    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        if !low.is_finite() || !high.is_finite() || high <= low {
            return low;
        }
        self.rng.gen_range(low..=high)
    }

    pub fn gamma(&mut self, mean: f64, sd: f64) -> f64 {
        if !mean.is_finite() || mean <= 0.0 {
            return 0.0;
        }
        if !sd.is_finite() || sd <= 0.0 {
            return mean;
        }
        let (shape, rate) = gamma_parameters(mean, sd);
        match Gamma::new(shape, 1.0 / rate) {
            Ok(dist) => dist.sample(&mut self.rng).max(0.0),
            Err(_) => mean,
        }
    }

    pub fn beta(&mut self, mean: f64, sd: f64) -> f64 {
        let mean = mean.clamp(1e-9, 1.0 - 1e-9);
        if !sd.is_finite() || sd <= 0.0 {
            return mean;
        }
        let variance = sd * sd;
        let alpha = mean * mean * (1.0 - mean) / variance - mean;
        let beta = alpha * (1.0 - mean) / mean;
        if !alpha.is_finite() || !beta.is_finite() || alpha <= 1e-9 || beta <= 1e-9 {
            return mean;
        }
        let x = self.gamma(alpha, 1.0);
        let y = self.gamma(beta, 1.0);
        if x + y <= 0.0 { mean } else { (x / (x + y)).clamp(0.0, 1.0) }
    }

    pub fn normal(&mut self, mean: f64, sd: f64) -> f64 {
        if !sd.is_finite() || sd <= 0.0 {
            return mean;
        }
        match Normal::new(mean, sd) {
            Ok(dist) => dist.sample(&mut self.rng),
            Err(_) => mean,
        }
    }

    pub fn index(&mut self, len: usize) -> usize {
        if len <= 1 { 0 } else { self.rng.gen_range(0..len) }
    }

    pub fn chance(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability.clamp(0.0, 1.0))
    }
}

/// Shape and rate of the gamma distribution used for cost and utility sampling.
///
/// Mirrors `sample_rgamma` in the R implementation
/// (`functions/RSV_get_burden.R`): `shape = mean² / sd²` and
/// `rate = mean / sd²` (the R code calls `rgamma(n, shape, rate = 1/beta)`
/// with `beta = sd² / mean`).
pub fn gamma_parameters(mean: f64, sd: f64) -> (f64, f64) {
    let mean = if mean.is_finite() && mean > 0.0 { mean } else { 1.0 };
    let sd = if sd.is_finite() && sd > 0.0 { sd } else { 0.0 };
    if sd == 0.0 {
        return (1.0, 0.0);
    }
    let shape = mean * mean / (sd * sd);
    let rate = mean / (sd * sd);
    (shape, rate)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_are_reproducible() {
        let mut a = DrawRng::new(42, 7);
        let mut b = DrawRng::new(42, 7);
        for _ in 0..20 {
            assert_eq!(a.uniform(0.0, 1.0), b.uniform(0.0, 1.0));
        }
    }

    #[test]
    fn gamma_mean_is_in_the_right_region() {
        let mut rng = DrawRng::new(1, 1);
        let values: Vec<f64> = (0..20_000).map(|_| rng.gamma(10.0, 2.0)).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        assert!((mean - 10.0).abs() < 0.25, "{mean}");
    }

    #[test]
    fn beta_is_bounded() {
        let mut rng = DrawRng::new(1, 2);
        for _ in 0..10_000 {
            let value = rng.beta(0.8, 0.1);
            assert!((0.0..=1.0).contains(&value));
        }
    }
}
