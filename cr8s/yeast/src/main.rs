// main.rs - Complete integration example
use std::sync::Arc;
use std::error::Error;

// Import all your existing modules
//mod tls;
mod indicators;
mod types;
mod options_math;
mod api; // The API layer we just created
mod og;

use api::*;
use crate::indicators::*;
use crate::og::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Stock Data API Server");

    // Initialize fetchers
    let chart_fetcher = Arc::new(AsyncFetcher::new());
    let options_fetcher = Arc::new(AsyncOptionsFetcher::new());
    
    // Build indicators
    let indicators = build_comprehensive_indicators();
    
    // Create API instance
    let api = StockDataApi::new(chart_fetcher, options_fetcher, indicators);

    // Option 1: Run examples
    if std::env::args().any(|arg| arg == "--examples") {
        run_api_examples(&api).await?;
        return Ok(());
    }

    // Option 2: Start HTTP server
    if std::env::args().any(|arg| arg == "--server") {
        #[cfg(feature = "simple-server")]
        {
            let server = http_server::StockApiServer::new(api);
            server.start("127.0.0.1:8080")?;
        }
        #[cfg(not(feature = "simple-server"))]
        {
            println!("Server feature not enabled. Compile with --features simple-server");
        }
        return Ok(());
    }

    // Option 3: Interactive CLI
    run_interactive_cli(&api).await?;

    Ok(())
}

async fn run_api_examples(api: &StockDataApi) -> Result<(), Box<dyn Error>> {
    println!("📊 Running API Examples\n");

    // Example 1: Multi-ticker historical data with indicators
    println!("=== Historical Data with Technical Indicators ===");
    let hist_request = HistoricalDataRequest {
        tickers: vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()],
        interval: Some("1d".to_string()),
        range: Some("3mo".to_string()),
        start_date: None,
        end_date: None,
        include_indicators: Some(true),
        indicators: Some(vec![
            IndicatorConfig {
                name: "SMA".to_string(),
                params: Some([
                    ("period".to_string(), serde_json::Value::Number(20.into()))
                ].iter().cloned().collect()),
            },
            IndicatorConfig {
                name: "RSI".to_string(),
                params: Some([
                    ("period".to_string(), serde_json::Value::Number(14.into()))
                ].iter().cloned().collect()),
            },
            IndicatorConfig {
                name: "MACD".to_string(),
                params: Some([
                    ("fast".to_string(), serde_json::Value::Number(12.into())),
                    ("slow".to_string(), serde_json::Value::Number(26.into()))
                ].iter().cloned().collect()),
            },
        ]),
    };

    match api.get_historical_data(hist_request).await {
        Ok(response) => {
            for (ticker, data) in &response.data {
                println!("📈 {}: {} candles, Current Price: ${:.2}", 
                    ticker, 
                    data.candles.len(),
                    data.meta.regular_market_price
                );
                
                if let Some(ref indicators) = data.indicators {
                    println!(
                        "   Indicators available: {}",
                        indicators.keys().map(|k| k.as_str()).collect::<Vec<&str>>().join(", ")
                    );


                    
                    // Show latest indicator values
                    if let Some(latest_candle) = data.candles.last() {
                        println!("   Latest Close: ${:.2} ({})", latest_candle.close, latest_candle.datetime);
                        for (name, values) in indicators {
                            if let Some(Some(latest_val)) = values.last() {
                                println!("   {}: {:.2}", name, latest_val);
                            }
                        }
                    }
                }
                println!();
            }
        }
        Err(e) => eprintln!("❌ Historical data error: {}", e),
    }

    // Example 2: Comprehensive options analysis
    println!("=== Options Chain Analysis with Greeks ===");
    let options_request = OptionsChainRequest {
        ticker: "AAPL".to_string(),
        expiration_dates: None,
        min_strike: Some(140.0),
        max_strike: Some(170.0),
        option_type: Some("both".to_string()),
        include_greeks: Some(true),
        volatility: Some(0.3),
        risk_free_rate: Some(0.02),
    };

    match api.get_options_chain(options_request).await {
        Ok(response) => {
            println!("🎯 Options for {} (Underlying: ${:.2})", response.symbol, response.underlying_price);
            
            for (expiry, data) in response.expirations.iter().take(2) { // Show first 2 expirations
                println!("\n📅 Expiration: {} ({:.0} days)", expiry, data.days_to_expiry);
                
                // Show top 5 calls by volume
                let mut top_calls = data.calls.clone();
                top_calls.sort_by(|a, b| b.volume.cmp(&a.volume));
                
                println!("   📞 Top Calls by Volume:");
                println!("   {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}", 
                    "Strike", "Last", "Bid", "Ask", "Volume", "OI", "Delta", "Gamma");
                
                for call in top_calls.iter().take(5) {
                    if let Some(ref greeks) = call.greeks {
                        println!("   {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8} {:>8} {:>8.3} {:>8.4}",
                            call.strike, call.last, call.bid, call.ask, 
                            call.volume, call.open_interest, greeks.delta, greeks.gamma);
                    }
                }

                // Show top 5 puts by volume
                let mut top_puts = data.puts.clone();
                top_puts.sort_by(|a, b| b.volume.cmp(&a.volume));
                
                println!("   📉 Top Puts by Volume:");
                println!("   {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}", 
                    "Strike", "Last", "Bid", "Ask", "Volume", "OI", "Delta", "Gamma");
                
                for put in top_puts.iter().take(5) {
                    if let Some(ref greeks) = put.greeks {
                        println!("   {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8} {:>8} {:>8.3} {:>8.4}",
                            put.strike, put.last, put.bid, put.ask, 
                            put.volume, put.open_interest, greeks.delta, greeks.gamma);
                    }
                }
            }
        }
        Err(e) => eprintln!("❌ Options chain error: {}", e),
    }

    // Example 3: Complex options strategy P&L analysis
    println!("\n=== Options Strategy P&L Analysis ===");
    println!("📊 Iron Condor Strategy Analysis");
    
    let pnl_request = OptionsPnLRequest {
        positions: vec![
            // Iron Condor: Sell call spread + sell put spread
            OptionPosition {
                option_type: "call".to_string(),
                strike: 155.0,    // Sell call
                quantity: -1,
                entry_price: 3.5,
                days_to_expiry: 30.0,
            },
            OptionPosition {
                option_type: "call".to_string(),
                strike: 160.0,    // Buy call (protection)
                quantity: 1,
                entry_price: 1.5,
                days_to_expiry: 30.0,
            },
            OptionPosition {
                option_type: "put".to_string(),
                strike: 145.0,    // Sell put
                quantity: -1,
                entry_price: 2.8,
                days_to_expiry: 30.0,
            },
            OptionPosition {
                option_type: "put".to_string(),
                strike: 140.0,    // Buy put (protection)
                quantity: 1,
                entry_price: 1.2,
                days_to_expiry: 30.0,
            },
        ],
        underlying_prices: (130..180).map(|x| x as f64).collect(),
        volatility: Some(0.25),
        risk_free_rate: Some(0.02),
        days_to_expiry: Some(30.0),
    };

    match api.calculate_options_pnl(pnl_request) {
        Ok(response) => {
            println!("✅ Strategy Analysis Complete");
            
            // Strategy summary
            let net_credit = 3.5 + 2.8 - 1.5 - 1.2; // Premium collected
            println!("   Net Credit Received: ${:.2}", net_credit);
            
            if let Some(max_profit) = response.portfolio.max_profit {
                println!("   Maximum Profit: ${:.2}", max_profit);
            }
            if let Some(max_loss) = response.portfolio.max_loss {
                println!("   Maximum Loss: ${:.2}", max_loss);
            }
            
            println!("   Break-even Points: {:?}", response.portfolio.break_even_points
                .iter().map(|&p| format!("${:.2}", p)).collect::<Vec<_>>());
            
            // Show P&L at key price levels
            println!("\n   P&L at Key Levels:");
            let key_prices = vec![140.0, 145.0, 150.0, 155.0, 160.0];
            for &price in &key_prices {
                if let Some(pnl_point) = response.portfolio.total_pnl_curve
                    .iter().find(|p| (p.underlying_price - price).abs() < 0.5) {
                    println!("   ${:.0}: ${:.2}", price, pnl_point.pnl);
                }
            }

            // Individual position analysis
            println!("\n   Individual Position Greeks:");
            for (i, pos_analysis) in response.positions.iter().enumerate() {
                println!("   Position {}: {} ${:.0} x{}", 
                    i + 1, 
                    pos_analysis.position.option_type,
                    pos_analysis.position.strike,
                    pos_analysis.position.quantity
                );
                println!("     Delta: {:.3}, Gamma: {:.4}, Theta: {:.3}, Vega: {:.3}",
                    pos_analysis.greeks.delta,
                    pos_analysis.greeks.gamma,
                    pos_analysis.greeks.theta,
                    pos_analysis.greeks.vega
                );
            }
        }
        Err(e) => eprintln!("❌ P&L analysis error: {}", e),
    }

    // Example 4: Real-time market snapshot
    println!("\n=== Real-time Market Snapshot ===");
    let quote_request = QuoteRequest {
        tickers: vec![
            "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(),
            "TSLA".to_string(), "NVDA".to_string(), "META".to_string()
        ],
        fields: None,
    };

    match api.get_quotes(quote_request).await {
        Ok(response) => {
            println!("📊 Market Quotes ({} tickers)", response.quotes.len());
            println!("{:>8} {:>10} {:>8} {:>8} {:>12} {:>8} {:>8}", 
                "Ticker", "Price", "Change", "Change%", "Volume", "52W High", "52W Low");
            
            for (ticker, quote) in &response.quotes {
                println!("{:>8} {:>10.2} {:>8.2} {:>7.2}% {:>12} {:>8.2} {:>8.2}",
                    ticker,
                    quote.price,
                    quote.change,
                    quote.change_percent,
                    format_volume(quote.volume),
                    quote.high_52w,
                    quote.low_52w
                );
            }
        }
        Err(e) => eprintln!("❌ Quotes error: {}", e),
    }

    // Example 5: Market summary with indices
    println!("\n=== Market Summary ===");
    match api.get_market_summary().await {
        Ok(summary) => {
            println!("🏛️  Major Indices (Status: {})", summary.market_status);
            for (index, quote) in &summary.indices {
                let index_name = match index.as_str() {
                    "^GSPC" => "S&P 500",
                    "^DJI" => "Dow Jones",
                    "^IXIC" => "NASDAQ",
                    _ => index,
                };
                println!("   {}: {:.2} ({:+.2}%)", 
                    index_name, quote.price, quote.change_percent);
            }
            println!("   Last Updated: {}", summary.last_updated);
        }
        Err(e) => eprintln!("❌ Market summary error: {}", e),
    }

    Ok(())
}

async fn run_interactive_cli(api: &StockDataApi) -> Result<(), Box<dyn Error>> {
    println!("🖥️  Interactive Stock Data CLI");
    println!("Commands: hist <ticker>, options <ticker>, quote <ticker>, help, quit");

    loop {
        print!("\n> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "help" => {
                println!("Available commands:");
                println!("  hist <ticker> [range]  - Get historical data (default: 1mo)");
                println!("  options <ticker>       - Get options chain with Greeks");
                println!("  quote <ticker>         - Get real-time quote");
                println!("  market                 - Get market summary");
                println!("  screen                 - Run basic stock screener");
                println!("  quit                   - Exit");
            }
            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }
            "hist" => {
                if parts.len() < 2 {
                    println!("Usage: hist <ticker> [range]");
                    continue;
                }
                let ticker = parts[1].to_uppercase();
                let range = parts.get(2).unwrap_or(&"1mo").to_string();
                
                let request = HistoricalDataRequest {
                    tickers: vec![ticker.clone()],
                    range: Some(range),
                    include_indicators: Some(true),
                    ..Default::default()
                };

                match api.get_historical_data(request).await {
                    Ok(response) => {
                        if let Some(data) = response.data.get(&ticker) {
                            println!("📈 {} - {} candles", ticker, data.candles.len());
                            if let Some(latest) = data.candles.last() {
                                println!("   Latest: ${:.2} on {}", latest.close, latest.datetime);
                            }
                            if let Some(ref indicators) = data.indicators {
                                for (name, values) in indicators.iter().take(5) {
                                    if let Some(Some(val)) = values.last() {
                                        println!("   {}: {:.2}", name, val);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "options" => {
                if parts.len() < 2 {
                    println!("Usage: options <ticker>");
                    continue;
                }
                let ticker = parts[1].to_uppercase();
                
                let request = OptionsChainRequest {
                    ticker: ticker.clone(),
                    include_greeks: Some(true),
                    ..Default::default()
                };

                match api.get_options_chain(request).await {
                    Ok(response) => {
                        println!("🎯 Options for {} (${:.2})", ticker, response.underlying_price);
                        for (expiry, data) in response.expirations.iter().take(2) {
                            println!("   {}: {} calls, {} puts", 
                                expiry, data.calls.len(), data.puts.len());
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "quote" => {
                if parts.len() < 2 {
                    println!("Usage: quote <ticker>");
                    continue;
                }
                let ticker = parts[1].to_uppercase();
                
                let request = QuoteRequest {
                    tickers: vec![ticker.clone()],
                    fields: None,
                };

                match api.get_quotes(request).await {
                    Ok(response) => {
                        if let Some(quote) = response.quotes.get(&ticker) {
                            println!("📊 {}: ${:.2} ({:+.2}%)", 
                                ticker, quote.price, quote.change_percent);
                            println!("   Volume: {}, 52W Range: ${:.2} - ${:.2}",
                                format_volume(quote.volume), quote.low_52w, quote.high_52w);
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "market" => {
                match api.get_market_summary().await {
                    Ok(summary) => {
                        println!("🏛️  Market Summary:");
                        for (index, quote) in &summary.indices {
                            println!("   {}: {:.2} ({:+.2}%)", 
                                index, quote.price, quote.change_percent);
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            // "screen" => {
            //     println!("📋 Basic Stock Screener");
            //     let request = ScreenerRequest {
            //         filters: vec![
            //             ScreenerFilter {
            //                 field: "price".to_string(),
            //                 operator: "gt".to_string(),
            //                 value: serde_json::Value::Number(50.into()),
            //             }
            //         ],
            //         sort_by: Some("volume".to_string()),
            //         sort_order: Some("desc".to_string()),
            //         limit: Some(10),
            //         indicators: None,
            //     };

            //     match api.screen_stocks(request).await {
            //         Ok(response) => {
            //             println!("   Found {} results", response.total_count);
            //             for result in &response.results {
            //                 println!("   {}: ${:.2} ({:+.2}%)", 
            //                     result.symbol, result.price, result.change_percent);
            //             }
            //         }
            //         Err(e) => println!("❌ Error: {}", e),
            //     }
            // }
            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
            }
        }
    }

    Ok(())
}

fn build_comprehensive_indicators() -> Vec<(String, Arc<dyn TechnicalIndicator + Send + Sync>)> {
    vec![
        // Moving Averages
        ("SMA(10)".to_string(), Arc::new(SMA { period: 10 })),
        ("SMA(20)".to_string(), Arc::new(SMA { period: 20 })),
        ("SMA(50)".to_string(), Arc::new(SMA { period: 50 })),
        ("EMA(12)".to_string(), Arc::new(EMA { period: 12 })),
        ("EMA(26)".to_string(), Arc::new(EMA { period: 26 })),
        ("WMA(20)".to_string(), Arc::new(WMA { period: 20 })),
        
        // Momentum Indicators
        ("RSI(14)".to_string(), Arc::new(RSI { period: 14 })),
        ("MACD(12,26)".to_string(), Arc::new(MACD { fast_period: 12, slow_period: 26 })),
        ("Stochastic(14,3)".to_string(), Arc::new(Stochastic { k_period: 14, d_period: 3 })),
        ("CCI(20)".to_string(), Arc::new(CCI { period: 20 })),
        ("WilliamsR(14)".to_string(), Arc::new(WilliamsR { period: 14 })),
        
        // Volatility Indicators
        ("BollingerBands(20)".to_string(), Arc::new(BollingerBands { period: 20, k: 2.0 })),
        ("ATR(14)".to_string(), Arc::new(ATR { period: 14 })),
        
        // Volume Indicators
        ("VWAP".to_string(), Arc::new(VWAP {})),
        ("OBV".to_string(), Arc::new(OBV {})),
        ("CMF(20)".to_string(), Arc::new(CMF { period: 20 })),
        
        // Trend Indicators
        ("ADX(14)".to_string(), Arc::new(ADX { period: 14 })),
        ("ParabolicSAR".to_string(), Arc::new(ParabolicSAR { step: 0.02, max_step: 0.2 })),
        
        // Advanced Indicators
        ("Ichimoku".to_string(), Arc::new(Ichimoku {
            conversion_period: 9,
            base_period: 26,
            leading_span_b_period: 52,
            displacement: 26,
        })),
    ]
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

// Default implementation for HistoricalDataRequest
impl Default for HistoricalDataRequest {
    fn default() -> Self {
        Self {
            tickers: vec!["AAPL".to_string()],
            interval: Some("1d".to_string()),
            range: Some("1mo".to_string()),
            start_date: None,
            end_date: None,
            include_indicators: Some(false),
            indicators: None,
        }
    }
}

impl Default for OptionsChainRequest {
    fn default() -> Self {
        Self {
            ticker: "AAPL".to_string(),
            expiration_dates: None,
            min_strike: None,
            max_strike: None,
            option_type: Some("both".to_string()),
            include_greeks: Some(false),
            volatility: Some(0.25),
            risk_free_rate: Some(0.01),
        }
    }
}

// Configuration for different deployment scenarios
pub struct ApiConfig {
    pub port: u16,
    pub enable_cors: bool,
    pub rate_limit: Option<RateLimit>,
    pub cache_ttl: u64, // seconds
    pub max_tickers_per_request: usize,
}

pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            enable_cors: true,
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                requests_per_hour: 1000,
            }),
            cache_ttl: 60, // 1 minute cache
            max_tickers_per_request: 10,
        }
    }
}


/*
curl "http://127.0.0.1:8080/api/v1/historical?tickers=AAPL,MSFT&range=1mo"

curl "http://127.0.0.1:8080/api/v1/options?ticker=AAPL&include_greeks=true"

curl -X POST "http://127.0.0.1:8080/api/v1/options/pnl" \
     -H "Content-Type: application/json" \
     -d '{
           "ticker": "AAPL",
           "type": "Call",
           "strike": 190.0,
           "expiry": "2025-08-16",
           "open_price": 3.5,
           "close_price": 5.0,
           "contracts": 10
         }'

curl "http://127.0.0.1:8080/api/v1/quotes?tickers=AAPL,MSFT"

curl "http://127.0.0.1:8080/api/v1/market/summary"

*/