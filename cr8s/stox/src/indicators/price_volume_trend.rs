use crate::{TechnicalIndicator, Candle};

pub struct PriceVolumeTrend;

impl TechnicalIndicator for PriceVolumeTrend {
    fn name(&self) -> &'static str {
        "Price Volume Trend"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let mut pvt = Vec::with_capacity(candles.len());
        let mut cum_pvt = 0.0;

        pvt.push(Some(0.0)); // start with zero

        for i in 1..candles.len() {
            let prev_close = candles[i - 1].close;
            let curr_close = candles[i].close;
            let volume = candles[i].volume.unwrap_or(0.0);

            if prev_close != 0.0 {
                let multiplier = (curr_close - prev_close) / prev_close;
                cum_pvt += volume * multiplier;
                pvt.push(Some(cum_pvt));
            } else {
                pvt.push(Some(cum_pvt)); // no change if prev_close zero
            }
        }

        pvt
    }
}