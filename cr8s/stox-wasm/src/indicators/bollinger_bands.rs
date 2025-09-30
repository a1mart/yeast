// src/indicators/bollinger_bands.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct BollingerBands;
impl BollingerBands {
    pub fn new() -> Self { BollingerBands }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize, std_dev: f64) -> Vec<Option<f64>> {
        use crate::indicators::SMA;
        let sma_values = SMA.calculate(candles, period);
        let mut bands = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            if i + 1 < period { bands.push(None); continue; }
            let mean = sma_values[i].unwrap();
            let variance = candles[i+1-period..=i].iter().map(|c| (c.close - mean).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            bands.push(Some(mean + std_dev * std)); // upper band
        }
        bands
    }
}
impl TechnicalIndicator for BollingerBands {
    fn name(&self) -> &'static str { "Bollinger Bands" }
    fn group(&self) -> &'static str { "Volatility" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![
            IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(20) },
            IndicatorParam { name: "std_dev".into(), param_type: "float".into(), default_value: json!(2.0) },
        ]
    }

    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
        let std_dev = options.values.get("std_dev").and_then(|v| v.as_f64()).unwrap_or(2.0);
        self.calculate(candles, period, std_dev)
    }
}
