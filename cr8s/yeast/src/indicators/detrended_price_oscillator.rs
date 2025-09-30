use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct DetrendedPriceOscillator {
    pub period: usize,
}

impl TechnicalIndicator for DetrendedPriceOscillator {
    fn name(&self) -> &'static str {
        "Detrended Price Oscillator"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let sma_vals = {
            let mut sma_vals = vec![None; closes.len()];
            let period = self.period;
            for i in period - 1..closes.len() {
                let window = &closes[i + 1 - period..=i];
                let sum: f64 = window.iter().sum();
                sma_vals[i] = Some(sum / period as f64);
            }
            sma_vals
        };

        closes.iter().zip(sma_vals.iter())
            .map(|(&close, &sma)| {
                if let Some(sma_val) = sma {
                    Some(close - sma_val)
                } else {
                    None
                }
            })
            .collect()
    }
}