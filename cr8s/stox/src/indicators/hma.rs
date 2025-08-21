use crate::indicators::TechnicalIndicator;
use crate::types::Candle;
use crate::indicators::WMA;

pub struct Hma {
    pub period: usize,
}

impl TechnicalIndicator for Hma {
    fn name(&self) -> &'static str {
        "Hull Moving Average (HMA)"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        if self.period < 1 {
            return vec![None; candles.len()];
        }

        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let half = self.period / 2;
        let sqrt = (self.period as f64).sqrt().round() as usize;

        let wma_half_indicator = WMA { period: half };
        let wma_full_indicator = WMA { period: self.period };
        let wma_ret_indicator = WMA { period: sqrt };

        // Wrap prices into dummy Candles for WMA input
        let prices_as_candles: Vec<Candle> = prices
            .iter()
            .map(|&price| Candle {
                timestamp: 0,
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: price,
                volume: None,
            })
            .collect();

        let wma_half = wma_half_indicator.compute(&prices_as_candles);

        let wma_full = wma_full_indicator.compute(&prices_as_candles);

        // Compute diff vector (raw f64)
        let diff: Vec<f64> = wma_half
            .iter()
            .zip(wma_full.iter())
            .map(|(h, f)| h.unwrap_or(0.0) * 2.0 - f.unwrap_or(0.0))
            .collect();

        // Wrap diff back to Candles to call compute again
        let diff_as_candles: Vec<Candle> = diff
            .iter()
            .map(|&price| Candle {
                timestamp: 0,
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: price,
                volume: None,
            })
            .collect();

        wma_ret_indicator.compute(&diff_as_candles)
    }
}
