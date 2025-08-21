

pub struct StochasticRSI {
    pub period: usize,
    pub smooth_k: usize,
    pub smooth_d: usize,
}

impl TechnicalIndicator for StochasticRSI {
    fn name(&self) -> &'static str {
        "Stochastic RSI"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        fn rsi(period: usize, prices: &[f64]) -> Vec<Option<f64>> {
            let mut gains = vec![0.0; prices.len()];
            let mut losses = vec![0.0; prices.len()];

            for i in 1..prices.len() {
                let change = prices[i] - prices[i - 1];
                if change > 0.0 {
                    gains[i] = change;
                } else {
                    losses[i] = -change;
                }
            }

            let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
            let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

            let mut rsi = vec![None; period];
            rsi.push(Some(100.0 - (100.0 / (1.0 + avg_gain / avg_loss))));

            for i in (period + 1)..prices.len() {
                avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
                avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;

                let rs = if avg_loss == 0.0 { 0.0 } else { avg_gain / avg_loss };
                rsi.push(Some(100.0 - (100.0 / (1.0 + rs))));
            }

            rsi
        }

        fn sma(period: usize, values: &[Option<f64>]) -> Vec<Option<f64>> {
            let mut sma_vals = vec![None; values.len()];
            for i in period - 1..values.len() {
                let window = &values[i + 1 - period..=i];
                if window.iter().any(|v| v.is_none()) {
                    sma_vals[i] = None;
                } else {
                    let sum: f64 = window.iter().map(|v| v.unwrap()).sum();
                    sma_vals[i] = Some(sum / period as f64);
                }
            }
            sma_vals
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let rsi_vals = rsi(self.period, &closes);

        // Stochastic RSI %K = SMA of RSI over smooth_k
        let mut stoch_rsi_k = sma(self.smooth_k, &rsi_vals);

        // Stochastic RSI %D = SMA of %K over smooth_d
        let stoch_rsi_d = sma(self.smooth_d, &stoch_rsi_k);

        stoch_rsi_k = stoch_rsi_k.into_iter().map(|v| v.map(|val| val / 100.0)).collect();

        stoch_rsi_k
    }
}