// src/indicators/cmf.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct CMF;
impl CMF {
    pub fn new() -> Self { CMF }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        let mut cmf = vec![None; candles.len()];
        if candles.len() < period { return cmf; }
        for i in period-1..candles.len() {
            let window = &candles[i+1-period..=i];
            let mfv: f64 = window.iter().map(|c| ((c.close - c.low) - (c.high - c.close)) / (c.high - c.low) * c.volume.unwrap_or(0.0)).sum();
            let vol: f64 = window.iter().map(|c| c.volume.unwrap_or(0.0)).sum();
            cmf[i] = Some(if vol != 0.0 { mfv / vol } else { 0.0 });
        }
        cmf
    }
}

impl TechnicalIndicator for CMF {
    fn name(&self) -> &'static str { "Chaikin Money Flow" }
    fn group(&self) -> &'static str { "Volume" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(20) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
        self.calculate(candles, period)
    }
}
