// src/indicators/heikin_ashi_slope.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct HeikinAshiSlope;
impl HeikinAshiSlope {
    pub fn new() -> Self { HeikinAshiSlope }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for HeikinAshiSlope {
    fn name(&self) -> &'static str { "Heikin-Ashi Slope" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
