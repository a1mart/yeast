use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct SchaffTrendCycle {
    pub short_period: usize,
    pub long_period: usize,
    pub cycle_period: usize,
    pub fast_k: usize,
    pub fast_d: usize,
}

impl TechnicalIndicator for SchaffTrendCycle {
    fn name(&self) -> &'static str {
        "Schaff Trend Cycle"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Reference: Combines MACD with stochastic oscillator for faster signals

        // 1. Calculate MACD line
        fn ema(period: usize, prices: &[f64]) -> Vec<Option<f64>> {
            let mut res = vec![None; prices.len()];
            let k = 2.0 / (period as f64 + 1.0);
            let mut prev = 0.0;
            for (i, &price) in prices.iter().enumerate() {
                if i < period - 1 {
                    res[i] = None;
                } else if i == period - 1 {
                    let sum: f64 = prices[i + 1 - period..=i].iter().sum();
                    prev = sum / period as f64;
                    res[i] = Some(prev);
                } else {
                    prev = price * k + prev * (1.0 - k);
                    res[i] = Some(prev);
                }
            }
            res
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let ema_short = ema(self.short_period, &closes);
        let ema_long = ema(self.long_period, &closes);

        let macd: Vec<Option<f64>> = ema_short.iter()
            .zip(ema_long.iter())
            .map(|(short, long)| match (short, long) {
                (Some(s), Some(l)) => Some(s - l),
                _ => None,
            })
            .collect();

        // 2. Stochastic calculation on MACD line
        fn stochastic_k(macd: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
            let mut k_vals = vec![None; macd.len()];
            for i in period - 1..macd.len() {
                let window = &macd[i + 1 - period..=i];
                if window.iter().any(|v| v.is_none()) {
                    k_vals[i] = None;
                    continue;
                }
                let values: Vec<f64> = window.iter().map(|v| v.unwrap()).collect();
                let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                if max_val - min_val == 0.0 {
                    k_vals[i] = Some(0.0);
                } else {
                    k_vals[i] = Some(100.0 * (values[period - 1] - min_val) / (max_val - min_val));
                }
            }
            k_vals
        }

        let mut fast_k = stochastic_k(&macd, self.cycle_period);

        // 3. Smooth %K with EMA fast_k times
        for _ in 0..self.fast_k {
            fast_k = {
                let mut smoothed = vec![None; fast_k.len()];
                let k = 2.0 / (self.cycle_period as f64 + 1.0);
                let mut prev = 0.0;
                for (i, val) in fast_k.iter().enumerate() {
                    if i < self.cycle_period - 1 || val.is_none() {
                        smoothed[i] = None;
                    } else if i == self.cycle_period - 1 {
                        let sum: f64 = fast_k[i + 1 - self.cycle_period..=i].iter().filter_map(|x| *x).sum();
                        prev = sum / self.cycle_period as f64;
                        smoothed[i] = Some(prev);
                    } else if let Some(v) = val {
                        prev = v * k + prev * (1.0 - k);
                        smoothed[i] = Some(prev);
                    }
                }
                smoothed
            };
        }

        // 4. Calculate %D as EMA of %K
        let mut fast_d = fast_k.clone();
        for _ in 0..self.fast_d {
            fast_d = {
                let mut smoothed = vec![None; fast_d.len()];
                let k = 2.0 / (self.cycle_period as f64 + 1.0);
                let mut prev = 0.0;
                for (i, val) in fast_d.iter().enumerate() {
                    if i < self.cycle_period - 1 || val.is_none() {
                        smoothed[i] = None;
                    } else if i == self.cycle_period - 1 {
                        let sum: f64 = fast_d[i + 1 - self.cycle_period..=i].iter().filter_map(|x| *x).sum();
                        prev = sum / self.cycle_period as f64;
                        smoothed[i] = Some(prev);
                    } else if let Some(v) = val {
                        prev = v * k + prev * (1.0 - k);
                        smoothed[i] = Some(prev);
                    }
                }
                smoothed
            };
        }

        fast_d
    }
}
