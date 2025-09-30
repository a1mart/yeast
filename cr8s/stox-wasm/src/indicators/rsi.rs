use crate::indicators::{TechnicalIndicator, IndicatorParam, IndicatorOptions, Candle};
use serde_json::json;

pub struct RSI;

impl RSI {
    pub fn new() -> Self {
        RSI
    }

    pub(crate) fn calculate(&self, candles: &[Candle], period: usize) -> Vec<Option<f64>> {
        if candles.len() < period {
            return vec![None; candles.len()];
        }

        let mut rsis = vec![None; candles.len()];
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..candles.len() {
            let change = candles[i].close - candles[i - 1].close;
            gains.push(change.max(0.0));
            losses.push((-change).max(0.0));

            if i >= period {
                let avg_gain = gains[(i + 1 - period)..=i].iter().sum::<f64>() / period as f64;
                let avg_loss = losses[(i + 1 - period)..=i].iter().sum::<f64>() / period as f64;

                let rs = if avg_loss == 0.0 {
                    100.0
                } else {
                    avg_gain / avg_loss
                };

                rsis[i] = Some(100.0 - (100.0 / (1.0 + rs)));
            }
        }

        rsis
    }
}

impl TechnicalIndicator for RSI {
    fn name(&self) -> &'static str {
        "Relative Strength Index"
    }

    fn group(&self) -> &'static str {
        "Oscillator"
    }

    fn params(&self) -> Vec<IndicatorParam> {
        vec![IndicatorParam {
            name: "period".to_string(),
            param_type: "int".to_string(),
            default_value: json!(14),
        }]
    }

    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>> {
        let period = options
            .values
            .get("period")
            .and_then(|v| v.as_u64())
            .unwrap_or(14) as usize;
        self.calculate(candles, period)
    }
}