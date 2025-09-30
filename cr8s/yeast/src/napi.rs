use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock as AsyncRwLock;
use chrono::{DateTime, Utc};
use regex::Regex;

// Enhanced Error Types
#[derive(Debug)]
pub enum ApiError {
    FetchError(String),
    DataNotFound(String),
    InvalidParameters(String),
    RateLimited(String),
    AuthenticationFailed(String),
    CacheError(String),
    NetworkError(String),
    ParseError(String),
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

// Market-wide Data Structures
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
    pub sentiment_score: f64, // -1 to 1 scale
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

// Enhanced Screener with Predefined Options
#[derive(Debug, Deserialize)]
pub struct EnhancedScreenerRequest {
    pub filters: Vec<ScreenerFilter>,
    pub indicators: Option<Vec<IndicatorConfig>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub screener_type: ScreenerType,
    pub market_cap_range: Option<MarketCapRange>,
    pub sector_filter: Option<Vec<String>>,
    pub exchange_filter: Option<Vec<String>>,
    pub country_filter: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub enum ScreenerType {
    Custom { filters: Vec<ScreenerFilter> },
    Predefined { name: PredefinedScreener },
}

#[derive(Debug, Deserialize)]
pub enum PredefinedScreener {
    MostActive,
    TopGainers,
    TopLosers,
    HighDividendYield,
    LowPERatio,
    HighGrowth,
    ValueStocks,
    SmallCap,
    MidCap,
    LargeCap,
    TechStocks,
    DividendAristocrats,
    PennyStocks,
    HighVolatility,
    LowVolatility,
}

#[derive(Debug, Deserialize)]
pub struct MarketCapRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ScreenerFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
    pub secondary_value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub enum FilterOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    Between,
    In,
    NotIn,
    Contains,
}

// Portfolio Management Structures
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
    pub weight: f64, // Percentage of portfolio
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

// Enhanced Client with Better Caching
pub struct EnhancedYahooFinanceClient {
    client: reqwest::Client,
    crumb_cache: Arc<AsyncRwLock<Option<CrumbCache>>>,
    rate_limiter: Arc<AsyncRwLock<RateLimiter>>,
    request_cache: Arc<AsyncRwLock<HashMap<String, CachedResponse>>>,
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
        
        // Reset window if needed
        if now.duration_since(self.window_start) > self.window_duration {
            self.request_count = 0;
            self.window_start = now;
        }

        // Check if we've exceeded rate limit
        if self.request_count >= self.requests_per_window {
            let wait_time = self.window_duration - now.duration_since(self.window_start);
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
                self.request_count = 0;
                self.window_start = Instant::now();
            }
        }

        // Ensure minimum interval between requests
        let time_since_last = now.duration_since(self.last_request);
        if time_since_last < self.min_interval {
            tokio::time::sleep(self.min_interval - time_since_last).await;
        }

        self.request_count += 1;
        self.last_request = Instant::now();
    }
}

impl EnhancedYahooFinanceClient {
    pub fn new() -> Self {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::Client::builder()
            .cookie_provider(jar)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            crumb_cache: Arc::new(AsyncRwLock::new(None)),
            rate_limiter: Arc::new(AsyncRwLock::new(RateLimiter::new(60))), // 60 requests per minute
            request_cache: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }

    // Enhanced crumb management with better error recovery
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
        
        // Update cache with longer TTL and better error handling
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

        // Method 3: Try alternative endpoints
        if let Ok(crumb) = self.try_alternative_crumb_sources().await {
            return Ok(crumb);
        }

        Err(ApiError::AuthenticationFailed("Unable to obtain crumb from any source".to_string()))
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

            if !crumb.is_empty() && crumb.len() < 50 {
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

        let html = response.text().await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        // Multiple regex patterns for crumb extraction
        let patterns = [
            r#""CrumbStore":\s*\{\s*"crumb":\s*"([^"]+)""#,
            r#""crumb"\s*:\s*"([^"]+)""#,
            r#"window\.crumb\s*=\s*"([^"]+)""#,
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(&html) {
                    if let Some(crumb_match) = captures.get(1) {
                        return Ok(crumb_match.as_str().to_string());
                    }
                }
            }
        }

        Err(ApiError::ParseError("Crumb not found in HTML".to_string()))
    }

    async fn try_alternative_crumb_sources(&self) -> Result<String, ApiError> {
        // Try chart endpoint which sometimes works without crumb
        let response = self.client
            .get("https://query1.finance.yahoo.com/v8/finance/chart/AAPL")
            .send()
            .await;

        if let Ok(resp) = response {
            if resp.status().is_success() {
                // If this works, we might be able to extract crumb from headers or response
                // For now, return a placeholder indicating we should retry
                return Err(ApiError::AuthenticationFailed("Alternative method needs implementation".to_string()));
            }
        }

        Err(ApiError::AuthenticationFailed("All alternative methods failed".to_string()))
    }

    // Market overview data fetching
    pub async fn fetch_market_overview(&self) -> Result<MarketOverview, ApiError> {
        let crumb = self.get_crumb().await?;
        
        // Fetch major indices
        let indices = self.fetch_major_indices(&crumb).await?;
        
        // Fetch sector performance
        let sectors = self.fetch_sector_performance(&crumb).await?;
        
        // Fetch market sentiment indicators
        let market_sentiment = self.fetch_market_sentiment(&crumb).await?;
        
        // Fetch top movers
        let top_movers = self.fetch_top_movers(&crumb).await?;
        
        // Calculate market statistics
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
        }

        Ok(sectors)
    }

    async fn fetch_market_sentiment(&self, crumb: &str) -> Result<MarketSentiment, ApiError> {
        // Fetch VIX for fear/greed indicator
        let vix_quote = self.fetch_single_quote("^VIX", crumb).await?;
        
        Ok(MarketSentiment {
            fear_greed_index: None, // Would need CNN Fear & Greed Index API
            vix: vix_quote.price,
            put_call_ratio: 1.0, // Would need options data
            advance_decline_ratio: 1.0, // Would need market breadth data
            sentiment_score: self.calculate_sentiment_score(vix_quote.price),
        })
    }

    async fn fetch_top_movers(&self, crumb: &str) -> Result<TopMovers, ApiError> {
        // Use predefined screeners for top movers
        let gainers = self.fetch_predefined_screener("day_gainers", Some(10), Some(0), crumb).await?;
        let losers = self.fetch_predefined_screener("day_losers", Some(10), Some(0), crumb).await?;
        let most_active = self.fetch_predefined_screener("most_actives", Some(10), Some(0), crumb).await?;

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
        // This would require more comprehensive market data
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
        // Simple sentiment calculation based on VIX
        // VIX < 12: Very bullish (0.8)
        // VIX 12-20: Bullish (0.2 to 0.8)
        // VIX 20-30: Neutral (-0.2 to 0.2)
        // VIX > 30: Bearish (-0.8 to -0.2)
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

    // Placeholder methods that would need full implementation
    async fn fetch_single_quote(&self, _symbol: &str, _crumb: &str) -> Result<Quote, ApiError> {
        // Implementation would fetch actual quote data
        Err(ApiError::FetchError("Not implemented".to_string()))
    }

    async fn fetch_predefined_screener(&self, _screener: &str, _limit: Option<u32>, _offset: Option<u32>, _crumb: &str) -> Result<Vec<serde_json::Value>, ApiError> {
        // Implementation would fetch screener data
        Err(ApiError::FetchError("Not implemented".to_string()))
    }

    fn convert_to_mover_data(&self, _data: &[serde_json::Value]) -> Result<Vec<MoverData>, ApiError> {
        // Implementation would convert screener results to MoverData
        Ok(Vec::new())
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
        let portfolio_id = uuid::Uuid::new_v4().to_string();
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
            let position_id = uuid::Uuid::new_v4().to_string();
            let position = Position {
                id: position_id.clone(),
                symbol: symbol.clone(),
                quantity,
                average_cost: price,
                current_price: price, // Will be updated with market data
                market_value: price * quantity,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                day_change: 0.0,
                day_change_percent: 0.0,
                weight: 0.0, // Will be calculated
                first_bought: Utc::now(),
                last_updated: Utc::now(),
                transactions: vec![Transaction {
                    id: uuid::Uuid::new_v4().to_string(),
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

        let mut total_value = portfolio.cash_balance;
        
        for position in &mut portfolio.positions {
            // Fetch current price from Yahoo Finance
            match self.client.fetch_single_quote(&position.symbol, &self.client.get_crumb().await?).await {
                Ok(quote) => {
                    position.current_price = quote.price;
                    position.market_value = position.quantity * quote.price;
                    position.unrealized_pnl = position.market_value - (position.quantity * position.average_cost);
                    position.unrealized_pnl_percent = (position.unrealized_pnl / (position.quantity * position.average_cost)) * 100.0;
                    position.day_change = quote.change * position.quantity;
                    position.day_change_percent = quote.change_percent;
                    position.last_updated = Utc::now();
                    
                    total_value += position.market_value;
                }
                Err(e) => {
                    eprintln!("Failed to update price for {}: {}", position.symbol, e);
                }
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

    pub async fn add_alert(&self, portfolio_id: &str, alert: PortfolioAlert) -> Result<(), ApiError> {
        let mut portfolios = self.portfolios.write().await;
        let portfolio = portfolios.get_mut(portfolio_id)
            .ok_or_else(|| ApiError::DataNotFound("Portfolio not found".to_string()))?;

        portfolio.alerts.push(alert);
        portfolio.updated_at = Utc::now();
        Ok(())
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

// Enhanced Screener Service
pub struct ScreenerService {
    client: Arc<EnhancedYahooFinanceClient>,
}

impl ScreenerService {
    pub fn new(client: Arc<EnhancedYahooFinanceClient>) -> Self {
        Self { client }
    }

    pub async fn run_screener(&self, request: EnhancedScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        match request.screener_type {
            ScreenerType::Predefined { name } => {
                self.run_predefined_screener(name, &request).await
            }
            ScreenerType::Custom { filters } => {
                self.run_custom_screener(&filters, &request).await
            }
        }
    }

    async fn run_predefined_screener(&self, screener: PredefinedScreener, request: &EnhancedScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        let yahoo_screener_id = match screener {
            PredefinedScreener::MostActive => "most_actives",
            PredefinedScreener::TopGainers => "day_gainers",
            PredefinedScreener::TopLosers => "day_losers",
            PredefinedScreener::HighDividendYield => "high_yield",
            PredefinedScreener::LowPERatio => "undervalued_large_caps",
            PredefinedScreener::HighGrowth => "growth_technology_stocks",
            PredefinedScreener::ValueStocks => "undervalued_large_caps",
            PredefinedScreener::SmallCap => "small_cap_gainers",
            PredefinedScreener::MidCap => "mid_cap_gainers",
            PredefinedScreener::LargeCap => "day_gainers",
            PredefinedScreener::TechStocks => "growth_technology_stocks",
            PredefinedScreener::DividendAristocrats => "dividend_yielders",
            PredefinedScreener::PennyStocks => "small_cap_gainers",
            PredefinedScreener::HighVolatility => "day_gainers",
            PredefinedScreener::LowVolatility => "conservative_foreign_funds",
        };

        let crumb = self.client.get_crumb().await?;
        let yahoo_response = self.client.fetch_predefined_screener(
            yahoo_screener_id,
            request.limit.map(|l| l as u32),
            request.offset.map(|o| o as u32),
            &crumb
        ).await?;

        self.convert_yahoo_response_to_screener_result(yahoo_response, request)
    }

    async fn run_custom_screener(&self, filters: &[ScreenerFilter], request: &EnhancedScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        let crumb = self.client.get_crumb().await?;
        
        // Convert our filters to Yahoo Finance format
        let yahoo_filters = self.convert_filters_to_yahoo_format(filters)?;
        
        let yahoo_response = self.client.fetch_custom_screener(
            &yahoo_filters,
            request.sort_by.as_deref(),
            request.sort_order.as_deref(),
            request.limit.map(|l| l as u32),
            request.offset.map(|o| o as u32),
            &crumb
        ).await?;

        self.convert_yahoo_response_to_screener_result(yahoo_response, request)
    }

    fn convert_filters_to_yahoo_format(&self, filters: &[ScreenerFilter]) -> Result<Vec<YahooScreenerFilter>, ApiError> {
        let mut yahoo_filters = Vec::new();

        for filter in filters {
            let yahoo_field = self.map_field_to_yahoo(&filter.field)?;
            let yahoo_operator = match filter.operator {
                FilterOperator::GreaterThan => "gt",
                FilterOperator::LessThan => "lt",
                FilterOperator::GreaterThanOrEqual => "gte",
                FilterOperator::LessThanOrEqual => "lte",
                FilterOperator::Equal => "eq",
                FilterOperator::Between => "between",
                FilterOperator::In => "in",
                FilterOperator::NotIn => "not_in",
                FilterOperator::Contains => "contains",
            };

            let yahoo_filter = YahooScreenerFilter {
                field: yahoo_field,
                operator: yahoo_operator.to_string(),
                value: filter.value.clone(),
                secondary_value: filter.secondary_value.clone(),
            };

            yahoo_filters.push(yahoo_filter);
        }

        Ok(yahoo_filters)
    }

    fn map_field_to_yahoo(&self, field: &str) -> Result<String, ApiError> {
        let mapped = match field {
            "price" | "current_price" => "intradayprice",
            "volume" | "avg_volume" => "intradayvolume",
            "market_cap" => "intradaymarketcap",
            "pe_ratio" | "trailing_pe" => "trailingpe",
            "forward_pe" => "forwardpe",
            "peg_ratio" => "pegratio",
            "price_to_book" | "pb_ratio" => "pb",
            "price_to_sales" | "ps_ratio" => "ps",
            "dividend_yield" => "dividendyield",
            "change_percent" | "percent_change" => "percentchange",
            "change" | "price_change" => "change",
            "fifty_two_week_high" | "52w_high" => "week52high",
            "fifty_two_week_low" | "52w_low" => "week52low",
            "beta" => "beta",
            "eps" | "trailing_eps" => "trailingeps",
            "forward_eps" => "forwardeps",
            "revenue" | "total_revenue" => "totalrevenue",
            "debt_to_equity" => "debttoequity",
            "return_on_equity" | "roe" => "returnonequity",
            "return_on_assets" | "roa" => "returnonassets",
            "profit_margin" => "profitmargin",
            "operating_margin" => "operatingmargin",
            "gross_margin" => "grossmargin",
            "current_ratio" => "currentratio",
            "quick_ratio" => "quickratio",
            "sector" => "sector",
            "industry" => "industry",
            "country" => "country",
            "exchange" => "exchange",
            "market_cap_category" => "marketcap",
            "dividend_rate" => "dividendrate",
            "ex_dividend_date" => "exdividenddate",
            "earnings_date" => "earningsdate",
            "book_value" => "bookvalue",
            "cash_per_share" => "cashpershare",
            "ebitda" => "ebitda",
            "enterprise_value" => "enterprisevalue",
            "float_shares" => "floatshares",
            "shares_outstanding" => "sharesoutstanding",
            "short_ratio" => "shortratio",
            "institutional_ownership" => "institutionalownership",
            _ => return Err(ApiError::InvalidParameters(format!("Unknown field: {}", field))),
        };
        Ok(mapped.to_string())
    }

    fn convert_yahoo_response_to_screener_result(&self, _yahoo_response: Vec<serde_json::Value>, _request: &EnhancedScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        // This would convert Yahoo Finance response format to our ScreenerResponse format
        // Implementation would parse the Yahoo response and create ScreenerResult objects
        
        Ok(ScreenerResponse {
            results: Vec::new(), // Would populate with converted results
            total_count: 0,
        })
    }
}

// Main Enhanced API Service
pub struct EnhancedStockDataApi {
    client: Arc<EnhancedYahooFinanceClient>,
    portfolio_manager: Arc<PortfolioManager>,
    screener_service: Arc<ScreenerService>,
}

impl EnhancedStockDataApi {
    pub fn new() -> Self {
        let client = Arc::new(EnhancedYahooFinanceClient::new());
        let portfolio_manager = Arc::new(PortfolioManager::new(client.clone()));
        let screener_service = Arc::new(ScreenerService::new(client.clone()));

        Self {
            client,
            portfolio_manager,
            screener_service,
        }
    }

    // Market Overview Endpoint
    pub async fn get_market_overview(&self) -> Result<MarketOverview, ApiError> {
        self.client.fetch_market_overview().await
    }

    // Enhanced Screener Endpoint
    pub async fn screen_stocks(&self, request: EnhancedScreenerRequest) -> Result<ScreenerResponse, ApiError> {
        self.screener_service.run_screener(request).await
    }

    // Portfolio Management Endpoints
    pub async fn create_portfolio(&self, name: String, description: Option<String>) -> Result<String, ApiError> {
        self.portfolio_manager.create_portfolio(name, description).await
    }

    pub async fn add_position_to_portfolio(&self, portfolio_id: &str, symbol: String, quantity: f64, price: f64) -> Result<(), ApiError> {
        self.portfolio_manager.add_position(portfolio_id, symbol, quantity, price).await
    }

    pub async fn get_portfolio(&self, portfolio_id: &str) -> Result<Portfolio, ApiError> {
        // Update portfolio with latest prices before returning
        self.portfolio_manager.update_portfolio_values(portfolio_id).await?;
        self.portfolio_manager.get_portfolio(portfolio_id).await
    }

    pub async fn list_portfolios(&self) -> Result<Vec<Portfolio>, ApiError> {
        self.portfolio_manager.list_portfolios().await
    }

    pub async fn add_portfolio_alert(&self, portfolio_id: &str, alert: PortfolioAlert) -> Result<(), ApiError> {
        self.portfolio_manager.add_alert(portfolio_id, alert).await
    }

    pub async fn check_portfolio_alerts(&self, portfolio_id: &str) -> Result<Vec<PortfolioAlert>, ApiError> {
        self.portfolio_manager.check_alerts(portfolio_id).await
    }

    // Sector Analysis Endpoint
    pub async fn get_sector_analysis(&self) -> Result<HashMap<String, SectorPerformance>, ApiError> {
        let market_overview = self.client.fetch_market_overview().await?;
        Ok(market_overview.sectors)
    }

    // Batch Quote Endpoint
    pub async fn get_batch_quotes(&self, symbols: Vec<String>) -> Result<HashMap<String, Quote>, ApiError> {
        let mut quotes = HashMap::new();
        let crumb = self.client.get_crumb().await?;

        // Process in batches to avoid rate limiting
        for symbol in symbols {
            match self.client.fetch_single_quote(&symbol, &crumb).await {
                Ok(quote) => {
                    quotes.insert(symbol, quote);
                }
                Err(e) => {
                    eprintln!("Failed to fetch quote for {}: {}", symbol, e);
                }
            }
            
            // Brief delay between requests
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(quotes)
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

// Supporting types for missing implementations
#[derive(Debug, Serialize)]
pub struct HealthStatus<'a> {
    pub status: &'a str,
    pub crumb_cache_status: &'a str,
    pub uptime: u64,
    pub version: &'a str,
}

#[derive(Debug, Clone)]
pub struct YahooScreenerFilter {
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
    pub indicators: Option<HashMap<String, f64>>,
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
    pub last_updated: String,
}

// Placeholder for indicator configuration
#[derive(Debug, Deserialize)]
pub struct IndicatorConfig {
    pub name: String,
    pub parameters: HashMap<String, f64>,
}