// Complete implementation of the API methods and usage examples

use chrono::{DateTime, Utc, TimeZone};
use std::time::{UNIX_EPOCH, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use serde_json::from_str;

// Re-export your existing types
use crate::types::Candle;
use crate::indicators::{TechnicalIndicator, IndicatorRunner};
use crate::options_math::{black_scholes_greeks, calculate_pnl, OptionData, OptionType, OptionGreeks};
use crate::og::*;

// API Error Types
#[derive(Debug, Serialize)]
pub enum ApiError {
    InvalidTicker(String),
    InvalidDateRange(String),
    DataNotFound(String),
    FetchError(String),
    CalculationError(String),
    InvalidParameters(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::InvalidTicker(t) => write!(f, "Invalid ticker: {}", t),
            ApiError::InvalidDateRange(r) => write!(f, "Invalid date range: {}", r),
            ApiError::DataNotFound(msg) => write!(f, "Data not found: {}", msg),
            ApiError::FetchError(msg) => write!(f, "Fetch error: {}", msg),
            ApiError::CalculationError(msg) => write!(f, "Calculation error: {}", msg),
            ApiError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl Error for ApiError {}

// API Request/Response Types
#[derive(Debug, Deserialize)]
pub struct HistoricalDataRequest {
    pub tickers: Vec<String>,
    pub interval: Option<String>,  // "1m", "5m", "15m", "30m", "1h", "1d", "1wk", "1mo"
    pub range: Option<String>,     // "1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"
    pub start_date: Option<String>, // YYYY-MM-DD format
    pub end_date: Option<String>,   // YYYY-MM-DD format
    pub include_indicators: Option<bool>,
    pub indicators: Option<Vec<IndicatorConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct IndicatorConfig {
    pub name: String,
    pub params: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct HistoricalDataResponse {
    pub data: HashMap<String, TickerData>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TickerData {
    pub symbol: String,
    pub candles: Vec<CandleData>,
    pub indicators: Option<HashMap<String, Vec<Option<f64>>>>,
    pub meta: TickerMeta,
}

#[derive(Debug, Serialize, Clone)]
pub struct CandleData {
    pub timestamp: i64,
    pub datetime: String, // ISO 8601 format
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
    pub adj_close: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TickerMeta {
    pub currency: String,
    pub exchange: String,
    pub instrument_type: String,
    pub timezone: String,
    pub regular_market_price: f64,
    pub fifty_two_week_high: f64,
    pub fifty_two_week_low: f64,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
}

// Options Chain API
#[derive(Debug, Deserialize)]
pub struct OptionsChainRequest {
    pub ticker: String,
    pub expiration_dates: Option<Vec<String>>, // YYYY-MM-DD format
    pub min_strike: Option<f64>,
    pub max_strike: Option<f64>,
    pub option_type: Option<String>, // "call", "put", "both"
    pub include_greeks: Option<bool>,
    pub volatility: Option<f64>,      // For Greeks calculation
    pub risk_free_rate: Option<f64>,  // For Greeks calculation
}

#[derive(Debug, Serialize)]
pub struct OptionsChainResponse {
    pub symbol: String,
    pub underlying_price: f64,
    pub expirations: HashMap<String, ExpirationData>,
    pub greeks_params: Option<GreeksParams>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExpirationData {
    pub expiration_date: String,
    pub days_to_expiry: f64,
    pub calls: Vec<OptionContractData>,
    pub puts: Vec<OptionContractData>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OptionContractData {
    pub strike: f64,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
    pub open_interest: u64,
    pub implied_volatility: Option<f64>,
    pub greeks: Option<GreeksData>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GreeksData {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub theoretical_price: f64,
}

#[derive(Debug, Serialize)]
pub struct GreeksParams {
    pub volatility: f64,
    pub risk_free_rate: f64,
}

// Options Math API
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsPnLRequest {
    pub positions: Vec<OptionPosition>,
    pub underlying_prices: Vec<f64>, // Array of prices to calculate P&L at
    pub volatility: Option<f64>,
    pub risk_free_rate: Option<f64>,
    pub days_to_expiry: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionPosition {
    pub option_type: String, // "call" or "put"
    pub strike: f64,
    pub quantity: i32, // Positive for long, negative for short
    pub entry_price: f64,
    pub days_to_expiry: f64,
}

#[derive(Debug, Serialize)]
pub struct OptionsPnLResponse {
    pub positions: Vec<PositionAnalysis>,
    pub portfolio: PortfolioAnalysis,
}

#[derive(Debug, Serialize)]
pub struct PositionAnalysis {
    pub position: OptionPosition,
    pub greeks: GreeksData,
    pub pnl_curve: Vec<PnLPoint>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioAnalysis {
    pub total_greeks: GreeksData,
    pub total_pnl_curve: Vec<PnLPoint>,
    pub break_even_points: Vec<f64>,
    pub max_profit: Option<f64>,
    pub max_loss: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PnLPoint {
    pub underlying_price: f64,
    pub pnl: f64,
    pub total_value: f64,
}

// Screener API
#[derive(Debug, Deserialize)]
pub struct ScreenerRequest {
    pub filters: Vec<ScreenerFilter>,
    pub indicators: Option<Vec<IndicatorConfig>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ScreenerFilter {
    pub field: String, // "price", "volume", "market_cap", "pe_ratio", etc.
    pub operator: String, // "gt", "lt", "gte", "lte", "eq", "between"
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ScreenerResponse {
    pub results: Vec<ScreenerResult>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ScreenerResult {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub indicators: Option<HashMap<String, f64>>,
}

// Real-time Quote API
#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub tickers: Vec<String>,
    pub fields: Option<Vec<String>>, // Specific fields to return
}

#[derive(Debug, Serialize)]
pub struct QuoteResponse {
    pub quotes: HashMap<String, Quote>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Quote {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub bid_size: Option<u64>,
    pub ask_size: Option<u64>,
    pub high_52w: f64,
    pub low_52w: f64,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub last_updated: String, // ISO 8601 timestamp
}

// Watchlist API
#[derive(Debug, Deserialize)]
pub struct WatchlistRequest {
    pub name: String,
    pub tickers: Vec<String>,
    pub alerts: Option<Vec<AlertConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct AlertConfig {
    pub ticker: String,
    pub condition: String, // "price_above", "price_below", "volume_above", etc.
    pub value: f64,
    pub indicator: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WatchlistResponse {
    pub name: String,
    pub quotes: Vec<Quote>,
    pub alerts: Vec<TriggeredAlert>,
}

#[derive(Debug, Serialize)]
pub struct TriggeredAlert {
    pub ticker: String,
    pub condition: String,
    pub current_value: f64,
    pub trigger_value: f64,
    pub timestamp: String,
}

// Main API Service
pub struct StockDataApi {
    chart_fetcher: Arc<dyn ChartFetcher + Send + Sync>,
    options_fetcher: Arc<dyn OptionsFetcher + Send + Sync>,
    indicator_runner: IndicatorRunner,
}

impl StockDataApi {
    pub fn new(
        chart_fetcher: Arc<dyn ChartFetcher + Send + Sync>,
        options_fetcher: Arc<dyn OptionsFetcher + Send + Sync>,
        indicators: Vec<(String, Arc<dyn TechnicalIndicator + Send + Sync>)>,
    ) -> Self {
        Self {
            chart_fetcher,
            options_fetcher,
            indicator_runner: IndicatorRunner { indicators },
        }
    }

    // Historical Data Endpoint
    pub async fn get_historical_data(&self, request: HistoricalDataRequest) -> Result<HistoricalDataResponse, ApiError> {
        let mut data = HashMap::new();
        let mut errors = Vec::new();

        let options = ChartQueryOptions {
            interval: request.interval.as_deref().unwrap_or("1d"),
            range: request.range.as_deref().unwrap_or("1mo"),
        };

        for ticker in &request.tickers {
            match self.fetch_ticker_data(ticker, &options).await {
                Ok(ticker_data) => {
                    let processed_data = self.process_ticker_data(ticker_data, &request)?;
                    data.insert(ticker.clone(), processed_data);
                }
                Err(e) => {
                    errors.push(format!("Error fetching {}: {}", ticker, e));
                }
            }
        }

        Ok(HistoricalDataResponse { data, errors })
    }

    // Options Chain Endpoint
    pub async fn get_options_chain(&self, request: OptionsChainRequest) -> Result<OptionsChainResponse, ApiError> {
        // Get underlying price first
        let chart_options = ChartQueryOptions::default();
        let chart_data = self.fetch_ticker_data(&request.ticker, &chart_options).await?;
        let underlying_price = self.extract_current_price(&chart_data)?;

        // Fetch options data
        let options_data = self.options_fetcher.fetch_async(&request.ticker).await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        // Process and filter options data
        let processed_data = self.process_options_data(
            options_data,
            &request,
            underlying_price,
        )?;

        Ok(processed_data)
    }

    // Options P&L Analysis Endpoint
    pub fn calculate_options_pnl(&self, request: OptionsPnLRequest) -> Result<OptionsPnLResponse, ApiError> {
        let volatility = request.volatility.unwrap_or(0.25);
        let risk_free_rate = request.risk_free_rate.unwrap_or(0.01);

        let mut positions = Vec::new();
        let mut portfolio_pnl_curves: Vec<Vec<PnLPoint>> = Vec::new();

        // Calculate P&L for each position
        for position in &request.positions {
            let option_type = match position.option_type.as_str() {
                "call" => OptionType::Call,
                "put" => OptionType::Put,
                _ => return Err(ApiError::InvalidParameters("Invalid option type".to_string())),
            };

            let greeks = black_scholes_greeks(
                request.underlying_prices[0], // Use first price for Greeks calculation
                position.strike,
                position.days_to_expiry / 365.0,
                risk_free_rate,
                volatility,
                option_type,
            );

            let mut pnl_curve = Vec::new();
            for &price in &request.underlying_prices {
                let current_greeks = black_scholes_greeks(
                    price,
                    position.strike,
                    position.days_to_expiry / 365.0,
                    risk_free_rate,
                    volatility,
                    option_type,
                );

                pnl_curve.push(PnLPoint {
                    underlying_price: price,
                    pnl: calculate_pnl(position.quantity.into(), position.entry_price, current_greeks.price),
                    total_value: current_greeks.price * position.quantity as f64,
                });
            }

            portfolio_pnl_curves.push(pnl_curve.clone());

            positions.push(PositionAnalysis {
                position: position.clone(),
                greeks: GreeksData {
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
                    theoretical_price: greeks.price,
                },
                pnl_curve,
            });
        }

        // Calculate portfolio totals
        let portfolio = self.calculate_portfolio_analysis(&portfolio_pnl_curves, &request.underlying_prices);

        Ok(OptionsPnLResponse {
            positions,
            portfolio,
        })
    }

    // Real-time Quotes Endpoint
    pub async fn get_quotes(&self, request: QuoteRequest) -> Result<QuoteResponse, ApiError> {
        let mut quotes = HashMap::new();
        let mut errors = Vec::new();

        let options = ChartQueryOptions {
            interval: "1m",
            range: "1d",
        };

        for ticker in &request.tickers {
            match self.fetch_ticker_data(ticker, &options).await {
                Ok(data) => {
                    if let Ok(quote) = self.extract_quote_from_data(data) {
                        quotes.insert(ticker.clone(), quote);
                    } else {
                        errors.push(format!("Could not extract quote for {}", ticker));
                    }
                }
                Err(e) => {
                    errors.push(format!("Error fetching quote for {}: {}", ticker, e));
                }
            }
        }

        Ok(QuoteResponse { quotes, errors })
    }

    // Helper methods
    async fn fetch_ticker_data(&self, ticker: &str, options: &ChartQueryOptions<'_>) -> Result<ChartResponse, ApiError> {
        self.chart_fetcher.fetch_async(ticker, options).await
            .map_err(|e| ApiError::FetchError(e.to_string()))
    }

    // Implementation of process_ticker_data
    fn process_ticker_data(&self, chart_data: ChartResponse, request: &HistoricalDataRequest) -> Result<TickerData, ApiError> {
        let result = chart_data.chart.result
            .as_ref()
            .and_then(|results| results.get(0))
            .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

        let candles = to_candles(result);
        if candles.is_empty() {
            return Err(ApiError::DataNotFound("No valid candles found".to_string()));
        }

        // Convert candles to API format
        let mut candle_data = Vec::new();
        for candle in &candles {
            let datetime = UNIX_EPOCH + Duration::from_secs(candle.timestamp.try_into().unwrap());
            let dt: DateTime<Utc> = datetime.into();
            
            candle_data.push(CandleData {
                timestamp: candle.timestamp,
                datetime: dt.to_rfc3339(),
                open: candle.open,
                high: candle.high,
                low: candle.low,
                close: candle.close,
                volume: candle.volume,
                adj_close: None, // You'd extract this from adjclose indicators
            });
        }

        // Calculate indicators if requested
        let indicators = if request.include_indicators.unwrap_or(false) {
            Some(self.indicator_runner.run(&candles))
        } else {
            None
        };

        // Build metadata
        let meta = TickerMeta {
            currency: result.meta.currency.clone(),
            exchange: result.meta.exchangeName.clone(),
            instrument_type: result.meta.instrumentType.clone(),
            timezone: result.meta.timezone.clone(),
            regular_market_price: result.meta.regularMarketPrice,
            fifty_two_week_high: result.meta.fiftyTwoWeekHigh,
            fifty_two_week_low: result.meta.fiftyTwoWeekLow,
            market_cap: None, // Not available in basic chart data
            pe_ratio: None,
            dividend_yield: None,
        };

        Ok(TickerData {
            symbol: result.meta.symbol.clone(),
            candles: candle_data,
            indicators,
            meta,
        })
    }

    // Implementation of process_options_data
    fn process_options_data(
        &self,
        options_data: OptionProfitCalculatorResponse,
        request: &OptionsChainRequest,
        underlying_price: f64,
    ) -> Result<OptionsChainResponse, ApiError> {
        let mut expirations = HashMap::new();
        
        let volatility = request.volatility.unwrap_or(0.25);
        let risk_free_rate = request.risk_free_rate.unwrap_or(0.01);
        let include_greeks = request.include_greeks.unwrap_or(false);

        for (expiry_str, exp_data) in options_data.options {
            // Calculate days to expiry (simplified - you'd want proper date parsing)
            let days_to_expiry = 30.0; // Placeholder - parse expiry_str properly
            let time_to_expiry = days_to_expiry / 365.0;

            let mut calls = Vec::new();
            let mut puts = Vec::new();

            // Process calls
            for (strike_str, quote) in exp_data.c {
                let strike: f64 = strike_str.parse().unwrap_or(0.0);
                
                // Apply filters
                if let Some(min_strike) = request.min_strike {
                    if strike < min_strike { continue; }
                }
                if let Some(max_strike) = request.max_strike {
                    if strike > max_strike { continue; }
                }
                if let Some(ref option_type) = request.option_type {
                    if option_type == "put" { continue; }
                }

                let greeks = if include_greeks {
                    let g = black_scholes_greeks(
                        underlying_price,
                        strike,
                        time_to_expiry,
                        risk_free_rate,
                        volatility,
                        OptionType::Call,
                    );
                    Some(GreeksData {
                        delta: g.delta,
                        gamma: g.gamma,
                        theta: g.theta,
                        vega: g.vega,
                        rho: g.rho,
                        theoretical_price: g.price,
                    })
                } else {
                    None
                };

                calls.push(OptionContractData {
                    strike,
                    bid: quote.b,
                    ask: quote.a,
                    last: quote.l,
                    volume: quote.v,
                    open_interest: quote.oi,
                    implied_volatility: None, // Not available in this data source
                    greeks,
                });
            }

            // Process puts (similar logic)
            for (strike_str, quote) in exp_data.p {
                let strike: f64 = strike_str.parse().unwrap_or(0.0);
                
                // Apply filters
                if let Some(min_strike) = request.min_strike {
                    if strike < min_strike { continue; }
                }
                if let Some(max_strike) = request.max_strike {
                    if strike > max_strike { continue; }
                }
                if let Some(ref option_type) = request.option_type {
                    if option_type == "call" { continue; }
                }

                let greeks = if include_greeks {
                    let g = black_scholes_greeks(
                        underlying_price,
                        strike,
                        time_to_expiry,
                        risk_free_rate,
                        volatility,
                        OptionType::Put,
                    );
                    Some(GreeksData {
                        delta: g.delta,
                        gamma: g.gamma,
                        theta: g.theta,
                        vega: g.vega,
                        rho: g.rho,
                        theoretical_price: g.price,
                    })
                } else {
                    None
                };

                puts.push(OptionContractData {
                    strike,
                    bid: quote.b,
                    ask: quote.a,
                    last: quote.l,
                    volume: quote.v,
                    open_interest: quote.oi,
                    implied_volatility: None,
                    greeks,
                });
            }

            // Sort by strike price
            calls.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap());
            puts.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap());

            expirations.insert(expiry_str.clone(), ExpirationData {
                expiration_date: expiry_str,
                days_to_expiry,
                calls,
                puts,
            });
        }

        let greeks_params = if include_greeks {
            Some(GreeksParams {
                volatility,
                risk_free_rate,
            })
        } else {
            None
        };

        Ok(OptionsChainResponse {
            symbol: request.ticker.clone(),
            underlying_price,
            expirations,
            greeks_params,
        })
    }

    fn extract_current_price(&self, chart_data: &ChartResponse) -> Result<f64, ApiError> {
        chart_data.chart.result
            .as_ref()
            .and_then(|results| results.get(0))
            .map(|result| result.meta.regularMarketPrice)
            .ok_or_else(|| ApiError::DataNotFound("No price data found".to_string()))
    }

    fn extract_quote_from_data(&self, chart_data: ChartResponse) -> Result<Quote, ApiError> {
        let result = chart_data.chart.result
            .as_ref()
            .and_then(|results| results.get(0))
            .ok_or_else(|| ApiError::DataNotFound("No quote data found".to_string()))?;

        let candles = to_candles(result);
        let latest_candle = candles.last()
            .ok_or_else(|| ApiError::DataNotFound("No candle data found".to_string()))?;

        // Calculate change from previous close
        let prev_close = result.meta.chartPreviousClose;
        let current_price = result.meta.regularMarketPrice;
        let change = current_price - prev_close;
        let change_percent = (change / prev_close) * 100.0;

        Ok(Quote {
            symbol: result.meta.symbol.clone(),
            price: current_price,
            change,
            change_percent,
            volume: result.meta.regularMarketVolume,
            bid: None, // Not available in this data
            ask: None,
            bid_size: None,
            ask_size: None,
            high_52w: result.meta.fiftyTwoWeekHigh,
            low_52w: result.meta.fiftyTwoWeekLow,
            market_cap: None,
            pe_ratio: None,
            dividend_yield: None,
            last_updated: Utc::now().to_rfc3339(),
        })
    }

    fn calculate_portfolio_analysis(
        &self,
        pnl_curves: &[Vec<PnLPoint>],
        underlying_prices: &[f64],
    ) -> PortfolioAnalysis {
        let mut total_pnl_curve = Vec::new();
        
        // Calculate total P&L at each price point
        for (i, &price) in underlying_prices.iter().enumerate() {
            let total_pnl: f64 = pnl_curves.iter()
                .map(|curve| curve.get(i).map_or(0.0, |point| point.pnl))
                .sum();
            
            let total_value: f64 = pnl_curves.iter()
                .map(|curve| curve.get(i).map_or(0.0, |point| point.total_value))
                .sum();

            total_pnl_curve.push(PnLPoint {
                underlying_price: price,
                pnl: total_pnl,
                total_value,
            });
        }

        // Find break-even points (where P&L crosses zero)
        let mut break_even_points = Vec::new();
        for i in 1..total_pnl_curve.len() {
            let prev = &total_pnl_curve[i - 1];
            let curr = &total_pnl_curve[i];
            
            if (prev.pnl <= 0.0 && curr.pnl >= 0.0) || (prev.pnl >= 0.0 && curr.pnl <= 0.0) {
                // Linear interpolation to find exact break-even point
                let ratio = prev.pnl.abs() / (prev.pnl.abs() + curr.pnl.abs());
                let break_even = prev.underlying_price + ratio * (curr.underlying_price - prev.underlying_price);
                break_even_points.push(break_even);
            }
        }

        // Find max profit and max loss
        let max_profit = total_pnl_curve.iter()
            .map(|point| point.pnl)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let max_loss = total_pnl_curve.iter()
            .map(|point| point.pnl)
            .fold(f64::INFINITY, f64::min);

        // Calculate total Greeks (simplified - sum of all position Greeks)
        let total_greeks = GreeksData {
            delta: 0.0, // Would sum individual position deltas
            gamma: 0.0, // Would sum individual position gammas
            theta: 0.0, // Would sum individual position thetas
            vega: 0.0,  // Would sum individual position vegas
            rho: 0.0,   // Would sum individual position rhos
            theoretical_price: 0.0, // Not applicable for portfolio
        };

        PortfolioAnalysis {
            total_greeks,
            total_pnl_curve,
            break_even_points,
            max_profit: if max_profit.is_finite() { Some(max_profit) } else { None },
            max_loss: if max_loss.is_finite() { Some(max_loss) } else { None },
        }
    }

    // Screener implementation
    pub async fn screen_stocks(&self, request: ScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        // This would typically involve fetching data for many stocks
        // For demonstration, we'll show the structure
        
        let mut results = Vec::new();
        // In a real implementation, you'd:
        // 1. Get list of all available stocks
        // 2. Fetch recent data for each
        // 3. Apply filters
        // 4. Calculate indicators if requested
        // 5. Sort and limit results
        
        // Placeholder implementation
        let sample_result = ScreenerResult {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            price: 150.0,
            change: 2.5,
            change_percent: 1.7,
            volume: 50000000,
            market_cap: Some(2_400_000_000_000.0),
            pe_ratio: Some(25.0),
            indicators: None,
        };
        results.push(sample_result);

        Ok(ScreenerResponse {
            results,
            total_count: 1,
        })
    }

    // Market data aggregation
    pub async fn get_market_summary(&self) -> Result<MarketSummary, ApiError> {
        // Fetch major indices and market stats
        let indices = vec!["^GSPC", "^DJI", "^IXIC"]; // S&P 500, Dow, NASDAQ
        let options = ChartQueryOptions::default();
        
        let mut index_data = HashMap::new();
        for index in &indices {
            if let Ok(data) = self.fetch_ticker_data(index, &options).await {
                if let Ok(quote) = self.extract_quote_from_data(data) {
                    index_data.insert(index.to_string(), quote);
                }
            }
        }

        Ok(MarketSummary {
            indices: index_data,
            market_status: "OPEN".to_string(), // You'd determine this from market hours
            last_updated: Utc::now().to_rfc3339(),
        })
    }
}

// HTTP Server traits (you can implement with your preferred web framework)
pub trait ApiServer {
    fn start(&self, port: u16) -> Result<(), Box<dyn Error>>;
}

// REST endpoints would be implemented here using your preferred web framework
// For example, with a simple HTTP server or actix-web, warp, etc.

#[cfg(feature = "simple-server")]
mod simple_server {
    use std::net::TcpListener;
    use std::io::prelude::*;
    use std::net::TcpStream;
    use std::sync::Arc;
    use std::error::Error;
    use crate::StockDataApi;

    pub struct SimpleApiServer {
        api: Arc<StockDataApi>,
    }
    
    impl SimpleApiServer {
        pub fn new(api: StockDataApi) -> Self {
            Self {
                api: Arc::new(api),
            }
        }
        
        pub fn start(&self, port: u16) -> Result<(), Box<dyn Error>> {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
            println!("Server running on http://127.0.0.1:{}", port);
            
            for stream in listener.incoming() {
                let stream = stream?;
                let api = Arc::clone(&self.api);
                
                // Handle request (this is a simplified example)
                std::thread::spawn(move || {
                    handle_connection(stream, api);
                });
            }
            
            Ok(())
        }
    }
    
    fn handle_connection(mut stream: TcpStream, _api: Arc<StockDataApi>) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        
        // Parse HTTP request and route to appropriate API method
        // This is where you'd implement the actual HTTP routing logic
        
        let response = "HTTP/1.1 200 OK\r\n\r\nAPI Server Running";
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

// Additional response types
#[derive(Debug, Serialize)]
pub struct MarketSummary {
    pub indices: HashMap<String, Quote>,
    pub market_status: String,
    pub last_updated: String,
}

// Usage Examples and Integration
pub mod examples {
    use super::*;
    use tokio;
    use crate::StockDataApi;

    pub async fn example_usage() -> Result<(), Box<dyn Error>> {
        // Initialize the API
        let chart_fetcher = Arc::new(AsyncFetcher::new());
        let options_fetcher = Arc::new(AsyncOptionsFetcher::new());
        let indicators = build_indicators();
        
        let api = StockDataApi::new(chart_fetcher, options_fetcher, indicators);

        // Example 1: Get historical data with indicators
        let hist_request = HistoricalDataRequest {
            tickers: vec!["AAPL".to_string(), "MSFT".to_string()],
            interval: Some("1d".to_string()),
            range: Some("3mo".to_string()),
            start_date: None,
            end_date: None,
            include_indicators: Some(true),
            indicators: Some(vec![
                IndicatorConfig {
                    name: "SMA".to_string(),
                    params: Some([("period".to_string(), serde_json::Value::Number(20.into()))].iter().cloned().collect()),
                },
                IndicatorConfig {
                    name: "RSI".to_string(),
                    params: Some([("period".to_string(), serde_json::Value::Number(14.into()))].iter().cloned().collect()),
                },
            ]),
        };

        match api.get_historical_data(hist_request).await {
            Ok(response) => {
                println!("Historical data retrieved for {} tickers", response.data.len());
                for (ticker, data) in &response.data {
                    println!("{}: {} candles", ticker, data.candles.len());
                    if let Some(ref indicators) = data.indicators {
                        println!("  Indicators: {:?}", indicators.keys());
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }

        // Example 2: Get options chain with Greeks
        let options_request = OptionsChainRequest {
            ticker: "AAPL".to_string(),
            expiration_dates: None,
            min_strike: Some(140.0),
            max_strike: Some(160.0),
            option_type: Some("both".to_string()),
            include_greeks: Some(true),
            volatility: Some(0.3),
            risk_free_rate: Some(0.02),
        };

        match api.get_options_chain(options_request).await {
            Ok(response) => {
                println!("Options chain for {}: {} expirations", response.symbol, response.expirations.len());
                for (expiry, data) in &response.expirations {
                    println!("  {}: {} calls, {} puts", expiry, data.calls.len(), data.puts.len());
                }
            }
            Err(e) => eprintln!("Options error: {}", e),
        }

        // Example 3: Calculate options P&L
        let pnl_request = OptionsPnLRequest {
            positions: vec![
                OptionPosition {
                    option_type: "call".to_string(),
                    strike: 150.0,
                    quantity: 10,
                    entry_price: 5.0,
                    days_to_expiry: 30.0,
                },
                OptionPosition {
                    option_type: "put".to_string(),
                    strike: 145.0,
                    quantity: -5, // Short position
                    entry_price: 3.0,
                    days_to_expiry: 30.0,
                },
            ],
            underlying_prices: (140..161).map(|x| x as f64).collect(),
            volatility: Some(0.25),
            risk_free_rate: Some(0.02),
            days_to_expiry: Some(30.0),
        };

        match api.calculate_options_pnl(pnl_request) {
            Ok(response) => {
                println!("P&L analysis completed for {} positions", response.positions.len());
                println!("Break-even points: {:?}", response.portfolio.break_even_points);
                if let Some(max_profit) = response.portfolio.max_profit {
                    println!("Max profit: ${:.2}", max_profit);
                }
                if let Some(max_loss) = response.portfolio.max_loss {
                    println!("Max loss: ${:.2}", max_loss);
                }
            }
            Err(e) => eprintln!("P&L calculation error: {}", e),
        }

        // Example 4: Get real-time quotes
        let quote_request = QuoteRequest {
            tickers: vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()],
            fields: None,
        };

        match api.get_quotes(quote_request).await {
            Ok(response) => {
                println!("Retrieved {} quotes", response.quotes.len());
                for (ticker, quote) in &response.quotes {
                    println!("{}: ${:.2} ({:+.2}%)", ticker, quote.price, quote.change_percent);
                }
            }
            Err(e) => eprintln!("Quote error: {}", e),
        }

        Ok(())
    }

    // JSON API Examples (what the HTTP endpoints would return)
    pub fn print_json_examples() {
        // Historical Data Response Example
        let hist_response = HistoricalDataResponse {
            data: [(
                "AAPL".to_string(),
                TickerData {
                    symbol: "AAPL".to_string(),
                    candles: vec![
                        CandleData {
                            timestamp: 1640995200,
                            datetime: "2022-01-01T00:00:00Z".to_string(),
                            open: 150.0,
                            high: 155.0,
                            low: 149.0,
                            close: 154.0,
                            volume: Some(50000000.0),
                            adj_close: Some(154.0),
                        }
                    ],
                    indicators: Some([
                        ("SMA(20)".to_string(), vec![Some(152.5)]),
                        ("RSI(14)".to_string(), vec![Some(65.2)]),
                    ].iter().cloned().collect()),
                    meta: TickerMeta {
                        currency: "USD".to_string(),
                        exchange: "NASDAQ".to_string(),
                        instrument_type: "EQUITY".to_string(),
                        timezone: "EST".to_string(),
                        regular_market_price: 154.0,
                        fifty_two_week_high: 180.0,
                        fifty_two_week_low: 120.0,
                        market_cap: Some(2_400_000_000_000.0),
                        pe_ratio: Some(25.0),
                        dividend_yield: Some(0.6),
                    },
                },
            )].into_iter().collect(),
            errors: vec![],
        };

        println!("Historical Data JSON:");
        println!("{}", serde_json::to_string_pretty(&hist_response).unwrap());

        // Options Chain Response Example
        let options_response = OptionsChainResponse {
            symbol: "AAPL".to_string(),
            underlying_price: 154.0,
            expirations: [(
                "2024-02-16".to_string(),
                ExpirationData {
                    expiration_date: "2024-02-16".to_string(),
                    days_to_expiry: 30.0,
                    calls: vec![
                        OptionContractData {
                            strike: 150.0,
                            bid: 8.5,
                            ask: 9.0,
                            last: 8.75,
                            volume: 1250,
                            open_interest: 5000,
                            implied_volatility: Some(0.28),
                            greeks: Some(GreeksData {
                                delta: 0.65,
                                gamma: 0.025,
                                theta: -0.12,
                                vega: 0.35,
                                rho: 0.08,
                                theoretical_price: 8.72,
                            }),
                        }
                    ],
                    puts: vec![
                        OptionContractData {
                            strike: 150.0,
                            bid: 2.8,
                            ask: 3.2,
                            last: 3.0,
                            volume: 800,
                            open_interest: 3200,
                            implied_volatility: Some(0.26),
                            greeks: Some(GreeksData {
                                delta: -0.35,
                                gamma: 0.025,
                                theta: -0.08,
                                vega: 0.35,
                                rho: -0.04,
                                theoretical_price: 2.95,
                            }),
                        }
                    ],
                },
            )].iter().cloned().collect(),
            greeks_params: Some(GreeksParams {
                volatility: 0.25,
                risk_free_rate: 0.02,
            }),
        };

        println!("\nOptions Chain JSON:");
        println!("{}", serde_json::to_string_pretty(&options_response).unwrap());
    }
}

// HTTP Server Implementation using std library only
#[cfg(feature = "simple-server")]
pub mod http_server {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::io::{Read, Write, BufRead, BufReader};
    use std::collections::HashMap;
    use crate::StockDataApi;

    pub struct StockApiServer {
        api: Arc<StockDataApi>,
    }

    impl StockApiServer {
        pub fn new(api: StockDataApi) -> Self {
            Self {
                api: Arc::new(api),
            }
        }

        pub fn start(&self, addr: &str) -> Result<(), Box<dyn Error>> {
            let listener = TcpListener::bind(addr)?;
            println!("Stock API Server running on http://{}", addr);
            println!("Available endpoints:");
            println!("  GET  /api/v1/historical?tickers=AAPL,MSFT&range=1mo");
            println!("  GET  /api/v1/options?ticker=AAPL&include_greeks=true");
            println!("  POST /api/v1/options/pnl");
            println!("  GET  /api/v1/quotes?tickers=AAPL,MSFT");
            println!("  GET  /api/v1/market/summary");

            for stream in listener.incoming() {
                let stream = stream?;
                let api = Arc::clone(&self.api);
                
                tokio::spawn(async move {
                    if let Err(e) = handle_request(stream, api).await {
                        eprintln!("Request handling error: {}", e);
                    }
                });
            }

            Ok(())
        }
    }

    async fn handle_request(mut stream: TcpStream, api: Arc<StockDataApi>) -> Result<(), Box<dyn Error>> {
        let reader_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() < 2 {
            send_response(&mut stream, 400, "Bad Request", "Invalid request line")?;
            return Ok(());
        }

        let method = parts[0];
        let path_with_query = parts[1];
        let (path, query) = parse_path_query(path_with_query);

        // CORS headers to be reused
        let cors_headers = concat!(
            "Access-Control-Allow-Origin: http://localhost:3000\r\n",
            "Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n",
            "Access-Control-Allow-Headers: Content-Type, Authorization\r\n",
            "Access-Control-Allow-Credentials: true\r\n",
        );

        // Handle OPTIONS preflight request
        if method == "OPTIONS" {
            // Usually you just reply with headers + 204 No Content
            let response = format!(
                "HTTP/1.1 204 No Content\r\n{}\r\n",
                cors_headers
            );
            stream.write_all(response.as_bytes())?;
            stream.flush()?;
            return Ok(());
        }

        // For non-OPTIONS methods, you must include CORS headers in the response
        // For example in your send_json_response function:
        // add Access-Control-Allow-Origin and other headers there

        match (method, path.as_str()) {
            ("GET", "/api/v1/historical") => {
                handle_historical_data(&mut stream, &*api, query).await?;
            }
            ("GET", "/api/v1/options") => {
                handle_options_chain(&mut stream, &*api, query).await?;
            }
            ("GET", "/api/v1/quotes") => {
                handle_quotes(&mut stream, &*api, query).await?;
            }
            ("GET", "/api/v1/market/summary") => {
                handle_market_summary(&mut stream, &*api).await?;
            }
            ("POST", "/api/v1/options/pnl") => {
                handle_options_pnl(&mut stream, &*api, &mut reader).await?;
            }
            _ => {
                send_response(&mut stream, 404, "Not Found", "Endpoint not found")?;
            }
        }

        Ok(())
    }

    fn parse_path_query(path_with_query: &str) -> (String, HashMap<String, String>) {
        let mut query_params = HashMap::new();
        
        if let Some(query_start) = path_with_query.find('?') {
            let path = path_with_query[..query_start].to_string();
            let query_string = &path_with_query[query_start + 1..];
            
            for param in query_string.split('&') {
                if let Some(eq_pos) = param.find('=') {
                    let key = param[..eq_pos].to_string();
                    let value = param[eq_pos + 1..].to_string();
                    query_params.insert(key, value);
                }
            }
            
            (path, query_params)
        } else {
            (path_with_query.to_string(), query_params)
        }
    }

    async fn handle_historical_data(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let tickers = query.get("tickers")
            .map(|t| t.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        let request = HistoricalDataRequest {
            tickers,
            interval: query.get("interval").cloned(),
            range: query.get("range").cloned(),
            start_date: query.get("start_date").cloned(),
            end_date: query.get("end_date").cloned(),
            include_indicators: query.get("include_indicators").map(|v| v == "true"),
            indicators: None, // Could parse from query params
        };

        match api.get_historical_data(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                send_response(stream, 500, "Internal Server Error", &e.to_string())?;
            }
        }

        Ok(())
    }

    async fn handle_options_chain(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let ticker = query.get("ticker")
            .cloned()
            .unwrap_or_else(|| "AAPL".to_string());

        let request = OptionsChainRequest {
            ticker,
            expiration_dates: None,
            min_strike: query.get("min_strike").and_then(|s| s.parse().ok()),
            max_strike: query.get("max_strike").and_then(|s| s.parse().ok()),
            option_type: query.get("option_type").cloned(),
            include_greeks: query.get("include_greeks").map(|v| v == "true"),
            volatility: query.get("volatility").and_then(|s| s.parse().ok()),
            risk_free_rate: query.get("risk_free_rate").and_then(|s| s.parse().ok()),
        };

        match api.get_options_chain(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                send_response(stream, 500, "Internal Server Error", &e.to_string())?;
            }
        }

        Ok(())
    }

    async fn handle_quotes(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let tickers = query.get("tickers")
            .map(|t| t.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        let request = QuoteRequest {
            tickers,
            fields: None,
        };

        match api.get_quotes(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                send_response(stream, 500, "Internal Server Error", &e.to_string())?;
            }
        }

        Ok(())
    }

    async fn handle_market_summary(
        stream: &mut TcpStream,
        api: &StockDataApi,
    ) -> Result<(), Box<dyn Error>> {
        match api.get_market_summary().await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                send_response(stream, 500, "Internal Server Error", &e.to_string())?;
            }
        }

        Ok(())
    }

    pub async fn handle_options_pnl(
        stream: &mut TcpStream,
        api: &StockDataApi,
        reader: &mut BufReader<TcpStream>,
    ) -> Result<(), Box<dyn Error>> {
        // Step 1: Read headers
        let mut content_length = None;
        let mut line = String::new();

        loop {
            line.clear();
            reader.read_line(&mut line)?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                break; // End of headers
            }

            if let Some(cl) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(cl.trim().parse::<usize>()?);
            }
        }

        let content_length = match content_length {
            Some(len) => len,
            None => {
                send_response(stream, 400, "Bad Request", "Missing Content-Length")?;
                return Ok(());
            }
        };

        // Step 2: Read body
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;

        // Step 3: Parse JSON
        let pnl_request: OptionsPnLRequest = match from_str(std::str::from_utf8(&body)?) {
            Ok(req) => req,
            Err(_) => {
                send_response(stream, 400, "Bad Request", "Invalid JSON in body")?;
                return Ok(());
            }
        };

        // Step 4: Call API
        let result = api.calculate_options_pnl(pnl_request);
        match result {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?
            }
            Err(e) => {
                eprintln!("P&L calculation error: {}", e);
                send_response(stream, 500, "Internal Server Error", &format!("Error: {}", e))?;
            }
        }

        Ok(())
    }

    fn send_response(
        stream: &mut TcpStream,
        status_code: u16,
        status_text: &str,
        body: &str,
    ) -> Result<(), Box<dyn Error>> {
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            status_code, status_text, body.len(), body
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn send_json_response(
        stream: &mut TcpStream,
        status_code: u16,
        json: &str,
    ) -> Result<(), Box<dyn Error>> {
        let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: http://localhost:3000\r\nAccess-Control-Allow-Credentials: true\r\n\r\n{}",
            status_code, json.len(), json
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}