/// Benchmark metrics collection for low-light video enhancement inference.
///
/// Latencies are accumulated in `f64` for precision over long benchmark runs.
/// Warm-up frames can be excluded from all statistics via
/// [`BenchmarkMetrics::with_warmup`]: the first `n` recorded samples are kept
/// separately and never included in average/median/p99/FPS (see
/// `docs/BENCHMARK_METHODOLOGY.md`).
#[derive(Clone)]
pub struct BenchmarkMetrics {
    latencies_ms: Vec<f64>,
    warmup_latencies_ms: Vec<f64>,
    warmup_budget: usize,
    warmup_remaining: usize,
}

impl BenchmarkMetrics {
    pub fn new() -> Self {
        Self {
            latencies_ms: Vec::new(),
            warmup_latencies_ms: Vec::new(),
            warmup_budget: 0,
            warmup_remaining: 0,
        }
    }

    /// Creates a metrics handle that excludes the first `warmup_frames`
    /// recorded samples from all statistics (warm-up exclusion).
    ///
    /// The budget is restored on [`BenchmarkMetrics::reset`], so a handle can
    /// be reused for repeat benchmark runs with a fresh warm-up each time.
    pub fn with_warmup(warmup_frames: usize) -> Self {
        Self {
            latencies_ms: Vec::new(),
            warmup_latencies_ms: Vec::new(),
            warmup_budget: warmup_frames,
            warmup_remaining: warmup_frames,
        }
    }

    /// Records a latency sample in milliseconds. Samples recorded while the
    /// warm-up budget is not exhausted are excluded from the statistics.
    pub fn record(&mut self, latency_ms: f64) {
        if self.warmup_remaining > 0 {
            self.warmup_remaining -= 1;
            self.warmup_latencies_ms.push(latency_ms);
        } else {
            self.latencies_ms.push(latency_ms);
        }
    }

    /// Average latency in milliseconds over non-warm-up samples.
    pub fn average_latency(&self) -> f64 {
        if self.latencies_ms.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.latencies_ms.iter().sum();
        sum / self.latencies_ms.len() as f64
    }

    /// Median latency in milliseconds over non-warm-up samples.
    pub fn median_latency(&self) -> f64 {
        let n = self.latencies_ms.len();
        if n == 0 {
            return 0.0;
        }
        let mut sorted = self.latencies_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if n % 2 == 1 {
            sorted[n / 2]
        } else {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        }
    }

    /// 99th percentile latency in milliseconds over non-warm-up samples.
    pub fn percentile_p99(&self) -> f64 {
        if self.latencies_ms.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((99.0 / 100.0) * sorted.len() as f64) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Frames per second derived from the average non-warm-up latency.
    pub fn fps(&self) -> f64 {
        let avg = self.average_latency();
        if avg == 0.0 {
            return 0.0;
        }
        1000.0 / avg
    }

    /// Number of non-warm-up samples recorded.
    pub fn count(&self) -> usize {
        self.latencies_ms.len()
    }

    /// Number of warm-up samples recorded (excluded from statistics).
    pub fn warmup_count(&self) -> usize {
        self.warmup_latencies_ms.len()
    }

    /// Clears all samples and restores the warm-up budget, so the next run
    /// gets a fresh warm-up.
    pub fn reset(&mut self) {
        self.latencies_ms.clear();
        self.warmup_latencies_ms.clear();
        self.warmup_remaining = self.warmup_budget;
    }
}

impl Default for BenchmarkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_metrics() {
        let m = BenchmarkMetrics::new();
        assert_eq!(m.average_latency(), 0.0);
        assert_eq!(m.median_latency(), 0.0);
        assert_eq!(m.percentile_p99(), 0.0);
        assert_eq!(m.fps(), 0.0);
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_single_sample() {
        let mut m = BenchmarkMetrics::new();
        m.record(10.0);
        assert_eq!(m.average_latency(), 10.0);
        assert_eq!(m.median_latency(), 10.0);
        assert_eq!(m.percentile_p99(), 10.0);
        assert_eq!(m.fps(), 100.0);
        assert_eq!(m.count(), 1);
    }

    #[test]
    fn test_median_odd_and_even() {
        let mut m = BenchmarkMetrics::new();
        for v in [1.0, 3.0, 2.0] {
            m.record(v);
        }
        assert_eq!(m.median_latency(), 2.0);

        let mut m2 = BenchmarkMetrics::new();
        for v in [4.0, 1.0, 3.0, 2.0] {
            m2.record(v);
        }
        assert_eq!(m2.median_latency(), 2.5);
    }

    #[test]
    fn test_p99_with_ties() {
        let mut m = BenchmarkMetrics::new();
        for _ in 0..100 {
            m.record(5.0);
        }
        m.record(100.0);
        // 101 samples: idx = floor(99.99) = 99 -> sorted[99] = 5.0
        assert_eq!(m.percentile_p99(), 5.0);
    }

    #[test]
    fn test_warmup_exclusion() {
        let mut m = BenchmarkMetrics::with_warmup(2);
        m.record(1000.0); // warm-up
        m.record(2000.0); // warm-up
        m.record(10.0);
        m.record(20.0);
        assert_eq!(m.count(), 2);
        assert_eq!(m.warmup_count(), 2);
        assert_eq!(m.average_latency(), 15.0);
        assert_eq!(m.median_latency(), 15.0);
    }

    #[test]
    fn test_reset_clears_warmup_budget() {
        let mut m = BenchmarkMetrics::with_warmup(1);
        m.record(1.0);
        m.reset();
        assert_eq!(m.count(), 0);
        assert_eq!(m.warmup_count(), 0);
        m.record(5.0); // warm-up again
        m.record(10.0);
        assert_eq!(m.count(), 1);
        assert_eq!(m.average_latency(), 10.0);
    }
}
