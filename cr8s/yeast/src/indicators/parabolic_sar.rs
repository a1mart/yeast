// src/indicators/parabolic_sar.rs

use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct ParabolicSAR {
    pub step: f64,
    pub max_step: f64,
}

impl TechnicalIndicator for ParabolicSAR {
    fn name(&self) -> &'static str {
        "ParabolicSAR"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut sar = vec![None; candles.len()];
        if candles.len() < 2 {
            return sar;
        }

        let mut is_long = true; // start trend assumed long
        let mut af = self.step;
        let max_af = self.max_step;

        let mut ep = candles[0].low; // extreme point
        let mut sar_value = candles[0].high; // start SAR

        sar[0] = Some(sar_value);

        for i in 1..candles.len() {
            sar_value += af * (ep - sar_value);

            if is_long {
                if candles[i].low < sar_value {
                    // switch to short
                    is_long = false;
                    sar_value = ep;
                    ep = candles[i].high;
                    af = self.step;
                    sar[i] = Some(sar_value);
                } else {
                    if candles[i].high > ep {
                        ep = candles[i].high;
                        af = (af + self.step).min(max_af);
                    }
                    sar[i] = Some(sar_value.min(candles[i].low));
                }
            } else {
                if candles[i].high > sar_value {
                    // switch to long
                    is_long = true;
                    sar_value = ep;
                    ep = candles[i].low;
                    af = self.step;
                    sar[i] = Some(sar_value);
                } else {
                    if candles[i].low < ep {
                        ep = candles[i].low;
                        af = (af + self.step).min(max_af);
                    }
                    sar[i] = Some(sar_value.max(candles[i].high));
                }
            }
        }

        sar
    }
}
