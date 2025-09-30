// src/indicators/schaff_trend_cycle.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct SchaffTrendCycle;
impl SchaffTrendCycle {
    pub fn new() -> Self { SchaffTrendCycle }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for SchaffTrendCycle {
    fn name(&self) -> &'static str { "Schaff Trend Cycle" }
    fn group(&self) -> &'static str { "Oscillator" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
