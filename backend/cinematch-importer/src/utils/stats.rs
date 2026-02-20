/// Welford's online algorithm for computing mean and standard deviation
/// in a single pass without storing all values.
pub struct WelfordStats {
    n: u64,
    mean: f64,
    m2: f64,
}

impl WelfordStats {
    pub fn new() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.n += 1;
        let delta = value - self.mean;
        self.mean += delta / self.n as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn finalize(&self) -> (f64, f64) {
        if self.n < 2 {
            return (self.mean, 1.0); // avoid division by zero, use 1.0 as fallback std dev
        }
        let variance = self.m2 / self.n as f64;
        (self.mean, variance.sqrt())
    }

    pub fn count(&self) -> u64 {
        self.n
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }
}

impl Default for WelfordStats {
    fn default() -> Self {
        Self::new()
    }
}
