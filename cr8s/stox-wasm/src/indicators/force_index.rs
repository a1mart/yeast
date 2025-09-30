// src/indicators/force_index.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct ForceIndex;
impl ForceIndex {
    pub fn new() -> Self { ForceIndex }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for ForceIndex {
    fn name(&self) -> &'static str { "Force Index" }
    fn group(&self) -> &'static str { "Volume" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
