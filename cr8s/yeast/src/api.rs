// Complete implementation of the API methods and usage examples

use chrono::{DateTime, Utc, TimeZone};
use std::time::{UNIX_EPOCH, Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use serde_json::from_str;
use regex::Regex;
use tokio::sync::RwLock as AsyncRwLock;

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
// Enhanced screener request types
#[derive(Debug, Deserialize)]
pub struct ScreenerRequest {
    pub filters: Vec<ScreenerFilter>,
    pub indicators: Option<Vec<IndicatorConfig>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub screener_type: Option<String>, // "predefined" or "custom"
    pub predefined_screener: Option<String>, // "most_actives", "gainers", "losers", etc.
}

#[derive(Debug, Deserialize)]
pub struct ScreenerFilter {
    pub field: String, // "price", "volume", "market_cap", "pe_ratio", "change_percent", etc.
    pub operator: String, // "gt", "lt", "gte", "lte", "eq", "between", "in"
    pub value: serde_json::Value,
    pub secondary_value: Option<serde_json::Value>, // For "between" operator
}

// Yahoo Finance screener response structures
#[derive(Debug, Deserialize)]
pub struct YahooScreenerResponse {
    pub finance: YahooScreenerFinance,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YahooScreenerFinance {
    pub result: Vec<YahooScreenerResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YahooScreenerResult {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub canonical_name: Option<String>,
    pub criteria: Option<serde_json::Value>,
    pub predefined: Option<bool>,
    pub count: Option<u32>,
    pub quotes: Option<Vec<YahooScreenerQuote>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YahooScreenerQuote {
    pub language: Option<String>,
    pub region: Option<String>,
    pub quote_type: Option<String>,
    pub type_disp: Option<String>,
    pub quote_source_name: Option<String>,
    pub triggerable: Option<bool>,
    pub custom_price_alert_confidence: Option<String>,
    pub currency: Option<String>,
    pub market_state: Option<String>,
    pub regular_market_change_percent: Option<f64>,
    pub regular_market_price: Option<f64>,
    pub exchange: Option<String>,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub message_board_id: Option<String>,
    pub exchange_timezone_name: Option<String>,
    pub exchange_timezone_short_name: Option<String>,
    pub gmt_off_set_milliseconds: Option<i64>,
    pub market: Option<String>,
    pub esg_populated: Option<bool>,
    pub first_trade_date_milliseconds: Option<i64>,
    pub price_hint: Option<u8>,
    pub regular_market_change: Option<f64>,
    pub regular_market_time: Option<i64>,
    pub regular_market_day_high: Option<f64>,
    pub regular_market_day_range: Option<String>,
    pub regular_market_day_low: Option<f64>,
    pub regular_market_volume: Option<u64>,
    pub regular_market_previous_close: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub bid_size: Option<u32>,
    pub ask_size: Option<u32>,
    pub full_exchange_name: Option<String>,
    pub financial_currency: Option<String>,
    pub regular_market_open: Option<f64>,
    pub average_daily_volume_3_month: Option<u64>,
    pub average_daily_volume_10_day: Option<u64>,
    pub fifty_two_week_low_change: Option<f64>,
    pub fifty_two_week_low_change_percent: Option<f64>,
    pub fifty_two_week_range: Option<String>,
    pub fifty_two_week_high_change: Option<f64>,
    pub fifty_two_week_high_change_percent: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub dividend_date: Option<i64>,
    pub earnings_timestamp: Option<i64>,
    pub earnings_timestamp_start: Option<i64>,
    pub earnings_timestamp_end: Option<i64>,
    pub trailing_annual_dividend_rate: Option<f64>,
    pub trailing_annual_dividend_yield: Option<f64>,
    pub symbol: String,
    pub book_value: Option<f64>,
    pub price_to_book: Option<f64>,
    pub source_interval: Option<u32>,
    pub exchange_data_delayed_by: Option<u32>,
    pub tradeable: Option<bool>,
    pub crypto_tradeable: Option<bool>,
    pub market_cap: Option<u64>,
    pub forward_pe: Option<f64>,
    pub trailing_pe: Option<f64>,
    pub price_eps_current_year: Option<f64>,
    pub shares_outstanding: Option<u64>,
    pub price_to_sales_trailing_12_months: Option<f64>,
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


#[derive(Debug, Serialize)]
pub struct ScreenerResponse {
    pub results: Vec<ScreenerResult>,
    pub total_count: usize,
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

// Quote Summary API Types
#[derive(Debug, Serialize)]
pub struct QuoteSummaryResponse {
    pub symbol: String,
    pub asset_profile: Option<AssetProfile>,
    pub financial_data: Option<FinancialData>,
    pub default_key_statistics: Option<DefaultKeyStatistics>,
    pub summary_detail: Option<SummaryDetail>,
    pub price: Option<PriceData>,
    pub summary_profile: Option<SummaryProfile>,
}

#[derive(Debug, Serialize)]
pub struct AssetProfile {
    pub address1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub sector: Option<String>,
    pub long_business_summary: Option<String>,
    pub full_time_employees: Option<u64>,
    pub company_officers: Vec<CompanyOfficer>,
}

#[derive(Debug, Serialize)]
pub struct CompanyOfficer {
    pub name: String,
    pub title: String,
    pub age: Option<u32>,
    pub total_pay: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct FinancialData {
    pub current_price: Option<f64>,
    pub target_high_price: Option<f64>,
    pub target_low_price: Option<f64>,
    pub target_mean_price: Option<f64>,
    pub recommendation_mean: Option<f64>,
    pub recommendation_key: Option<String>,
    pub number_of_analyst_opinions: Option<u32>,
    pub total_cash: Option<f64>,
    pub total_cash_per_share: Option<f64>,
    pub ebitda: Option<f64>,
    pub total_debt: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub current_ratio: Option<f64>,
    pub total_revenue: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub revenue_per_share: Option<f64>,
    pub return_on_assets: Option<f64>,
    pub return_on_equity: Option<f64>,
    pub gross_profits: Option<f64>,
    pub free_cashflow: Option<f64>,
    pub operating_cashflow: Option<f64>,
    pub earnings_growth: Option<f64>,
    pub revenue_growth: Option<f64>,
    pub gross_margins: Option<f64>,
    pub ebitda_margins: Option<f64>,
    pub operating_margins: Option<f64>,
    pub profit_margins: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DefaultKeyStatistics {
    pub forward_pe: Option<f64>,
    pub trailing_pe: Option<f64>,
    pub peg_ratio: Option<f64>,
    pub price_to_sales_trailing_12_months: Option<f64>,
    pub price_to_book: Option<f64>,
    pub enterprise_to_revenue: Option<f64>,
    pub enterprise_to_ebitda: Option<f64>,
    pub beta: Option<f64>,
    pub fifty_two_week_change: Option<f64>,
    pub sp_500_52_week_change: Option<f64>,
    pub shares_outstanding: Option<f64>,
    pub float_shares: Option<f64>,
    pub shares_short: Option<f64>,
    pub short_ratio: Option<f64>,
    pub book_value: Option<f64>,
    //pub price_to_book: Option<f64>,
    pub earnings_quarterly_growth: Option<f64>,
    pub net_income_to_common: Option<f64>,
    pub trailing_eps: Option<f64>,
    pub forward_eps: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SummaryDetail {
    pub previous_close: Option<f64>,
    pub regular_market_open: Option<f64>,
    pub two_hundred_day_average: Option<f64>,
    pub trailing_annual_dividend_yield: Option<f64>,
    pub pay_out_ratio: Option<f64>,
    pub volume_24hr: Option<u64>,
    pub regular_market_previous_close: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub bid_size: Option<u32>,
    pub ask_size: Option<u32>,
    pub market_cap: Option<f64>,
    pub yield_: Option<f64>,
    pub ytd_return: Option<f64>,
    pub total_assets: Option<f64>,
    pub expense_ratio: Option<f64>,
    pub beta: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PriceData {
    pub regular_market_price: f64,
    pub regular_market_change: f64,
    pub regular_market_change_percent: f64,
    pub regular_market_time: i64,
    pub regular_market_day_high: f64,
    pub regular_market_day_low: f64,
    pub regular_market_volume: u64,
    pub pre_market_price: Option<f64>,
    pub pre_market_change: Option<f64>,
    pub pre_market_change_percent: Option<f64>,
    pub post_market_price: Option<f64>,
    pub post_market_change: Option<f64>,
    pub post_market_change_percent: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SummaryProfile {
    pub address1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub sector: Option<String>,
    pub long_business_summary: Option<String>,
    pub full_time_employees: Option<u64>,
}

// News API Types
#[derive(Debug, Serialize)]
pub struct NewsResponse {
    pub stories: Vec<NewsStory>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct NewsStory {
    pub uuid: String,
    pub title: String,
    pub link: String,
    pub summary: Option<String>,
    pub publisher: String,
    pub author: Option<String>,
    pub publish_time: i64,
    pub provider_publish_time: i64,
    pub news_type: String,
    pub thumbnail: Option<String>,
    pub related_tickers: Vec<String>,
}

// Calendar API Types
#[derive(Debug, Serialize)]
pub struct CalendarResponse {
    pub earnings: Vec<EarningsEvent>,
    pub dividends: Vec<DividendEvent>,
    pub splits: Vec<SplitEvent>,
    pub ipos: Vec<IpoEvent>,
}

#[derive(Debug, Serialize)]
pub struct EarningsEvent {
    pub ticker: String,
    pub company_name: String,
    pub earnings_date: String,
    pub earnings_call_time: Option<String>,
    pub eps_estimate: Option<f64>,
    pub reported_eps: Option<f64>,
    pub surprise_percent: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DividendEvent {
    pub ticker: String,
    pub company_name: String,
    pub ex_dividend_date: String,
    pub dividend_rate: f64,
    pub annual_dividend_rate: f64,
    pub annual_dividend_yield: f64,
    pub pay_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SplitEvent {
    pub ticker: String,
    pub company_name: String,
    pub ex_date: String,
    pub split_ratio: String,
    pub from_factor: f64,
    pub to_factor: f64,
}

#[derive(Debug, Serialize)]
pub struct IpoEvent {
    pub ticker: String,
    pub company_name: String,
    pub ipo_date: String,
    pub price_range_low: Option<f64>,
    pub price_range_high: Option<f64>,
    pub currency: String,
    pub exchange: String,
}

// Reports API Types
#[derive(Debug, Serialize)]
pub struct ReportsResponse {
    pub financials: FinancialReports,
    pub analysis: AnalysisReports,
}

#[derive(Debug, Serialize)]
pub struct FinancialReports {
    pub income_statement: Vec<FinancialStatement>,
    pub balance_sheet: Vec<FinancialStatement>,
    pub cash_flow: Vec<FinancialStatement>,
}

#[derive(Debug, Serialize)]
pub struct FinancialStatement {
    pub date: String,
    pub period_type: String, // "annual" or "quarterly"
    pub data: HashMap<String, Option<f64>>,
}

#[derive(Debug, Serialize)]
pub struct AnalysisReports {
    pub analyst_recommendations: Vec<AnalystRecommendation>,
    pub earnings_estimates: Vec<EarningsEstimate>,
    pub revenue_estimates: Vec<RevenueEstimate>,
}

#[derive(Debug, Serialize)]
pub struct AnalystRecommendation {
    pub period: String,
    pub strong_buy: u32,
    pub buy: u32,
    pub hold: u32,
    pub sell: u32,
    pub strong_sell: u32,
    pub mean_recommendation: f64,
}

#[derive(Debug, Serialize)]
pub struct EarningsEstimate {
    pub period: String,
    pub avg: Option<f64>,
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub year_ago_eps: Option<f64>,
    pub number_of_estimates: Option<u32>,
    pub growth: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RevenueEstimate {
    pub period: String,
    pub avg: Option<f64>,
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub year_ago_sales: Option<f64>,
    pub number_of_estimates: Option<u32>,
    pub sales_growth: Option<f64>,
}

// Crumb cache structure
#[derive(Clone)]
pub struct CrumbCache {
    pub crumb: String,
    pub expires_at: Instant,
}

impl CrumbCache {
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

pub struct YahooFinanceClient {
    client: reqwest::Client,
    crumb: Option<String>,
}

impl YahooFinanceClient {
    pub fn new() -> Self {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::Client::builder()
            .cookie_provider(jar)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(30))
            //.gzip(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            crumb: None,
        }
    }

    // Enhanced crumb caching with TTL
    pub async fn get_cached_crumb(&mut self, symbol: &str, cache: &AsyncRwLock<Option<CrumbCache>>) -> Result<String, ApiError> {
        // Check cache first
        {
            let cache_read = cache.read().await;
            if let Some(cached) = cache_read.as_ref() {
                if !cached.is_expired() {
                    return Ok(cached.crumb.clone());
                }
            }
        }

        // Cache miss or expired, fetch new crumb
        let new_crumb = self.get_crumb(symbol).await?;
        
        // Update cache with 1 hour TTL
        {
            let mut cache_write = cache.write().await;
            *cache_write = Some(CrumbCache {
                crumb: new_crumb.clone(),
                expires_at: Instant::now() + Duration::from_secs(3600), // 1 hour
            });
        }

        Ok(new_crumb)
    }

    pub async fn get_crumb(&mut self, symbol: &str) -> Result<String, ApiError> {
        if let Some(ref crumb) = self.crumb {
            return Ok(crumb.clone());
        }

        // Method 1: Try the dedicated crumb endpoint first (most reliable)
        println!("Trying dedicated crumb endpoint...");
        match self.get_crumb_from_endpoint().await {
            Ok(crumb) => {
                println!("Successfully got crumb from endpoint: {}", crumb);
                self.crumb = Some(crumb.clone());
                return Ok(crumb);
            }
            Err(e) => {
                println!("Crumb endpoint failed: {}", e);
            }
        }

        // Method 2: Try HTML parsing approach
        println!("Trying HTML parsing approach...");
        match self.get_crumb_from_html(symbol).await {
            Ok(crumb) => {
                println!("Successfully got crumb from HTML: {}", crumb);
                self.crumb = Some(crumb.clone());
                return Ok(crumb);
            }
            Err(e) => {
                println!("HTML parsing failed: {}", e);
            }
        }

        // Method 3: Try alternative approach without crumb
        println!("All crumb methods failed, trying crumbless approach...");
        Err(ApiError::FetchError("Could not obtain crumb from any method".to_string()))
    }

    async fn get_crumb_from_endpoint(&self) -> Result<String, ApiError> {
        // First establish session by visiting main page
        let main_response = self.client
            .get("https://finance.yahoo.com/")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| ApiError::FetchError(format!("Failed to establish session: {}", e)))?;

        if main_response.status() != 200 {
            return Err(ApiError::FetchError(format!("Session establishment failed: {}", main_response.status())));
        }

        // Brief delay
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Now try the dedicated crumb endpoint
        let crumb_response = self.client
            .get("https://query2.finance.yahoo.com/v1/test/getcrumb")
            .header("Accept", "*/*")
            .header("Referer", "https://finance.yahoo.com/")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await
            .map_err(|e| ApiError::FetchError(format!("Crumb endpoint request failed: {}", e)))?;

        if crumb_response.status() != 200 {
            return Err(ApiError::FetchError(format!("Crumb endpoint returned: {}", crumb_response.status())));
        }

        let crumb = crumb_response
            .text()
            .await
            .map_err(|e| ApiError::FetchError(format!("Failed to read crumb response: {}", e)))?;

        // Clean up the crumb (remove quotes if present)
        let clean_crumb = crumb.trim().trim_matches('"').to_string();
        
        if clean_crumb.is_empty() || clean_crumb.len() > 50 {
            return Err(ApiError::FetchError("Invalid crumb format from endpoint".to_string()));
        }

        Ok(clean_crumb)
    }

    async fn get_crumb_from_html(&self, symbol: &str) -> Result<String, ApiError> {
        // Visit quote page to get HTML
        let quote_url = format!("https://finance.yahoo.com/quote/{}", symbol);
        let quote_response = self.client
            .get(&quote_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "same-origin")
            .send()
            .await
            .map_err(|e| ApiError::FetchError(format!("Failed to fetch quote page: {}", e)))?;

        if quote_response.status() != 200 {
            return Err(ApiError::FetchError(format!("Quote page fetch failed: {}", quote_response.status())));
        }

        let html = quote_response
            .text()
            .await
            .map_err(|e| ApiError::FetchError(format!("Failed to read HTML: {}", e)))?;

        // Check for blocking
        if html.contains("Access Denied") || html.contains("blocked") || html.len() < 10000 {
            return Err(ApiError::FetchError("Page appears to be blocked or incomplete".to_string()));
        }

        // Multiple crumb patterns to try
        let crumb_patterns = [
            r#""CrumbStore":\s*\{\s*"crumb":\s*"([^"]+)"\s*\}"#,
            r#"CrumbStore":\s*\{\s*crumb:\s*"([^"]+)"\s*\}"#,
            r#""crumb"\s*:\s*"([^"]+)""#,
            r#"crumb["\']?\s*:\s*["\']([^"\']+)["\']"#,
            r#"window\.crumb\s*=\s*["\']([^"\']+)["\']"#,
            r#"data-crumb\s*=\s*["\']([^"\']+)["\']"#,
        ];

        // Try each pattern
        for (i, pattern) in crumb_patterns.iter().enumerate() {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(&html) {
                    if let Some(crumb_match) = captures.get(1) {
                        let crumb = crumb_match.as_str().to_string();
                        println!("Found crumb with pattern {}: {}", i + 1, crumb);
                        return Ok(crumb);
                    }
                }
            }
        }

        // Also try searching in individual script tags
        let script_regex = Regex::new(r"<script[^>]*>(.*?)</script>").unwrap();
        for script_match in script_regex.captures_iter(&html) {
            if let Some(script_content) = script_match.get(1) {
                let script_text = script_content.as_str();
                
                for (i, pattern) in crumb_patterns.iter().enumerate() {
                    if let Ok(re) = Regex::new(pattern) {
                        if let Some(captures) = re.captures(script_text) {
                            if let Some(crumb_match) = captures.get(1) {
                                let crumb = crumb_match.as_str().to_string();
                                println!("Found crumb in script tag with pattern {}: {}", i + 1, crumb);
                                return Ok(crumb);
                            }
                        }
                    }
                }
            }
        }

        // Save HTML snippet for debugging
        let preview = &html[..std::cmp::min(2000, html.len())];
        println!("HTML preview (first 2000 chars): {}", preview);

        Err(ApiError::FetchError("Could not find crumb in HTML".to_string()))
    }

    // Alternative method that tries to work without crumb for some endpoints
    pub async fn try_crumbless_request(&self, ticker: &str) -> Result<serde_json::Value, ApiError> {
        println!("Attempting crumbless request...");
        
        // Some endpoints might work without crumb
        let endpoints_to_try = [
            format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", ticker),
            format!("https://query2.finance.yahoo.com/v10/finance/quoteSummary/{}?modules=price,summaryDetail", ticker),
            format!("https://query1.finance.yahoo.com/v7/finance/quote?symbols={}", ticker),
        ];

        for endpoint in &endpoints_to_try {
            println!("Trying endpoint: {}", endpoint);
            
            let response = self.client
                .get(endpoint)
                .header("Accept", "application/json")
                .header("Referer", &format!("https://finance.yahoo.com/quote/{}", ticker))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status() == 200 => {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            println!("Successful response from: {}", endpoint);
                            return Ok(json);
                        }
                        Err(e) => {
                            println!("JSON parse error from {}: {}", endpoint, e);
                        }
                    }
                }
                Ok(resp) => {
                    println!("HTTP error from {}: {}", endpoint, resp.status());
                }
                Err(e) => {
                    println!("Request error from {}: {}", endpoint, e);
                }
            }
        }

        Err(ApiError::FetchError("All crumbless endpoints failed".to_string()))
    }

    pub async fn fetch_quote_summary(&mut self, ticker: &str) -> Result<QuoteSummaryResponse, ApiError> {
        let crumb = self.get_crumb(ticker).await?;
        
        let modules = "assetProfile,financialData,defaultKeyStatistics,summaryDetail,price,summaryProfile";
        let url = format!(
            "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{}?modules={}&crumb={}",
            ticker, modules, crumb
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        if response.status() != 200 {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        // Parse Yahoo's complex nested JSON structure
        self.parse_quote_summary(ticker, json)
    }

    pub async fn fetch_news(&mut self, ticker: &str, count: Option<u32>) -> Result<NewsResponse, ApiError> {
        let crumb = self.get_crumb(ticker).await?;
        let count = count.unwrap_or(20);
        
        let url = format!(
            "https://query1.finance.yahoo.com/v1/finance/search?q={}&quotesCount=0&newsCount={}&crumb={}",
            ticker, count, crumb
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        if response.status() != 200 {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        self.parse_news(json)
    }

    pub async fn fetch_calendar(&mut self, from: &str, to: &str) -> Result<CalendarResponse, ApiError> {
        let crumb = self.get_crumb("AAPL").await?; // Use any symbol to get crumb
        
        let earnings_url = format!(
            "https://query1.finance.yahoo.com/v1/finance/calendar/earnings?from={}&to={}&crumb={}",
            from, to, crumb
        );

        let dividends_url = format!(
            "https://query1.finance.yahoo.com/v1/finance/calendar/dividends?from={}&to={}&crumb={}",
            from, to, crumb
        );

        // Fetch earnings data
        let earnings_response = self.client
            .get(&earnings_url)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        let dividends_response = self.client
            .get(&dividends_url)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        let earnings_json: serde_json::Value = if earnings_response.status() == 200 {
            earnings_response.json().await.unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        let dividends_json: serde_json::Value = if dividends_response.status() == 200 {
            dividends_response.json().await.unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        self.parse_calendar(earnings_json, dividends_json)
    }

    pub async fn fetch_reports(&mut self, ticker: &str) -> Result<ReportsResponse, ApiError> {
        let crumb = self.get_crumb(ticker).await?;
        
        let financials_url = format!(
            "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{}?modules=incomeStatementHistory,balanceSheetHistory,cashflowStatementHistory&crumb={}",
            ticker, crumb
        );

        let analysis_url = format!(
            "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{}?modules=recommendationTrend,earningsEstimate,revenueEstimate&crumb={}",
            ticker, crumb
        );

        let financials_response = self.client
            .get(&financials_url)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        let analysis_response = self.client
            .get(&analysis_url)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        let financials_json: serde_json::Value = if financials_response.status() == 200 {
            financials_response.json().await.unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        let analysis_json: serde_json::Value = if analysis_response.status() == 200 {
            analysis_response.json().await.unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        self.parse_reports(financials_json, analysis_json)
    }

    // Predefined screener fetch
    pub async fn fetch_predefined_screener(
        &mut self,
        screener_id: &str,
        count: Option<u32>,
        offset: Option<u32>,
        cache: &AsyncRwLock<Option<CrumbCache>>,
    ) -> Result<YahooScreenerResponse, ApiError> {
        let crumb = self.get_cached_crumb("AAPL", cache).await?;
        let count = count.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let url = format!(
            "https://query2.finance.yahoo.com/v1/finance/screener/predefined/saved?count={}&offset={}&scrIds={}&crumb={}",
            count, offset, screener_id, crumb
        );

        println!("Fetching predefined screener: {}", url);

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://finance.yahoo.com/screener")
            .send()
            .await
            .map_err(|e| ApiError::FetchError(format!("Screener request failed: {}", e)))?;

        if response.status() != 200 {
            return Err(ApiError::FetchError(format!("HTTP {}: {}", response.status(), response.status())));
        }

        let json: YahooScreenerResponse = response
            .json()
            .await
            .map_err(|e| ApiError::FetchError(format!("JSON parsing failed: {}", e)))?;

        Ok(json)
    }

    // Custom screener with filters
    pub async fn fetch_custom_screener(
        &mut self,
        filters: &[ScreenerFilter],
        sort_by: Option<&str>,
        sort_order: Option<&str>,
        count: Option<u32>,
        offset: Option<u32>,
        cache: &AsyncRwLock<Option<CrumbCache>>,
    ) -> Result<YahooScreenerResponse, ApiError> {
        let crumb = self.get_cached_crumb("AAPL", cache).await?;
        let count = count.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        // Build the screener criteria
        let criteria = self.build_screener_criteria(filters, sort_by, sort_order)?;
        
        let url = format!(
            "https://query2.finance.yahoo.com/v1/finance/screener?crumb={}",
            crumb
        );

        println!("Fetching custom screener with criteria: {}", serde_json::to_string(&criteria).unwrap_or_default());

        let body = serde_json::json!({
            "size": count,
            "offset": offset,
            "sortField": sort_by.unwrap_or("ticker"),
            "sortType": sort_order.unwrap_or("desc"),
            "quoteType": "EQUITY",
            "query": criteria
        });

        let response = self.client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Referer", "https://finance.yahoo.com/screener")
            .json(&body)
            .send()
            .await
            .map_err(|e| ApiError::FetchError(format!("Custom screener request failed: {}", e)))?;

        if response.status() != 200 {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::FetchError(format!("HTTP {}: {}", status, error_text)));
        }

        let json: YahooScreenerResponse = response
            .json()
            .await
            .map_err(|e| ApiError::FetchError(format!("JSON parsing failed: {}", e)))?;

        Ok(json)
    }

    // Build Yahoo Finance screener criteria from our filters
    fn build_screener_criteria(&self, filters: &[ScreenerFilter], sort_by: Option<&str>, sort_order: Option<&str>) -> Result<serde_json::Value, ApiError> {
        let mut operator_filters = Vec::new();

        for filter in filters {
            let yahoo_field = self.map_field_to_yahoo(&filter.field)?;
            let criterion = match filter.operator.as_str() {
                "gt" => {
                    serde_json::json!({
                        "operator": "GT",
                        "operands": [yahoo_field, filter.value]
                    })
                }
                "lt" => {
                    serde_json::json!({
                        "operator": "LT",
                        "operands": [yahoo_field, filter.value]
                    })
                }
                "gte" => {
                    serde_json::json!({
                        "operator": "GTE",
                        "operands": [yahoo_field, filter.value]
                    })
                }
                "lte" => {
                    serde_json::json!({
                        "operator": "LTE",
                        "operands": [yahoo_field, filter.value]
                    })
                }
                "eq" => {
                    serde_json::json!({
                        "operator": "EQ",
                        "operands": [yahoo_field, filter.value]
                    })
                }
                "between" => {
                    if let Some(secondary) = &filter.secondary_value {
                        serde_json::json!({
                            "operator": "BTWN",
                            "operands": [yahoo_field, filter.value, secondary]
                        })
                    } else {
                        return Err(ApiError::InvalidParameters("Between operator requires secondary_value".to_string()));
                    }
                }
                "in" => {
                    if let Some(values) = filter.value.as_array() {
                        serde_json::json!({
                            "operator": "IN",
                            "operands": [yahoo_field, values]
                        })
                    } else {
                        return Err(ApiError::InvalidParameters("In operator requires array value".to_string()));
                    }
                }
                _ => return Err(ApiError::InvalidParameters(format!("Unknown operator: {}", filter.operator))),
            };

            operator_filters.push(criterion);
        }

        // Build the main query structure
        let query = if operator_filters.len() == 1 {
            operator_filters.into_iter().next().unwrap()
        } else if operator_filters.len() > 1 {
            serde_json::json!({
                "operator": "AND",
                "operands": operator_filters
            })
        } else {
            // No filters, return all equities
            serde_json::json!({
                "operator": "EQ",
                "operands": ["region", "us"]
            })
        };

        Ok(query)
    }

    // Map our field names to Yahoo Finance field names
    fn map_field_to_yahoo(&self, field: &str) -> Result<String, ApiError> {
        let mapped = match field {
            "price" => "intradayprice",
            "volume" => "intradayvolume",
            "market_cap" => "intradaymarketcap",
            "pe_ratio" => "pe",
            "trailing_pe" => "trailingpe",
            "forward_pe" => "forwardpe",
            "peg_ratio" => "pegratio",
            "price_to_book" => "pb",
            "price_to_sales" => "ps",
            "dividend_yield" => "dividendyield",
            "change_percent" => "percentchange",
            "change" => "change",
            "fifty_two_week_high" => "week52high",
            "fifty_two_week_low" => "week52low",
            "beta" => "beta",
            "eps" => "trailingeps",
            "revenue" => "totalrevenue",
            "debt_to_equity" => "debttoequity",
            "return_on_equity" => "returnonequity",
            "return_on_assets" => "returnonassets",
            "profit_margin" => "profitmargin",
            "operating_margin" => "operatingmargin",
            "gross_margin" => "grossmargin",
            "current_ratio" => "currentratio",
            "quick_ratio" => "quickratio",
            "sector" => "sector",
            "industry" => "industry",
            "country" => "country",
            "exchange" => "exchange",
            _ => return Err(ApiError::InvalidParameters(format!("Unknown field: {}", field))),
        };
        Ok(mapped.to_string())
    }

    // Convert Yahoo screener quote to our format
    fn convert_yahoo_quote_to_screener_result(
        &self,
        quote: &YahooScreenerQuote,
        indicators: Option<HashMap<String, f64>>,
    ) -> ScreenerResult {
        ScreenerResult {
            symbol: quote.symbol.clone(),
            name: quote.short_name.clone().unwrap_or_else(|| quote.long_name.clone().unwrap_or_default()),
            price: quote.regular_market_price.unwrap_or(0.0),
            change: quote.regular_market_change.unwrap_or(0.0),
            change_percent: quote.regular_market_change_percent.unwrap_or(0.0),
            volume: quote.regular_market_volume.unwrap_or(0),
            market_cap: quote.market_cap.map(|mc| mc as f64),
            pe_ratio: quote.trailing_pe,
            indicators,
        }
    }

    // Helper parsing methods
    fn parse_quote_summary(&self, ticker: &str, json: serde_json::Value) -> Result<QuoteSummaryResponse, ApiError> {
        let result = json
            .get("quoteSummary")
            .and_then(|qs| qs.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ApiError::DataNotFound("No quote summary data".to_string()))?;

        // Extract each module (this is simplified - in reality you'd parse all the nested data)
        let asset_profile = result.get("assetProfile").map(|ap| AssetProfile {
            address1: ap.get("address1").and_then(|v| v.as_str()).map(String::from),
            city: ap.get("city").and_then(|v| v.as_str()).map(String::from),
            state: ap.get("state").and_then(|v| v.as_str()).map(String::from),
            zip: ap.get("zip").and_then(|v| v.as_str()).map(String::from),
            country: ap.get("country").and_then(|v| v.as_str()).map(String::from),
            phone: ap.get("phone").and_then(|v| v.as_str()).map(String::from),
            website: ap.get("website").and_then(|v| v.as_str()).map(String::from),
            industry: ap.get("industry").and_then(|v| v.as_str()).map(String::from),
            sector: ap.get("sector").and_then(|v| v.as_str()).map(String::from),
            long_business_summary: ap.get("longBusinessSummary").and_then(|v| v.as_str()).map(String::from),
            full_time_employees: ap.get("fullTimeEmployees").and_then(|v| v.as_u64()),
            company_officers: Vec::new(), // Would parse officers array
        });

        let financial_data = result.get("financialData").map(|fd| FinancialData {
            current_price: fd.get("currentPrice").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            target_high_price: fd.get("targetHighPrice").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            target_low_price: fd.get("targetLowPrice").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            target_mean_price: fd.get("targetMeanPrice").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            recommendation_mean: fd.get("recommendationMean").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            recommendation_key: fd.get("recommendationKey").and_then(|v| v.as_str()).map(String::from),
            number_of_analyst_opinions: fd.get("numberOfAnalystOpinions").and_then(|v| v.get("raw")).and_then(|v| v.as_u64()).map(|v| v as u32),
            total_cash: fd.get("totalCash").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            total_cash_per_share: fd.get("totalCashPerShare").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            ebitda: fd.get("ebitda").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            total_debt: fd.get("totalDebt").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            quick_ratio: fd.get("quickRatio").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            current_ratio: fd.get("currentRatio").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            total_revenue: fd.get("totalRevenue").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            debt_to_equity: fd.get("debtToEquity").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            revenue_per_share: fd.get("revenuePerShare").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            return_on_assets: fd.get("returnOnAssets").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            return_on_equity: fd.get("returnOnEquity").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            gross_profits: fd.get("grossProfits").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            free_cashflow: fd.get("freeCashflow").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            operating_cashflow: fd.get("operatingCashflow").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            earnings_growth: fd.get("earningsGrowth").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            revenue_growth: fd.get("revenueGrowth").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            gross_margins: fd.get("grossMargins").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            ebitda_margins: fd.get("ebitdaMargins").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            operating_margins: fd.get("operatingMargins").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            profit_margins: fd.get("profitMargins").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
        });

        // Continue with other modules...
        let default_key_statistics = result.get("defaultKeyStatistics").map(|dks| DefaultKeyStatistics {
            forward_pe: dks.get("forwardPE").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            trailing_pe: dks.get("trailingPE").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            peg_ratio: dks.get("pegRatio").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            price_to_sales_trailing_12_months: dks.get("priceToSalesTrailing12Months").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            price_to_book: dks.get("priceToBook").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            enterprise_to_revenue: dks.get("enterpriseToRevenue").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            enterprise_to_ebitda: dks.get("enterpriseToEbitda").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            beta: dks.get("beta").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            fifty_two_week_change: dks.get("52WeekChange").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            sp_500_52_week_change: dks.get("SandP52WeekChange").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            shares_outstanding: dks.get("sharesOutstanding").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            float_shares: dks.get("floatShares").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            shares_short: dks.get("sharesShort").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            short_ratio: dks.get("shortRatio").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            book_value: dks.get("bookValue").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            earnings_quarterly_growth: dks.get("earningsQuarterlyGrowth").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            net_income_to_common: dks.get("netIncomeToCommon").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            trailing_eps: dks.get("trailingEps").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
            forward_eps: dks.get("forwardEps").and_then(|v| v.get("raw")).and_then(|v| v.as_f64()),
        });

        // Parse other modules similarly...
        let summary_detail = None; // Implement similar parsing
        let price = None; // Implement similar parsing  
        let summary_profile = None; // Implement similar parsing

        Ok(QuoteSummaryResponse {
            symbol: ticker.to_string(),
            asset_profile,
            financial_data,
            default_key_statistics,
            summary_detail,
            price,
            summary_profile,
        })
    }

    fn parse_news(&self, json: serde_json::Value) -> Result<NewsResponse, ApiError> {
        let news_array = json
            .get("news")
            .and_then(|n| n.as_array())
            .ok_or_else(|| ApiError::DataNotFound("No news data".to_string()))?;

        let mut stories = Vec::new();
        for item in news_array {
            if let Some(story) = self.parse_news_item(item) {
                stories.push(story);
            }
        }

        Ok(NewsResponse {
            total_count: stories.len(),
            stories,
        })
    }

    fn parse_news_item(&self, item: &serde_json::Value) -> Option<NewsStory> {
        Some(NewsStory {
            uuid: item.get("uuid")?.as_str()?.to_string(),
            title: item.get("title")?.as_str()?.to_string(),
            link: item.get("link")?.as_str()?.to_string(),
            summary: item.get("summary").and_then(|s| s.as_str()).map(String::from),
            publisher: item.get("publisher")?.as_str()?.to_string(),
            author: item.get("author").and_then(|a| a.as_str()).map(String::from),
            publish_time: item.get("providerPublishTime")?.as_i64()?,
            provider_publish_time: item.get("providerPublishTime")?.as_i64()?,
            news_type: item.get("type").and_then(|t| t.as_str()).unwrap_or("news").to_string(),
            thumbnail: item.get("thumbnail")
                .and_then(|t| t.get("resolutions"))
                .and_then(|r| r.as_array())
                .and_then(|arr| arr.first())
                .and_then(|res| res.get("url"))
                .and_then(|u| u.as_str())
                .map(String::from),
            related_tickers: item.get("relatedTickers")
                .and_then(|rt| rt.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|t| t.as_str())
                    .map(String::from)
                    .collect())
                .unwrap_or_default(),
        })
    }

    fn parse_calendar(&self, earnings_json: serde_json::Value, dividends_json: serde_json::Value) -> Result<CalendarResponse, ApiError> {
        let mut earnings = Vec::new();
        let mut dividends = Vec::new();

        // Parse earnings events
        if let Some(earnings_array) = earnings_json.get("finance")
            .and_then(|f| f.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("earnings"))
            .and_then(|e| e.as_array())
        {
            for event in earnings_array {
                if let Some(earnings_event) = self.parse_earnings_event(event) {
                    earnings.push(earnings_event);
                }
            }
        }

        // Parse dividend events
        if let Some(dividends_array) = dividends_json.get("finance")
            .and_then(|f| f.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("dividends"))
            .and_then(|d| d.as_array())
        {
            for event in dividends_array {
                if let Some(dividend_event) = self.parse_dividend_event(event) {
                    dividends.push(dividend_event);
                }
            }
        }

        Ok(CalendarResponse {
            earnings,
            dividends,
            splits: Vec::new(), // Would implement split parsing
            ipos: Vec::new(),   // Would implement IPO parsing
        })
    }

    fn parse_earnings_event(&self, event: &serde_json::Value) -> Option<EarningsEvent> {
        Some(EarningsEvent {
            ticker: event.get("ticker")?.as_str()?.to_string(),
            company_name: event.get("companyshortname")?.as_str()?.to_string(),
            earnings_date: event.get("startdatetime")?.as_str()?.to_string(),
            earnings_call_time: event.get("startdatetime").and_then(|s| s.as_str()).map(String::from),
            eps_estimate: event.get("epsestimate").and_then(|e| e.as_f64()),
            reported_eps: event.get("epsactual").and_then(|e| e.as_f64()),
            surprise_percent: event.get("epssurprisepct").and_then(|e| e.as_f64()),
        })
    }

    fn parse_dividend_event(&self, event: &serde_json::Value) -> Option<DividendEvent> {
        Some(DividendEvent {
            ticker: event.get("ticker")?.as_str()?.to_string(),
            company_name: event.get("companyshortname")?.as_str()?.to_string(),
            ex_dividend_date: event.get("exdividenddate")?.as_str()?.to_string(),
            dividend_rate: event.get("amount")?.as_f64()?,
            annual_dividend_rate: event.get("amount")?.as_f64()? * 4.0, // Assume quarterly
            annual_dividend_yield: event.get("yield")?.as_f64()?,
            pay_date: event.get("payoutdate").and_then(|p| p.as_str()).map(String::from),
        })
    }

    fn parse_reports(&self, financials_json: serde_json::Value, analysis_json: serde_json::Value) -> Result<ReportsResponse, ApiError> {
        let financials = self.parse_financial_reports(financials_json);
        let analysis = self.parse_analysis_reports(analysis_json);

        Ok(ReportsResponse {
            financials,
            analysis,
        })
    }

    fn parse_financial_reports(&self, json: serde_json::Value) -> FinancialReports {
        let result = json.get("quoteSummary")
            .and_then(|qs| qs.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first());

        let mut income_statement = Vec::new();
        let mut balance_sheet = Vec::new();
        let mut cash_flow = Vec::new();

        if let Some(result) = result {
            // Parse income statement
            if let Some(is_history) = result.get("incomeStatementHistory")
                .and_then(|ish| ish.get("incomeStatementHistory"))
                .and_then(|ish| ish.as_array())
            {
                for statement in is_history {
                    if let Some(parsed) = self.parse_financial_statement(statement, "annual") {
                        income_statement.push(parsed);
                    }
                }
            }

            // Parse balance sheet (similar pattern)
            if let Some(bs_history) = result.get("balanceSheetHistory")
                .and_then(|bsh| bsh.get("balanceSheetHistory"))
                .and_then(|bsh| bsh.as_array())
            {
                for statement in bs_history {
                    if let Some(parsed) = self.parse_financial_statement(statement, "annual") {
                        balance_sheet.push(parsed);
                    }
                }
            }

            // Parse cash flow (similar pattern)
            if let Some(cf_history) = result.get("cashflowStatementHistory")
                .and_then(|cfh| cfh.get("cashflowStatementHistory"))
                .and_then(|cfh| cfh.as_array())
            {
                for statement in cf_history {
                    if let Some(parsed) = self.parse_financial_statement(statement, "annual") {
                        cash_flow.push(parsed);
                    }
                }
            }
        }

        FinancialReports {
            income_statement,
            balance_sheet,
            cash_flow,
        }
    }

    fn parse_financial_statement(&self, statement: &serde_json::Value, period_type: &str) -> Option<FinancialStatement> {
        let date = statement.get("endDate")
            .and_then(|d| d.get("fmt"))
            .and_then(|f| f.as_str())?
            .to_string();

        let mut data = HashMap::new();
        
        // Extract all financial metrics (this is a simplified version)
        if let Some(obj) = statement.as_object() {
            for (key, value) in obj {
                if let Some(raw_value) = value.get("raw").and_then(|r| r.as_f64()) {
                    data.insert(key.clone(), Some(raw_value));
                }
            }
        }

        Some(FinancialStatement {
            date,
            period_type: period_type.to_string(),
            data,
        })
    }

    fn parse_analysis_reports(&self, json: serde_json::Value) -> AnalysisReports {
        // Parse analyst recommendations, earnings estimates, etc.
        // This is simplified - you'd parse the actual nested structures
        AnalysisReports {
            analyst_recommendations: Vec::new(),
            earnings_estimates: Vec::new(),
            revenue_estimates: Vec::new(),
        }
    }
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
    pub async fn screen_stocks(
        &self,
        request: ScreenerRequest,
        cache: &AsyncRwLock<Option<CrumbCache>>,
    ) -> Result<ScreenerResponse, ApiError> {
        let mut yahoo_client = YahooFinanceClient::new();

        let yahoo_response = match request.screener_type.as_deref() {
            Some("predefined") => {
                let screener_id = request.predefined_screener
                    .as_deref()
                    .unwrap_or("most_actives");
                
                yahoo_client.fetch_predefined_screener(
                    screener_id,
                    request.limit.map(|l| l as u32),
                    request.offset.map(|o| o as u32),
                    cache,
                ).await?
            }
            _ => {
                yahoo_client.fetch_custom_screener(
                    &request.filters,
                    request.sort_by.as_deref(),
                    request.sort_order.as_deref(),
                    request.limit.map(|l| l as u32),
                    request.offset.map(|o| o as u32),
                    cache,
                ).await?
            }
        };

        // Process the results
        let mut results = Vec::new();
        let total_count = yahoo_response.finance.result.len();

        for result in &yahoo_response.finance.result {
            if let Some(quotes) = &result.quotes {
                for quote in quotes {
                    // Calculate indicators if requested
                    let indicators = if request.indicators.is_some() {
                        // Would fetch historical data and calculate indicators
                        // For now, return None to keep response times reasonable
                        None
                    } else {
                        None
                    };

                    let screener_result = yahoo_client.convert_yahoo_quote_to_screener_result(quote, indicators);
                    results.push(screener_result);
                }
            }
        }

        // Apply additional sorting if specified
        if let Some(sort_field) = &request.sort_by {
            let ascending = request.sort_order.as_deref() != Some("desc");
            self.sort_screener_results(&mut results, sort_field, ascending)?;
        }

        // Apply limit after sorting
        if let Some(limit) = request.limit {
            results.truncate(limit);
        }

        Ok(ScreenerResponse {
            results,
            total_count,
        })
    }

    fn sort_screener_results(
        &self,
        results: &mut [ScreenerResult],
        sort_field: &str,
        ascending: bool,
    ) -> Result<(), ApiError> {
        match sort_field {
            "symbol" => {
                results.sort_by(|a, b| {
                    if ascending {
                        a.symbol.cmp(&b.symbol)
                    } else {
                        b.symbol.cmp(&a.symbol)
                    }
                });
            }
            "price" => {
                results.sort_by(|a, b| {
                    if ascending {
                        a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            "change_percent" => {
                results.sort_by(|a, b| {
                    if ascending {
                        a.change_percent.partial_cmp(&b.change_percent).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b.change_percent.partial_cmp(&a.change_percent).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            "volume" => {
                results.sort_by(|a, b| {
                    if ascending {
                        a.volume.cmp(&b.volume)
                    } else {
                        b.volume.cmp(&a.volume)
                    }
                });
            }
            "market_cap" => {
                results.sort_by(|a, b| {
                    let a_mc = a.market_cap.unwrap_or(0.0);
                    let b_mc = b.market_cap.unwrap_or(0.0);
                    if ascending {
                        a_mc.partial_cmp(&b_mc).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b_mc.partial_cmp(&a_mc).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            "pe_ratio" => {
                results.sort_by(|a, b| {
                    let a_pe = a.pe_ratio.unwrap_or(f64::INFINITY);
                    let b_pe = b.pe_ratio.unwrap_or(f64::INFINITY);
                    if ascending {
                        a_pe.partial_cmp(&b_pe).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b_pe.partial_cmp(&a_pe).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            _ => {
                return Err(ApiError::InvalidParameters(format!("Unknown sort field: {}", sort_field)));
            }
        }
        Ok(())
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

    pub async fn get_quote_summary(&self, ticker: &str) -> Result<QuoteSummaryResponse, ApiError> {
        let mut yahoo_client = YahooFinanceClient::new();
        yahoo_client.fetch_quote_summary(ticker).await
    }

    pub async fn get_news(&self, ticker: &str, count: Option<u32>) -> Result<NewsResponse, ApiError> {
        let mut yahoo_client = YahooFinanceClient::new();
        yahoo_client.fetch_news(ticker, count).await
    }

    pub async fn get_calendar(&self, from: &str, to: &str) -> Result<CalendarResponse, ApiError> {
        let mut yahoo_client = YahooFinanceClient::new();
        yahoo_client.fetch_calendar(from, to).await
    }

    pub async fn get_reports(&self, ticker: &str) -> Result<ReportsResponse, ApiError> {
        let mut yahoo_client = YahooFinanceClient::new();
        yahoo_client.fetch_reports(ticker).await
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
            println!("  GET  /api/v1/quotesummary?ticker=AAPL");
            println!("  GET  /api/v1/market/summary");
            println!("  GET /api/v1/news?ticker=AAPL&count=10");
            println!("  GET /api/v1/calendar?from=2024-01-01&to=2024-01-31");
            println!("  GET /api/v1/reports?ticker=AAPL");

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
            ("GET", "/api/v1/quotesummary") => {
                handle_quote_summary(&mut stream, &*api, query).await?;
            }
            ("GET", "/api/v1/news") => {
                handle_news(&mut stream, &*api, query).await?;
            }
            ("GET", "/api/v1/calendar") => {
                handle_calendar(&mut stream, &*api, query).await?;
            }
            ("GET", "/api/v1/reports") => {
                handle_reports(&mut stream, &*api, query).await?;
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

    pub async fn handle_quote_summary(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = query.get("ticker")
            .cloned()
            .unwrap_or_else(|| "AAPL".to_string());
    
        match api.get_quote_summary(&ticker).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "ticker": ticker
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json)?;
            }
        }
        Ok(())
    }
    
    pub async fn handle_news(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = query.get("ticker")
            .cloned()
            .unwrap_or_else(|| "AAPL".to_string());
        
        let count = query.get("count")
            .and_then(|c| c.parse::<u32>().ok());
    
        match api.get_news(&ticker, count).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "ticker": ticker
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json)?;
            }
        }
        Ok(())
    }
    
    pub async fn handle_calendar(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let from = query.get("from")
            .cloned()
            .unwrap_or_else(|| "2024-01-01".to_string());
        
        let to = query.get("to")
            .cloned()
            .unwrap_or_else(|| "2024-12-31".to_string());
    
        match api.get_calendar(&from, &to).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "from": from,
                    "to": to
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json)?;
            }
        }
        Ok(())
    }
    
    pub async fn handle_reports(
        stream: &mut TcpStream,
        api: &StockDataApi,
        query: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = query.get("ticker")
            .cloned()
            .unwrap_or_else(|| "AAPL".to_string());
    
        match api.get_reports(&ticker).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "ticker": ticker
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json)?;
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