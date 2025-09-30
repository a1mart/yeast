use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock as AsyncRwLock;
use chrono::{DateTime, Utc, TimeZone};
use regex::Regex;
use uuid::Uuid;

// Enhanced Error Types
#[derive(Debug, Clone)]
pub enum ApiError {
    FetchError(String),
    DataNotFound(String),
    InvalidParameters(String),
    RateLimited(String),
    AuthenticationFailed(String),
    CacheError(String),
    NetworkError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::FetchError(msg) => write!(f, "Fetch error: {}", msg),
            ApiError::DataNotFound(msg) => write!(f, "Data not found: {}", msg),
            ApiError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            ApiError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            ApiError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            ApiError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

// Enhanced Caching System
#[derive(Clone, Debug)]
pub struct CrumbCache {
    pub crumb: String,
    pub expires_at: Instant,
    pub session_cookies: String,
}

impl CrumbCache {
    pub fn new(crumb: String, ttl_seconds: u64) -> Self {
        Self {
            crumb,
            expires_at: Instant::now() + Duration::from_secs(ttl_seconds),
            session_cookies: String::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    pub fn remaining_ttl(&self) -> Option<Duration> {
        if self.is_expired() {
            None
        } else {
            Some(self.expires_at - Instant::now())
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub data: serde_json::Value,
    pub expires_at: Instant,
    pub etag: Option<String>,
}

#[derive(Debug)]
pub struct RateLimiter {
    last_request: Instant,
    min_interval: Duration,
    request_count: u32,
    window_start: Instant,
    requests_per_window: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            last_request: Instant::now() - Duration::from_secs(60),
            min_interval: Duration::from_millis(1000 / requests_per_minute.max(1) as u64),
            request_count: 0,
            window_start: Instant::now(),
            requests_per_window: requests_per_minute,
            window_duration: Duration::from_secs(60),
        }
    }

    pub async fn wait_if_needed(&mut self) {
        let now = Instant::now();
        
        if now.duration_since(self.window_start) > self.window_duration {
            self.request_count = 0;
            self.window_start = now;
        }

        if self.request_count >= self.requests_per_window {
            let wait_time = self.window_duration - now.duration_since(self.window_start);
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
                self.request_count = 0;
                self.window_start = Instant::now();
            }
        }

        let time_since_last = now.duration_since(self.last_request);
        if time_since_last < self.min_interval {
            tokio::time::sleep(self.min_interval - time_since_last).await;
        }

        self.request_count += 1;
        self.last_request = Instant::now();
    }
}

// Core Data Structures
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOverview {
    pub indices: HashMap<String, IndexData>,
    pub sectors: HashMap<String, SectorPerformance>,
    pub market_sentiment: MarketSentiment,
    pub top_movers: TopMovers,
    pub market_stats: MarketStatistics,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexData {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectorPerformance {
    pub sector: String,
    pub change_percent: f64,
    pub market_cap: f64,
    pub pe_ratio: Option<f64>,
    pub top_stocks: Vec<String>,
    pub performance_1d: f64,
    pub performance_5d: f64,
    pub performance_1m: f64,
    pub performance_3m: f64,
    pub performance_ytd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketSentiment {
    pub fear_greed_index: Option<f64>,
    pub vix: f64,
    pub put_call_ratio: f64,
    pub advance_decline_ratio: f64,
    pub sentiment_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopMovers {
    pub gainers: Vec<MoverData>,
    pub losers: Vec<MoverData>,
    pub most_active: Vec<MoverData>,
    pub unusual_volume: Vec<MoverData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoverData {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub avg_volume: u64,
    pub market_cap: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketStatistics {
    pub total_market_cap: f64,
    pub advancing_issues: u32,
    pub declining_issues: u32,
    pub unchanged_issues: u32,
    pub new_highs: u32,
    pub new_lows: u32,
    pub volume_leaders: Vec<String>,
}

// Portfolio Management
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub positions: Vec<Position>,
    pub cash_balance: f64,
    pub total_value: f64,
    pub total_return: f64,
    pub total_return_percent: f64,
    pub day_change: f64,
    pub day_change_percent: f64,
    pub alerts: Vec<PortfolioAlert>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub id: String,
    pub symbol: String,
    pub quantity: f64,
    pub average_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
    pub day_change: f64,
    pub day_change_percent: f64,
    pub weight: f64,
    pub first_bought: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub symbol: String,
    pub quantity: f64,
    pub price: f64,
    pub amount: f64,
    pub fees: f64,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransactionType {
    Buy,
    Sell,
    Dividend,
    Split,
    CashDeposit,
    CashWithdrawal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioAlert {
    pub id: String,
    pub alert_type: AlertType,
    pub condition: AlertCondition,
    pub target_value: f64,
    pub current_value: f64,
    pub is_triggered: bool,
    pub created_at: DateTime<Utc>,
    pub triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AlertType {
    PriceAlert { symbol: String },
    PortfolioValue,
    PositionWeight { symbol: String },
    DayChange,
    TotalReturn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AlertCondition {
    Above,
    Below,
    PercentChange,
}

// Screener Types
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenerRequest {
    pub filters: Vec<ScreenerFilter>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub screener_type: Option<String>,
    pub predefined_screener: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenerFilter {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub secondary_value: Option<serde_json::Value>,
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
}

// Yahoo Finance Response Types
#[derive(Debug, Deserialize)]
pub struct YahooChartResponse {
    pub chart: YahooChart,
}

#[derive(Debug, Deserialize)]
pub struct YahooChart {
    pub result: Option<Vec<YahooChartResult>>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct YahooChartResult {
    pub meta: YahooChartMeta,
    pub timestamp: Option<Vec<i64>>,
    pub indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
pub struct YahooChartMeta {
    pub currency: String,
    pub symbol: String,
    #[serde(rename = "exchangeName")]
    pub exchange_name: String,
    #[serde(rename = "instrumentType")]
    pub instrument_type: String,
    #[serde(rename = "firstTradeDate")]
    pub first_trade_date: i64,
    #[serde(rename = "regularMarketTime")]
    pub regular_market_time: i64,
    #[serde(rename = "gmtoffset")]
    pub gmt_offset: i64,
    pub timezone: String,
    #[serde(rename = "exchangeTimezoneName")]
    pub exchange_timezone_name: String,
    #[serde(rename = "regularMarketPrice")]
    pub regular_market_price: f64,
    #[serde(rename = "chartPreviousClose")]
    pub chart_previous_close: f64,
    #[serde(rename = "previousClose")]
    pub previous_close: Option<f64>,
    pub scale: Option<i32>,
    #[serde(rename = "priceHint")]
    pub price_hint: i32,
    #[serde(rename = "currentTradingPeriod")]
    pub current_trading_period: Option<serde_json::Value>,
    #[serde(rename = "tradingPeriods")]
    pub trading_periods: Option<serde_json::Value>,
    #[serde(rename = "dataGranularity")]
    pub data_granularity: String,
    pub range: String,
    #[serde(rename = "validRanges")]
    pub valid_ranges: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct YahooIndicators {
    pub quote: Option<Vec<YahooQuoteData>>,
    pub adjclose: Option<Vec<YahooAdjCloseData>>,
}

#[derive(Debug, Deserialize)]
pub struct YahooQuoteData {
    pub open: Option<Vec<Option<f64>>>,
    pub high: Option<Vec<Option<f64>>>,
    pub low: Option<Vec<Option<f64>>>,
    pub close: Option<Vec<Option<f64>>>,
    pub volume: Option<Vec<Option<u64>>>,
}

#[derive(Debug, Deserialize)]
pub struct YahooAdjCloseData {
    pub adjclose: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
pub struct YahooScreenerResponse {
    pub finance: YahooScreenerFinance,
}

#[derive(Debug, Deserialize)]
pub struct YahooScreenerFinance {
    pub result: Vec<YahooScreenerResult>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct YahooScreenerQuote {
    pub symbol: String,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub regular_market_price: Option<f64>,
    pub regular_market_change: Option<f64>,
    pub regular_market_change_percent: Option<f64>,
    pub regular_market_volume: Option<u64>,
    pub market_cap: Option<u64>,
    pub trailing_pe: Option<f64>,
    pub forward_pe: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    pub exchange: Option<String>,
    pub currency: Option<String>,
}

// Enhanced Yahoo Finance Client
pub struct EnhancedYahooFinanceClient {
    client: reqwest::Client,
    crumb_cache: Arc<AsyncRwLock<Option<CrumbCache>>>,
    rate_limiter: Arc<AsyncRwLock<RateLimiter>>,
    request_cache: Arc<AsyncRwLock<HashMap<String, CachedResponse>>>,
}

impl EnhancedYahooFinanceClient {
    pub fn new() -> Self {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::Client::builder()
            .cookie_provider(jar)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            crumb_cache: Arc::new(AsyncRwLock::new(None)),
            rate_limiter: Arc::new(AsyncRwLock::new(RateLimiter::new(30))), // Conservative 30 req/min
            request_cache: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }

    pub async fn get_crumb(&self) -> Result<String, ApiError> {
        // Check cache first
        {
            let cache_read = self.crumb_cache.read().await;
            if let Some(cached) = cache_read.as_ref() {
                if !cached.is_expired() {
                    return Ok(cached.crumb.clone());
                }
            }
        }

        // Rate limit
        self.rate_limiter.write().await.wait_if_needed().await;

        // Try multiple methods to get crumb
        let crumb = self.fetch_fresh_crumb().await?;
        
        // Update cache
        {
            let mut cache_write = self.crumb_cache.write().await;
            *cache_write = Some(CrumbCache::new(crumb.clone(), 3600)); // 1 hour TTL
        }

        Ok(crumb)
    }

    async fn fetch_fresh_crumb(&self) -> Result<String, ApiError> {
        // Method 1: Try dedicated crumb endpoint
        if let Ok(crumb) = self.try_crumb_endpoint().await {
            return Ok(crumb);
        }

        // Method 2: Try HTML parsing from quote page
        if let Ok(crumb) = self.try_crumb_from_html("AAPL").await {
            return Ok(crumb);
        }

        Err(ApiError::AuthenticationFailed("Unable to obtain crumb".to_string()))
    }

    async fn try_crumb_endpoint(&self) -> Result<String, ApiError> {
        // First establish session
        let _ = self.client
            .get("https://finance.yahoo.com/")
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        let response = self.client
            .get("https://query2.finance.yahoo.com/v1/test/getcrumb")
            .header("Referer", "https://finance.yahoo.com/")
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let crumb = response.text().await
                .map_err(|e| ApiError::ParseError(e.to_string()))?
                .trim()
                .trim_matches('"')
                .to_string();

            if !crumb.is_empty() && crumb.len() < 50 && !crumb.contains("<!DOCTYPE") {
                return Ok(crumb);
            }
        }

        Err(ApiError::AuthenticationFailed("Crumb endpoint failed".to_string()))
    }

    async fn try_crumb_from_html(&self, symbol: &str) -> Result<String, ApiError> {
        let url = format!("https://finance.yahoo.com/quote/{}", symbol);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let yahoo_response: YahooScreenerResponse = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let mut results = Vec::new();

        for result in &yahoo_response.finance.result {
            if let Some(quotes) = &result.quotes {
                for quote in quotes {
                    results.push(ScreenerResult {
                        symbol: quote.symbol.clone(),
                        name: quote.short_name.clone().unwrap_or_else(|| quote.long_name.clone().unwrap_or_default()),
                        price: quote.regular_market_price.unwrap_or(0.0),
                        change: quote.regular_market_change.unwrap_or(0.0),
                        change_percent: quote.regular_market_change_percent.unwrap_or(0.0),
                        volume: quote.regular_market_volume.unwrap_or(0),
                        market_cap: quote.market_cap.map(|mc| mc as f64),
                        pe_ratio: quote.trailing_pe,
                    });
                }
            }
        }

        Ok(results)
    }

    // Market Overview Implementation
    pub async fn fetch_market_overview(&self) -> Result<MarketOverview, ApiError> {
        let crumb = self.get_crumb().await?;
        
        let indices = self.fetch_major_indices(&crumb).await?;
        let sectors = self.fetch_sector_performance(&crumb).await?;
        let market_sentiment = self.fetch_market_sentiment(&crumb).await?;
        let top_movers = self.fetch_top_movers().await?;
        let market_stats = self.calculate_market_statistics(&indices, &sectors).await?;

        Ok(MarketOverview {
            indices,
            sectors,
            market_sentiment,
            top_movers,
            market_stats,
            last_updated: Utc::now().to_rfc3339(),
        })
    }

    async fn fetch_major_indices(&self, crumb: &str) -> Result<HashMap<String, IndexData>, ApiError> {
        let symbols = ["^GSPC", "^DJI", "^IXIC", "^RUT", "^VIX"];
        let mut indices = HashMap::new();

        for symbol in &symbols {
            if let Ok(quote) = self.fetch_single_quote(symbol, crumb).await {
                let index_data = IndexData {
                    symbol: symbol.to_string(),
                    name: self.get_index_name(symbol),
                    price: quote.price,
                    change: quote.change,
                    change_percent: quote.change_percent,
                    volume: quote.volume,
                    market_cap: quote.market_cap,
                    pe_ratio: quote.pe_ratio,
                };
                indices.insert(symbol.to_string(), index_data);
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        Ok(indices)
    }

    async fn fetch_sector_performance(&self, crumb: &str) -> Result<HashMap<String, SectorPerformance>, ApiError> {
        let sector_etfs = [
            ("XLK", "Technology"),
            ("XLF", "Financials"),
            ("XLV", "Healthcare"),
            ("XLI", "Industrials"),
            ("XLE", "Energy"),
            ("XLB", "Materials"),
            ("XLP", "Consumer Staples"),
            ("XLY", "Consumer Discretionary"),
            ("XLRE", "Real Estate"),
            ("XLU", "Utilities"),
        ];

        let mut sectors = HashMap::new();

        for (etf_symbol, sector_name) in &sector_etfs {
            if let Ok(quote) = self.fetch_single_quote(etf_symbol, crumb).await {
                let sector_perf = SectorPerformance {
                    sector: sector_name.to_string(),
                    change_percent: quote.change_percent,
                    market_cap: quote.market_cap.unwrap_or(0.0),
                    pe_ratio: quote.pe_ratio,
                    top_stocks: Vec::new(), // Would need additional API call
                    performance_1d: quote.change_percent,
                    performance_5d: 0.0, // Would need historical data
                    performance_1m: 0.0,
                    performance_3m: 0.0,
                    performance_ytd: 0.0,
                };
                sectors.insert(sector_name.to_string(), sector_perf);
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        Ok(sectors)
    }

    async fn fetch_market_sentiment(&self, crumb: &str) -> Result<MarketSentiment, ApiError> {
        let vix_quote = self.fetch_single_quote("^VIX", crumb).await?;
        
        Ok(MarketSentiment {
            fear_greed_index: None,
            vix: vix_quote.price,
            put_call_ratio: 1.0, // Would need options data
            advance_decline_ratio: 1.0, // Would need market breadth data
            sentiment_score: self.calculate_sentiment_score(vix_quote.price),
        })
    }

    async fn fetch_top_movers(&self) -> Result<TopMovers, ApiError> {
        let gainers = self.fetch_predefined_screener("day_gainers", Some(10), Some(0)).await?;
        let losers = self.fetch_predefined_screener("day_losers", Some(10), Some(0)).await?;
        let most_active = self.fetch_predefined_screener("most_actives", Some(10), Some(0)).await?;

        Ok(TopMovers {
            gainers: self.convert_to_mover_data(&gainers)?,
            losers: self.convert_to_mover_data(&losers)?,
            most_active: self.convert_to_mover_data(&most_active)?,
            unusual_volume: Vec::new(), // Would need volume comparison logic
        })
    }

    async fn calculate_market_statistics(
        &self,
        _indices: &HashMap<String, IndexData>,
        _sectors: &HashMap<String, SectorPerformance>,
    ) -> Result<MarketStatistics, ApiError> {
        Ok(MarketStatistics {
            total_market_cap: 0.0,
            advancing_issues: 0,
            declining_issues: 0,
            unchanged_issues: 0,
            new_highs: 0,
            new_lows: 0,
            volume_leaders: Vec::new(),
        })
    }

    // Helper methods
    fn get_index_name(&self, symbol: &str) -> String {
        match symbol {
            "^GSPC" => "S&P 500",
            "^DJI" => "Dow Jones Industrial Average",
            "^IXIC" => "NASDAQ Composite",
            "^RUT" => "Russell 2000",
            "^VIX" => "VIX",
            _ => symbol,
        }.to_string()
    }

    fn calculate_sentiment_score(&self, vix: f64) -> f64 {
        if vix < 12.0 {
            0.8
        } else if vix < 20.0 {
            0.8 - (vix - 12.0) / 8.0 * 0.6
        } else if vix < 30.0 {
            0.2 - (vix - 20.0) / 10.0 * 0.4
        } else {
            -0.2 - (vix - 30.0) / 20.0 * 0.6
        }
    }

    fn convert_to_mover_data(&self, results: &[ScreenerResult]) -> Result<Vec<MoverData>, ApiError> {
        Ok(results.iter().map(|result| MoverData {
            symbol: result.symbol.clone(),
            name: result.name.clone(),
            price: result.price,
            change: result.change,
            change_percent: result.change_percent,
            volume: result.volume,
            avg_volume: result.volume, // Simplified - would need historical average
            market_cap: result.market_cap,
        }).collect())
    }
}

// Portfolio Management Service
pub struct PortfolioManager {
    portfolios: Arc<AsyncRwLock<HashMap<String, Portfolio>>>,
    client: Arc<EnhancedYahooFinanceClient>,
}

impl PortfolioManager {
    pub fn new(client: Arc<EnhancedYahooFinanceClient>) -> Self {
        Self {
            portfolios: Arc::new(AsyncRwLock::new(HashMap::new())),
            client,
        }
    }

    pub async fn create_portfolio(&self, name: String, description: Option<String>) -> Result<String, ApiError> {
        let portfolio_id = Uuid::new_v4().to_string();
        let portfolio = Portfolio {
            id: portfolio_id.clone(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            positions: Vec::new(),
            cash_balance: 0.0,
            total_value: 0.0,
            total_return: 0.0,
            total_return_percent: 0.0,
            day_change: 0.0,
            day_change_percent: 0.0,
            alerts: Vec::new(),
        };

        let mut portfolios = self.portfolios.write().await;
        portfolios.insert(portfolio_id.clone(), portfolio);

        Ok(portfolio_id)
    }

    pub async fn add_position(&self, portfolio_id: &str, symbol: String, quantity: f64, price: f64) -> Result<(), ApiError> {
        let mut portfolios = self.portfolios.write().await;
        let portfolio = portfolios.get_mut(portfolio_id)
            .ok_or_else(|| ApiError::DataNotFound("Portfolio not found".to_string()))?;

        // Check if position already exists
        if let Some(existing_position) = portfolio.positions.iter_mut().find(|p| p.symbol == symbol) {
            // Update existing position (average cost calculation)
            let total_cost = existing_position.average_cost * existing_position.quantity + price * quantity;
            existing_position.quantity += quantity;
            existing_position.average_cost = total_cost / existing_position.quantity;
        } else {
            // Create new position
            let position_id = Uuid::new_v4().to_string();
            let position = Position {
                id: position_id.clone(),
                symbol: symbol.clone(),
                quantity,
                average_cost: price,
                current_price: price,
                market_value: price * quantity,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                day_change: 0.0,
                day_change_percent: 0.0,
                weight: 0.0,
                first_bought: Utc::now(),
                last_updated: Utc::now(),
                transactions: vec![Transaction {
                    id: Uuid::new_v4().to_string(),
                    transaction_type: TransactionType::Buy,
                    symbol,
                    quantity,
                    price,
                    amount: price * quantity,
                    fees: 0.0,
                    timestamp: Utc::now(),
                    notes: None,
                }],
            };
            portfolio.positions.push(position);
        }

        portfolio.updated_at = Utc::now();
        Ok(())
    }

    pub async fn update_portfolio_values(&self, portfolio_id: &str) -> Result<(), ApiError> {
        let mut portfolios = self.portfolios.write().await;
        let portfolio = portfolios.get_mut(portfolio_id)
            .ok_or_else(|| ApiError::DataNotFound("Portfolio not found".to_string()))?;

        let symbols: Vec<String> = portfolio.positions.iter().map(|p| p.symbol.clone()).collect();
        let quotes = self.client.fetch_batch_quotes(&symbols).await?;

        let mut total_value = portfolio.cash_balance;
        
        for position in &mut portfolio.positions {
            if let Some(quote) = quotes.get(&position.symbol) {
                position.current_price = quote.price;
                position.market_value = position.quantity * quote.price;
                position.unrealized_pnl = position.market_value - (position.quantity * position.average_cost);
                position.unrealized_pnl_percent = if position.average_cost > 0.0 {
                    (position.unrealized_pnl / (position.quantity * position.average_cost)) * 100.0
                } else {
                    0.0
                };
                position.day_change = quote.change * position.quantity;
                position.day_change_percent = quote.change_percent;
                position.last_updated = Utc::now();
                
                total_value += position.market_value;
            }
        }

        // Calculate portfolio-level metrics
        let total_cost: f64 = portfolio.positions.iter()
            .map(|p| p.quantity * p.average_cost)
            .sum();

        portfolio.total_value = total_value;
        portfolio.total_return = total_value - total_cost - portfolio.cash_balance;
        portfolio.total_return_percent = if total_cost > 0.0 {
            (portfolio.total_return / total_cost) * 100.0
        } else {
            0.0
        };

        portfolio.day_change = portfolio.positions.iter()
            .map(|p| p.day_change)
            .sum();

        let prev_total = total_value - portfolio.day_change;
        portfolio.day_change_percent = if prev_total > 0.0 {
            (portfolio.day_change / prev_total) * 100.0
        } else {
            0.0
        };

        // Calculate position weights
        for position in &mut portfolio.positions {
            position.weight = if total_value > 0.0 {
                (position.market_value / total_value) * 100.0
            } else {
                0.0
            };
        }

        portfolio.updated_at = Utc::now();
        Ok(())
    }

    pub async fn get_portfolio(&self, portfolio_id: &str) -> Result<Portfolio, ApiError> {
        let portfolios = self.portfolios.read().await;
        portfolios.get(portfolio_id)
            .cloned()
            .ok_or_else(|| ApiError::DataNotFound("Portfolio not found".to_string()))
    }

    pub async fn list_portfolios(&self) -> Result<Vec<Portfolio>, ApiError> {
        let portfolios = self.portfolios.read().await;
        Ok(portfolios.values().cloned().collect())
    }

    pub async fn check_alerts(&self, portfolio_id: &str) -> Result<Vec<PortfolioAlert>, ApiError> {
        let mut portfolios = self.portfolios.write().await;
        let portfolio = portfolios.get_mut(portfolio_id)
            .ok_or_else(|| ApiError::DataNotFound("Portfolio not found".to_string()))?;

        let mut triggered_alerts = Vec::new();

        for alert in &mut portfolio.alerts {
            if alert.is_triggered {
                continue;
            }

            let should_trigger = match &alert.alert_type {
                AlertType::PriceAlert { symbol } => {
                    if let Some(position) = portfolio.positions.iter().find(|p| &p.symbol == symbol) {
                        alert.current_value = position.current_price;
                        match alert.condition {
                            AlertCondition::Above => position.current_price >= alert.target_value,
                            AlertCondition::Below => position.current_price <= alert.target_value,
                            AlertCondition::PercentChange => {
                                let change_percent = ((position.current_price - position.average_cost) / position.average_cost) * 100.0;
                                change_percent.abs() >= alert.target_value
                            }
                        }
                    } else {
                        false
                    }
                }
                AlertType::PortfolioValue => {
                    alert.current_value = portfolio.total_value;
                    match alert.condition {
                        AlertCondition::Above => portfolio.total_value >= alert.target_value,
                        AlertCondition::Below => portfolio.total_value <= alert.target_value,
                        AlertCondition::PercentChange => {
                            portfolio.total_return_percent.abs() >= alert.target_value
                        }
                    }
                }
                AlertType::PositionWeight { symbol } => {
                    if let Some(position) = portfolio.positions.iter().find(|p| &p.symbol == symbol) {
                        alert.current_value = position.weight;
                        match alert.condition {
                            AlertCondition::Above => position.weight >= alert.target_value,
                            AlertCondition::Below => position.weight <= alert.target_value,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                AlertType::DayChange => {
                    alert.current_value = portfolio.day_change_percent;
                    match alert.condition {
                        AlertCondition::Above => portfolio.day_change_percent >= alert.target_value,
                        AlertCondition::Below => portfolio.day_change_percent <= alert.target_value,
                        AlertCondition::PercentChange => portfolio.day_change_percent.abs() >= alert.target_value,
                    }
                }
                AlertType::TotalReturn => {
                    alert.current_value = portfolio.total_return_percent;
                    match alert.condition {
                        AlertCondition::Above => portfolio.total_return_percent >= alert.target_value,
                        AlertCondition::Below => portfolio.total_return_percent <= alert.target_value,
                        AlertCondition::PercentChange => portfolio.total_return_percent.abs() >= alert.target_value,
                    }
                }
            };

            if should_trigger {
                alert.is_triggered = true;
                alert.triggered_at = Some(Utc::now());
                triggered_alerts.push(alert.clone());
            }
        }

        Ok(triggered_alerts)
    }
}

// Main Enhanced API Service
pub struct EnhancedStockDataApi {
    client: Arc<EnhancedYahooFinanceClient>,
    portfolio_manager: Arc<PortfolioManager>,
}

impl EnhancedStockDataApi {
    pub fn new() -> Self {
        let client = Arc::new(EnhancedYahooFinanceClient::new());
        let portfolio_manager = Arc::new(PortfolioManager::new(client.clone()));

        Self {
            client,
            portfolio_manager,
        }
    }

    // Market Overview Endpoint
    pub async fn get_market_overview(&self) -> Result<MarketOverview, ApiError> {
        self.client.fetch_market_overview().await
    }

    // Historical Data Endpoint
    pub async fn get_historical_data(&self, symbols: Vec<String>, range: &str, interval: &str) -> Result<HashMap<String, Vec<CandleData>>, ApiError> {
        let mut data = HashMap::new();
        
        for symbol in symbols {
            match self.client.fetch_historical_data(&symbol, range, interval).await {
                Ok(candles) => {
                    data.insert(symbol, candles);
                }
                Err(e) => {
                    eprintln!("Failed to fetch data for {}: {}", symbol, e);
                }
            }
        }

        Ok(data)
    }

    // Quote Endpoints
    pub async fn get_single_quote(&self, symbol: &str) -> Result<Quote, ApiError> {
        let crumb = self.client.get_crumb().await?;
        self.client.fetch_single_quote(symbol, &crumb).await
    }

    pub async fn get_batch_quotes(&self, symbols: Vec<String>) -> Result<HashMap<String, Quote>, ApiError> {
        self.client.fetch_batch_quotes(&symbols).await
    }

    // Screener Endpoint
    pub async fn run_screener(&self, request: ScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        let results = match request.screener_type.as_deref() {
            Some("predefined") => {
                let screener_id = request.predefined_screener.as_deref().unwrap_or("most_actives");
                self.client.fetch_predefined_screener(screener_id, request.limit.map(|l| l as u32), request.offset.map(|o| o as u32)).await?
            }
            _ => {
                // For now, just return most active as fallback
                self.client.fetch_predefined_screener("most_actives", request.limit.map(|l| l as u32), request.offset.map(|o| o as u32)).await?
            }
        };

        Ok(ScreenerResponse {
            total_count: results.len(),
            results,
        })
    }

    // Portfolio Management Endpoints
    pub async fn create_portfolio(&self, name: String, description: Option<String>) -> Result<String, ApiError> {
        self.portfolio_manager.create_portfolio(name, description).await
    }

    pub async fn add_position_to_portfolio(&self, portfolio_id: &str, symbol: String, quantity: f64, price: f64) -> Result<(), ApiError> {
        self.portfolio_manager.add_position(portfolio_id, symbol, quantity, price).await
    }

    pub async fn get_portfolio(&self, portfolio_id: &str) -> Result<Portfolio, ApiError> {
        self.portfolio_manager.update_portfolio_values(portfolio_id).await?;
        self.portfolio_manager.get_portfolio(portfolio_id).await
    }

    pub async fn list_portfolios(&self) -> Result<Vec<Portfolio>, ApiError> {
        self.portfolio_manager.list_portfolios().await
    }

    pub async fn check_portfolio_alerts(&self, portfolio_id: &str) -> Result<Vec<PortfolioAlert>, ApiError> {
        self.portfolio_manager.check_alerts(portfolio_id).await
    }

    // Cache management
    pub async fn clear_cache(&self) -> Result<(), ApiError> {
        {
            let mut cache = self.client.crumb_cache.write().await;
            *cache = None;
        }
        {
            let mut request_cache = self.client.request_cache.write().await;
            request_cache.clear();
        }
        Ok(())
    }

    // Health check endpoint
    pub async fn health_check(&self) -> Result<HealthStatus, ApiError> {
        let crumb_status = match self.client.get_crumb().await {
            Ok(_) => "healthy",
            Err(_) => "unhealthy",
        };

        Ok(HealthStatus {
            status: if crumb_status == "healthy" { "healthy" } else { "degraded" },
            crumb_cache_status: crumb_status,
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: "1.0.0",
        })
    }
}

// Supporting types
#[derive(Debug, Serialize)]
pub struct HealthStatus<'a> {
    pub status: &'a str,
    pub crumb_cache_status: &'a str,
    pub uptime: u64,
    pub version: &'a str,
}

#[derive(Debug, Serialize, Clone)]
pub struct CandleData {
    pub timestamp: i64,
    pub datetime: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
    pub adj_close: Option<f64>,
}

// HTTP Server Implementation
pub mod http_server {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::io::{Read, Write, BufRead, BufReader};
    use std::collections::HashMap;

    pub struct StockApiServer {
        api: Arc<EnhancedStockDataApi>,
    }

    impl StockApiServer {
        pub fn new(api: EnhancedStockDataApi) -> Self {
            Self {
                api: Arc::new(api),
            }
        }

        pub async fn start(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
            let listener = TcpListener::bind(addr)?;
            println!("Enhanced Stock API Server running on http://{}", addr);
            println!("Available endpoints:");
            println!("  GET  /api/v1/quote?symbol=AAPL");
            println!("  GET  /api/v1/quotes?symbols=AAPL,MSFT");
            println!("  GET  /api/v1/historical?symbol=AAPL&range=1mo&interval=1d");
            println!("  GET  /api/v1/market/overview");
            println!("  GET  /api/v1/screener?type=predefined&screener=most_actives");
            println!("  POST /api/v1/portfolio");
            println!("  GET  /api/v1/portfolio/{id}");
            println!("  GET  /api/v1/health");

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

    async fn handle_request(mut stream: TcpStream, api: Arc<EnhancedStockDataApi>) -> Result<(), Box<dyn std::error::Error>> {
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

        let cors_headers = concat!(
            "Access-Control-Allow-Origin: *\r\n",
            "Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n",
            "Access-Control-Allow-Headers: Content-Type, Authorization\r\n",
        );

        if method == "OPTIONS" {
            let response = format!(
                "HTTP/1.1 204 No Content\r\n{}\r\n",
                cors_headers
            );
            stream.write_all(response.as_bytes())?;
            stream.flush()?;
            return Ok(());
        }

        match (method, path.as_str()) {
            ("GET", "/api/v1/quote") => {
                handle_single_quote(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/quotes") => {
                handle_batch_quotes(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/historical") => {
                handle_historical_data(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/market/overview") => {
                handle_market_overview(&mut stream, &*api, cors_headers).await?;
            }
            ("GET", "/api/v1/screener") => {
                handle_screener(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/health") => {
                handle_health_check(&mut stream, &*api, cors_headers).await?;
            }
            ("POST", "/api/v1/portfolio") => {
                handle_create_portfolio(&mut stream, &*api, &mut reader, cors_headers).await?;
            }
            (_, _) if path.starts_with("/api/v1/portfolio/") => {
                let portfolio_id = &path[18..]; // Remove "/api/v1/portfolio/"
                handle_get_portfolio(&mut stream, &*api, portfolio_id, cors_headers).await?;
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

    async fn handle_single_quote(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbol = query.get("symbol")
            .cloned()
            .unwrap_or_else(|| "AAPL".to_string());

        match api.get_single_quote(&symbol).await {
            Ok(quote) => {
                let json = serde_json::to_string(&quote)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "symbol": symbol
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_batch_quotes(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbols = query.get("symbols")
            .map(|s| s.split(',').map(|symbol| symbol.trim().to_uppercase()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        match api.get_batch_quotes(symbols).await {
            Ok(quotes) => {
                let response = serde_json::json!({
                    "quotes": quotes,
                    "count": quotes.len()
                });
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string()
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_historical_data(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbols = query.get("symbols")
            .or_else(|| query.get("symbol"))
            .map(|s| s.split(',').map(|symbol| symbol.trim().to_uppercase()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        let range = query.get("range").cloned().unwrap_or_else(|| "1mo".to_string());
        let interval = query.get("interval").cloned().unwrap_or_else(|| "1d".to_string());

        match api.get_historical_data(symbols, &range, &interval).await {
            Ok(data) => {
                let response = serde_json::json!({
                    "data": data,
                    "range": range,
                    "interval": interval
                });
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string()
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_market_overview(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match api.get_market_overview().await {
            Ok(overview) => {
                let json = serde_json::to_string(&overview)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string()
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_screener(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let screener_type = query.get("type").cloned();
        let predefined_screener = query.get("screener").cloned();
        let limit = query.get("limit").and_then(|l| l.parse().ok());
        let offset = query.get("offset").and_then(|o| o.parse().ok());

        let request = ScreenerRequest {
            filters: Vec::new(), // Would parse from query parameters in real implementation
            sort_by: query.get("sort_by").cloned(),
            sort_order: query.get("sort_order").cloned(),
            limit,
            offset,
            screener_type,
            predefined_screener,
        };

        match api.run_screener(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string()
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_health_check(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match api.health_check().await {
            Ok(health) => {
                let json = serde_json::to_string(&health)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string()
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_create_portfolio(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        reader: &mut BufReader<TcpStream>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Read headers
        let mut content_length = None;
        let mut line = String::new();

        loop {
            line.clear();
            reader.read_line(&mut line)?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                break;
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

        // Read body
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;

        // Parse JSON
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(req) => req,
            Err(_) => {
                send_response(stream, 400, "Bad Request", "Invalid JSON")?;
                return Ok(());
            }
        };

        let name = request.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("New Portfolio")
            .to_string();

        let description = request.get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        match api.create_portfolio(name, description).await {
            Ok(portfolio_id) => {
                let response = serde_json::json!({
                    "portfolio_id": portfolio_id,
                    "status": "created"
                });
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 201, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string()
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_get_portfolio(
        stream: &mut TcpStream,
        api: &EnhancedStockDataApi,
        portfolio_id: &str,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match api.get_portfolio(portfolio_id).await {
            Ok(portfolio) => {
                let json = serde_json::to_string(&portfolio)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "portfolio_id": portfolio_id
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 404, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    fn send_response(
        stream: &mut TcpStream,
        status_code: u16,
        status_text: &str,
        body: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{}\r\n{}",
            status_code, json.len(), cors_headers, json
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

// Main function and example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Enhanced Stock Data API Server");

    // Create API instance
    let api = EnhancedStockDataApi::new();

    // Check if we should run examples
    if std::env::args().any(|arg| arg == "--examples") {
        run_examples(&api).await?;
        return Ok(());
    }

    // Start HTTP server
    if std::env::args().any(|arg| arg == "--server") {
        let server = http_server::StockApiServer::new(api);
        server.start("127.0.0.1:8080").await?;
        return Ok(());
    }

    // Default: run interactive CLI
    run_interactive_cli(&api).await?;

    Ok(())
}

async fn run_examples(api: &EnhancedStockDataApi) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running API Examples\n");

    // Example 1: Single Quote
    println!("=== Single Quote ===");
    match api.get_single_quote("AAPL").await {
        Ok(quote) => {
            println!("AAPL: ${:.2} ({:+.2}%)", quote.price, quote.change_percent);
            println!("Volume: {}, 52W Range: ${:.2} - ${:.2}", 
                format_volume(quote.volume), quote.low_52w, quote.high_52w);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 2: Batch Quotes
    println!("\n=== Batch Quotes ===");
    let symbols = vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()];
    match api.get_batch_quotes(symbols).await {
        Ok(quotes) => {
            println!("Retrieved {} quotes:", quotes.len());
            for (symbol, quote) in quotes {
                println!("  {}: ${:.2} ({:+.2}%)", symbol, quote.price, quote.change_percent);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 3: Historical Data
    println!("\n=== Historical Data ===");
    let symbols = vec!["AAPL".to_string()];
    match api.get_historical_data(symbols, "1mo", "1d").await {
        Ok(data) => {
            for (symbol, candles) in data {
                println!("{}: {} candles", symbol, candles.len());
                if let Some(latest) = candles.last() {
                    println!("  Latest: ${:.2} on {}", latest.close, latest.datetime);
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 4: Market Overview
    println!("\n=== Market Overview ===");
    match api.get_market_overview().await {
        Ok(overview) => {
            println!("Market Overview ({})", overview.last_updated);
            println!("Major Indices:");
            for (symbol, index) in overview.indices {
                println!("  {}: {:.2} ({:+.2}%)", index.name, index.price, index.change_percent);
            }
            println!("Market Sentiment Score: {:.2}", overview.market_sentiment.sentiment_score);
            println!("VIX: {:.2}", overview.market_sentiment.vix);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 5: Stock Screener
    println!("\n=== Stock Screener ===");
    let screener_request = ScreenerRequest {
        filters: Vec::new(),
        sort_by: Some("volume".to_string()),
        sort_order: Some("desc".to_string()),
        limit: Some(10),
        offset: Some(0),
        screener_type: Some("predefined".to_string()),
        predefined_screener: Some("most_actives".to_string()),
    };

    match api.run_screener(screener_request).await {
        Ok(response) => {
            println!("Most Active Stocks ({} results):", response.total_count);
            for result in response.results.iter().take(5) {
                println!("  {}: ${:.2} ({:+.2}%) Vol: {}", 
                    result.symbol, result.price, result.change_percent, format_volume(result.volume));
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 6: Portfolio Management
    println!("\n=== Portfolio Management ===");
    match api.create_portfolio("Demo Portfolio".to_string(), Some("Example portfolio for testing".to_string())).await {
        Ok(portfolio_id) => {
            println!("Created portfolio: {}", portfolio_id);
            
            // Add some positions
            let _ = api.add_position_to_portfolio(&portfolio_id, "AAPL".to_string(), 100.0, 150.0).await;
            let _ = api.add_position_to_portfolio(&portfolio_id, "MSFT".to_string(), 50.0, 300.0).await;
            
            // Get portfolio details
            match api.get_portfolio(&portfolio_id).await {
                Ok(portfolio) => {
                    println!("Portfolio Value: ${:.2}", portfolio.total_value);
                    println!("Total Return: ${:.2} ({:.2}%)", portfolio.total_return, portfolio.total_return_percent);
                    println!("Positions:");
                    for position in portfolio.positions {
                        println!("  {}: {} shares @ ${:.2} (Current: ${:.2})", 
                            position.symbol, position.quantity, position.average_cost, position.current_price);
                    }
                }
                Err(e) => println!("Error getting portfolio: {}", e),
            }
        }
        Err(e) => println!("Error creating portfolio: {}", e),
    }

    Ok(())
}

async fn run_interactive_cli(api: &EnhancedStockDataApi) -> Result<(), Box<dyn std::error::Error>> {
    println!("Interactive Stock Data CLI");
    println!("Commands: quote <symbol>, quotes <symbols>, hist <symbol>, market, screen, portfolio, help, quit");

    loop {
        print!("\n> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "help" => {
                println!("Available commands:");
                println!("  quote <symbol>         - Get single quote");
                println!("  quotes <sym1,sym2,...> - Get multiple quotes");
                println!("  hist <symbol> [range]  - Get historical data");
                println!("  market                 - Get market overview");
                println!("  screen                 - Run stock screener");
                println!("  portfolio              - Portfolio operations");
                println!("  health                 - API health check");
                println!("  quit                   - Exit");
            }
            "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            "quote" => {
                if parts.len() < 2 {
                    println!("Usage: quote <symbol>");
                    continue;
                }
                let symbol = parts[1].to_uppercase();
                
                match api.get_single_quote(&symbol).await {
                    Ok(quote) => {
                        println!("{}: ${:.2} ({:+.2}%)", symbol, quote.price, quote.change_percent);
                        println!("  Volume: {}, 52W: ${:.2}-${:.2}", 
                            format_volume(quote.volume), quote.low_52w, quote.high_52w);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "quotes" => {
                if parts.len() < 2 {
                    println!("Usage: quotes <symbol1,symbol2,...>");
                    continue;
                }
                let symbols: Vec<String> = parts[1]
                    .split(',')
                    .map(|s| s.trim().to_uppercase())
                    .collect();
                
                match api.get_batch_quotes(symbols).await {
                    Ok(quotes) => {
                        for (symbol, quote) in quotes {
                            println!("{}: ${:.2} ({:+.2}%)", symbol, quote.price, quote.change_percent);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "hist" => {
                if parts.len() < 2 {
                    println!("Usage: hist <symbol> [range]");
                    continue;
                }
                let symbol = parts[1].to_uppercase();
                let range = parts.get(2).unwrap_or(&"1mo");
                
                let symbols = vec![symbol.clone()];
                match api.get_historical_data(symbols, range, "1d").await {
                    Ok(mut data) => {
                        if let Some(candles) = data.remove(&symbol) {
                            println!("{} - {} candles ({})", symbol, candles.len(), range);
                            if let Some(latest) = candles.last() {
                                println!("  Latest: ${:.2} ({})", latest.close, latest.datetime);
                            }
                            if candles.len() >= 2 {
                                let first = &candles[0];
                                let last = &candles[candles.len() - 1];
                                let change = ((last.close - first.close) / first.close) * 100.0;
                                println!("  Period change: {:+.2}%", change);
                            }
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "market" => {
                match api.get_market_overview().await {
                    Ok(overview) => {
                        println!("Market Overview:");
                        for (symbol, index) in overview.indices.iter().take(5) {
                            println!("  {}: {:.2} ({:+.2}%)", index.name, index.price, index.change_percent);
                        }
                        println!("  VIX: {:.2}", overview.market_sentiment.vix);
                        println!("  Sentiment: {:.2}", overview.market_sentiment.sentiment_score);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "screen" => {
                let request = ScreenerRequest {
                    filters: Vec::new(),
                    sort_by: Some("volume".to_string()),
                    sort_order: Some("desc".to_string()),
                    limit: Some(10),
                    offset: Some(0),
                    screener_type: Some("predefined".to_string()),
                    predefined_screener: Some("most_actives".to_string()),
                };

                match api.run_screener(request).await {
                    Ok(response) => {
                        println!("Most Active Stocks:");
                        for result in response.results.iter().take(10) {
                            println!("  {}: ${:.2} ({:+.2}%)", 
                                result.symbol, result.price, result.change_percent);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "health" => {
                match api.health_check().await {
                    Ok(health) => {
                        println!("API Health: {}", health.status);
                        println!("  Crumb Status: {}", health.crumb_cache_status);
                        println!("  Uptime: {}s", health.uptime);
                        println!("  Version: {}", health.version);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "portfolio" => {
                println!("Portfolio management - create a demo portfolio...");
                match api.create_portfolio("CLI Portfolio".to_string(), None).await {
                    Ok(id) => {
                        println!("Created portfolio: {}", id);
                        let _ = api.add_position_to_portfolio(&id, "AAPL".to_string(), 10.0, 150.0).await;
                        println!("Added AAPL position");
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
            }
        }
    }

    Ok(())
}

fn format_volume(volume: u64) -> String {
    if volume >= 1_000_000_000 {
        format!("{:.1}B", volume as f64 / 1_000_000_000.0)
    } else if volume >= 1_000_000 {
        format!("{:.1}M", volume as f64 / 1_000_000.0)
    } else if volume >= 1_000 {
        format!("{:.1}K", volume as f64 / 1_000.0)
    } else {
        volume.to_string()
    }
}

// // Example usage and testing
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_quote_fetch() {
//         let api = EnhancedStockDataApi::new();
//         let result = api.get_single_quote("AAPL").await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test] 
//     async fn test_portfolio_creation() {
//         let api = EnhancedStockDataApi::new();
//         let portfolio_id = api.create_portfolio("Test Portfolio".to_string(), None).await.unwrap();
//         assert!(!portfolio_id.is_empty());
        
//         let portfolio = api.get_portfolio(&portfolio_id).await.unwrap();
//         assert_eq!(portfolio.name, "Test Portfolio");
//     }

//     #[tokio::test]
//     async fn test_health_check() {
//         let api = EnhancedStockDataApi::new();
//         let health = api.health_check().await.unwrap();
//         assert_eq!(health.version, "1.0.0");
//     }
// } 
//     // // REAL IMPLEMENTATION - Predefined Screener
//     // pub async fn fetch_predefined_screener(&self, screener_id: &str, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<ScreenerResult>, ApiError> {
//     //     let crumb = self.get_crumb().await?;
//     //     let limit = limit.unwrap_or(25);
//     //     let offset = offset.unwrap_or(0);

//     //     self.rate_limiter.write().await.wait_if_needed().await;

//     //     let url = format!(
//     //         "https://query2.finance.yahoo.com/v1/finance/screener/predefined/saved?count={}&offset={}&scrIds={}&crumb={}",
//     //         limit, offset, screener_id, crumb
//     //     );

//     //     let response = self.client
//     //         .get(&url)
//     //         .header("Accept", "application/json")
//     //         .header("Referer", "https://finance.yahoo.com/screener")
//     //         .send()
//     //         .await
//     //         .map_err(|e| ApiError::NetworkError(e.to_string()))?;

//     //     if !response.status().is_success() {
//     //         return Err(ApiError::FetchError(format!("HTTP {}",response.status())));
//         }

//         let html = response.text().await
//             .map_err(|e| ApiError::ParseError(e.to_string()))?;

//         let patterns = [
//             r#""CrumbStore":\s*\{\s*"crumb":\s*"([^"]+)""#,
//             r#""crumb"\s*:\s*"([^"]+)""#,
//             r#"window\.crumb\s*=\s*"([^"]+)""#,
//             r#"crumb["\']?\s*:\s*["\']([^"\']+)["\']"#,
//         ];

//         for pattern in &patterns {
//             if let Ok(re) = Regex::new(pattern) {
//                 if let Some(captures) = re.captures(&html) {
//                     if let Some(crumb_match) = captures.get(1) {
//                         let crumb = crumb_match.as_str().to_string();
//                         if !crumb.is_empty() && crumb.len() < 50 {
//                             return Ok(crumb);
//                         }
//                     }
//                 }
//             }
//         }

//         Err(ApiError::ParseError("Crumb not found in HTML".to_string()))
//     }

//     // REAL IMPLEMENTATION - Single Quote
//     pub async fn fetch_single_quote(&self, symbol: &str, crumb: &str) -> Result<Quote, ApiError> {
//         self.rate_limiter.write().await.wait_if_needed().await;

//         let url = format!(
//             "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d&crumb={}",
//             symbol, crumb
//         );

//         let response = self.client
//             .get(&url)
//             .header("Accept", "application/json")
//             .header("Referer", &format!("https://finance.yahoo.com/quote/{}", symbol))
//             .send()
//             .await
//             .map_err(|e| ApiError::NetworkError(e.to_string()))?;

//         if !response.status().is_success() {
//             return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
//         }

//         let yahoo_response: YahooChartResponse = response
//             .json()
//             .await
//             .map_err(|e| ApiError::ParseError(e.to_string()))?;

//         if let Some(ref error) = yahoo_response.chart.error {
//             return Err(ApiError::FetchError(format!("Yahoo error: {:?}", error)));
//         }

//         let result = yahoo_response.chart.result
//             .as_ref()
//             .and_then(|results| results.get(0))
//             .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

//         let meta = &result.meta;
//         let current_price = meta.regular_market_price;
//         let prev_close = meta.chart_previous_close;
//         let change = current_price - prev_close;
//         let change_percent = (change / prev_close) * 100.0;

//         // Get latest volume from indicators
//         let volume = result.indicators.quote
//             .as_ref()
//             .and_then(|quotes| quotes.get(0))
//             .and_then(|quote| quote.volume.as_ref())
//             .and_then(|volumes| volumes.last())
//             .and_then(|&vol| vol)
//             .unwrap_or(0);

//         // Get 52-week high/low (simplified - would need additional API call for real data)
//         let high_52w = current_price * 1.3; // Placeholder
//         let low_52w = current_price * 0.7;  // Placeholder

//         Ok(Quote {
//             symbol: symbol.to_string(),
//             price: current_price,
//             change,
//             change_percent,
//             volume,
//             bid: None,
//             ask: None,
//             bid_size: None,
//             ask_size: None,
//             high_52w,
//             low_52w,
//             market_cap: None,
//             pe_ratio: None,
//             dividend_yield: None,
//             last_updated: Utc::now().to_rfc3339(),
//         })
//     }

//     // REAL IMPLEMENTATION - Batch Quotes
//     pub async fn fetch_batch_quotes(&self, symbols: &[String]) -> Result<HashMap<String, Quote>, ApiError> {
//         let mut quotes = HashMap::new();
//         let crumb = self.get_crumb().await?;

//         // Process in batches of 5 to avoid overwhelming the API
//         for chunk in symbols.chunks(5) {
//             for symbol in chunk {
//                 match self.fetch_single_quote(symbol, &crumb).await {
//                     Ok(quote) => {
//                         quotes.insert(symbol.clone(), quote);
//                     }
//                     Err(e) => {
//                         eprintln!("Failed to fetch quote for {}: {}", symbol, e);
//                     }
//                 }
                
//                 // Brief delay between requests
//                 tokio::time::sleep(Duration::from_millis(200)).await;
//             }
//         }

//         Ok(quotes)
//     }

//     // REAL IMPLEMENTATION - Historical Data
//     pub async fn fetch_historical_data(&self, symbol: &str, range: &str, interval: &str) -> Result<Vec<CandleData>, ApiError> {
//         let crumb = self.get_crumb().await?;
//         self.rate_limiter.write().await.wait_if_needed().await;

//         let url = format!(
//             "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}&crumb={}",
//             symbol, range, interval, crumb
//         );

//         let response = self.client
//             .get(&url)
//             .header("Accept", "application/json")
//             .header("Referer", &format!("https://finance.yahoo.com/quote/{}", symbol))
//             .send()
//             .await
//             .map_err(|e| ApiError::NetworkError(e.to_string()))?;

//         if !response.status().is_success() {
//             return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
//         }

//         let yahoo_response: YahooChartResponse = response
//             .json()
//             .await
//             .map_err(|e| ApiError::ParseError(e.to_string()))?;

//         let result = yahoo_response.chart.result
//             .as_ref()
//             .and_then(|results| results.get(0))
//             .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

//         let timestamps = result.timestamp.as_ref()
//             .ok_or_else(|| ApiError::DataNotFound("No timestamp data".to_string()))?;

//         let quote_data = result.indicators.quote
//             .as_ref()
//             .and_then(|quotes| quotes.get(0))
//             .ok_or_else(|| ApiError::DataNotFound("No quote data".to_string()))?;

//         let opens = quote_data.open.as_ref().unwrap_or(&vec![]);
//         let highs = quote_data.high.as_ref().unwrap_or(&vec![]);
//         let lows = quote_data.low.as_ref().unwrap_or(&vec![]);
//         let closes = quote_data.close.as_ref().unwrap_or(&vec![]);
//         let volumes = quote_data.volume.as_ref().unwrap_or(&vec![]);

//         let adj_closes = result.indicators.adjclose
//             .as_ref()
//             .and_then(|adj| adj.get(0))
//             .and_then(|adj_data| adj_data.adjclose.as_ref())
//             .unwrap_or(&vec![]);

//         let mut candles = Vec::new();

//         for (i, &timestamp) in timestamps.iter().enumerate() {
//             if let (Some(Some(open)), Some(Some(high)), Some(Some(low)), Some(Some(close))) = (
//                 opens.get(i).cloned().flatten(),
//                 highs.get(i).cloned().flatten(),
//                 lows.get(i).cloned().flatten(),
//                 closes.get(i).cloned().flatten(),
//             ) {
//                 let volume = volumes.get(i).cloned().flatten();
//                 let adj_close = adj_closes.get(i).cloned().flatten();
                
//                 let datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
//                 let dt: DateTime<Utc> = datetime.into();

//                 candles.push(CandleData {
//                     timestamp,
//                     datetime: dt.to_rfc3339(),
//                     open,
//                     high,
//                     low,
//                     close,
//                     volume: volume.map(|v| v as f64),
//                     adj_close,
//                 });
//             }
//         }

//         Ok(candles)
//     }
