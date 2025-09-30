use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct ForceIndex {
    pub period: usize,
}

impl TechnicalIndicator for ForceIndex {
    fn name(&self) -> &'static str {
        "Force Index"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut force = vec![None; candles.len()];

        for i in 1..candles.len() {
            if let Some(volume) = candles[i].volume {
                force[i] = Some((candles[i].close - candles[i - 1].close) * volume);
            }
        }

        // Smooth with EMA of period if needed
        if self.period <= 1 {
            return force;
        }

        // EMA helper:
        fn ema(period: usize, values: &[Option<f64>]) -> Vec<Option<f64>> {
            let mut result = Vec::with_capacity(values.len());
            let k = 2.0 / (period as f64 + 1.0);
            let mut prev_ema = 0.0;

            for (i, val) in values.iter().enumerate() {
                if i < period - 1 {
                    result.push(None);
                } else if i == period - 1 {
                    let sum: f64 = values[i + 1 - period..=i]
                        .iter()
                        .filter_map(|x| *x)
                        .sum();
                    prev_ema = sum / period as f64;
                    result.push(Some(prev_ema));
                } else if let Some(v) = val {
                    let ema = v * k + prev_ema * (1.0 - k);
                    prev_ema = ema;
                    result.push(Some(ema));
                } else {
                    result.push(None);
                }
            }

            result
        }

        ema(self.period, &force)
    }
}