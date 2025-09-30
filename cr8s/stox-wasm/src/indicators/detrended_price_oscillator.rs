// src/indicators/detrended_price_oscillator.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct DetrendedPriceOscillator;
impl DetrendedPriceOscillator {
    pub fn new() -> Self { DetrendedPriceOscillator }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for DetrendedPriceOscillator {
    fn name(&self) -> &'static str { "Detrended Price Oscillator" }
    fn group(&self) -> &'static str { "Oscillator" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
