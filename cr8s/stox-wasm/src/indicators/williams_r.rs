// src/indicators/williams_r.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct WilliamsR;
impl WilliamsR {
    pub fn new() -> Self { WilliamsR }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        let mut result = vec![None; candles.len()];
        if candles.len() < period { return result; }
        for i in period-1..candles.len() {
            let window = &candles[i+1-period..=i];
            let highest = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let lowest = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            result[i] = Some(if highest != lowest { (highest - candles[i].close) / (highest - lowest) * -100.0 } else { 0.0 });
        }
        result
    }
}

impl TechnicalIndicator for WilliamsR {
    fn name(&self) -> &'static str { "Williams %R" }
    fn group(&self) -> &'static str { "Oscillator" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(14) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
