use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct TRIX {
    pub period: usize,
}

impl TechnicalIndicator for TRIX {
    fn name(&self) -> &'static str {
        "TRIX"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Helper EMA function
        fn ema(period: usize, prices: &[f64]) -> Vec<Option<f64>> {
            let mut result = Vec::with_capacity(prices.len());
            let k = 2.0 / (period as f64 + 1.0);
            let mut prev_ema = 0.0;

            for (i, &price) in prices.iter().enumerate() {
                if i < period - 1 {
                    result.push(None);
                } else if i == period - 1 {
                    let sum: f64 = prices[i + 1 - period..=i].iter().sum();
                    prev_ema = sum / period as f64;
                    result.push(Some(prev_ema));
                } else {
                    let ema = price * k + prev_ema * (1.0 - k);
                    result.push(Some(ema));
                    prev_ema = ema;
                }
            }

            result
        }

        // Extract closes
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        // First EMA
        let ema1 = ema(self.period, &closes);
        // Second EMA over ema1 (unwrap None as 0.0)
        let ema1_vals: Vec<f64> = ema1.iter().map(|x| x.unwrap_or(0.0)).collect();
        let ema2 = ema(self.period, &ema1_vals);
        // Third EMA over ema2
        let ema2_vals: Vec<f64> = ema2.iter().map(|x| x.unwrap_or(0.0)).collect();
        let ema3 = ema(self.period, &ema2_vals);

        // TRIX = percent rate of change of ema3
        let mut trix = Vec::with_capacity(candles.len());
        trix.push(None); // no previous value for i=0

        for i in 1..candles.len() {
            match (ema3.get(i), ema3.get(i - 1)) {
                (Some(Some(curr)), Some(Some(prev))) if *prev != 0.0 => {
                    trix.push(Some(((curr - prev) / prev) * 100.0))
                }
                _ => trix.push(None),
            }
        }

        trix
    }
}