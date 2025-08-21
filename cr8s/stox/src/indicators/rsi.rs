use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct RSI {
    pub period: usize,
}

impl TechnicalIndicator for RSI {
    fn name(&self) -> &'static str {
        "RSI"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        let period = self.period;

        if candles.len() < period {
            return vec![None; candles.len()];
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        // Calculate initial average gain/loss
        for i in 1..=period {
            let change = candles[i].close - candles[i-1].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let mut avg_gain = gains / period as f64;
        let mut avg_loss = losses / period as f64;

        result.extend(vec![None; period]); // no RSI before period

        // Calculate RSI for the rest
        for i in (period + 1)..candles.len() {
            let change = candles[i].close - candles[i-1].close;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;

            if avg_loss == 0.0 {
                result.push(Some(100.0));
            } else {
                let rs = avg_gain / avg_loss;
                let rsi = 100.0 - (100.0 / (1.0 + rs));
                result.push(Some(rsi));
            }
        }

        result
    }
}
