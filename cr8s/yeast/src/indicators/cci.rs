// src/indicators/cci.rs

use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct CCI {
    pub period: usize,
}

impl TechnicalIndicator for CCI {
    fn name(&self) -> &'static str {
        "CCI"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut cci = Vec::with_capacity(candles.len());
        let period = self.period;

        for i in 0..candles.len() {
            if i + 1 < period {
                cci.push(None);
                continue;
            }

            let window = &candles[i + 1 - period..=i];

            // Typical Price = (High + Low + Close) / 3
            let typical_prices: Vec<f64> = window.iter()
                .map(|c| (c.high + c.low + c.close) / 3.0)
                .collect();

            let sma_tp = typical_prices.iter().sum::<f64>() / period as f64;

            // Mean deviation
            let mean_dev = typical_prices.iter()
                .map(|tp| (tp - sma_tp).abs())
                .sum::<f64>() / period as f64;

            let current_tp = (candles[i].high + candles[i].low + candles[i].close) / 3.0;

            if mean_dev.abs() < std::f64::EPSILON {
                cci.push(None);
            } else {
                let cci_value = (current_tp - sma_tp) / (0.015 * mean_dev);
                cci.push(Some(cci_value));
            }
        }

        cci
    }
}
