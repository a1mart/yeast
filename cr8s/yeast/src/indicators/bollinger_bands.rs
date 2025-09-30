use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct BollingerBands {
    pub period: usize,
    pub k: f64,  // number of std devs
}

impl TechnicalIndicator for BollingerBands {
    fn name(&self) -> &'static str {
        "BollingerBands_Middle"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut middle_band = Vec::with_capacity(candles.len());
        let period = self.period;

        for i in 0..candles.len() {
            if i + 1 < period {
                middle_band.push(None);
                continue;
            }

            let window = &candles[i + 1 - period..=i];
            let mean: f64 = window.iter().map(|c| c.close).sum::<f64>() / period as f64;
            middle_band.push(Some(mean));
        }

        middle_band
    }
}
