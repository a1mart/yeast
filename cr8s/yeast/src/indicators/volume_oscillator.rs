use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct VolumeOscillator {
    pub short_period: usize,
    pub long_period: usize,
}

impl TechnicalIndicator for VolumeOscillator {
    fn name(&self) -> &'static str {
        "Volume Oscillator"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Helper: simple moving average for volumes
        fn sma(period: usize, volumes: &[Option<f64>]) -> Vec<Option<f64>> {
            let mut sma_vals = vec![None; volumes.len()];
            for i in period - 1..volumes.len() {
                let window = &volumes[i + 1 - period..=i];
                let sum: f64 = window.iter().filter_map(|v| *v).sum();
                sma_vals[i] = Some(sum / period as f64);
            }
            sma_vals
        }

        let volumes: Vec<Option<f64>> = candles.iter().map(|c| c.volume).collect();
        let short_ma = sma(self.short_period, &volumes);
        let long_ma = sma(self.long_period, &volumes);

        short_ma.iter()
            .zip(long_ma.iter())
            .map(|(s, l)| match (s, l) {
                (Some(sv), Some(lv)) => Some(sv - lv),
                _ => None,
            })
            .collect()
    }
}
