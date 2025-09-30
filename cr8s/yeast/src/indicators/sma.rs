use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct SMA {
    pub period: usize,
}

impl TechnicalIndicator for SMA {
    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        let window = self.period;

        for i in 0..candles.len() {
            if i + 1 < window {
                result.push(None);
                continue;
            }
            let sum: f64 = candles[i + 1 - window..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            result.push(Some(sum / window as f64));
        }

        result
    }

    fn name(&self) -> &'static str {
        "SMA"
    }
}