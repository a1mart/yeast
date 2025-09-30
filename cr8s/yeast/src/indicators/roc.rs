use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct RateOfChange {
    pub period: usize,
}

impl TechnicalIndicator for RateOfChange {
    fn name(&self) -> &'static str {
        "Rate of Change"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut roc = vec![None; candles.len()];

        for i in self.period..closes.len() {
            if closes[i - self.period] != 0.0 {
                roc[i] = Some(((closes[i] - closes[i - self.period]) / closes[i - self.period]) * 100.0);
            }
        }

        roc
    }
}