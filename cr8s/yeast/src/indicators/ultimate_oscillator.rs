use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct UltimateOscillator {
    pub short_period: usize,
    pub mid_period: usize,
    pub long_period: usize,
}

impl TechnicalIndicator for UltimateOscillator {
    fn name(&self) -> &'static str {
        "Ultimate Oscillator"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let len = candles.len();
        let mut bp = vec![None; len]; // Buying Pressure
        let mut tr = vec![None; len]; // True Range

        for i in 1..len {
            let low = candles[i].low;
            let prev_close = candles[i - 1].close;
            let high = candles[i].high;

            let min_low_close = low.min(prev_close);
            bp[i] = Some(candles[i].close - min_low_close);

            let max_high_close = high.max(prev_close);
            tr[i] = Some(max_high_close - min_low_close);
        }

        fn sum_period(vals: &[Option<f64>], period: usize, idx: usize) -> Option<f64> {
            if idx + 1 < period {
                return None;
            }
            let window = &vals[idx + 1 - period..=idx];
            let s: f64 = window.iter().filter_map(|v| *v).sum();
            Some(s)
        }

        let mut ult_osc = vec![None; len];

        for i in 0..len {
            let short_bp = sum_period(&bp, self.short_period, i);
            let mid_bp = sum_period(&bp, self.mid_period, i);
            let long_bp = sum_period(&bp, self.long_period, i);

            let short_tr = sum_period(&tr, self.short_period, i);
            let mid_tr = sum_period(&tr, self.mid_period, i);
            let long_tr = sum_period(&tr, self.long_period, i);

            if let (Some(sb), Some(mb), Some(lb), Some(st), Some(mt), Some(lt)) =
                (short_bp, mid_bp, long_bp, short_tr, mid_tr, long_tr)
            {
                let avg = (4.0 * (sb / st)) + (2.0 * (mb / mt)) + (lb / lt);
                ult_osc[i] = Some((avg / 7.0) * 100.0);
            }
        }

        ult_osc
    }
}