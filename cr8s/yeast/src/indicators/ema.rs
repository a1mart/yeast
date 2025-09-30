use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct EMA {
    pub period: usize,
}

impl TechnicalIndicator for EMA {
    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        let mut prev_ema = 0.0;

        let k = 2.0 / (self.period as f64 + 1.0);

        for (i, candle) in candles.iter().enumerate() {
            if i < self.period - 1 {
                result.push(None);
            } else if i == self.period - 1 {
                // Initialize EMA with SMA at period
                let sum: f64 = candles[i + 1 - self.period..=i]
                    .iter()
                    .map(|c| c.close)
                    .sum();
                prev_ema = sum / self.period as f64;
                result.push(Some(prev_ema));
            } else {
                let ema = (candle.close * k) + (prev_ema * (1.0 - k));
                result.push(Some(ema));
                prev_ema = ema;
            }
        }

        result
    }

    fn name(&self) -> &'static str {
        "EMA"
    }
}
