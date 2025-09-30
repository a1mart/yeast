// src/indicators/price_volume_trend.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct PriceVolumeTrend;
impl PriceVolumeTrend {
    pub fn new() -> Self { PriceVolumeTrend }
    pub(crate) fn calculate(&self, candles:&[Candle]) -> Vec<Option<f64>> { vec![None; candles.len()] }
}
impl TechnicalIndicator for PriceVolumeTrend {
    fn name(&self) -> &'static str { "Price Volume Trend" }
    fn group(&self) -> &'static str { "Volume" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles:&[Candle], _options:&IndicatorOptions) -> Vec<Option<f64>> { self.calculate(candles) }
}
