use crate::indicators::TechnicalIndicator;
use crate::types::Candle;
use crate::indicators::EMA;

pub struct Tema {
    pub period: usize,
}

impl TechnicalIndicator for Tema {
    fn name(&self) -> &'static str {
        "Triple Exponential Moving Average (TEMA)"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();

        let ema_indicator = EMA { period: self.period };

        // Compute EMA1 on prices
        let ema1 = ema_indicator.compute(candles);

        // Convert EMA1 Option<f64> to dummy Candle vec
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

        // Compute EMA2 on EMA1 candles
        let ema2 = ema_indicator.compute(&ema1_candles);

        let ema2_candles: Vec<Candle> = ema2.iter()
            .map(|&v| Candle {
                close: v.unwrap_or(0.0),
                open: 0.0,
                high: 0.0,
                low: 0.0,
                volume: None,
                timestamp: 0,
            })
            .collect();

        // Compute EMA3 on EMA2 candles
        let ema3 = ema_indicator.compute(&ema2_candles);

        // Zip ema1, ema2, ema3 with correct destructuring
        ema1.iter()
            .zip(ema2.iter())
            .zip(ema3.iter())
            .map(|((e1_opt, e2_opt), e3_opt)| match (e1_opt, e2_opt, e3_opt) {
                (Some(e1), Some(e2), Some(e3)) => Some(3.0 * e1 - 3.0 * e2 + e3),
                _ => None,
            })
            .collect()
    }
}
