use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct AccumDistLine;

impl TechnicalIndicator for AccumDistLine {
    fn name(&self) -> &'static str {
        "Accumulation/Distribution Line"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut ad = Vec::with_capacity(candles.len());
        let mut cum_sum = 0.0;

        for c in candles {
            let range = c.high - c.low;
            let clv = if range != 0.0 {
                ((c.close - c.low) - (c.high - c.close)) / range
            } else {
                0.0
            };

            let mf = clv * c.volume.unwrap_or(0.0);
            cum_sum += mf;
            ad.push(Some(cum_sum));
        }

        ad
    }
}