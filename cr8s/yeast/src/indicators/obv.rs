// src/indicators/obv.rs

use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct OBV;

impl TechnicalIndicator for OBV {
    fn name(&self) -> &'static str {
        "OBV"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut obv = Vec::with_capacity(candles.len());

        if candles.is_empty() {
            return obv;
        }

        let mut total = 0.0;
        obv.push(Some(total)); // first value

        for i in 1..candles.len() {
            if let Some(vol) = candles[i].volume {
                if candles[i].close > candles[i - 1].close {
                    total += vol;
                } else if candles[i].close < candles[i - 1].close {
                    total -= vol;
                }
            }
            obv.push(Some(total));
        }

        obv
    }
}
