use crate::{TechnicalIndicator, Candle};
use crate::indicators::EMA;

pub struct Dema {
    pub period: usize,
}

impl TechnicalIndicator for Dema {
    fn name(&self) -> &'static str {
        "Double Exponential Moving Average (DEMA)"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let ema_indicator = EMA { period: self.period };

        // Compute EMA1 on input candles
        let ema1 = ema_indicator.compute(candles);

        // Convert EMA1 Option<f64> results into dummy Candles to reuse compute()
        let ema1_candles: Vec<Candle> = ema1.iter()
            .map(|&v| Candle {
                close: v.unwrap_or(0.0),
                open: 0.0,
                high: 0.0,
                low: 0.0,
                volume: None,
                timestamp: 0,
            })
            .collect();

        // Compute EMA2 on EMA1 results (as Candles)
        let ema2 = ema_indicator.compute(&ema1_candles);

        // Combine for DEMA formula: 2*EMA1 - EMA2
        ema1.iter()
            .zip(ema2.iter())
            .map(|(e1_opt, e2_opt)| match (e1_opt, e2_opt) {
                (Some(e1), Some(e2)) => Some(2.0 * e1 - e2),
                _ => None,
            })
            .collect()
    }
}
