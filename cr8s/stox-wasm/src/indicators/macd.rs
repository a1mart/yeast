// src/indicators/macd.rs
use crate::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct MACD;
impl MACD {
    pub fn new() -> Self { MACD }

    pub(crate) fn calculate(&self, candles: &[Candle], short_period: usize, long_period: usize, signal_period: usize) -> Vec<Option<f64>> {
        use crate::indicators::EMA;
        let ema_short = EMA.calculate(candles, short_period);
        let ema_long = EMA.calculate(candles, long_period);
        let mut macd_line = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            let macd = match (ema_short.get(i), ema_long.get(i)) {
                (Some(Some(s)), Some(Some(l))) => Some(s - l),
                _ => None,
            };
            macd_line.push(macd);
        }
        

        // Compute signal line
        let mut signal_line = Vec::with_capacity(candles.len());
        let mut prev = 0.0;
        for i in 0..macd_line.len() {
            let val = macd_line[i].unwrap_or(0.0);
            prev = if i == 0 { val } else { val * 2.0/(signal_period as f64 + 1.0) + prev * (1.0 - 2.0/(signal_period as f64 + 1.0)) };
            if i + 1 >= signal_period { signal_line.push(Some(prev)); } else { signal_line.push(None); }
        }

        signal_line
    }
}

impl TechnicalIndicator for MACD {
    fn name(&self) -> &'static str { "MACD" }
    fn group(&self) -> &'static str { "Trend" }
    fn params(&self) -> Vec<IndicatorParam> {
        vec![
            IndicatorParam { name: "short_period".into(), param_type: "int".into(), default_value: json!(12) },
            IndicatorParam { name: "long_period".into(), param_type: "int".into(), default_value: json!(26) },
            IndicatorParam { name: "signal_period".into(), param_type: "int".into(), default_value: json!(9) },
        ]
    }

    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let short_period = options.values.get("short_period").and_then(|v| v.as_u64()).unwrap_or(12) as usize;
        let long_period = options.values.get("long_period").and_then(|v| v.as_u64()).unwrap_or(26) as usize;
        let signal_period = options.values.get("signal_period").and_then(|v| v.as_u64()).unwrap_or(9) as usize;
        self.calculate(candles, short_period, long_period, signal_period)
    }
}
