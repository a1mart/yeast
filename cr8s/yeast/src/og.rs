use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::error::Error;
use std::sync::Arc;
use std::collections::HashMap;
use serde::Deserialize;
use reqwest;

// For async
use futures::future::BoxFuture;

// // internal
// mod ml_models;
// use ml_models::{
//     MLPipeline, LinearRegression, RandomForest, KMeans, ModelEvaluator, 
//     FeatureEngineer, FeatureSet
// };
// mod indicators;
// mod types;
// mod options_math;
use crate::types::Candle;
use crate::indicators::{
    SMA, EMA, RSI, MACD, BollingerBands, VWAP, ATR, Stochastic, CCI, ADX, ParabolicSAR, OBV,
    CMF, WilliamsR, Ichimoku, Momentum, Tema, Dema, Kama, WMA, Hma, Frama, ChandelierExit,
    TRIX, MFI, ForceIndex, EaseOfMovement, AccumDistLine, PriceVolumeTrend, VolumeOscillator,
    UltimateOscillator, DetrendedPriceOscillator, RateOfChange, ZScore, GMMA, SchaffTrendCycle,
    FibonacciRetracement, KalmanFilterSmoother, HeikinAshiSlope, PercentB,
    TechnicalIndicator, IndicatorRunner
};
use crate::options_math::{black_scholes_greeks, calculate_pnl, OptionData, OptionType};


// --- TLS module remains unchanged ---
mod tls { 
    use std::net::TcpStream;
    use std::sync::Arc;
    use rustls::client::{ClientConfig, ClientConnection};
    use rustls::{RootCertStore, StreamOwned};
    use rustls_native_certs::{load_native_certs, Certificate};
    use rustls::pki_types::CertificateDer;

    pub fn connect(domain: &'static str, port: u16) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
        let addr = format!("{}:{}", domain, port);
        let stream = TcpStream::connect(&addr).map_err(|e| format!("TCP connect error: {}", e))?;

        let mut root_store = RootCertStore::empty();

        let native_certs = load_native_certs()
            .map_err(|e| format!("Failed to load native certs: {:?}", e))?;

        let certs: Vec<Vec<u8>> = native_certs.into_iter().map(|cert| cert.0).collect();
        let cert_refs: Vec<&[u8]> = certs.iter().map(|v| v.as_slice()).collect();

        let certs_der: Vec<CertificateDer<'_>> = cert_refs.iter()
            .map(|&slice| CertificateDer::from(slice))
            .collect();

        root_store.add_parsable_certificates(certs_der);

        let config = ClientConfig::builder()
            //.with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let rc_config = Arc::new(config);

        let conn = ClientConnection::new(rc_config, domain.try_into().unwrap())
            .map_err(|e| format!("TLS connection error: {}", e))?;

        Ok(StreamOwned::new(conn, stream))
    }
}

#[derive(Debug)]
pub struct ChartQueryOptions<'a> {
    pub interval: &'a str,  // e.g., "1d", "1h"
    pub range: &'a str,     // e.g., "5d", "1mo"
}

impl Default for ChartQueryOptions<'_> {
    fn default() -> Self {
        Self {
            interval: "1d",
            range: "5d",
        }
    }
}

// Trait for both sync and async fetching
pub trait ChartFetcher {
    fn fetch_sync(&self, ticker: &str, opts: &ChartQueryOptions) -> Result<ChartResponse, Box<dyn Error>>;

    fn fetch_async<'a>(&'a self, ticker: &'a str, opts: &'a ChartQueryOptions) -> BoxFuture<'a, Result<ChartResponse, Box<dyn Error>>>;
}

// Sync implementation using native TLS + TcpStream
struct SyncFetcher;

impl SyncFetcher {
    fn fetch_yahoo_chart_for_ticker(ticker: &str, opts: &ChartQueryOptions) -> Result<String, String> {
        let domain = "query1.finance.yahoo.com";
        let path = format!(
            "/v8/finance/chart/{}?interval={}&range={}",
            ticker, opts.interval, opts.range
        );

        let mut stream = tls::connect(domain, 443)?;
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: stock-client/1.0\r\nConnection: close\r\n\r\n",
            path, domain
        );

        stream.write_all(request.as_bytes()).map_err(|e| format!("Write error: {}", e))?;

        let mut response = String::new();
        stream.read_to_string(&mut response).map_err(|e| format!("Read error: {}", e))?;

        if let Some(pos) = response.find("\r\n\r\n") {
            Ok(response[pos + 4..].to_string())
        } else {
            Err("Malformed HTTP response".into())
        }
    }
}

impl ChartFetcher for SyncFetcher {
    fn fetch_sync(&self, ticker: &str, opts: &ChartQueryOptions) -> Result<ChartResponse, Box<dyn Error>> {
        let json = Self::fetch_yahoo_chart_for_ticker(ticker, opts)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        let parsed = extract_all_data(&json)?;
        Ok(parsed)
    }

    fn fetch_async<'a>(&'a self, _ticker: &'a str, _opts: &'a ChartQueryOptions) -> BoxFuture<'a, Result<ChartResponse, Box<dyn Error>>> {
        // SyncFetcher doesn't support async fetch
        Box::pin(async { Err("SyncFetcher does not support async fetch".into()) })
    }
}

// Async implementation using reqwest
pub struct AsyncFetcher {
    client: reqwest::Client,
}

impl AsyncFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl ChartFetcher for AsyncFetcher {
    fn fetch_sync(&self, _ticker: &str, _opts: &ChartQueryOptions) -> Result<ChartResponse, Box<dyn Error>> {
        Err("AsyncFetcher does not support sync fetch".into())
    }

    fn fetch_async<'a>(&'a self, ticker: &'a str, opts: &'a ChartQueryOptions) -> BoxFuture<'a, Result<ChartResponse, Box<dyn Error>>> {
        let client = &self.client;
        let interval = opts.interval.to_string();
        let range = opts.range.to_string();
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range={}", ticker, interval, range);

        Box::pin(async move {
            let resp = client.get(&url)
                .header("User-Agent", "stock-client/1.0")
                .send()
                .await?
                .text()
                .await?;

            let parsed = extract_all_data(&resp)?;
            Ok(parsed)
        })
    }
}

// Your parsing structs & function remain unchanged here
#[derive(Debug, Deserialize)]
pub struct ChartResponse {
    pub chart: Chart,
}

#[derive(Debug, Deserialize)]
pub struct Chart {
    pub result: Option<Vec<ResultItem>>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ResultItem {
    pub meta: Meta,
    pub timestamp: Vec<u64>,
    pub indicators: Indicators,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub currency: String,
    pub symbol: String,
    pub exchangeName: String,
    pub fullExchangeName: String,
    pub instrumentType: String,
    pub firstTradeDate: u64,
    pub regularMarketTime: u64,
    pub hasPrePostMarketData: bool,
    pub gmtoffset: i64,
    pub timezone: String,
    pub exchangeTimezoneName: String,
    pub regularMarketPrice: f64,
    pub fiftyTwoWeekHigh: f64,
    pub fiftyTwoWeekLow: f64,
    pub regularMarketDayHigh: f64,
    pub regularMarketDayLow: f64,
    pub regularMarketVolume: u64,
    pub longName: String,
    pub shortName: String,
    pub chartPreviousClose: f64,
    pub priceHint: u8,
    pub currentTradingPeriod: TradingPeriodWrapper,
    pub dataGranularity: String,
    pub range: String,
    pub validRanges: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TradingPeriodWrapper {
    pub pre: TradingPeriod,
    pub regular: TradingPeriod,
    pub post: TradingPeriod,
}

#[derive(Debug, Deserialize)]
pub struct TradingPeriod {
    pub timezone: String,
    pub end: u64,
    pub start: u64,
    pub gmtoffset: i64,
}

#[derive(Debug, Deserialize)]
pub struct Indicators {
    pub quote: Option<Vec<Quote>>,
    pub adjclose: Option<Vec<AdjClose>>,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub close: Option<Vec<Option<f64>>>,
    pub open: Option<Vec<Option<f64>>>,
    pub volume: Option<Vec<Option<u64>>>,
    pub high: Option<Vec<Option<f64>>>,
    pub low: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
pub struct AdjClose {
    pub adjclose: Option<Vec<Option<f64>>>,
}

fn extract_all_data(json: &str) -> Result<ChartResponse, Box<dyn Error>> {
    serde_json::from_str(json).map_err(|e| -> Box<dyn std::error::Error> { e.into() })
}

async fn fetch_nasdaq_symbols_csv() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = "https://datahub.io/core/nasdaq-listings/r/nasdaq-listed-symbols.csv";
    let resp = reqwest::get(url).await?.text().await?;
    let mut lines = resp.lines();
    let header = lines.next(); // ignore header

    let mut tickers = Vec::new();
    for line in lines {
        if line.trim().is_empty() { continue; }
        let symbol = line.split(',').next().unwrap_or("").to_string();
        tickers.push(symbol);
    }
    Ok(tickers)
}

pub fn to_candles(result: &ResultItem) -> Vec<Candle> {
    let mut candles = Vec::new();
    if let Some(quote_vec) = &result.indicators.quote {
        if let Some(quote) = quote_vec.get(0) {
            let close = quote.close.as_ref();
            let open = quote.open.as_ref();
            let high = quote.high.as_ref();
            let low = quote.low.as_ref();
            let volume = quote.volume.as_ref();
            let timestamps = &result.timestamp;

            for i in 0..timestamps.len() {
                if let (Some(Some(c)), Some(Some(o)), Some(Some(h)), Some(Some(l)), Some(Some(v))) =
                    (close.and_then(|v| v.get(i)), open.and_then(|v| v.get(i)),
                     high.and_then(|v| v.get(i)), low.and_then(|v| v.get(i)),
                     volume.and_then(|v| v.get(i)))
                {
                    candles.push(Candle {
                        timestamp: timestamps[i].try_into().unwrap(),
                        open: *o,
                        high: *h,
                        low: *l,
                        close: *c,
                        volume: Some(*v as f64),
                    });
                }
            }
        }
    }
    candles
}

pub fn build_indicators() -> Vec<(String, Arc<dyn TechnicalIndicator + Send + Sync>)> {
    vec![
        ("SMA(5)".to_string(), Arc::new(SMA { period: 5 })),
        ("EMA(5)".to_string(), Arc::new(EMA { period: 5 })),
        ("RSI(14)".to_string(), Arc::new(RSI { period: 14 })),
        ("MACD(12,26)".to_string(), Arc::new(MACD { fast_period: 12, slow_period: 26 })),
        ("BollingerBands(20)".to_string(), Arc::new(BollingerBands { period: 20, k: 2.0 })),
        ("VWAP".to_string(), Arc::new(VWAP {})),
        ("ATR(14)".to_string(), Arc::new(ATR { period: 14 })),
        ("Stochastic(14,3)".to_string(), Arc::new(Stochastic { k_period: 14, d_period: 3 })),
        ("CCI(20)".to_string(), Arc::new(CCI { period: 20 })),
        ("ADX(14)".to_string(), Arc::new(ADX { period: 14 })),
        ("ParabolicSAR".to_string(), Arc::new(ParabolicSAR { step: 0.02, max_step: 0.2 })),
        ("OBV".to_string(), Arc::new(OBV {})),
        ("CMF(20)".to_string(), Arc::new(CMF { period: 20 })),
        ("WilliamsR(14)".to_string(), Arc::new(WilliamsR { period: 14 })),
        ("Ichimoku".to_string(), Arc::new(Ichimoku {
            conversion_period: 9,
            base_period: 26,
            leading_span_b_period: 52,
            displacement: 26,
        })),
        ("Momentum(10)".to_string(), Arc::new(Momentum { period: 10 })),
        ("Tema(10)".to_string(), Arc::new(Tema { period: 10 })),
        ("Dema(10)".to_string(), Arc::new(Dema { period: 10 })),
        ("Kama(10)".to_string(), Arc::new(Kama { period: 10 })),
        ("WMA(10)".to_string(), Arc::new(WMA { period: 10 })),
        ("HMA(10)".to_string(), Arc::new(Hma { period: 10 })),
        ("Frama(10)".to_string(), Arc::new(Frama { period: 10 })),
        ("ChandelierExit(22, 3.0)".to_string(), Arc::new(ChandelierExit { period: 22, atr_multiplier: 3.0 })),
        ("TRIX(15)".to_string(), Arc::new(TRIX { period: 15 })),
        ("MFI(14)".to_string(), Arc::new(MFI { period: 14 })),
        ("ForceIndex(13)".to_string(), Arc::new(ForceIndex { period: 13 })),
        ("EaseOfMovement(14)".to_string(), Arc::new(EaseOfMovement { period: 14 })),
        ("AccumDistLine".to_string(), Arc::new(AccumDistLine {})),
        ("PriceVolumeTrend".to_string(), Arc::new(PriceVolumeTrend {})),
        ("VolumeOscillator(14,28)".to_string(), Arc::new(VolumeOscillator { short_period: 14, long_period: 28 })),
        ("UltimateOscillator(7,14,28)".to_string(), Arc::new(UltimateOscillator {
            short_period: 7,
            mid_period: 14,
            long_period: 28,
        })),
        ("DetrendedPriceOscillator(20)".to_string(), Arc::new(DetrendedPriceOscillator { period: 20 })),
        ("RateOfChange(12)".to_string(), Arc::new(RateOfChange { period: 12 })),
        ("ZScore(20)".to_string(), Arc::new(ZScore { period: 20 })),
        ("GMMA".to_string(), Arc::new(GMMA {
            short_periods: vec![3, 5, 8, 10, 12, 15],
            long_periods: vec![30, 35, 40, 45, 50, 60],
        })),
        ("SchaffTrendCycle".to_string(), Arc::new(SchaffTrendCycle {
            cycle_period: 10,
            fast_k: 23,
            fast_d: 50,
            short_period: 50,
            long_period: 50,
        })),
        ("FibonacciRetracement(14)".to_string(), Arc::new(FibonacciRetracement { period: 14 })),
        ("KalmanFilterSmoother".to_string(), Arc::new(KalmanFilterSmoother {
            measurement_variance: 1.0,
            process_variance: 1.0,
        })),
        ("HeikinAshiSlope(10)".to_string(), Arc::new(HeikinAshiSlope { period: 10 })),
        ("PercentB(20, 2.0)".to_string(), Arc::new(PercentB { period: 20, std_dev_mult: 2.0 })),
    ]
}

// OPTIONS CHAIN
#[derive(Debug, Deserialize)]
pub struct OptionProfitCalculatorResponse {
    pub options: HashMap<String, ExpiryOptionData>,
}

#[derive(Debug, Deserialize)]
pub struct ExpiryOptionData {
    pub c: HashMap<String, OptionQuote>,
    pub p: HashMap<String, OptionQuote>,
}

#[derive(Debug, Deserialize)]
pub struct OptionQuote {
    pub oi: u64,
    pub l: f64,
    pub b: f64,
    pub a: f64,
    pub v: u64,
}

pub trait OptionsFetcher {
    fn fetch_sync(&self, ticker: &str) -> Result<OptionProfitCalculatorResponse, Box<dyn Error>>;

    fn fetch_async<'a>(&'a self, ticker: &'a str) -> BoxFuture<'a, Result<OptionProfitCalculatorResponse, Box<dyn Error>>>;
}

struct SyncOptionsFetcher;

impl SyncOptionsFetcher {
    fn fetch_options_for_ticker(ticker: &str) -> Result<String, String> {
        let domain = "www.optionsprofitcalculator.com";
        let path = format!("/ajax/getOptions?stock={}&reqId=1", ticker);

        let mut stream = tls::connect(domain, 443)?;
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: stock-client/1.0\r\nConnection: close\r\n\r\n",
            path, domain
        );

        stream.write_all(request.as_bytes()).map_err(|e| format!("Write error: {}", e))?;

        let mut response = String::new();
        stream.read_to_string(&mut response).map_err(|e| format!("Read error: {}", e))?;

        if let Some(pos) = response.find("\r\n\r\n") {
            Ok(response[pos + 4..].to_string())
        } else {
            Err("Malformed HTTP response".into())
        }
    }
}

impl OptionsFetcher for SyncOptionsFetcher {
    fn fetch_sync(&self, ticker: &str) -> Result<OptionProfitCalculatorResponse, Box<dyn Error>> {
        let json = Self::fetch_options_for_ticker(ticker)
            .map_err(|e| -> Box<dyn Error> { e.into() })?;
        let parsed: OptionProfitCalculatorResponse = serde_json::from_str(&json)?;
        Ok(parsed)
    }

    fn fetch_async<'a>(&'a self, _ticker: &'a str) -> BoxFuture<'a, Result<OptionProfitCalculatorResponse, Box<dyn Error>>> {
        // SyncOptionsFetcher doesn't support async fetch
        Box::pin(async { Err("SyncOptionsFetcher does not support async fetch".into()) })
    }
}

pub struct AsyncOptionsFetcher {
    client: reqwest::Client,
}

impl AsyncOptionsFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl OptionsFetcher for AsyncOptionsFetcher {
    fn fetch_sync(&self, _ticker: &str) -> Result<OptionProfitCalculatorResponse, Box<dyn Error>> {
        Err("AsyncOptionsFetcher does not support sync fetch".into())
    }

    fn fetch_async<'a>(&'a self, ticker: &'a str) -> BoxFuture<'a, Result<OptionProfitCalculatorResponse, Box<dyn Error>>> {
        let client = &self.client;
        let url = format!("https://www.optionsprofitcalculator.com/ajax/getOptions?stock={}&reqId=1", ticker);

        Box::pin(async move {
            let resp = client.get(&url)
                .header("User-Agent", "stock-client/1.0")
                .send()
                .await?
                .text()
                .await?;

            let parsed: OptionProfitCalculatorResponse = serde_json::from_str(&resp)?;
            Ok(parsed)
        })
    }
}

fn print_opc_option_chain(data: OptionProfitCalculatorResponse) {
    for (expiry, exp_data) in data.options {
        println!("Expiration Date: {}", expiry);

        println!("Calls:");
        for (strike, quote) in exp_data.c {
            println!(
                "  Strike: {:>8} | Bid: {:>7.2} | Ask: {:>7.2} | Last: {:>7.2} | Vol: {:>5} | OI: {:>5}",
                strike, quote.b, quote.a, quote.l, quote.v, quote.oi
            );
        }

        println!("Puts:");
        for (strike, quote) in exp_data.p {
            println!(
                "  Strike: {:>8} | Bid: {:>7.2} | Ask: {:>7.2} | Last: {:>7.2} | Vol: {:>5} | OI: {:>5}",
                strike, quote.b, quote.a, quote.l, quote.v, quote.oi
            );
        }

        println!("---");
    }
}

fn convert_to_option_data(
    strike_str: &str,
    option_type: OptionType,
    quote: &OptionQuote,
) -> OptionData {
    OptionData {
        strike: strike_str.parse().unwrap_or(0.0),
        option_type,
        open_interest: quote.oi,
        bid: quote.b,
        ask: quote.a,
        last: quote.l,
        volume: quote.v,
    }
}

fn print_opc_option_chain_with_greeks(data: OptionProfitCalculatorResponse, underlying_price: f64, time_to_expiry: f64, risk_free_rate: f64, volatility: f64) {
    for (expiry, exp_data) in data.options {
        println!("Expiration Date: {}", expiry);

        println!("Calls:");
        for (strike, quote) in &exp_data.c {
            let option = convert_to_option_data(strike, OptionType::Call, quote);
            let greeks = black_scholes_greeks(
                underlying_price,
                option.strike,
                time_to_expiry,
                risk_free_rate,
                volatility,
                option.option_type,
            );
            let pnl = calculate_pnl(10.0, option.last, greeks.price);
            println!(
                "  Strike: {:>8.2} | Price: {:>7.2} | Delta: {:>7.4} | Gamma: {:>7.4} | PnL(10): {:>7.2} | Bid: {:>7.2} | Ask: {:>7.2} | OI: {:>5}",
                option.strike, greeks.price, greeks.delta, greeks.gamma, pnl, option.bid, option.ask, option.open_interest
            );
        }

        println!("Puts:");
        for (strike, quote) in &exp_data.p {
            let option = convert_to_option_data(strike, OptionType::Put, quote);
            let greeks = black_scholes_greeks(
                underlying_price,
                option.strike,
                time_to_expiry,
                risk_free_rate,
                volatility,
                option.option_type,
            );
            let pnl = calculate_pnl(10.0, option.last, greeks.price);
            println!(
                "  Strike: {:>8.2} | Price: {:>7.2} | Delta: {:>7.4} | Gamma: {:>7.4} | PnL(10): {:>7.2} | Bid: {:>7.2} | Ask: {:>7.2} | OI: {:>5}",
                option.strike, greeks.price, greeks.delta, greeks.gamma, pnl, option.bid, option.ask, option.open_interest
            );
        }

        println!("---");
    }
}

// Enhanced analysis with ML capabilities
#[derive(Debug, Clone)]
pub struct MLAnalysisConfig {
    pub prediction_model: String, // "linear", "random_forest"
    pub lookback_window: usize,
    pub train_test_split: f64,
    pub enable_clustering: bool,
    pub n_clusters: usize,
    pub risk_free_rate: f64,
}

impl Default for MLAnalysisConfig {
    fn default() -> Self {
        Self {
            prediction_model: "random_forest".to_string(),
            lookback_window: 20,
            train_test_split: 0.8,
            enable_clustering: true,
            n_clusters: 3,
            risk_free_rate: 0.02, // 2% annual risk-free rate
        }
    }
}

// pub struct StockMLAnalyzer {
//     pub symbol: String,
//     pub candles: Vec<Candle>,
//     pub indicators: HashMap<String, Vec<Option<f64>>>,
//     pub ml_pipeline: MLPipeline,
//     pub config: MLAnalysisConfig,
// }

// impl StockMLAnalyzer {
//     pub fn new(symbol: String, config: MLAnalysisConfig) -> Self {
//         let model: Box<dyn ml_models::MLModel> = match config.prediction_model.as_str() {
//             "linear" => Box::new(LinearRegression::new(0.01, 1000)),
//             "random_forest" => Box::new(RandomForest::new(20, 8)),
//             _ => Box::new(RandomForest::new(20, 8)),
//         };

//         Self {
//             symbol,
//             candles: Vec::new(),
//             indicators: HashMap::new(),
//             ml_pipeline: MLPipeline::new(model),
//             config,
//         }
//     }

//     pub fn load_data(&mut self, candles: Vec<Candle>, indicators: HashMap<String, Vec<Option<f64>>>) {
//         self.candles = candles;
//         self.indicators = indicators;
//     }

//     pub fn run_analysis(&mut self) -> Result<MLAnalysisResults, Box<dyn Error>> {
//         if self.candles.len() < self.config.lookback_window * 2 {
//             return Err("Insufficient data for ML analysis".into());
//         }

//         println!("🤖 Running ML Analysis for {}", self.symbol);
//         println!("═══════════════════════════════════════");

//         // 1. Feature Engineering
//         let features = self.extract_and_prepare_features()?;
//         println!("✓ Extracted {} feature sets with {} features each", 
//                  features.len(), features[0].features.len());

//         // 2. Train-Test Split
//         let split_idx = (features.len() as f64 * self.config.train_test_split) as usize;
//         let (train_features, test_features) = features.split_at(split_idx);

//         // 3. Train Prediction Model
//         self.ml_pipeline.train_prediction_model(
//             &self.candles, 
//             &self.indicators, 
//             self.config.lookback_window
//         )?;
//         println!("✓ Trained {} model", self.config.prediction_model);

//         // 4. Make Predictions and Evaluate
//         let predictions = self.evaluate_model(test_features)?;
//         println!("✓ Generated {} predictions", predictions.len());

//         // 5. Feature Importance Analysis
//         let feature_importance = self.ml_pipeline.get_feature_importance();
        
//         // 6. Risk Analysis
//         let risk_metrics = self.calculate_risk_metrics()?;

//         // 7. Trading Signals
//         let signals = self.generate_trading_signals(&predictions)?;

//         // 8. Clustering Analysis (if enabled)
//         let cluster_results = if self.config.enable_clustering {
//             Some(self.perform_clustering_analysis()?)
//         } else {
//             None
//         };

//         Ok(MLAnalysisResults {
//             symbol: self.symbol.clone(),
//             predictions,
//             feature_importance,
//             risk_metrics,
//             signals,
//             cluster_results,
//         })
//     }

//     fn extract_and_prepare_features(&self) -> Result<Vec<FeatureSet>, Box<dyn Error>> {
//         let features = FeatureEngineer::extract_features(
//             &self.candles,
//             &self.indicators,
//             self.config.lookback_window,
//         );

//         if features.is_empty() {
//             return Err("No features could be extracted".into());
//         }

//         Ok(features)
//     }

//     fn evaluate_model(&self, test_features: &[FeatureSet]) -> Result<Vec<PredictionResult>, Box<dyn Error>> {
//         let mut predictions = Vec::new();
//         let mut actual_values = Vec::new();
//         let mut predicted_values = Vec::new();

//         for (i, feature_set) in test_features.iter().enumerate() {
//             if let Some(actual) = feature_set.target {
//                 match self.ml_pipeline.model.predict(&feature_set.features) {
//                     Ok(predicted) => {
//                         predictions.push(PredictionResult {
//                             index: i,
//                             actual_return: actual,
//                             predicted_return: predicted,
//                             confidence: self.calculate_prediction_confidence(predicted),
//                         });
//                         actual_values.push(actual);
//                         predicted_values.push(predicted);
//                     }
//                     Err(e) => eprintln!("Prediction error: {}", e),
//                 }
//             }
//         }

//         // Calculate evaluation metrics
//         if !actual_values.is_empty() {
//             let mse = ModelEvaluator::mean_squared_error(&actual_values, &predicted_values);
//             let mae = ModelEvaluator::mean_absolute_error(&actual_values, &predicted_values);
//             let r2 = ModelEvaluator::r_squared(&actual_values, &predicted_values);

//             println!("📊 Model Performance Metrics:");
//             println!("   Mean Squared Error: {:.6}", mse);
//             println!("   Mean Absolute Error: {:.6}", mae);
//             println!("   R-squared: {:.4}", r2);
            
//             // Directional accuracy
//             let correct_direction = actual_values.iter()
//                 .zip(&predicted_values)
//                 .filter(|(actual, predicted)| {
//                     (**actual > 0.0 && **predicted > 0.0) || (**actual <= 0.0 && **predicted <= 0.0)
//                 })
//                 .count();
            
//             let directional_accuracy = correct_direction as f64 / actual_values.len() as f64;
//             println!("   Directional Accuracy: {:.2}%", directional_accuracy * 100.0);
//         }

//         Ok(predictions)
//     }

//     fn calculate_prediction_confidence(&self, predicted_return: f64) -> f64 {
//         // Simple confidence based on magnitude of prediction
//         // In practice, you might use model uncertainty estimates
//         1.0 - (-predicted_return.abs() * 10.0).exp()
//     }

//     fn calculate_risk_metrics(&self) -> Result<RiskMetrics, Box<dyn Error>> {
//         if self.candles.len() < 2 {
//             return Err("Insufficient data for risk calculation".into());
//         }

//         // Calculate returns
//         let returns: Vec<f64> = self.candles.windows(2)
//             .map(|w| (w[1].close - w[0].close) / w[0].close)
//             .collect();

//         // Extract prices for drawdown calculation
//         let prices: Vec<f64> = self.candles.iter().map(|c| c.close).collect();

//         let volatility = returns.iter()
//             .map(|r| r.powi(2))
//             .sum::<f64>()
//             .sqrt() / (returns.len() as f64).sqrt();

//         let sharpe_ratio = ModelEvaluator::sharpe_ratio(&returns, self.config.risk_free_rate / 252.0);
//         let max_drawdown = ModelEvaluator::maximum_drawdown(&prices);
        
//         // Value at Risk (95% confidence)
//         let mut sorted_returns = returns.clone();
//         sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
//         let var_95 = sorted_returns[(returns.len() as f64 * 0.05) as usize];

//         Ok(RiskMetrics {
//             volatility,
//             sharpe_ratio,
//             max_drawdown,
//             var_95,
//             skewness: self.calculate_skewness(&returns),
//             kurtosis: self.calculate_kurtosis(&returns),
//         })
//     }

//     fn calculate_skewness(&self, returns: &[f64]) -> f64 {
//         let mean = returns.iter().sum::<f64>() / returns.len() as f64;
//         let variance = returns.iter()
//             .map(|r| (r - mean).powi(2))
//             .sum::<f64>() / returns.len() as f64;
//         let std_dev = variance.sqrt();

//         if std_dev == 0.0 {
//             return 0.0;
//         }

//         let skewness = returns.iter()
//             .map(|r| ((r - mean) / std_dev).powi(3))
//             .sum::<f64>() / returns.len() as f64;

//         skewness
//     }

//     fn calculate_kurtosis(&self, returns: &[f64]) -> f64 {
//         let mean = returns.iter().sum::<f64>() / returns.len() as f64;
//         let variance = returns.iter()
//             .map(|r| (r - mean).powi(2))
//             .sum::<f64>() / returns.len() as f64;
//         let std_dev = variance.sqrt();

//         if std_dev == 0.0 {
//             return 3.0; // Normal distribution kurtosis
//         }

//         let kurtosis = returns.iter()
//             .map(|r| ((r - mean) / std_dev).powi(4))
//             .sum::<f64>() / returns.len() as f64;

//         kurtosis - 3.0 // Excess kurtosis
//     }

//     fn generate_trading_signals(&self, predictions: &[PredictionResult]) -> Result<Vec<TradingSignal>, Box<dyn Error>> {
//         let mut signals = Vec::new();
        
//         for prediction in predictions {
//             let signal_strength = prediction.predicted_return.abs() * prediction.confidence;
            
//             let signal_type = if prediction.predicted_return > 0.02 && prediction.confidence > 0.6 {
//                 SignalType::StrongBuy
//             } else if prediction.predicted_return > 0.01 && prediction.confidence > 0.5 {
//                 SignalType::Buy
//             } else if prediction.predicted_return < -0.02 && prediction.confidence > 0.6 {
//                 SignalType::StrongSell
//             } else if prediction.predicted_return < -0.01 && prediction.confidence > 0.5 {
//                 SignalType::Sell
//             } else {
//                 SignalType::Hold
//             };

//             signals.push(TradingSignal {
//                 signal_type,
//                 strength: signal_strength,
//                 predicted_return: prediction.predicted_return,
//                 confidence: prediction.confidence,
//                 risk_adjusted_return: prediction.predicted_return / (signal_strength + 0.01), // Simple risk adjustment
//             });
//         }

//         Ok(signals)
//     }

//     fn perform_clustering_analysis(&mut self) -> Result<ClusterAnalysis, Box<dyn Error>> {
//         // Create feature vectors for clustering (using recent data)
//         let recent_window = 50.min(self.candles.len());
//         let start_idx = self.candles.len().saturating_sub(recent_window);
        
//         let mut feature_vectors = Vec::new();
        
//         for i in start_idx..self.candles.len() {
//             let mut features = Vec::new();
            
//             // Price-based features
//             if i > 0 {
//                 features.push((self.candles[i].close - self.candles[i-1].close) / self.candles[i-1].close);
//             } else {
//                 features.push(0.0);
//             }
            
//             // Volume features
//             features.push(self.candles[i].volume.unwrap_or(0.0));
            
//             // Technical indicators (using last available values)
//             for (_, indicator_values) in &self.indicators {
//                 if let Some(Some(value)) = indicator_values.get(i) {
//                     features.push(*value);
//                 } else {
//                     features.push(0.0); // Fill missing with 0
//                 }
//             }
            
//             feature_vectors.push(features);
//         }

//         if feature_vectors.is_empty() {
//             return Err("No feature vectors for clustering".into());
//         }

//         // Normalize features for clustering
//         let normalized_features = self.normalize_features(&feature_vectors);
        
//         let assignments = self.ml_pipeline.cluster_stocks(&normalized_features, self.config.n_clusters)?;
        
//         // Analyze clusters
//         let mut cluster_stats = HashMap::new();
//         for (i, &cluster_id) in assignments.iter().enumerate() {
//             let stats = cluster_stats.entry(cluster_id).or_insert_with(|| ClusterStats {
//                 count: 0,
//                 avg_return: 0.0,
//                 avg_volatility: 0.0,
//             });
            
//             stats.count += 1;
//             if i > 0 && i < feature_vectors.len() {
//                 stats.avg_return += feature_vectors[i][0]; // First feature is return
//             }
//         }

//         // Calculate averages
//         for stats in cluster_stats.values_mut() {
//             if stats.count > 0 {
//                 stats.avg_return /= stats.count as f64;
//             }
//         }

//         Ok(ClusterAnalysis {
//             n_clusters: self.config.n_clusters,
//             assignments,
//             cluster_stats,
//         })
//     }

//     fn normalize_features(&self, features: &[Vec<f64>]) -> Vec<Vec<f64>> {
//         if features.is_empty() || features[0].is_empty() {
//             return features.to_vec();
//         }

//         let n_features = features[0].len();
//         let mut means = vec![0.0; n_features];
//         let mut stds = vec![0.0; n_features];

//         // Calculate means
//         for feature_vec in features {
//             for (i, &value) in feature_vec.iter().enumerate() {
//                 means[i] += value;
//             }
//         }
//         for mean in &mut means {
//             *mean /= features.len() as f64;
//         }

//         // Calculate standard deviations
//         for feature_vec in features {
//             for (i, &value) in feature_vec.iter().enumerate() {
//                 stds[i] += (value - means[i]).powi(2);
//             }
//         }
//         for std in &mut stds {
//             *std = (*std / features.len() as f64).sqrt();
//             if *std == 0.0 {
//                 *std = 1.0; // Avoid division by zero
//             }
//         }

//         // Normalize
//         features.iter()
//             .map(|feature_vec| {
//                 feature_vec.iter()
//                     .enumerate()
//                     .map(|(i, &value)| (value - means[i]) / stds[i])
//                     .collect()
//             })
//             .collect()
//     }
// }

// // Result structures
// #[derive(Debug)]
// pub struct MLAnalysisResults {
//     pub symbol: String,
//     pub predictions: Vec<PredictionResult>,
//     pub feature_importance: Vec<(String, f64)>,
//     pub risk_metrics: RiskMetrics,
//     pub signals: Vec<TradingSignal>,
//     pub cluster_results: Option<ClusterAnalysis>,
// }

// #[derive(Debug)]
// pub struct PredictionResult {
//     pub index: usize,
//     pub actual_return: f64,
//     pub predicted_return: f64,
//     pub confidence: f64,
// }

// #[derive(Debug)]
// pub struct RiskMetrics {
//     pub volatility: f64,
//     pub sharpe_ratio: f64,
//     pub max_drawdown: f64,
//     pub var_95: f64,
//     pub skewness: f64,
//     pub kurtosis: f64,
// }

// #[derive(Debug)]
// pub struct TradingSignal {
//     pub signal_type: SignalType,
//     pub strength: f64,
//     pub predicted_return: f64,
//     pub confidence: f64,
//     pub risk_adjusted_return: f64,
// }

// #[derive(Debug)]
// pub enum SignalType {
//     StrongBuy,
//     Buy,
//     Hold,
//     Sell,
//     StrongSell,
// }

// #[derive(Debug)]
// pub struct ClusterAnalysis {
//     pub n_clusters: usize,
//     pub assignments: Vec<usize>,
//     pub cluster_stats: HashMap<usize, ClusterStats>,
// }

// #[derive(Debug)]
// pub struct ClusterStats {
//     pub count: usize,
//     pub avg_return: f64,
//     pub avg_volatility: f64,
// }

// // Enhanced main function with ML analysis
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     println!("🚀 Advanced Stock Analysis with Machine Learning");
//     println!("================================================\n");

//     let tickers = ["AAPL", "MSFT", "GOOG", "TSLA", "NVDA"];
//     let chart_options = ChartQueryOptions {
//         interval: "1d",
//         range: "1y", // More data for ML
//     };

//     let ml_config = MLAnalysisConfig {
//         prediction_model: "random_forest".to_string(),
//         lookback_window: 30,
//         train_test_split: 0.8,
//         enable_clustering: true,
//         n_clusters: 3,
//         risk_free_rate: 0.02,
//     };

//     let indicators = build_indicators();
//     let runner = IndicatorRunner { indicators };
//     let fetcher = AsyncFetcher::new();

//     let mut all_results = Vec::new();

//     // Process each ticker
//     for ticker in &tickers {
//         println!("📈 Analyzing {}", ticker);
//         println!("─────────────────────────");

//         match fetcher.fetch_async(ticker, &chart_options).await {
//             Ok(chart_response) => {
//                 if let Some(results) = &chart_response.chart.result {
//                     for result in results {
//                         let candles = to_candles(result);
//                         if candles.len() < 60 { // Need sufficient data for ML
//                             println!("⚠️  Insufficient data for {} ({})", ticker, candles.len());
//                             continue;
//                         }

//                         let indicator_map = runner.run(&candles);
                        
//                         // Create ML analyzer
//                         let mut analyzer = StockMLAnalyzer::new(
//                             ticker.to_string(), 
//                             ml_config.clone()
//                         );
                        
//                         analyzer.load_data(candles.clone(), indicator_map.clone());
                        
//                         // Run ML analysis
//                         match analyzer.run_analysis() {
//                             Ok(ml_results) => {
//                                 print_ml_results(&ml_results);
//                                 all_results.push(ml_results);
//                             }
//                             Err(e) => {
//                                 eprintln!("❌ ML analysis failed for {}: {}", ticker, e);
//                             }
//                         }
//                     }
//                 }
//             }
//             Err(e) => eprintln!("❌ Error fetching {}: {}", ticker, e),
//         }
        
//         println!("\n");
//     }

//     // Portfolio-level analysis
//     if !all_results.is_empty() {
//         print_portfolio_analysis(&all_results);
//     }

//     Ok(())
// }

// fn print_ml_results(results: &MLAnalysisResults) {
//     println!("🎯 Predictions Summary:");
//     if !results.predictions.is_empty() {
//         let avg_predicted = results.predictions.iter()
//             .map(|p| p.predicted_return)
//             .sum::<f64>() / results.predictions.len() as f64;
        
//         let high_confidence_predictions: Vec<_> = results.predictions.iter()
//             .filter(|p| p.confidence > 0.7)
//             .collect();
        
//         println!("   Average Predicted Return: {:.2}%", avg_predicted * 100.0);
//         println!("   High Confidence Predictions: {}/{}", 
//                  high_confidence_predictions.len(), results.predictions.len());
//     }

//     println!("\n📊 Risk Metrics:");
//     println!("   Volatility: {:.2}%", results.risk_metrics.volatility * 100.0);
//     println!("   Sharpe Ratio: {:.3}", results.risk_metrics.sharpe_ratio);
//     println!("   Max Drawdown: {:.2}%", results.risk_metrics.max_drawdown * 100.0);
//     println!("   VaR (95%): {:.2}%", results.risk_metrics.var_95 * 100.0);
//     println!("   Skewness: {:.3}", results.risk_metrics.skewness);
//     println!("   Excess Kurtosis: {:.3}", results.risk_metrics.kurtosis);

//     println!("\n🔝 Top 5 Most Important Features:");
//     for (name, importance) in results.feature_importance.iter().take(5) {
//         println!("   {}: {:.4}", name, importance);
//     }

//     println!("\n📡 Latest Trading Signals:");
//     let recent_signals: Vec<_> = results.signals.iter().rev().take(3).collect();
//     for signal in recent_signals {
//         let signal_emoji = match signal.signal_type {
//             SignalType::StrongBuy => "🟢",
//             SignalType::Buy => "🔵",
//             SignalType::Hold => "⚪",
//             SignalType::Sell => "🔴",
//             SignalType::StrongSell => "⚫",
//         };
        
//         println!("   {} {:?} - Strength: {:.3}, Confidence: {:.1}%", 
//                  signal_emoji, signal.signal_type, signal.strength, signal.confidence * 100.0);
//     }

//     if let Some(cluster_results) = &results.cluster_results {
//         println!("\n🎲 Cluster Analysis:");
//         println!("   Assigned to cluster: {}", 
//                  cluster_results.assignments.last().unwrap_or(&0));
        
//         for (cluster_id, stats) in &cluster_results.cluster_stats {
//             println!("   Cluster {}: {} observations, avg return: {:.2}%", 
//                      cluster_id, stats.count, stats.avg_return * 100.0);
//         }
//     }
// }

// fn print_portfolio_analysis(all_results: &[MLAnalysisResults]) {
//     println!("📊 PORTFOLIO-LEVEL ANALYSIS");
//     println!("═══════════════════════════════════════");

//     // Calculate portfolio metrics
//     let total_stocks = all_results.len();
//     let avg_sharpe = all_results.iter()
//         .map(|r| r.risk_metrics.sharpe_ratio)
//         .sum::<f64>() / total_stocks as f64;

//     let strong_buy_signals = all_results.iter()
//         .flat_map(|r| &r.signals)
//         .filter(|s| matches!(s.signal_type, SignalType::StrongBuy))
//         .count();

//     let buy_signals = all_results.iter()
//         .flat_map(|r| &r.signals)
//         .filter(|s| matches!(s.signal_type, SignalType::Buy))
//         .count();

//     println!("📈 Portfolio Summary:");
//     println!("   Total Stocks Analyzed: {}", total_stocks);
//     println!("   Average Sharpe Ratio: {:.3}", avg_sharpe);
//     println!("   Strong Buy Signals: {}", strong_buy_signals);
//     println!("   Buy Signals: {}", buy_signals);

//     // Top performers by predicted return
//     let mut stock_predictions: Vec<_> = all_results.iter()
//         .filter_map(|r| {
//             r.predictions.last().map(|p| (&r.symbol, p.predicted_return, p.confidence))
//         })
//         .collect();

//     stock_predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

//     println!("\n🏆 Top Predicted Performers:");
//     for (symbol, predicted_return, confidence) in stock_predictions.iter().take(3) {
//         println!("   {}: {:.2}% (confidence: {:.1}%)", 
//                  symbol, predicted_return * 100.0, confidence * 100.0);
//     }

//     // Risk ranking
//     let mut risk_ranking: Vec<_> = all_results.iter()
//         .map(|r| (&r.symbol, r.risk_metrics.volatility, r.risk_metrics.max_drawdown))
//         .collect();

//     risk_ranking.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

//     println!("\n🛡️  Lowest Risk Stocks:");
//     for (symbol, volatility, max_dd) in risk_ranking.iter().take(3) {
//         println!("   {}: {:.1}% volatility, {:.1}% max drawdown", 
//                  symbol, volatility * 100.0, max_dd * 100.0);
//     }
// }

// // --- Main: example usage ---

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let tickers = ["AAPL", "MSFT", "GOOG"];
//     // // Fetch first 10 NASDAQ tickers dynamically
//     // let tickers = fetch_nasdaq_symbols_csv().await?
//     //     .into_iter()
//     //     .take(10)
//     //     .collect::<Vec<_>>();
//     // println!("{:?}", tickers);
//     let options = ChartQueryOptions {
//         interval: "1d",
//         range: "6mo",
//     };

//     // Hardcoded example params for Greeks calculation
//     let risk_free_rate = 0.01;
//     let volatility = 0.25;
//     let time_to_expiry = 30.0 / 365.0;

//     let indicators = build_indicators();
//     let runner = IndicatorRunner { indicators };

//     let use_async = true;

//     // Map to store underlying prices keyed by ticker
//     let mut underlying_prices: HashMap<String, f64> = HashMap::new();

//     if use_async {
//         let fetcher = AsyncFetcher::new();

//         for ticker in &tickers {
//             match fetcher.fetch_async(ticker, &options).await {
//                 Ok(chart_response) => {
//                     if let Some((symbol, price)) = print_chart_response(chart_response, &runner) {
//                         underlying_prices.insert(symbol, price);
//                     }
//                 }
//                 Err(e) => eprintln!("Error fetching {}: {}", ticker, e),
//             }
//         }
//     } else {
//         let fetcher = SyncFetcher;

//         for ticker in &tickers {
//             match fetcher.fetch_sync(ticker, &options) {
//                 Ok(chart_response) => {
//                     if let Some((symbol, price)) = print_chart_response(chart_response, &runner) {
//                         underlying_prices.insert(symbol, price);
//                     }
//                 }
//                 Err(e) => eprintln!("Error fetching {}: {}", ticker, e),
//             }
//         }
//     }

//     // Use the collected underlying prices for options Greeks
//     if use_async {
//         let fetcher = AsyncOptionsFetcher::new();
//         for ticker in &tickers {
//             let underlying_price = *underlying_prices.get(&ticker.to_string()).unwrap_or(&100.0);
//             match fetcher.fetch_async(ticker).await {
//                 Ok(resp) => print_opc_option_chain_with_greeks(resp, underlying_price, time_to_expiry, risk_free_rate, volatility),
//                 Err(e) => eprintln!("Async fetch error for {}: {}", ticker, e),
//             }
//         }
//     } else {
//         let fetcher = SyncOptionsFetcher;
//         for ticker in &tickers {
//             let underlying_price = *underlying_prices.get(&ticker.to_string()).unwrap_or(&100.0);
//             match fetcher.fetch_sync(ticker) {
//                 Ok(resp) => print_opc_option_chain_with_greeks(resp, underlying_price, time_to_expiry, risk_free_rate, volatility),
//                 Err(e) => eprintln!("Sync fetch error for {}: {}", ticker, e),
//             }
//         }
//     }

//     Ok(())
// }

fn print_chart_response(chart_response: ChartResponse, runner: &IndicatorRunner) -> Option<(String, f64)> {
    if let Some(results) = &chart_response.chart.result {
        for result in results {
            println!("Ticker: {}", result.meta.symbol);

            let candles = to_candles(result);
            if candles.is_empty() {
                println!("No valid candles.");
                continue;
            }

            let indicator_map = runner.run(&candles);

            for (i, candle) in candles.iter().enumerate() {
                let datetime = UNIX_EPOCH + Duration::from_secs(candle.timestamp.try_into().unwrap());
                let dt: chrono::DateTime<chrono::Utc> = datetime.into();

                print!("  {}: Close ${:.2}", dt.format("%Y-%m-%d"), candle.close);

                for (name, values) in &indicator_map {
                    if let Some(Some(val)) = values.get(i) {
                        print!(" | {}: {:.2}", name, val);
                    }
                }

                println!();
            }

            println!("---");

            // Return last candle close price as underlying price
            return Some((result.meta.symbol.clone(), candles.last().unwrap().close));
        }
    } else {
        eprintln!("No results found; error: {:?}", chart_response.chart.error);
    }
    None
}



/*
// OPTIONS FETCHER
#[derive(Debug, Deserialize)]
struct OptionChainResponse {
    optionChain: OptionChain,
}

#[derive(Debug, Deserialize)]
struct OptionChain {
    result: Option<Vec<OptionChainResult>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OptionChainResult {
    underlyingSymbol: String,
    expirationDates: Vec<u64>,
    options: Vec<OptionSet>,
}

#[derive(Debug, Deserialize)]
struct OptionSet {
    calls: Vec<OptionContract>,
    puts: Vec<OptionContract>,
}

#[derive(Debug, Deserialize)]
struct OptionContract {
    contractSymbol: String,
    strike: f64,
    lastPrice: f64,
    bid: f64,
    ask: f64,
    volume: Option<u64>,
    openInterest: Option<u64>,
    impliedVolatility: f64,
    inTheMoney: bool,
    expiration: u64,
}

async fn fetch_option_chain(ticker: &str, expiration: Option<u64>) -> Result<OptionChainResponse, Box<dyn Error>> {
    let mut url = format!("https://query1.finance.yahoo.com/v7/finance/options/{}", ticker);
    // https://www.optionsprofitcalculator.com/ajax/getOptions?stock=tsla&reqId=1
    if let Some(date) = expiration {
        url.push_str(&format!("?date={}", date));
    }

    let resp = reqwest::get(&url).await?.text().await?;
    println!("{:?}", resp);
    let parsed: OptionChainResponse = serde_json::from_str(&resp)?;
    Ok(parsed)
}

fn print_option_chain(response: OptionChainResponse) {
    if let Some(chain) = response.optionChain.result {
        for result in chain {
            println!("Options for: {}", result.underlyingSymbol);
            println!("Available Expirations: {:?}", result.expirationDates);

            for option_set in result.options {
                println!("Calls:");
                for call in &option_set.calls {
                    println!(
                        "  {} | Strike: ${:.2} | Last: {:.2} | Bid: {:.2} | Ask: {:.2} | Vol: {:?} | OI: {:?} | IV: {:.2}%",
                        call.contractSymbol,
                        call.strike,
                        call.lastPrice,
                        call.bid,
                        call.ask,
                        call.volume,
                        call.openInterest,
                        call.impliedVolatility * 100.0,
                    );
                }

                println!("Puts:");
                for put in &option_set.puts {
                    println!(
                        "  {} | Strike: ${:.2} | Last: {:.2} | Bid: {:.2} | Ask: {:.2} | Vol: {:?} | OI: {:?} | IV: {:.2}%",
                        put.contractSymbol,
                        put.strike,
                        put.lastPrice,
                        put.bid,
                        put.ask,
                        put.volume,
                        put.openInterest,
                        put.impliedVolatility * 100.0,
                    );
                }

                println!("---");
            }
        }
    } else {
        eprintln!("Error: {:?}", response.optionChain.error);
    }
}
*/