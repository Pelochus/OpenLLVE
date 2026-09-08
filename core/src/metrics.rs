/// Benchmark metrics collection for low-light video enhancement inference.
#[derive(Clone)]
pub struct BenchmarkMetrics {
    latencies_ms: Vec<f32>,
}

impl BenchmarkMetrics {
    pub fn new() -> Self {
        Self {
            latencies_ms: Vec::new(),
        }
    }

    pub fn record(&mut self, latency_ms: f32) {
        self.latencies_ms.push(latency_ms);
    }

    pub fn average_latency(&self) -> f32 {
        if self.latencies_ms.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.latencies_ms.iter().sum();
        sum / self.latencies_ms.len() as f32
    }

    pub fn fps(&self) -> f32 {
        let avg = self.average_latency();
        if avg == 0.0 {
            return 0.0;
        }
        1000.0 / avg
    }

    pub fn percentile_p99(&self) -> f32 {
        if self.latencies_ms.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((99.0 / 100.0) * sorted.len() as f32) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    pub fn count(&self) -> usize {
        self.latencies_ms.len()
    }

    pub fn reset(&mut self) {
        self.latencies_ms.clear();
    }
}

impl Default for BenchmarkMetrics {
    fn default() -> Self {
        Self::new()
    }
}
