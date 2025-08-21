// src/indicators/cmf.rs

use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct CMF {
    pub period: usize,
}

impl TechnicalIndicator for CMF {
    fn name(&self) -> &'static str {
        "CMF"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut cmf = Vec::with_capacity(candles.len());
        let period = self.period;

        for i in 0..candles.len() {
            if i + 1 < period {
                cmf.push(None);
                continue;
            }
            let window = &candles[i + 1 - period..=i];

            let mut mfv_sum = 0.0;
            let mut volume_sum = 0.0;

            for c in window {
                let denom = c.high - c.low;
                if denom == 0.0 {
                    continue;
                }
                if let Some(vol) = c.volume {
                    let mfm = ((c.close - c.low) - (c.high - c.close)) / denom;
                    mfv_sum += mfm * vol;
                    volume_sum += vol;
                }
            }

            if volume_sum == 0.0 {
                cmf.push(None);
            } else {
                cmf.push(Some(mfv_sum / volume_sum));
            }
        }

        cmf
    }
}
