pub mod ema; 
pub mod rsi; 
pub mod sma;
pub mod macd;
pub mod bollinger_bands;
pub mod vwap;
pub mod atr;
pub mod stochastic;
pub mod cci;
pub mod adx;
pub mod parabolic_sar;
pub mod obv;
pub mod cmf;
pub mod williams_r;
pub mod ichimoku;
pub mod momentum;
pub mod tema;
pub mod dema;
pub mod kama;
pub mod wma;
pub mod hma;
pub mod frama;
pub mod chandelier_exit;
pub mod trix;
pub mod mfi;
pub mod force_index;
pub mod ease_of_movement;
pub mod accum_dist_line;
pub mod price_volume_trend;
pub mod volume_oscillator;
pub mod ultimate_oscillator;
pub mod detrended_price_oscillator;
pub mod roc;
pub mod z_score;
pub mod gmma;
pub mod schaff_trend_cycle;
pub mod fibonacci_retracement;
pub mod kalman_filter_smoother; 
pub mod heikin_ashi_slope; 
pub mod percent_b; 

pub use sma::SMA;
pub use ema::EMA;
pub use rsi::RSI;
pub use macd::MACD;
pub use bollinger_bands::BollingerBands;
pub use vwap::VWAP;
pub use atr::ATR;
pub use stochastic::Stochastic;
pub use cci::CCI;
pub use adx::ADX;
pub use parabolic_sar::ParabolicSAR;
pub use obv::OBV;
pub use cmf::CMF;
pub use williams_r::WilliamsR;
pub use ichimoku::Ichimoku;
pub use momentum::Momentum;
pub use tema::Tema;
pub use dema::Dema;
pub use kama::Kama;
pub use wma::WMA;
pub use hma::Hma;
pub use frama::Frama;
pub use chandelier_exit::ChandelierExit;
pub use trix::TRIX;
pub use mfi::MFI;
pub use force_index::ForceIndex;
pub use ease_of_movement::EaseOfMovement;
pub use accum_dist_line::AccumDistLine;
pub use price_volume_trend::PriceVolumeTrend;
pub use volume_oscillator::VolumeOscillator;
pub use ultimate_oscillator::UltimateOscillator;
pub use detrended_price_oscillator::DetrendedPriceOscillator;
pub use roc::RateOfChange;
pub use z_score::ZScore;
pub use gmma::GMMA;
pub use schaff_trend_cycle::SchaffTrendCycle;
pub use fibonacci_retracement::FibonacciRetracement;
pub use heikin_ashi_slope::HeikinAshiSlope;
pub use kalman_filter_smoother::KalmanFilterSmoother;
pub use percent_b::PercentB;


use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndicatorParam {
    pub name: String,
    pub param_type: String, // "int", "float", "bool"
    pub default_value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndicatorOptions {
    pub values: HashMap<String, serde_json::Value>,
}

pub trait TechnicalIndicator: Sync + Send {
    fn name(&self) -> &'static str;
    fn group(&self) -> &'static str; // e.g., "Trend", "Volume", "Oscillator"
    fn params(&self) -> Vec<IndicatorParam>;
    fn compute(&self, candles: &[Candle], options: &IndicatorOptions) -> Vec<Option<f64>>;
}