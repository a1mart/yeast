// src/indicators/ease_of_movement.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct EaseOfMovement;
impl EaseOfMovement {
    pub fn new() -> Self { EaseOfMovement }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for EaseOfMovement {
    fn name(&self) -> &'static str { "Ease of Movement" }
    fn group(&self) -> &'static str { "Volume" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
