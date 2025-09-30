// src/indicators/cci.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct CCI;
impl CCI {
    pub fn new() -> Self { CCI }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(candles.len());
        for i in 0..candles.len() {
            if i + 1 < period { result.push(None); continue; }
            let window = &candles[i+1-period..=i];
            let typical_prices: Vec<f64> = window.iter().map(|c| (c.high + c.low + c.close)/3.0).collect();
            let sma: f64 = typical_prices.iter().sum::<f64>() / period as f64;
            let mean_dev: f64 = typical_prices.iter().map(|tp| (tp - sma).abs()).sum::<f64>() / period as f64;
            if mean_dev == 0.0 { result.push(Some(0.0)); continue; }
            let cci = (typical_prices[period-1] - sma) / (0.015 * mean_dev);
            result.push(Some(cci));
        }
        result
    }
}

impl TechnicalIndicator for CCI {
    fn name(&self) -> &'static str { "Commodity Channel Index" }
    fn group(&self) -> &'static str { "Momentum" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(20) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
        self.calculate(candles, period)
    }
}
