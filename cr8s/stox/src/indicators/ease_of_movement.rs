use crate::{TechnicalIndicator, Candle};

pub struct EaseOfMovement {
    pub period: usize,
}

impl TechnicalIndicator for EaseOfMovement {
    fn name(&self) -> &'static str {
        "Ease of Movement"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut eom = vec![None; candles.len()];

        for i in 1..candles.len() {
            let distance_moved = ((candles[i].high + candles[i].low) / 2.0) - ((candles[i - 1].high + candles[i - 1].low) / 2.0);
            let box_ratio = if candles[i].volume.unwrap_or(0.0) != 0.0 {
                (candles[i].high - candles[i].low) / candles[i].volume.unwrap()
            } else {
                0.0
            };

            eom[i] = Some(distance_moved / box_ratio);
        }

        // Smooth with SMA of period
        let mut smoothed = vec![None; candles.len()];
        for i in self.period - 1..candles.len() {
            let sum: f64 = eom[i + 1 - self.period..=i].iter().filter_map(|x| *x).sum();
            smoothed[i] = Some(sum / self.period as f64);
        }

        smoothed
    }
}
