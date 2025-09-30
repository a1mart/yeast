// src/indicators/atr.rs

use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct ATR {
    pub period: usize,
}

impl TechnicalIndicator for ATR {
    fn name(&self) -> &'static str {
        "ATR"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut atr = Vec::with_capacity(candles.len());
        let period = self.period;

        let mut trs = Vec::with_capacity(candles.len());
        for i in 0..candles.len() {
            if i == 0 {
                trs.push(candles[0].high - candles[0].low);
            } else {
                let high_low = candles[i].high - candles[i].low;
                let high_close = (candles[i].high - candles[i-1].close).abs();
                let low_close = (candles[i].low - candles[i-1].close).abs();
                trs.push(high_low.max(high_close).max(low_close));
            }
        }

        for i in 0..candles.len() {
            if i + 1 < period {
                atr.push(None);
                continue;
            }
            let window = &trs[i + 1 - period..=i];
            let avg_tr = window.iter().sum::<f64>() / period as f64;
            atr.push(Some(avg_tr));
        }

        atr
    }
}
