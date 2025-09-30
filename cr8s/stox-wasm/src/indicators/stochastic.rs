// src/indicators/stochastic.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct Stochastic;
impl Stochastic {
    pub fn new() -> Self { Stochastic }

    pub(crate) fn calculate(&self, candles: &[Candle], k_period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        for i in 0..candles.len() {
            if i + 1 < k_period { result.push(None); continue; }
            let window = &candles[i+1-k_period..=i];
            let lowest_low = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            let highest_high = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let value = if highest_high - lowest_low == 0.0 { 0.0 } else { (candles[i].close - lowest_low) / (highest_high - lowest_low) * 100.0 };
            result.push(Some(value));
        }
        result
    }
}

impl TechnicalIndicator for Stochastic {
    fn name(&self) -> &'static str { "Stochastic Oscillator" }
    fn group(&self) -> &'static str { "Oscillator" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "k_period".into(), param_type: "int".into(), default_value: json!(14) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let k_period = options.values.get("k_period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, k_period)
    }
}
