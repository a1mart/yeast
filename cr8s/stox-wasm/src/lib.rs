use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use lazy_static::lazy_static;
use wasm_bindgen::prelude::*;

mod indicators;

use crate::indicators::{
    TechnicalIndicator, IndicatorOptions, IndicatorParam, Candle, 
    SMA, EMA, RSI, MACD, BollingerBands, VWAP, ATR, Stochastic, CCI, ADX, ParabolicSAR, OBV,
    CMF, WilliamsR, Ichimoku, Momentum, Tema, Dema, Kama, WMA, Hma, Frama, ChandelierExit,
    TRIX, MFI, ForceIndex, EaseOfMovement, AccumDistLine, PriceVolumeTrend, VolumeOscillator,
    UltimateOscillator, DetrendedPriceOscillator, RateOfChange, ZScore, GMMA, SchaffTrendCycle,
    FibonacciRetracement, KalmanFilterSmoother, HeikinAshiSlope, PercentB,
};


// ======================
// Indicator Registry
// ======================
lazy_static! {
    pub static ref INDICATOR_REGISTRY: HashMap<&'static str, Arc<dyn TechnicalIndicator>> = {
        let mut map = HashMap::new();
        map.insert("rsi", Arc::new(RSI::new()) as Arc<dyn TechnicalIndicator>);
        map.insert("ema", Arc::new(EMA::new()) as Arc<dyn TechnicalIndicator>);
        map.insert("sma", Arc::new(SMA::new()) as Arc<dyn TechnicalIndicator>);

        map.insert("williams_r", Arc::new(WilliamsR::new()));
        map.insert("ichimoku", Arc::new(Ichimoku::new()));
        map.insert("momentum", Arc::new(Momentum::new()));
        map.insert("tema", Arc::new(Tema::new()));
        map.insert("dema", Arc::new(Dema::new()));
        map.insert("kama", Arc::new(Kama::new()));
        map.insert("wma", Arc::new(WMA::new()));
        map.insert("hma", Arc::new(Hma::new()));
        map.insert("frama", Arc::new(Frama::new()));
        map.insert("chandelier_exit", Arc::new(ChandelierExit::new()));
        map.insert("trix", Arc::new(TRIX::new()));
        map.insert("mfi", Arc::new(MFI::new()));
        map.insert("force_index", Arc::new(ForceIndex::new()));
        map.insert("ease_of_movement", Arc::new(EaseOfMovement::new()));
        map.insert("accum_dist_line", Arc::new(AccumDistLine::new()));
        map.insert("price_volume_trend", Arc::new(PriceVolumeTrend::new()));
        map.insert("volume_oscillator", Arc::new(VolumeOscillator::new()));
        map.insert("ultimate_oscillator", Arc::new(UltimateOscillator::new()));
        map.insert("detrended_price_oscillator", Arc::new(DetrendedPriceOscillator::new()));
        map.insert("roc", Arc::new(RateOfChange::new()));
        map.insert("z_score", Arc::new(ZScore::new()));
        map.insert("gmma", Arc::new(GMMA::new()));
        map.insert("schaff_trend_cycle", Arc::new(SchaffTrendCycle::new()));
        map.insert("fibonacci_retracement", Arc::new(FibonacciRetracement::new()));
        map.insert("kalman_filter_smoother", Arc::new(KalmanFilterSmoother::new()));
        map.insert("heikin_ashi_slope", Arc::new(HeikinAshiSlope::new()));
        map.insert("percent_b", Arc::new(PercentB::new()));

        map
    };
}

// ======================
// WASM Exports
// ======================
#[wasm_bindgen]
pub fn get_indicators() -> JsValue {
    let indicators: Vec<_> = INDICATOR_REGISTRY
        .iter()
        .map(|(key, indicator)| {
            json!({
                "key": key,
                "name": indicator.name(),
                "group": indicator.group(),
                "params": indicator.params()
            })
        })
        .collect();

    JsValue::from_serde(&indicators).unwrap()
}

#[wasm_bindgen]
pub fn compute_indicator(key: &str, candles: JsValue, options: JsValue) -> JsValue {
    let candles: Vec<Candle> = candles.into_serde().unwrap();
    let options: IndicatorOptions = options.into_serde().unwrap();

    if let Some(indicator) = INDICATOR_REGISTRY.get(key) {
        let result = indicator.compute(&candles, &options);
        JsValue::from_serde(&result).unwrap()
    } else {
        JsValue::from_str("Indicator not found")
    }
}

#[wasm_bindgen]
pub fn compute_batch(requests: JsValue) -> JsValue {
    let requests: Vec<(String, Vec<Candle>, IndicatorOptions)> = requests.into_serde().unwrap();
    let mut results = HashMap::new();
    for (key, candles, options) in requests {
        if let Some(indicator) = INDICATOR_REGISTRY.get(key.as_str()) {
            results.insert(key.clone(), indicator.compute(&candles, &options));
        }
    }
    JsValue::from_serde(&results).unwrap()
}

/*
const wasm = await import('/wasm/wasm.js');
await wasm.default();

// Get all indicators
const indicators = JSON.parse(wasm.get_indicators());
console.log(indicators);

// Compute RSI
const candles = [
  { timestamp: 1, open: 100, high: 102, low: 99, close: 101, volume: null },
  { timestamp: 2, open: 101, high: 103, low: 100, close: 102, volume: null },
  // ... more candles
];

const options = { values: { period: 14 } };

const rsiResult = JSON.parse(
  wasm.compute_indicator("rsi", JSON.stringify(candles), JSON.stringify(options))
);
console.log(rsiResult);
*/