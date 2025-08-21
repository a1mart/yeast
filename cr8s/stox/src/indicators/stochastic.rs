// src/indicators/stochastic.rs

use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct Stochastic {
    pub k_period: usize,
    pub d_period: usize,
}

impl TechnicalIndicator for Stochastic {
    fn name(&self) -> &'static str {
        "%K"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut percent_k = Vec::with_capacity(candles.len());
        let k_period = self.k_period;

        for i in 0..candles.len() {
            if i + 1 < k_period {
                percent_k.push(None);
                continue;
            }
            let window = &candles[i + 1 - k_period..=i];
            let highest_high = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let lowest_low = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            if (highest_high - lowest_low).abs() < std::f64::EPSILON {
                percent_k.push(None);
            } else {
                let k = (candles[i].close - lowest_low) / (highest_high - lowest_low) * 100.0;
                percent_k.push(Some(k));
            }
        }

        percent_k
    }
}

// You can add a separate %D indicator by taking SMA of %K, or extend this
