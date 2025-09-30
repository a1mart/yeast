// src/indicators/ichimoku.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct Ichimoku;
impl Ichimoku {
    pub fn new() -> Self { Ichimoku }

    pub(crate) fn calculate(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut result = vec![None; candles.len()];
        // stub: using conversion line (Tenkan-sen)
        for i in 8..candles.len() {
            let window = &candles[i-8..=i];
            let high = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let low = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            result[i] = Some((high + low) / 2.0);
        }
        result
    }
}

impl TechnicalIndicator for Ichimoku {
    fn name(&self) -> &'static str { "Ichimoku Kinko Hyo" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles: &[Candle], _options: &IndicatorOptions) -> Vec<Option<f64>> {
        self.calculate(candles)
    }
}
