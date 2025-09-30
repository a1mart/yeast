// src/indicators/momentum.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct Momentum;
impl Momentum {
    pub fn new() -> Self { Momentum }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        let mut result = vec![None; candles.len()];
        for i in period..candles.len() {
            result[i] = Some(candles[i].close - candles[i-period].close);
        }
        result
    }
}

impl TechnicalIndicator for Momentum {
    fn name(&self) -> &'static str { "Momentum" }
    fn group(&self) -> &'static str { "Oscillator" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(10) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        self.calculate(candles, period)
    }
}
