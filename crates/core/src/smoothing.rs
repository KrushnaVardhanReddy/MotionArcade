use crate::pose::Keypoint;

/// Configurable adaptive low-pass filter for noisy landmark streams.
///
/// Implements the 1€ Filter (Casiez, Roussel, Vogel, CHI 2012).
/// It provides low jitter at low speeds (high smoothing) while minimizing
/// latency during fast movements (adaptive cutoff).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OneEuroFilterConfig {
    /// Minimum cutoff frequency in Hz (higher = more jitter, less lag). Default: 1.0
    pub min_cutoff: f32,
    /// Speed coefficient parameter (higher = faster adaptation to high-speed movement). Default: 0.007
    pub beta: f32,
    /// Derivative cutoff frequency in Hz. Default: 1.0
    pub d_cutoff: f32,
}

impl Default for OneEuroFilterConfig {
    fn default() -> Self {
        Self {
            min_cutoff: 1.0,
            beta: 0.007,
            d_cutoff: 1.0,
        }
    }
}

/// Calculate smoothing factor alpha from sample interval dt and cutoff frequency.
/// alpha = r / (1.0 + r) where r = 2 * PI * cutoff * dt
#[inline]
fn smoothing_factor(dt: f32, cutoff: f32) -> f32 {
    let r = 2.0 * std::f32::consts::PI * cutoff.max(0.0) * dt.max(0.0);
    (r / (1.0 + r)).clamp(0.0, 1.0)
}

/// 1D One Euro Filter state.
#[derive(Debug, Clone)]
pub struct OneEuroFilter {
    config: OneEuroFilterConfig,
    x_prev: Option<f32>,
    dx_prev: Option<f32>,
    t_prev: Option<f64>,
}

impl OneEuroFilter {
    /// Creates a new filter with the given configuration.
    pub fn new(config: OneEuroFilterConfig) -> Self {
        Self {
            config,
            x_prev: None,
            dx_prev: None,
            t_prev: None,
        }
    }

    /// Resets the internal filter state.
    pub fn reset(&mut self) {
        self.x_prev = None;
        self.dx_prev = None;
        self.t_prev = None;
    }

    /// Filters a scalar measurement at the given monotonic timestamp (in seconds).
    pub fn filter(&mut self, value: f32, timestamp: f64) -> f32 {
        let (prev_x, prev_t) = match (self.x_prev, self.t_prev) {
            (Some(x), Some(t)) => (x, t),
            _ => {
                // First sample initialization
                self.x_prev = Some(value);
                self.dx_prev = Some(0.0);
                self.t_prev = Some(timestamp);
                return value;
            }
        };

        let dt = (timestamp - prev_t) as f32;
        if dt <= 1e-6 {
            // Non-positive or negligible dt; return previous filtered value
            return prev_x;
        }

        // 1. Calculate raw rate of change
        let dx = (value - prev_x) / dt;

        // 2. Filter rate of change with fixed derivative cutoff
        let alpha_d = smoothing_factor(dt, self.config.d_cutoff);
        let prev_dx = self.dx_prev.unwrap_or(0.0);
        let dx_hat = alpha_d * dx + (1.0 - alpha_d) * prev_dx;

        // 3. Adapt cutoff frequency based on filtered velocity
        let cutoff = self.config.min_cutoff + self.config.beta * dx_hat.abs();

        // 4. Filter value with adaptive cutoff
        let alpha_x = smoothing_factor(dt, cutoff);
        let x_hat = alpha_x * value + (1.0 - alpha_x) * prev_x;

        // 5. Update state
        self.x_prev = Some(x_hat);
        self.dx_prev = Some(dx_hat);
        self.t_prev = Some(timestamp);

        x_hat
    }
}

/// 2D One Euro Filter for smoothing normalized (x, y) coordinates.
#[derive(Debug, Clone)]
pub struct OneEuroFilter2D {
    pub x_filter: OneEuroFilter,
    pub y_filter: OneEuroFilter,
}

impl OneEuroFilter2D {
    pub fn new(config: OneEuroFilterConfig) -> Self {
        Self {
            x_filter: OneEuroFilter::new(config),
            y_filter: OneEuroFilter::new(config),
        }
    }

    pub fn filter(&mut self, x: f32, y: f32, timestamp: f64) -> (f32, f32) {
        (
            self.x_filter.filter(x, timestamp),
            self.y_filter.filter(y, timestamp),
        )
    }

    pub fn filter_keypoint(&mut self, keypoint: &Keypoint, timestamp: f64) -> Keypoint {
        let (x, y) = self.filter(keypoint.x, keypoint.y, timestamp);
        Keypoint {
            x,
            y,
            confidence: keypoint.confidence,
        }
    }

    pub fn reset(&mut self) {
        self.x_filter.reset();
        self.y_filter.reset();
    }
}

/// Full 17-keypoint skeleton smoothing filter.
#[derive(Debug, Clone)]
pub struct SkeletonSmoother {
    keypoint_filters: Vec<OneEuroFilter2D>,
}

impl SkeletonSmoother {
    pub fn new(config: OneEuroFilterConfig) -> Self {
        let mut filters = Vec::with_capacity(17);
        for _ in 0..17 {
            filters.push(OneEuroFilter2D::new(config));
        }
        Self {
            keypoint_filters: filters,
        }
    }

    pub fn filter_skeleton(
        &mut self,
        skeleton: &crate::pose::Skeleton,
        timestamp: f64,
    ) -> crate::pose::Skeleton {
        let mut smoothed_kps = [Keypoint::ZERO; 17];
        for (i, smoothed_kp) in smoothed_kps.iter_mut().enumerate() {
            if let Some(filter) = self.keypoint_filters.get_mut(i) {
                *smoothed_kp = filter.filter_keypoint(&skeleton.keypoints[i], timestamp);
            }
        }

        crate::pose::Skeleton::new(skeleton.id, smoothed_kps)
    }

    pub fn reset(&mut self) {
        for f in &mut self.keypoint_filters {
            f.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple deterministic pseudo-random number generator for tests
    struct Lcg {
        state: u64,
    }

    impl Lcg {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }

        fn next_f32(&mut self) -> f32 {
            self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((self.state >> 33) & 0x7FFFFFFF) as f32;
            (val / 0x7FFFFFFF as f32) * 2.0 - 1.0 // Range [-1.0, 1.0]
        }
    }

    #[test]
    fn test_one_euro_filter_initialization() {
        let mut filter = OneEuroFilter::new(OneEuroFilterConfig::default());
        let first = filter.filter(0.5, 0.0);
        assert_eq!(first, 0.5);
    }

    #[test]
    fn test_one_euro_filter_noise_reduction_static() {
        let config = OneEuroFilterConfig {
            min_cutoff: 0.5,
            beta: 0.001,
            d_cutoff: 1.0,
        };
        let mut filter = OneEuroFilter::new(config);
        let mut rng = Lcg::new(42);

        let ground_truth = 0.5f32;
        let mut noisy_step_diff_sum = 0.0f32;
        let mut filtered_step_diff_sum = 0.0f32;

        let mut prev_noisy = ground_truth;
        let mut prev_filtered = ground_truth;

        let num_samples = 300;
        let dt = 1.0 / 60.0;

        for i in 0..num_samples {
            let t = i as f64 * dt;
            let noise = rng.next_f32() * 0.05; // 5% jitter
            let noisy = ground_truth + noise;
            let filtered = filter.filter(noisy, t);

            if i > 10 {
                noisy_step_diff_sum += (noisy - prev_noisy).abs();
                filtered_step_diff_sum += (filtered - prev_filtered).abs();
            }

            prev_noisy = noisy;
            prev_filtered = filtered;
        }

        // Calculate jitter reduction percentage
        let jitter_reduction = 1.0 - (filtered_step_diff_sum / noisy_step_diff_sum);
        // Expect > 60% jitter reduction on static signal with noise
        assert!(
            jitter_reduction > 0.60,
            "Expected >60% jitter reduction, got {:.2}%",
            jitter_reduction * 100.0
        );
    }

    #[test]
    fn test_one_euro_filter_reset() {
        let mut filter = OneEuroFilter::new(OneEuroFilterConfig::default());
        filter.filter(0.5, 1.0);
        filter.filter(0.6, 1.1);
        filter.reset();

        // After reset, first sample should be returned directly
        let out = filter.filter(0.9, 5.0);
        assert_eq!(out, 0.9);
    }
}
