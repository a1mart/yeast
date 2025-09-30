use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct VWAP;

impl TechnicalIndicator for VWAP {
    fn name(&self) -> &'static str {
        "VWAP"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut vwap = Vec::with_capacity(candles.len());
        let mut cumulative_vol = 0.0;
        let mut cumulative_vol_price = 0.0;

        for candle in candles {
            if let Some(volume) = candle.volume {
                cumulative_vol += volume;
                cumulative_vol_price += candle.close * volume;
                if cumulative_vol > 0.0 {
                    vwap.push(Some(cumulative_vol_price / cumulative_vol));
                } else {
                    vwap.push(None);
                }
            } else {
                vwap.push(None);
            }
        }

        vwap
    }
}
