use crate::indicators::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct SMA;
impl SMA {
    pub fn new() -> Self { SMA }
    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        if candles.len() < period { return vec![None; candles.len()]; }
        let mut result = Vec::with_capacity(candles.len());
        for i in 0..candles.len() {
            if i + 1 < period {
                result.push(None);
            } else {
                let sum: f64 = candles[i+1-period..=i].iter().map(|c| c.close).sum();
                result.push(Some(sum / period as f64));
            }
        }
        result
    }
}
impl TechnicalIndicator for SMA {
    fn name(&self) -> &'static str { "Simple Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(14) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
