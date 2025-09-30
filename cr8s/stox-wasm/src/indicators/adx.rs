// src/indicators/adx.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct ADX;
impl ADX {
    pub fn new() -> Self { ADX }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        if candles.len() < period { return vec![None; candles.len()]; }
        let mut tr = Vec::with_capacity(candles.len());
        let mut plus_dm = Vec::with_capacity(candles.len());
        let mut minus_dm = Vec::with_capacity(candles.len());

        tr.push(None);
        plus_dm.push(None);
        minus_dm.push(None);

        for i in 1..candles.len() {
            let high_diff = candles[i].high - candles[i-1].high;
            let low_diff = candles[i-1].low - candles[i].low;
            plus_dm.push(Some(if high_diff > low_diff && high_diff > 0.0 { high_diff } else { 0.0 }));
            minus_dm.push(Some(if low_diff > high_diff && low_diff > 0.0 { low_diff } else { 0.0 }));
            let high_low = candles[i].high - candles[i].low;
            let high_close = (candles[i].high - candles[i-1].close).abs();
            let low_close = (candles[i].low - candles[i-1].close).abs();
            tr.push(Some(high_low.max(high_close).max(low_close)));
        }

        // Simple smoothing (EMA-style) for DI
        let mut adx = vec![None; candles.len()];
        let mut plus_di_prev = 0.0;
        let mut minus_di_prev = 0.0;
        let mut dx_prev = 0.0;

        for i in period..candles.len() {
            let tr_sum: f64 = tr[i+1-period..=i].iter().map(|v| v.unwrap_or(0.0)).sum();
            let plus_dm_sum: f64 = plus_dm[i+1-period..=i].iter().map(|v| v.unwrap_or(0.0)).sum();
            let minus_dm_sum: f64 = minus_dm[i+1-period..=i].iter().map(|v| v.unwrap_or(0.0)).sum();

            plus_di_prev = if tr_sum == 0.0 { 0.0 } else { (plus_dm_sum / tr_sum) * 100.0 };
            minus_di_prev = if tr_sum == 0.0 { 0.0 } else { (minus_dm_sum / tr_sum) * 100.0 };
            dx_prev = if (plus_di_prev + minus_di_prev) == 0.0 { 0.0 } else { ((plus_di_prev - minus_di_prev).abs() / (plus_di_prev + minus_di_prev)) * 100.0 };

            adx[i] = Some(dx_prev);
        }

        adx
    }
}

impl TechnicalIndicator for ADX {
    fn name(&self) -> &'static str { "Average Directional Index" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam { name: "period".into(), param_type: "int".into(), default_value: json!(14) }]
    }
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options.values.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}
