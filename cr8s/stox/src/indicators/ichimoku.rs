// src/indicators/ichimoku.rs

use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct Ichimoku {
    pub conversion_period: usize, // usually 9
    pub base_period: usize,       // usually 26
    pub leading_span_b_period: usize, // usually 52
    pub displacement: usize,      // usually 26
}

impl TechnicalIndicator for Ichimoku {
    fn name(&self) -> &'static str {
        "IchimokuCloud"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // For simplicity, we'll just return the Conversion Line (Tenkan-sen)
        // You can expand this to return multiple vectors or a struct later

        let mut tenkan_sen = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            if i + 1 < self.conversion_period {
                tenkan_sen.push(None);
                continue;
            }
            let window = &candles[i + 1 - self.conversion_period..=i];
            let highest_high = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let lowest_low = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            tenkan_sen.push(Some((highest_high + lowest_low) / 2.0));
        }

        tenkan_sen
    }
}
