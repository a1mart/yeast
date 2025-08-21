use crate::Candle;
use crate::indicators::TechnicalIndicator;

/// Kalman Filter Smoother (1D version)
/// Experimental signal/noise separation with simple Kalman filter
pub struct KalmanFilterSmoother {
    pub process_variance: f64,
    pub measurement_variance: f64,
}

impl TechnicalIndicator for KalmanFilterSmoother {
    fn name(&self) -> &'static str {
        "Kalman Filter Smoother"
    }

    fn compute(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        let measurements: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut estimates = vec![None; candles.len()];

        let mut x = measurements[0]; // initial estimate
        let mut p = 1.0; // initial estimation error covariance

        for (i, &z) in measurements.iter().enumerate() {
            // Prediction update
            p = p + self.process_variance;

            // Measurement update (Kalman Gain)
            let k = p / (p + self.measurement_variance);

            // Update estimate with measurement z
            x = x + k * (z - x);

            // Update error covariance
            p = (1.0 - k) * p;

            estimates[i] = Some(x);
        }

        estimates
    }
}