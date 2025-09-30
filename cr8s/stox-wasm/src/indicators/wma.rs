// src/indicators/wma.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct WMA;
impl WMA {
    pub fn new() -> Self { WMA }

    pub(crate) fn calculate(&self, candles:&[Candle], period:usize) -> Vec<Option<f64>> {
        let mut result = vec![None; candles.len()];
        for i in period-1..candles.len() {
            let mut sum = 0.0;
            let mut weight = 0.0;
            for j in 0..period {
                let w = (j+1) as f64;
                sum += candles[i-period+1+j].close * w;
                weight += w;
            }
            result[i] = Some(sum / weight);
        }
        result
    }
}

impl TechnicalIndicator for WMA {
    fn name(&self) -> &'static str { "Weighted Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name:"period".into(), param_type:"int".into(), default_value: json!(14)}]
    }
    fn compute(&self, candles:&[Candle], options:&IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v|v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
