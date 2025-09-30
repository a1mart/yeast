// src/indicators/frama.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct Frama;
impl Frama {
    pub fn new() -> Self { Frama }

    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> {
        vec![None; candles.len()] // stub
    }
}

impl TechnicalIndicator for Frama {
    fn name(&self) -> &'static str { "Fractal Adaptive Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
