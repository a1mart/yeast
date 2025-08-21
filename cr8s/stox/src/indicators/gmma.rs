use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct GMMA {
    pub short_periods: Vec<usize>, // usually 3-12 periods
    pub long_periods: Vec<usize>,  // usually 15-30 periods
}

impl TechnicalIndicator for GMMA {
    fn name(&self) -> &'static str {
        "Guppy Multiple Moving Averages"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Returns combined vector with short then long EMAs flattened as options (for demo)
        // Usually you'd expose separately or as grouped outputs
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut result = Vec::new();

        for &period in self.short_periods.iter().chain(self.long_periods.iter()) {
            let ema = {
                let mut res = vec![None; closes.len()];
                let k = 2.0 / (period as f64 + 1.0);
                let mut prev_ema = 0.0;
                for (i, &price) in closes.iter().enumerate() {
                    if i < period - 1 {
                        res[i] = None;
                    } else if i == period - 1 {
                        let sum: f64 = closes[i + 1 - period..=i].iter().sum();
                        prev_ema = sum / period as f64;
                        res[i] = Some(prev_ema);
                    } else {
                        prev_ema = price * k + prev_ema * (1.0 - k);
                        res[i] = Some(prev_ema);
                    }
                }
                res
            };
            result.extend(ema);
        }

        result
    }
}
