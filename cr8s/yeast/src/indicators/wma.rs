use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct WMA {
    pub period: usize,
}

impl TechnicalIndicator for WMA {
    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        let period = self.period;

        // sum of weights for normalization: 1 + 2 + ... + period = period * (period + 1) / 2
        let weight_sum = (period * (period + 1) / 2) as f64;

        for i in 0..candles.len() {
            if i + 1 < period {
                result.push(None);
            } else {
                let window = &candles[i + 1 - period..=i];
                let weighted_sum: f64 = window
                    .iter()
                    .enumerate()
                    .map(|(idx, candle)| candle.close * (idx as f64 + 1.0))
                    .sum();

                result.push(Some(weighted_sum / weight_sum));
            }
        }

        result
    }

    fn name(&self) -> &'static str {
        "Weighted Moving Average (WMA)"
    }
}
