// src/indicators/kalman_filter_smoother.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct KalmanFilterSmoother;
impl KalmanFilterSmoother {
    pub fn new() -> Self { KalmanFilterSmoother }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for KalmanFilterSmoother {
    fn name(&self) -> &'static str { "Kalman Filter Smoother" }
    fn group(&self) -> &'static str { "Filter" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
