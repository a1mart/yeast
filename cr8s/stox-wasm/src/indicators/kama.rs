// src/indicators/kama.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct Kama;
impl Kama {
    pub fn new() -> Self { Kama }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        // stub: use simple smoothing for demo
        let mut result = vec![None; candles.len()];
        let mut prev = candles.get(0).map(|c| c.close).unwrap_or(0.0);
        for i in 0..candles.len() {
            let close = candles[i].close;
            prev = prev + (close - prev) / period as f64;
            result[i] = Some(prev);
        }
        result
    }
}

impl TechnicalIndicator for Kama {
    fn name(&self) -> &'static str { "Kaufman's Adaptive Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name:"period".into(), param_type:"int".into(), default_value: json!(10)}]
    }
    fn compute(&self, candles:&[Candle], options:&IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v|v.as_u64()).unwrap_or(10) as usize;
        self.calculate(candles, period)
    }
}
