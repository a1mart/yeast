// src/indicators/obv.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct OBV;
impl OBV {
    pub fn new() -> Self { OBV }

    pub(crate) fn calculate(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut obv = vec![Some(0.0)];
        for i in 1..candles.len() {
            let prev = obv[i-1].unwrap_or(0.0);
            let change = if candles[i].close > candles[i-1].close { candles[i].volume.unwrap_or(0.0) }
                         else if candles[i].close < candles[i-1].close { -candles[i].volume.unwrap_or(0.0) }
                         else { 0.0 };
            obv.push(Some(prev + change));
        }
        obv
    }
}

impl TechnicalIndicator for OBV {
    fn name(&self) -> &'static str { "On-Balance Volume" }
    fn group(&self) -> &'static str { "Volume" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles: &[Candle], _options: &IndicatorOptions) -> Vec<Option<f64>> {
        self.calculate(candles)
    }
}
