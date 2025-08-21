use crate::{TechnicalIndicator, Candle};

pub struct MFI {
    pub period: usize,
}

impl TechnicalIndicator for MFI {
    fn name(&self) -> &'static str {
        "MFI"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let typical_prices: Vec<f64> = candles.iter()
            .map(|c| (c.high + c.low + c.close) / 3.0)
            .collect();

        let mut positive_flow = vec![0.0; candles.len()];
        let mut negative_flow = vec![0.0; candles.len()];

        for i in 1..candles.len() {
            let raw_flow = typical_prices[i] * candles[i].volume.unwrap_or(0.0);
            if typical_prices[i] > typical_prices[i - 1] {
                positive_flow[i] = raw_flow;
            } else if typical_prices[i] < typical_prices[i - 1] {
                negative_flow[i] = raw_flow;
            }
        }

        let mut mfi = vec![None; candles.len()];
        for i in self.period..candles.len() {
            let pos_sum: f64 = positive_flow[i + 1 - self.period..=i].iter().sum();
            let neg_sum: f64 = negative_flow[i + 1 - self.period..=i].iter().sum();
            if neg_sum == 0.0 {
                mfi[i] = Some(100.0);
            } else {
                let money_flow_ratio = pos_sum / neg_sum;
                mfi[i] = Some(100.0 - (100.0 / (1.0 + money_flow_ratio)));
            }
        }
        mfi
    }
}