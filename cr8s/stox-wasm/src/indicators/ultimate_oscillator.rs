// src/indicators/ultimate_oscillator.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct UltimateOscillator;
impl UltimateOscillator {
    pub fn new() -> Self { UltimateOscillator }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for UltimateOscillator {
    fn name(&self) -> &'static str { "Ultimate Oscillator" }
    fn group(&self) -> &'static str { "Oscillator" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
