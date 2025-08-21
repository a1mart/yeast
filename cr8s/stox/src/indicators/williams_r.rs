// src/indicators/williams_r.rs

use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct WilliamsR {
    pub period: usize,
}

impl TechnicalIndicator for WilliamsR {
    fn name(&self) -> &'static str {
        "Williams%R"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut wr = Vec::with_capacity(candles.len());
        let period = self.period;

        for i in 0..candles.len() {
            if i + 1 < period {
                wr.push(None);
                continue;
            }

            let window = &candles[i + 1 - period..=i];
            let highest_high = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let lowest_low = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);

            if (highest_high - lowest_low).abs() < std::f64::EPSILON {
                wr.push(None);
            } else {
                let value = (highest_high - candles[i].close) / (highest_high - lowest_low) * -100.0;
                wr.push(Some(value));
            }
        }

        wr
    }
}
