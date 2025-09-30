// src/indicators/parabolic_sar.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct ParabolicSAR;
impl ParabolicSAR {
    pub fn new() -> Self { ParabolicSAR }

    pub(crate) fn calculate(&self, candles: &[Candle], step: f64, max_af: f64) -> Vec<Option<f64>> {
        let mut sar = vec![None; candles.len()];
        if candles.is_empty() { return sar; }
        // simple stub implementation
        sar[0] = Some(candles[0].low);
        for i in 1..candles.len() {
            sar[i] = Some(candles[i].low + (candles[i].high - candles[i].low) * step.min(max_af));
        }
        sar
    }
}

impl TechnicalIndicator for ParabolicSAR {
    fn name(&self) -> &'static str { "Parabolic SAR" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![
            IndicatorParam { name: "step".into(), param_type: "float".into(), default_value: json!(0.02) },
            IndicatorParam { name: "max_af".into(), param_type: "float".into(), default_value: json!(0.2) },
        ]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let step = options.values.get("step").and_then(|v| v.as_f64()).unwrap_or(0.02);
        let max_af = options.values.get("max_af").and_then(|v| v.as_f64()).unwrap_or(0.2);
        self.calculate(candles, step, max_af)
    }
}
