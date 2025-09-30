// src/indicators/tema.rs
use crate::{Candle, TechnicalIndicator, IndicatorParam, IndicatorOptions};
use serde_json::json;

pub struct Tema;
impl Tema {
    pub fn new() -> Self { Tema }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        use super::ema::EMA;
        let ema1 = EMA::new().calculate(candles, period);
        let ema2 = EMA::new().calculate(&ema1.iter().enumerate().map(|(i,v)| Candle { timestamp: i as i64, open: v.unwrap_or(0.0), high: v.unwrap_or(0.0), low: v.unwrap_or(0.0), close: v.unwrap_or(0.0), volume: None }).collect::<Vec<_>>(), period);
        let ema3 = EMA::new().calculate(&ema2.iter().enumerate().map(|(i,v)| Candle { timestamp: i as i64, open: v.unwrap_or(0.0), high: v.unwrap_or(0.0), low: v.unwrap_or(0.0), close: v.unwrap_or(0.0), volume: None }).collect::<Vec<_>>(), period);
        ema1.iter().zip(ema2.iter()).zip(ema3.iter()).map(|((a,b),c)| a.map(|a| 3.0*a - 3.0*b.unwrap_or(0.0) + c.unwrap_or(0.0))).collect()
    }
}

impl TechnicalIndicator for Tema {
    fn name(&self) -> &'static str { "Triple Exponential Moving Average" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(14) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
