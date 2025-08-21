use crate::Candle;
use crate::indicators::TechnicalIndicator;

pub struct Kama {
    pub period: usize,
}

impl TechnicalIndicator for Kama {
    fn name(&self) -> &'static str {
        "Kaufman Adaptive Moving Average (KAMA)"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();

        if prices.len() < self.period + 1 {
            return vec![None; prices.len()];
        }

        let mut kama = vec![None; prices.len()];
        let mut prev_kama = prices[self.period - 1];

        for i in self.period..prices.len() {
            let change = (prices[i] - prices[i - self.period]).abs();
            let volatility: f64 = prices[i - self.period + 1..=i]
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .sum();

            let er = if volatility != 0.0 { change / volatility } else { 0.0 };
            let sc = (er * (2.0 / (2.0 + 1.0) - 2.0 / (30.0 + 1.0)) + 2.0 / (30.0 + 1.0)).powi(2);

            prev_kama = prev_kama + sc * (prices[i] - prev_kama);
            kama[i] = Some(prev_kama);
        }

        kama
    }
}