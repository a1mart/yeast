// src/indicators/atr.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct ATR;
impl ATR {
    pub fn new() -> Self { ATR }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        if candles.len() < period { return vec![None; candles.len()]; }
        let mut trs = Vec::with_capacity(candles.len());
        trs.push(None); // first candle TR undefined

        for i in 1..candles.len() {
            let high_low = candles[i].high - candles[i].low;
            let high_close = (candles[i].high - candles[i-1].close).abs();
            let low_close = (candles[i].low - candles[i-1].close).abs();
            trs.push(Some(high_low.max(high_close).max(low_close)));
        }

        let mut atr = Vec::with_capacity(candles.len());
        let mut sum = 0.0;
        for i in 0..candles.len() {
            if i + 1 < period { atr.push(None); continue; }
            if i + 1 == period {
                sum = trs[1..=i].iter().map(|v| v.unwrap_or(0.0)).sum();
                atr.push(Some(sum / period as f64));
            } else {
                let prev_atr = atr[i-1].unwrap();
                atr.push(Some((prev_atr * (period as f64 - 1.0) + trs[i].unwrap()) / period as f64));
            }
        }
        atr
    }
}
impl TechnicalIndicator for ATR {
    fn name(&self) -> &'static str { "Average True Range" }
    fn group(&self) -> &'static str { "Volatility" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(14) }]
    }

    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
