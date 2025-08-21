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

// internal
mod indicators;
mod types;
use crate::types::Candle;
use crate::indicators::{
    SMA, EMA, RSI, MACD, BollingerBands, VWAP, ATR, Stochastic, CCI, ADX, ParabolicSAR, OBV,
    CMF, WilliamsR, Ichimoku, Momentum, Tema, Dema, Kama, WMA, Hma, Frama, ChandelierExit,
    TRIX, MFI, ForceIndex, EaseOfMovement, AccumDistLine, PriceVolumeTrend, VolumeOscillator,
    UltimateOscillator, DetrendedPriceOscillator, RateOfChange, ZScore, GMMA, SchaffTrendCycle,
    FibonacciRetracement, KalmanFilterSmoother, HeikinAshiSlope, PercentB,
    TechnicalIndicator, IndicatorRunner
};
mod options_math;
use options_math::{black_scholes_greeks, calculate_pnl, OptionData, OptionType};


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
struct ChartQueryOptions<'a> {
    interval: &'a str,  // e.g., "1d", "1h"
    range: &'a str,     // e.g., "5d", "1mo"
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
trait ChartFetcher {
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
struct AsyncFetcher {
    client: reqwest::Client,
}

impl AsyncFetcher {
    fn new() -> Self {
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
struct ChartResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Option<Vec<ResultItem>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ResultItem {
    meta: Meta,
    timestamp: Vec<u64>,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Meta {
    currency: String,
    symbol: String,
    exchangeName: String,
    fullExchangeName: String,
    instrumentType: String,
    firstTradeDate: u64,
    regularMarketTime: u64,
    hasPrePostMarketData: bool,
    gmtoffset: i64,
    timezone: String,
    exchangeTimezoneName: String,
    regularMarketPrice: f64,
    fiftyTwoWeekHigh: f64,
    fiftyTwoWeekLow: f64,
    regularMarketDayHigh: f64,
    regularMarketDayLow: f64,
    regularMarketVolume: u64,
    longName: String,
    shortName: String,
    chartPreviousClose: f64,
    priceHint: u8,
    currentTradingPeriod: TradingPeriodWrapper,
    dataGranularity: String,
    range: String,
    validRanges: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TradingPeriodWrapper {
    pre: TradingPeriod,
    regular: TradingPeriod,
    post: TradingPeriod,
}

#[derive(Debug, Deserialize)]
struct TradingPeriod {
    timezone: String,
    end: u64,
    start: u64,
    gmtoffset: i64,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    quote: Option<Vec<Quote>>,
    adjclose: Option<Vec<AdjClose>>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    close: Option<Vec<Option<f64>>>,
    open: Option<Vec<Option<f64>>>,
    volume: Option<Vec<Option<u64>>>,
    high: Option<Vec<Option<f64>>>,
    low: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
struct AdjClose {
    adjclose: Option<Vec<Option<f64>>>,
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

fn to_candles(result: &ResultItem) -> Vec<Candle> {
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

fn build_indicators() -> Vec<(String, Arc<dyn TechnicalIndicator + Send + Sync>)> {
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
struct OptionProfitCalculatorResponse {
    options: HashMap<String, ExpiryOptionData>,
}

#[derive(Debug, Deserialize)]
struct ExpiryOptionData {
    c: HashMap<String, OptionQuote>,
    p: HashMap<String, OptionQuote>,
}

#[derive(Debug, Deserialize)]
struct OptionQuote {
    oi: u64,
    l: f64,
    b: f64,
    a: f64,
    v: u64,
}

trait OptionsFetcher {
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

struct AsyncOptionsFetcher {
    client: reqwest::Client,
}

impl AsyncOptionsFetcher {
    fn new() -> Self {
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


// --- Main: example usage ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tickers = ["AAPL", "MSFT", "GOOG"];
    // // Fetch first 10 NASDAQ tickers dynamically
    // let tickers = fetch_nasdaq_symbols_csv().await?
    //     .into_iter()
    //     .take(10)
    //     .collect::<Vec<_>>();
    // println!("{:?}", tickers);
    let options = ChartQueryOptions {
        interval: "1d",
        range: "6mo",
    };

    // Hardcoded example params for Greeks calculation
    let risk_free_rate = 0.01;
    let volatility = 0.25;
    let time_to_expiry = 30.0 / 365.0;

    let indicators = build_indicators();
    let runner = IndicatorRunner { indicators };

    let use_async = true;

    // Map to store underlying prices keyed by ticker
    let mut underlying_prices: HashMap<String, f64> = HashMap::new();

    if use_async {
        let fetcher = AsyncFetcher::new();

        for ticker in &tickers {
            match fetcher.fetch_async(ticker, &options).await {
                Ok(chart_response) => {
                    if let Some((symbol, price)) = print_chart_response(chart_response, &runner) {
                        underlying_prices.insert(symbol, price);
                    }
                }
                Err(e) => eprintln!("Error fetching {}: {}", ticker, e),
            }
        }
    } else {
        let fetcher = SyncFetcher;

        for ticker in &tickers {
            match fetcher.fetch_sync(ticker, &options) {
                Ok(chart_response) => {
                    if let Some((symbol, price)) = print_chart_response(chart_response, &runner) {
                        underlying_prices.insert(symbol, price);
                    }
                }
                Err(e) => eprintln!("Error fetching {}: {}", ticker, e),
            }
        }
    }

    // Use the collected underlying prices for options Greeks
    if use_async {
        let fetcher = AsyncOptionsFetcher::new();
        for ticker in &tickers {
            let underlying_price = *underlying_prices.get(&ticker.to_string()).unwrap_or(&100.0);
            match fetcher.fetch_async(ticker).await {
                Ok(resp) => print_opc_option_chain_with_greeks(resp, underlying_price, time_to_expiry, risk_free_rate, volatility),
                Err(e) => eprintln!("Async fetch error for {}: {}", ticker, e),
            }
        }
    } else {
        let fetcher = SyncOptionsFetcher;
        for ticker in &tickers {
            let underlying_price = *underlying_prices.get(&ticker.to_string()).unwrap_or(&100.0);
            match fetcher.fetch_sync(ticker) {
                Ok(resp) => print_opc_option_chain_with_greeks(resp, underlying_price, time_to_expiry, risk_free_rate, volatility),
                Err(e) => eprintln!("Sync fetch error for {}: {}", ticker, e),
            }
        }
    }

    Ok(())
}

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