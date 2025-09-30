// src/indicators/vwap.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct VWAP;
impl VWAP {
    pub fn new() -> Self { VWAP }

    pub(crate) fn calculate(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut cum_vol_price = 0.0;
        let mut cum_vol = 0.0;
        let mut result = Vec::with_capacity(candles.len());

        for c in candles {
            if let Some(vol) = c.volume {
                cum_vol_price += (c.high + c.low + c.close)/3.0 * vol;
                cum_vol += vol;
                result.push(Some(cum_vol_price / cum_vol));
            } else {
                result.push(None);
            }
        }
        result
    }
}
impl TechnicalIndicator for VWAP {
    fn name(&self) -> &'static str { "VWAP" }
    fn group(&self) -> &'static str { "Volume" }
    fn params(&self) -> Vec<IndicatorParam> { vec![] }
    fn compute(&self, candles: &[Candle], _options: &IndicatorOptions) -> Vec<Option<f64>> {
        self.calculate(candles)
    }
}
