// src/indicators/momentum.rs

use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct Momentum {
    pub period: usize,
}

impl TechnicalIndicator for Momentum {
    fn name(&self) -> &'static str {
        "Momentum"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut momentum = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            if i < self.period {
                momentum.push(None);
                continue;
            }
            let value = candles[i].close - candles[i - self.period].close;
            momentum.push(Some(value));
        }

        momentum
    }
}
