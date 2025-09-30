// src/indicators/hma.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct Hma;
impl Hma {
    pub fn new() -> Self { Hma }

    pub(crate) fn calculate(&self, candles:&[Candle], period:usize) -> Vec<Option<f64>> {
        // stub: use simple smoothing
        let wma_half = super::wma::WMA::new().calculate(candles, period/2);
        let wma_full = super::wma::WMA::new().calculate(candles, period);
        wma_half.iter().zip(wma_full.iter()).map(|(h,f)| h.map(|h| 2.0*h - f.unwrap_or(0.0))).collect()
    }
}

impl TechnicalIndicator for Hma {
    fn name(&self) -> &'static str { "Hull Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name:"period".into(), param_type:"int".into(), default_value: json!(14)}]
    }
    fn compute(&self, candles:&[Candle], options:&IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v|v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
