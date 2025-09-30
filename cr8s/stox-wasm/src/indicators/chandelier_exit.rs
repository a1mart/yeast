// src/indicators/chandelier_exit.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct ChandelierExit;
impl ChandelierExit {
    pub fn new() -> Self { ChandelierExit }

    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> {
        vec![None; candles.len()] // stub
    }
}

impl TechnicalIndicator for ChandelierExit {
    fn name(&self) -> &'static str { "Chandelier Exit" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
