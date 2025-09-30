use crate::indicators::{TechnicalIndicator, ATR};
use crate::types::Candle;

pub struct ChandelierExit {
    pub period: usize,
    pub atr_multiplier: f64,
}

impl TechnicalIndicator for ChandelierExit {
    fn name(&self) -> &'static str {
        "Chandelier Exit"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Instantiate an ATR indicator and compute ATR values on candles
        let atr_indicator = ATR { period: self.period };
        let atr_values = atr_indicator.compute(candles);

        let mut result = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            if i + 1 < self.period || atr_values[i].is_none() {
                result.push(None);
                continue;
            }

            // Find highest high in the lookback period
            let high_since = candles[i + 1 - self.period..=i]
                .iter()
                .map(|c| c.high)
                .fold(f64::MIN, f64::max);

            // Calculate Chandelier Exit value
            result.push(Some(high_since - self.atr_multiplier * atr_values[i].unwrap()));
        }

        result
    }
}
