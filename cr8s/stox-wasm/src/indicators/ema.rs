use crate::indicators::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct EMA;
impl EMA {
    pub fn new() -> Self { EMA }
    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        let k = 2.0 / (period as f64 + 1.0);
        let mut prev = candles.get(0).map(|c| c.close).unwrap_or(0.0);
        for i in 0..candles.len() {
            let close = candles[i].close;
            prev = if i == 0 { close } else { close * k + prev * (1.0 - k) };
            if i + 1 >= period { result.push(Some(prev)); } else { result.push(None); }
        }
        result
    }
}
impl TechnicalIndicator for EMA {
    fn name(&self) -> &'static str { "Exponential Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(14) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}