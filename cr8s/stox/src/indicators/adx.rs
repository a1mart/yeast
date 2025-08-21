// src/indicators/adx.rs

use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct ADX {
    pub period: usize,
}

impl TechnicalIndicator for ADX {
    fn name(&self) -> &'static str {
        "ADX"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let period = self.period;
        let len = candles.len();
        let mut adx = vec![None; len];

        if len < period + 1 {
            return adx; // Not enough data
        }

        let mut plus_dm = vec![0.0; len];
        let mut minus_dm = vec![0.0; len];
        let mut tr = vec![0.0; len];

        // Calculate +DM, -DM and TR
        for i in 1..len {
            let up_move = candles[i].high - candles[i - 1].high;
            let down_move = candles[i - 1].low - candles[i].low;

            plus_dm[i] = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
            minus_dm[i] = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };

            let high_low = candles[i].high - candles[i].low;
            let high_close = (candles[i].high - candles[i - 1].close).abs();
            let low_close = (candles[i].low - candles[i - 1].close).abs();

            tr[i] = high_low.max(high_close).max(low_close);
        }

        // Smooth TR, +DM, -DM with Wilder's smoothing
        let mut tr_smooth = vec![0.0; len];
        let mut plus_dm_smooth = vec![0.0; len];
        let mut minus_dm_smooth = vec![0.0; len];

        tr_smooth[period] = tr[1..=period].iter().sum();
        plus_dm_smooth[period] = plus_dm[1..=period].iter().sum();
        minus_dm_smooth[period] = minus_dm[1..=period].iter().sum();

        for i in (period + 1)..len {
            tr_smooth[i] = tr_smooth[i - 1] - (tr_smooth[i - 1] / period as f64) + tr[i];
            plus_dm_smooth[i] = plus_dm_smooth[i - 1] - (plus_dm_smooth[i - 1] / period as f64) + plus_dm[i];
            minus_dm_smooth[i] = minus_dm_smooth[i - 1] - (minus_dm_smooth[i - 1] / period as f64) + minus_dm[i];
        }

        // Calculate +DI and -DI
        let mut plus_di = vec![0.0; len];
        let mut minus_di = vec![0.0; len];
        for i in period..len {
            if tr_smooth[i] != 0.0 {
                plus_di[i] = 100.0 * plus_dm_smooth[i] / tr_smooth[i];
                minus_di[i] = 100.0 * minus_dm_smooth[i] / tr_smooth[i];
            }
        }

        // Calculate DX
        let mut dx = vec![0.0; len];
        for i in period..len {
            let sum = plus_di[i] + minus_di[i];
            let diff = (plus_di[i] - minus_di[i]).abs();
            if sum != 0.0 {
                dx[i] = 100.0 * diff / sum;
            }
        }

        // Early return if not enough data
        if len < period * 2 {
            return adx;
        }

        // Smooth DX to get ADX using Wilder's smoothing
        adx[period * 2 - 1] = Some(dx[period..period * 2].iter().sum::<f64>() / period as f64);

        for i in (period * 2)..len {
            adx[i] = Some(((adx[i - 1].unwrap_or(0.0) * (period as f64 - 1.0)) + dx[i]) / period as f64);
        }

        adx
    }
}
