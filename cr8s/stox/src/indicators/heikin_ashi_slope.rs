use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

/// Heikin-Ashi derived trend slope
/// Calculates slope of close prices of HA candles over period
pub struct HeikinAshiSlope {
    pub period: usize,
}

impl TechnicalIndicator for HeikinAshiSlope {
    fn name(&self) -> &'static str {
        "Heikin-Ashi Slope"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Compute Heikin-Ashi candles
        let mut ha_closes = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            if i == 0 {
                let ha_close = (candles[0].open + candles[0].high + candles[0].low + candles[0].close) / 4.0;
                ha_closes.push(ha_close);
            } else {
                let ha_close = (candles[i].open + candles[i].high + candles[i].low + candles[i].close) / 4.0;
                ha_closes.push(ha_close);
            }
        }

        // Calculate slope of HA close prices over rolling windows
        let mut slopes = vec![None; candles.len()];

        for i in self.period - 1..ha_closes.len() {
            let window = &ha_closes[i + 1 - self.period..=i];
            let n = self.period as f64;

            let sum_x = (0..self.period).map(|x| x as f64).sum::<f64>();
            let sum_y = window.iter().sum::<f64>();
            let sum_xx = (0..self.period).map(|x| (x as f64).powi(2)).sum::<f64>();
            let sum_xy = (0..self.period).map(|x| (x as f64) * window[x]).sum::<f64>();

            let denominator = n * sum_xx - sum_x * sum_x;
            if denominator != 0.0 {
                let slope = (n * sum_xy - sum_x * sum_y) / denominator;
                slopes[i] = Some(slope);
            } else {
                slopes[i] = None;
            }
        }

        slopes
    }
}