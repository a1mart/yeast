use crate::{TechnicalIndicator, Candle};

pub struct ZScore {
    pub period: usize,
}

impl TechnicalIndicator for ZScore {
    fn name(&self) -> &'static str {
        "Z-Score"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut z_scores = vec![None; closes.len()];

        for i in self.period - 1..closes.len() {
            let window = &closes[i + 1 - self.period..=i];
            let mean = window.iter().sum::<f64>() / self.period as f64;
            let variance = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.period as f64;
            let stddev = variance.sqrt();

            if stddev != 0.0 {
                z_scores[i] = Some((closes[i] - mean) / stddev);
            } else {
                z_scores[i] = Some(0.0);
            }
        }

        z_scores
    }
}