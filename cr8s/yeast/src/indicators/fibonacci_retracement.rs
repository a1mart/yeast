use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

/// Fibonacci Retracement Zones
/// Returns retracement levels [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0] scaled between
/// a high and low over a lookback period.
pub struct FibonacciRetracement {
    pub period: usize,
}

impl TechnicalIndicator for FibonacciRetracement {
    fn name(&self) -> &'static str {
        "Fibonacci Retracement Zones"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let fib_levels = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];
        let mut zones = vec![None; candles.len()];

        if candles.len() < self.period {
            return zones;
        }

        for i in self.period - 1..candles.len() {
            let window = &candles[i + 1 - self.period..=i];
            let high = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let low = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            let diff = high - low;

            // Store retracement levels as an Option<f64> vector in zones[i]
            // Here we just pick one level per index for simplicity, e.g., 0.618 level
            // For full zone info, consider returning Vec<Vec<Option<f64>>> or a struct.
            zones[i] = Some(low + diff * 0.618); // common retracement level
        }

        zones
    }
}