use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct MACD {
    pub fast_period: usize,
    pub slow_period: usize,
}

impl TechnicalIndicator for MACD {
    fn name(&self) -> &'static str {
        "MACD"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut macd_line = Vec::with_capacity(candles.len());

        // Helper to compute EMA with given period
        fn ema(candles: &[Candle], period: usize) -> Vec<Option<f64>> {
            let mut result = Vec::with_capacity(candles.len());
            let k = 2.0 / (period as f64 + 1.0);
            let mut prev_ema = 0.0;

            for (i, candle) in candles.iter().enumerate() {
                if i < period - 1 {
                    result.push(None);
                } else if i == period - 1 {
                    let sum: f64 = candles[i + 1 - period..=i].iter().map(|c| c.close).sum();
                    prev_ema = sum / period as f64;
                    result.push(Some(prev_ema));
                } else {
                    let ema = candle.close * k + prev_ema * (1.0 - k);
                    result.push(Some(ema));
                    prev_ema = ema;
                }
            }
            result
        }

        let fast_ema = ema(candles, self.fast_period);
        let slow_ema = ema(candles, self.slow_period);

        for i in 0..candles.len() {
            match (fast_ema.get(i), slow_ema.get(i)) {
                (Some(Some(fast)), Some(Some(slow))) => macd_line.push(Some(fast - slow)),
                _ => macd_line.push(None),
            }
        }

        macd_line
    }
}
