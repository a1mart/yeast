use crate::indicators::TechnicalIndicator;
use crate::types::Candle;

pub struct Frama {
    pub period: usize,
}

impl TechnicalIndicator for Frama {
    fn name(&self) -> &'static str {
        "Fractal Adaptive Moving Average (FRAMA)"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();

        if prices.len() < self.period {
            return vec![None; prices.len()];
        }

        let mut frama = vec![None; prices.len()];
        let mut prev = prices[self.period - 1];

        for i in self.period..prices.len() {
            let window = &prices[i - self.period..i];
            let half = self.period / 2;

            let hl1 = window[..half].iter().fold((f64::MIN, f64::MAX), |(h, l), &v| (h.max(v), l.min(v)));
            let hl2 = window[half..].iter().fold((f64::MIN, f64::MAX), |(h, l), &v| (h.max(v), l.min(v)));
            let hl_all = window.iter().fold((f64::MIN, f64::MAX), |(h, l), &v| (h.max(v), l.min(v)));

            let n1 = (hl1.0 - hl1.1) / (half as f64);
            let n2 = (hl2.0 - hl2.1) / (half as f64);
            let n3 = (hl_all.0 - hl_all.1) / (self.period as f64);

            let dim = if n1 > 0.0 && n2 > 0.0 && n3 > 0.0 {
                ((n1 + n2) / n3).log2().abs()
            } else {
                1.0
            };

            let alpha = (-4.6 * (dim - 1.0)).exp();
            prev = alpha * prices[i] + (1.0 - alpha) * prev;
            frama[i] = Some(prev);
        }

        frama
    }
}