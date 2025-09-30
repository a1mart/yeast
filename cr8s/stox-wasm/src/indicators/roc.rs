// src/indicators/roc.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct RateOfChange;
impl RateOfChange {
    pub fn new() -> Self { RateOfChange }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for RateOfChange {
    fn name(&self) -> &'static str { "Rate of Change" }
    fn group(&self) -> &'static str { "Momentum" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
