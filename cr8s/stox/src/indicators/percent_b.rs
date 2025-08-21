use crate::{TechnicalIndicator, Candle};

/// Percent B (from Bollinger Bands)
/// Measures position of price relative to Bollinger Bands [0..1]
pub struct PercentB {
    pub period: usize,
    pub std_dev_mult: f64,
}

impl TechnicalIndicator for PercentB {
    fn name(&self) -> &'static str {
        "Percent B"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut percent_b = vec![None; closes.len()];

        for i in self.period - 1..closes.len() {
            let window = &closes[i + 1 - self.period..=i];
            let mean = window.iter().sum::<f64>() / self.period as f64;
            let variance = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.period as f64;
            let stddev = variance.sqrt();

            let upper_band = mean + self.std_dev_mult * stddev;
            let lower_band = mean - self.std_dev_mult * stddev;
            let price = closes[i];

            if upper_band != lower_band {
                percent_b[i] = Some((price - lower_band) / (upper_band - lower_band));
            } else {
                percent_b[i] = None;
            }
        }

        percent_b
    }
}
