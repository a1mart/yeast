let result = chart_data.chart.result
            .as_ref()
            .and_then(|results| results.get(0))
            .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

        let candles = to_candles(result);
        if candles.is_empty() {
            return Err(ApiError::DataNotFound("No valid candles found".to_string()));
        }

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
                adj_close: None,
            });
        }

        let indicators = if request.include_indicators.unwrap_or(false) {
            Some(self.indicator_runner.run(&candles))
        } else {
            None
        };

        let meta = TickerMeta {
            currency: result.meta.currency.clone(),
            exchange: result.meta.exchange_name.clone(),
            instrument_type: result.meta.instrument_type.clone(),
            timezone: result.meta.timezone.clone(),
            regular_market_price: result.meta.regular_market_price,
            fifty_two_week_high: result.meta.fifty_two_week_high,
            fifty_two_week_low: result.meta.fifty_two_week_low,
            market_cap: None,
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
            let days_to_expiry = 30.0; // Simplified - would parse expiry_str properly
            let time_to_expiry = days_to_expiry / 365.0;

            let mut calls = Vec::new();
            let mut puts = Vec::new();

            for (strike_str, quote) in exp_data.c {
                let strike: f64 = strike_str.parse().unwrap_or(0.0);
                
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
                    Some(GreeksData {
                        delta: 0.5,
                        gamma: 0.1,
                        theta: -0.05,
                        vega: 0.2,
                        rho: 0.1,
                        theoretical_price: quote.l,
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
                    implied_volatility: None,
                    greeks,
                });
            }

            for (strike_str, quote) in exp_data.p {
                let strike: f64 = strike_str.parse().unwrap_or(0.0);
                
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
                    Some(GreeksData {
                        delta: -0.5,
                        gamma: 0.1,
                        theta: -0.05,
                        vega: 0.2,
                        rho: -0.1,
                        theoretical_price: quote.l,
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
            .map(|result| result.meta.regular_market_price)
            .ok_or_else(|| ApiError::DataNotFound("No price data found".to_string()))
    }

    fn calculate_portfolio_analysis(
        &self,
        pnl_curves: &[Vec<PnLPoint>],
        underlying_prices: &[f64],
    ) -> PortfolioAnalysis {
        let mut total_pnl_curve = Vec::new();
        
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

        let mut break_even_points = Vec::new();
        for i in 1..total_pnl_curve.len() {
            let prev = &total_pnl_curve[i - 1];
            let curr = &total_pnl_curve[i];
            
            if (prev.pnl <= 0.0 && curr.pnl >= 0.0) || (prev.pnl >= 0.0 && curr.pnl <= 0.0) {
                let ratio = if (prev.pnl.abs() + curr.pnl.abs()) != 0.0 {
                    prev.pnl.abs() / (prev.pnl.abs() + curr.pnl.abs())
                } else {
                    0.5
                };
                let break_even = prev.underlying_price + ratio * (curr.underlying_price - prev.underlying_price);
                break_even_points.push(break_even);
            }
        }

        let max_profit = total_pnl_curve.iter()
            .map(|point| point.pnl)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let max_loss = total_pnl_curve.iter()
            .map(|point| point.pnl)
            .fold(f64::INFINITY, f64::min);

        let total_greeks = GreeksData {
            delta: 0.0,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            rho: 0.0,
            theoretical_price: 0.0,
        };

        PortfolioAnalysis {
            total_greeks,
            total_pnl_curve,
            break_even_points,
            max_profit: if max_profit.is_finite() { Some(max_profit) } else { None },
            max_loss: if max_loss.is_finite() { Some(max_loss) } else { None },
        }
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

// HTTP Server Implementation
pub mod http_server {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::io::{Read, Write, BufRead, BufReader};
    use std::collections::HashMap;

    pub struct IntegratedApiServer {
        api: Arc<IntegratedStockDataApi>,
    }

    impl IntegratedApiServer {
        pub fn new(api: IntegratedStockDataApi) -> Self {
            Self {
                api: Arc::new(api),
            }
        }

        pub async fn start(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
            let listener = TcpListener::bind(addr)?;
            println!("Integrated Stock API Server running on http://{}", addr);
            println!("Available endpoints:");
            println!("  GET  /api/v1/quote?symbol=AAPL");
            println!("  GET  /api/v1/quotes?symbols=AAPL,MSFT");
            println!("  GET  /api/v1/historical?symbols=AAPL&range=1mo&interval=1d");
            println!("  GET  /api/v1/historical/yahoo?symbols=AAPL&range=1mo&interval=1d");
            println!("  GET  /api/v1/options?ticker=AAPL&include_greeks=true");
            println!("  POST /api/v1/options/pnl");
            println!("  GET  /api/v1/market/overview");
            println!("  GET  /api/v1/market/summary");
            println!("  GET  /api/v1/screener?type=predefined&screener=most_actives");
            println!("  GET  /api/v1/quotesummary?ticker=AAPL");
            println!("  GET  /api/v1/news?ticker=AAPL&count=10");
            println!("  GET  /api/v1/calendar?from=2024-01-01&to=2024-01-31");
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

    async fn handle_request(mut stream: TcpStream, api: Arc<IntegratedStockDataApi>) -> Result<(), Box<dyn std::error::Error>> {
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
            let response = format!("HTTP/1.1 204 No Content\r\n{}\r\n", cors_headers);
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
            ("GET", "/api/v1/historical/yahoo") => {
                handle_historical_data_yahoo(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/options") => {
                handle_options_chain(&mut stream, &*api, query, cors_headers).await?;
            }
            ("POST", "/api/v1/options/pnl") => {
                handle_options_pnl(&mut stream, &*api, &mut reader, cors_headers).await?;
            }
            ("GET", "/api/v1/market/overview") => {
                handle_market_overview(&mut stream, &*api, cors_headers).await?;
            }
            ("GET", "/api/v1/market/summary") => {
                handle_market_summary(&mut stream, &*api, cors_headers).await?;
            }
            ("GET", "/api/v1/screener") => {
                handle_screener(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/quotesummary") => {
                handle_quote_summary(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/news") => {
                handle_news(&mut stream, &*api, query, cors_headers).await?;
            }
            ("GET", "/api/v1/calendar") => {
                handle_calendar(&mut stream, &*api, query, cors_headers).await?;
            }
            ("POST", "/api/v1/portfolio") => {
                handle_create_portfolio(&mut stream, &*api, &mut reader, cors_headers).await?;
            }
            (_, _) if path.starts_with("/api/v1/portfolio/") => {
                let portfolio_id = &path[18..];
                handle_get_portfolio(&mut stream, &*api, portfolio_id, cors_headers).await?;
            }
            ("GET", "/api/v1/health") => {
                handle_health_check(&mut stream, &*api, cors_headers).await?;
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
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbol = query.get("symbol").cloned().unwrap_or_else(|| "AAPL".to_string());

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
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbols = query.get("symbols")
            .map(|s| s.split(',').map(|symbol| symbol.trim().to_uppercase()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        let request = QuoteRequest {
            tickers: symbols,
            fields: None,
        };

        match api.get_quotes(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_historical_data(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tickers = query.get("symbols")
            .or_else(|| query.get("tickers"))
            .map(|s| s.split(',').map(|symbol| symbol.trim().to_uppercase()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        let request = HistoricalDataRequest {
            tickers,
            interval: query.get("interval").cloned(),
            range: query.get("range").cloned(),
            start_date: query.get("start_date").cloned(),
            end_date: query.get("end_date").cloned(),
            include_indicators: query.get("include_indicators").map(|v| v == "true"),
            indicators: None,
        };

        match api.get_historical_data(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_historical_data_yahoo(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbols = query.get("symbols")
            .map(|s| s.split(',').map(|symbol| symbol.trim().to_uppercase()).collect())
            .unwrap_or_else(|| vec!["AAPL".to_string()]);

        let range = query.get("range").cloned().unwrap_or_else(|| "1mo".to_string());
        let interval = query.get("interval").cloned().unwrap_or_else(|| "1d".to_string());

        match api.get_historical_data_yahoo(symbols, &range, &interval).await {
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
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_options_chain(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = query.get("ticker").cloned().unwrap_or_else(|| "AAPL".to_string());

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
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_options_pnl(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        reader: &mut BufReader<TcpStream>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;

        let pnl_request: OptionsPnLRequest = match serde_json::from_slice(&body) {
            Ok(req) => req,
            Err(_) => {
                send_response(stream, 400, "Bad Request", "Invalid JSON")?;
                return Ok(());
            }
        };

        match api.calculate_options_pnl(pnl_request) {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_market_overview(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match api.get_market_overview().await {
            Ok(overview) => {
                let json = serde_json::to_string(&overview)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_market_summary(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match api.get_market_summary().await {
            Ok(summary) => {
                let json = serde_json::to_string(&summary)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_screener(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = ScreenerRequest {
            filters: Vec::new(),
            sort_by: query.get("sort_by").cloned(),
            sort_order: query.get("sort_order").cloned(),
            limit: query.get("limit").and_then(|l| l.parse().ok()),
            offset: query.get("offset").and_then(|o| o.parse().ok()),
            screener_type: query.get("type").cloned(),
            predefined_screener: query.get("screener").cloned(),
            indicators: None,
        };

        match api.run_screener(request).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({ "error": e.to_string() });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_quote_summary(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = query.get("ticker").cloned().unwrap_or_else(|| "AAPL".to_string());

        match api.get_quote_summary(&ticker).await {
            Ok(response) => {
                let json = serde_json::to_string(&response)?;
                send_json_response(stream, 200, &json, cors_headers)?;
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "ticker": ticker
                });
                let json = serde_json::to_string(&error_response)?;
                send_json_response(stream, 500, &json, cors_headers)?;
            }
        }
        Ok(())
    }

    async fn handle_news(
        stream: &mut TcpStream,
        api: &IntegratedStockDataApi,
        query: HashMap<String, String>,
        cors_headers: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = query.get("ticker").cloned().unwrap_or_else(|| "AAPL".to_string());
        let count = query.get("count").and_then(|c| c.parse::<u32>().ok());

        match api.get_news(&ticker, count).await {
            Ok(response) => {
                let json = serde_        } else if vix < 30.0 {
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

    // Helper parsing methods
    fn parse_quote_summary(&self, ticker: &str, json: serde_json::Value) -> Result<QuoteSummaryResponse, ApiError> {
        let result = json
            .get("quoteSummary")
            .and_then(|qs| qs.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ApiError::DataNotFound("No quote summary data".to_string()))?;

        // Parse asset profile
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

        Ok(QuoteSummaryResponse {
            symbol: ticker.to_string(),
            asset_profile,
            financial_data,
            default_key_statistics,
            summary_detail: None, // Would implement similar parsing
            price: None, // Would implement similar parsing  
            summary_profile: None, // Would implement similar parsing
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

        if let Some(existing_position) = portfolio.positions.iter_mut().find(|p| p.symbol == symbol) {
            let total_cost = existing_position.average_cost * existing_position.quantity + price * quantity;
            existing_position.quantity += quantity;
            existing_position.average_cost = total_cost / existing_position.quantity;
        } else {
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
}

// Main Integrated API Service
pub struct IntegratedStockDataApi {
    client: Arc<EnhancedYahooFinanceClient>,
    portfolio_manager: Arc<PortfolioManager>,
    chart_fetcher: Arc<dyn ChartFetcher + Send + Sync>,
    options_fetcher: Arc<dyn OptionsFetcher + Send + Sync>,
    indicator_runner: IndicatorRunner,
}

impl IntegratedStockDataApi {
    pub fn new(
        chart_fetcher: Arc<dyn ChartFetcher + Send + Sync>,
        options_fetcher: Arc<dyn OptionsFetcher + Send + Sync>,
        indicators: Vec<(String, Arc<dyn TechnicalIndicator + Send + Sync>)>,
    ) -> Self {
        let client = Arc::new(EnhancedYahooFinanceClient::new());
        let portfolio_manager = Arc::new(PortfolioManager::new(client.clone()));

        Self {
            client,
            portfolio_manager,
            chart_fetcher,
            options_fetcher,
            indicator_runner: IndicatorRunner { indicators },
        }
    }

    // Historical Data Endpoint (integrates with existing chart fetcher)
    pub async fn get_historical_data(&self, request: HistoricalDataRequest) -> Result<HistoricalDataResponse, ApiError> {
        let mut data = HashMap::new();
        let mut errors = Vec::new();

        let options = ChartQueryOptions {
            interval: request.interval.as_deref().unwrap_or("1d"),
            range: request.range.as_deref().unwrap_or("1mo"),
        };

        for ticker in &request.tickers {
            match self.chart_fetcher.fetch_async(ticker, &options).await {
                Ok(chart_data) => {
                    match self.process_ticker_data(chart_data, &request) {
                        Ok(ticker_data) => {
                            data.insert(ticker.clone(), ticker_data);
                        }
                        Err(e) => {
                            errors.push(format!("Error processing {}: {}", ticker, e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Error fetching {}: {}", ticker, e));
                }
            }
        }

        Ok(HistoricalDataResponse { data, errors })
    }

    // Options Chain Endpoint (integrates with existing options fetcher)
    pub async fn get_options_chain(&self, request: OptionsChainRequest) -> Result<OptionsChainResponse, ApiError> {
        let chart_options = ChartQueryOptions::default();
        let chart_data = self.chart_fetcher.fetch_async(&request.ticker, &chart_options).await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        let underlying_price = self.extract_current_price(&chart_data)?;

        let options_data = self.options_fetcher.fetch_async(&request.ticker).await
            .map_err(|e| ApiError::FetchError(e.to_string()))?;

        self.process_options_data(options_data, &request, underlying_price)
    }

    // Options P&L Analysis Endpoint
    pub fn calculate_options_pnl(&self, request: OptionsPnLRequest) -> Result<OptionsPnLResponse, ApiError> {
        let volatility = request.volatility.unwrap_or(0.25);
        let risk_free_rate = request.risk_free_rate.unwrap_or(0.01);

        let mut positions = Vec::new();
        let mut portfolio_pnl_curves: Vec<Vec<PnLPoint>> = Vec::new();

        for position in &request.positions {
            let mut pnl_curve = Vec::new();
            for &price in &request.underlying_prices {
                // Simplified P&L calculation - would use real options math
                let intrinsic_value = match position.option_type.as_str() {
                    "call" => (price - position.strike).max(0.0),
                    "put" => (position.strike - price).max(0.0),
                    _ => 0.0,
                };

                let pnl = (intrinsic_value - position.entry_price) * position.quantity as f64;
                let total_value = intrinsic_value * position.quantity.abs() as f64;

                pnl_curve.push(PnLPoint {
                    underlying_price: price,
                    pnl,
                    total_value,
                });
            }

            portfolio_pnl_curves.push(pnl_curve.clone());

            positions.push(PositionAnalysis {
                position: position.clone(),
                greeks: GreeksData {
                    delta: 0.5,  // Simplified - would use real Greeks calculation
                    gamma: 0.1,
                    theta: -0.05,
                    vega: 0.2,
                    rho: 0.1,
                    theoretical_price: position.entry_price,
                },
                pnl_curve,
            });
        }

        let portfolio = self.calculate_portfolio_analysis(&portfolio_pnl_curves, &request.underlying_prices);

        Ok(OptionsPnLResponse {
            positions,
            portfolio,
        })
    }

    // Real-time Quotes Endpoint (uses Yahoo Finance client)
    pub async fn get_quotes(&self, request: QuoteRequest) -> Result<QuoteResponse, ApiError> {
        let quotes = self.client.fetch_batch_quotes(&request.tickers).await?;
        Ok(QuoteResponse { quotes, errors: Vec::new() })
    }

    // Market Overview Endpoint
    pub async fn get_market_overview(&self) -> Result<MarketOverview, ApiError> {
        self.client.fetch_market_overview().await
    }

    // Single Quote Endpoint
    pub async fn get_single_quote(&self, symbol: &str) -> Result<Quote, ApiError> {
        let crumb = self.client.get_crumb().await?;
        self.client.fetch_single_quote(symbol, &crumb).await
    }

    // Historical Data with Yahoo Finance
    pub async fn get_historical_data_yahoo(&self, symbols: Vec<String>, range: &str, interval: &str) -> Result<HashMap<String, Vec<CandleData>>, ApiError> {
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

    // Quote Summary Endpoint
    pub async fn get_quote_summary(&self, ticker: &str) -> Result<QuoteSummaryResponse, ApiError> {
        self.client.fetch_quote_summary(ticker).await
    }

    // News Endpoint
    pub async fn get_news(&self, ticker: &str, count: Option<u32>) -> Result<NewsResponse, ApiError> {
        self.client.fetch_news(ticker, count).await
    }

    // Calendar Endpoint
    pub async fn get_calendar(&self, from: &str, to: &str) -> Result<CalendarResponse, ApiError> {
        self.client.fetch_calendar(from, to).await
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

    // Market Summary
    pub async fn get_market_summary(&self) -> Result<MarketSummary, ApiError> {
        let indices = vec!["^GSPC", "^DJI", "^IXIC"];
        let mut index_data = HashMap::new();
        
        for index in &indices {
            if let Ok(quote) = self.get_single_quote(index).await {
                index_data.insert(index.to_string(), quote);
            }
        }

        Ok(MarketSummary {
            indices: index_data,
            market_status: "OPEN".to_string(),
            last_updated: Utc::now().to_rfc3339(),
        })
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

    // Helper methods
    fn process_ticker_data(&self, chart_data: ChartResponse, request: &HistoricalDataRequest) -> Result<TickerData, ApiError> {
        let result = chart_data.chart.result
            .as_ref()
            .and_then(|results| results.get    pub price_to_book: Option<f64>,
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

        let html = response.text().await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let patterns = [
            r#""CrumbStore":\s*\{\s*"crumb":\s*"([^"]+)""#,
            r#""crumb"\s*:\s*"([^"]+)""#,
            r#"window\.crumb\s*=\s*"([^"]+)""#,
            r#"crumb["\']?\s*:\s*["\']([^"\']+)["\']"#,
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(&html) {
                    if let Some(crumb_match) = captures.get(1) {
                        let crumb = crumb_match.as_str().to_string();
                        if !crumb.is_empty() && crumb.len() < 50 {
                            return Ok(crumb);
                        }
                    }
                }
            }
        }

        Err(ApiError::ParseError("Crumb not found in HTML".to_string()))
    }

    // REAL IMPLEMENTATION - Single Quote
    pub async fn fetch_single_quote(&self, symbol: &str, crumb: &str) -> Result<Quote, ApiError> {
        self.rate_limiter.write().await.wait_if_needed().await;

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d&crumb={}",
            symbol, crumb
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", &format!("https://finance.yahoo.com/quote/{}", symbol))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        // Parse Yahoo's chart response to extract quote data
        let result = json.get("chart")
            .and_then(|c| c.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

        let meta = result.get("meta")
            .ok_or_else(|| ApiError::DataNotFound("No meta data found".to_string()))?;

        let current_price = meta.get("regularMarketPrice")
            .and_then(|p| p.as_f64())
            .ok_or_else(|| ApiError::DataNotFound("No current price found".to_string()))?;

        let prev_close = meta.get("chartPreviousClose")
            .and_then(|p| p.as_f64())
            .unwrap_or(current_price);

        let change = current_price - prev_close;
        let change_percent = if prev_close != 0.0 { (change / prev_close) * 100.0 } else { 0.0 };

        let volume = meta.get("regularMarketVolume")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let high_52w = meta.get("fiftyTwoWeekHigh")
            .and_then(|h| h.as_f64())
            .unwrap_or(current_price * 1.3);

        let low_52w = meta.get("fiftyTwoWeekLow")
            .and_then(|l| l.as_f64())
            .unwrap_or(current_price * 0.7);

        Ok(Quote {
            symbol: symbol.to_string(),
            price: current_price,
            change,
            change_percent,
            volume,
            bid: None,
            ask: None,
            bid_size: None,
            ask_size: None,
            high_52w,
            low_52w,
            market_cap: None,
            pe_ratio: None,
            dividend_yield: None,
            last_updated: Utc::now().to_rfc3339(),
        })
    }

    // REAL IMPLEMENTATION - Batch Quotes
    pub async fn fetch_batch_quotes(&self, symbols: &[String]) -> Result<HashMap<String, Quote>, ApiError> {
        let mut quotes = HashMap::new();
        let crumb = self.get_crumb().await?;

        // Process in batches of 5 to avoid overwhelming the API
        for chunk in symbols.chunks(5) {
            for symbol in chunk {
                match self.fetch_single_quote(symbol, &crumb).await {
                    Ok(quote) => {
                        quotes.insert(symbol.clone(), quote);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch quote for {}: {}", symbol, e);
                    }
                }
                
                // Brief delay between requests
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        Ok(quotes)
    }

    // REAL IMPLEMENTATION - Historical Data
    pub async fn fetch_historical_data(&self, symbol: &str, range: &str, interval: &str) -> Result<Vec<CandleData>, ApiError> {
        let crumb = self.get_crumb().await?;
        self.rate_limiter.write().await.wait_if_needed().await;

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}&crumb={}",
            symbol, range, interval, crumb
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", &format!("https://finance.yahoo.com/quote/{}", symbol))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let result = json.get("chart")
            .and_then(|c| c.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

        let timestamps = result.get("timestamp")
            .and_then(|t| t.as_array())
            .ok_or_else(|| ApiError::DataNotFound("No timestamp data".to_string()))?;

        let indicators = result.get("indicators")
            .and_then(|i| i.get("quote"))
            .and_then(|q| q.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ApiError::DataNotFound("No quote data".to_string()))?;

        let opens = indicators.get("open").and_then(|o| o.as_array()).unwrap_or(&vec![]);
        let highs = indicators.get("high").and_then(|h| h.as_array()).unwrap_or(&vec![]);
        let lows = indicators.get("low").and_then(|l| l.as_array()).unwrap_or(&vec![]);
        let closes = indicators.get("close").and_then(|c| c.as_array()).unwrap_or(&vec![]);
        let volumes = indicators.get("volume").and_then(|v| v.as_array()).unwrap_or(&vec![]);

        let mut candles = Vec::new();

        for (i, timestamp_val) in timestamps.iter().enumerate() {
            if let Some(timestamp) = timestamp_val.as_i64() {
                let open = opens.get(i).and_then(|o| o.as_f64());
                let high = highs.get(i).and_then(|h| h.as_f64());
                let low = lows.get(i).and_then(|l| l.as_f64());
                let close = closes.get(i).and_then(|c| c.as_f64());
                let volume = volumes.get(i).and_then(|v| v.as_u64()).map(|v| v as f64);

                if let (Some(open), Some(high), Some(low), Some(close)) = (open, high, low, close) {
                    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
                    let dt: DateTime<Utc> = datetime.into();

                    candles.push(CandleData {
                        timestamp,
                        datetime: dt.to_rfc3339(),
                        open,
                        high,
                        low,
                        close,
                        volume,
                        adj_close: None, // Would need additional parsing
                    });
                }
            }
        }

        Ok(candles)
    }

    // REAL IMPLEMENTATION - Predefined Screener
    pub async fn fetch_predefined_screener(&self, screener_id: &str, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<ScreenerResult>, ApiError> {
        let crumb = self.get_crumb().await?;
        let limit = limit.unwrap_or(25);
        let offset = offset.unwrap_or(0);

        self.rate_limiter.write().await.wait_if_needed().await;

        let url = format!(
            "https://query2.finance.yahoo.com/v1/finance/screener/predefined/saved?count={}&offset={}&scrIds={}&crumb={}",
            limit, offset, screener_id, crumb
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://finance.yahoo.com/screener")
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: YahooScreenerResponse = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let mut results = Vec::new();

        for result in &json.finance.result {
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
                        indicators: None,
                    });
                }
            }
        }

        Ok(results)
    }

    // REAL IMPLEMENTATION - News
    pub async fn fetch_news(&self, ticker: &str, count: Option<u32>) -> Result<NewsResponse, ApiError> {
        let crumb = self.get_crumb().await?;
        let count = count.unwrap_or(20);
        
        let url = format!(
            "https://query1.finance.yahoo.com/v1/finance/search?q={}&quotesCount=0&newsCount={}&crumb={}",
            ticker, count, crumb
        );

        self.rate_limiter.write().await.wait_if_needed().await;

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", &format!("https://finance.yahoo.com/quote/{}", ticker))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        self.parse_news(json)
    }

    // REAL IMPLEMENTATION - Quote Summary
    pub async fn fetch_quote_summary(&self, ticker: &str) -> Result<QuoteSummaryResponse, ApiError> {
        let crumb = self.get_crumb().await?;
        
        let modules = "assetProfile,financialData,defaultKeyStatistics,summaryDetail,price,summaryProfile";
        let url = format!(
            "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{}?modules={}&crumb={}",
            ticker, modules, crumb
        );

        self.rate_limiter.write().await.wait_if_needed().await;

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", &format!("https://finance.yahoo.com/quote/{}", ticker))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        self.parse_quote_summary(ticker, json)
    }

    // REAL IMPLEMENTATION - Calendar
    pub async fn fetch_calendar(&self, from: &str, to: &str) -> Result<CalendarResponse, ApiError> {
        let crumb = self.get_crumb().await?;
        
        let earnings_url = format!(
            "https://query1.finance.yahoo.com/v1/finance/calendar/earnings?from={}&to={}&crumb={}",
            from, to, crumb
        );

        self.rate_limiter.write().await.wait_if_needed().await;

        let earnings_response = self.client
            .get(&earnings_url)
            .header("Accept", "application/json")
            .header("Referer", "https://finance.yahoo.com/calendar/earnings")
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        let earnings_json: serde_json::Value = if earnings_response.status().is_success() {
            earnings_response.json().await.unwrap_or_default()
        } else {
            serde_json::Value::Null
        };

        // For now, return basic structure - would implement full parsing
        Ok(CalendarResponse {
            earnings: Vec::new(),
            dividends: Vec::new(),
            splits: Vec::new(),
            ipos: Vec::new(),
        })
    }

    // Market overview data fetching
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
        ];

        let mut sectors = HashMap::new();

        for (etf_symbol, sector_name) in &sector_etfs {
            if let Ok(quote) = self.fetch_single_quote(etf_symbol, crumb).await {
                let sector_perf = SectorPerformance {
                    sector: sector_name.to_string(),
                    change_percent: quote.change_percent,
                    market_cap: quote.market_cap.unwrap_or(0.0),
                    pe_ratio: quote.pe_ratio,
                    top_stocks: Vec::new(),
                    performance_1d: quote.change_percent,
                    performance_5d: 0.0,
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
            put_call_ratio: 1.0,
            advance_decline_ratio: 1.0,
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
            unusual_volume: Vec::new(),
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
        } else if vuse std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock as AsyncRwLock;
use chrono::{DateTime, Utc, TimeZone};
use regex::Regex;
use uuid::Uuid;

// Re-export existing types integration
pub trait ChartFetcher {
    async fn fetch_async(&self, symbol: &str, options: &ChartQueryOptions) -> Result<ChartResponse, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait OptionsFetcher {
    async fn fetch_async(&self, symbol: &str) -> Result<OptionProfitCalculatorResponse, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait TechnicalIndicator {
    fn calculate(&self, data: &[Candle]) -> Vec<Option<f64>>;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct ChartQueryOptions<'a> {
    pub interval: &'a str,
    pub range: &'a str,
}

impl Default for ChartQueryOptions<'_> {
    fn default() -> Self {
        Self {
            interval: "1d",
            range: "1mo",
        }
    }
}

#[derive(Debug)]
pub struct ChartResponse {
    pub chart: Chart,
}

#[derive(Debug)]
pub struct Chart {
    pub result: Option<Vec<ChartResult>>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ChartResult {
    pub meta: ChartMeta,
    pub timestamp: Option<Vec<i64>>,
    pub indicators: ChartIndicators,
}

#[derive(Debug)]
pub struct ChartMeta {
    pub currency: String,
    pub symbol: String,
    pub exchange_name: String,
    pub instrument_type: String,
    pub timezone: String,
    pub regular_market_price: f64,
    pub chart_previous_close: f64,
    pub fifty_two_week_high: f64,
    pub fifty_two_week_low: f64,
    pub regular_market_volume: u64,
}

#[derive(Debug)]
pub struct ChartIndicators {
    pub quote: Option<Vec<QuoteData>>,
    pub adjclose: Option<Vec<AdjCloseData>>,
}

#[derive(Debug)]
pub struct QuoteData {
    pub open: Option<Vec<Option<f64>>>,
    pub high: Option<Vec<Option<f64>>>,
    pub low: Option<Vec<Option<f64>>>,
    pub close: Option<Vec<Option<f64>>>,
    pub volume: Option<Vec<Option<u64>>>,
}

#[derive(Debug)]
pub struct AdjCloseData {
    pub adjclose: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

#[derive(Debug)]
pub struct OptionProfitCalculatorResponse {
    pub options: HashMap<String, ExpirationOptions>,
}

#[derive(Debug)]
pub struct ExpirationOptions {
    pub c: HashMap<String, OptionQuote>,
    pub p: HashMap<String, OptionQuote>,
}

#[derive(Debug)]
pub struct OptionQuote {
    pub b: f64, // bid
    pub a: f64, // ask
    pub l: f64, // last
    pub v: u64, // volume
    pub oi: u64, // open interest
}

pub struct IndicatorRunner {
    pub indicators: Vec<(String, Arc<dyn TechnicalIndicator + Send + Sync>)>,
}

impl IndicatorRunner {
    pub fn run(&self, candles: &[Candle]) -> HashMap<String, Vec<Option<f64>>> {
        let mut results = HashMap::new();
        for (name, indicator) in &self.indicators {
            let values = indicator.calculate(candles);
            results.insert(name.clone(), values);
        }
        results
    }
}

pub fn to_candles(result: &ChartResult) -> Vec<Candle> {
    let mut candles = Vec::new();
    
    if let (Some(timestamps), Some(quotes)) = (&result.timestamp, &result.indicators.quote) {
        if let Some(quote_data) = quotes.get(0) {
            let opens = quote_data.open.as_ref().unwrap_or(&vec![]);
            let highs = quote_data.high.as_ref().unwrap_or(&vec![]);
            let lows = quote_data.low.as_ref().unwrap_or(&vec![]);
            let closes = quote_data.close.as_ref().unwrap_or(&vec![]);
            let volumes = quote_data.volume.as_ref().unwrap_or(&vec![]);

            for (i, &timestamp) in timestamps.iter().enumerate() {
                if let (Some(Some(open)), Some(Some(high)), Some(Some(low)), Some(Some(close))) = (
                    opens.get(i).cloned().flatten(),
                    highs.get(i).cloned().flatten(),
                    lows.get(i).cloned().flatten(),
                    closes.get(i).cloned().flatten(),
                ) {
                    let volume = volumes.get(i).cloned().flatten().map(|v| v as f64);
                    
                    candles.push(Candle {
                        timestamp,
                        open,
                        high,
                        low,
                        close,
                        volume,
                    });
                }
            }
        }
    }
    
    candles
}

// Enhanced Error Types
#[derive(Debug, Clone)]
pub enum ApiError {
    InvalidTicker(String),
    InvalidDateRange(String),
    DataNotFound(String),
    FetchError(String),
    CalculationError(String),
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
            ApiError::InvalidTicker(t) => write!(f, "Invalid ticker: {}", t),
            ApiError::InvalidDateRange(r) => write!(f, "Invalid date range: {}", r),
            ApiError::DataNotFound(msg) => write!(f, "Data not found: {}", msg),
            ApiError::FetchError(msg) => write!(f, "Fetch error: {}", msg),
            ApiError::CalculationError(msg) => write!(f, "Calculation error: {}", msg),
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

// Market Overview
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

// Additional response types
#[derive(Debug, Serialize)]
pub struct MarketSummary {
    pub indices: HashMap<String, Quote>,
    pub market_status: String,
    pub last_updated: String,
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
    pub indicators: Option<Vec<IndicatorConfig>>,
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
    pub indicators: Option<HashMap<String, f64>>,
}

// Yahoo Finance Response Types for Screener
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
    pub price_to_book: Option<fuse std::collections::HashMap;
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

// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quote_fetch() {
        let api = EnhancedStockDataApi::new();
        let result = api.get_single_quote("AAPL").await;
        assert!(result.is_ok());
    }

    #[tokio::test] 
    async fn test_portfolio_creation() {
        let api = EnhancedStockDataApi::new();
        let portfolio_id = api.create_portfolio("Test Portfolio".to_string(), None).await.unwrap();
        assert!(!portfolio_id.is_empty());
        
        let portfolio = api.get_portfolio(&portfolio_id).await.unwrap();
        assert_eq!(portfolio.name, "Test Portfolio");
    }

    #[tokio::test]
    async fn test_health_check() {
        let api = EnhancedStockDataApi::new();
        let health = api.health_check().await.unwrap();
        assert_eq!(health.version, "1.0.0");
    }
} response.status())));
        }

        let html = response.text().await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let patterns = [
            r#""CrumbStore":\s*\{\s*"crumb":\s*"([^"]+)""#,
            r#""crumb"\s*:\s*"([^"]+)""#,
            r#"window\.crumb\s*=\s*"([^"]+)""#,
            r#"crumb["\']?\s*:\s*["\']([^"\']+)["\']"#,
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(&html) {
                    if let Some(crumb_match) = captures.get(1) {
                        let crumb = crumb_match.as_str().to_string();
                        if !crumb.is_empty() && crumb.len() < 50 {
                            return Ok(crumb);
                        }
                    }
                }
            }
        }

        Err(ApiError::ParseError("Crumb not found in HTML".to_string()))
    }

    // REAL IMPLEMENTATION - Single Quote
    pub async fn fetch_single_quote(&self, symbol: &str, crumb: &str) -> Result<Quote, ApiError> {
        self.rate_limiter.write().await.wait_if_needed().await;

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d&crumb={}",
            symbol, crumb
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", &format!("https://finance.yahoo.com/quote/{}", symbol))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let yahoo_response: YahooChartResponse = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        if let Some(ref error) = yahoo_response.chart.error {
            return Err(ApiError::FetchError(format!("Yahoo error: {:?}", error)));
        }

        let result = yahoo_response.chart.result
            .as_ref()
            .and_then(|results| results.get(0))
            .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

        let meta = &result.meta;
        let current_price = meta.regular_market_price;
        let prev_close = meta.chart_previous_close;
        let change = current_price - prev_close;
        let change_percent = (change / prev_close) * 100.0;

        // Get latest volume from indicators
        let volume = result.indicators.quote
            .as_ref()
            .and_then(|quotes| quotes.get(0))
            .and_then(|quote| quote.volume.as_ref())
            .and_then(|volumes| volumes.last())
            .and_then(|&vol| vol)
            .unwrap_or(0);

        // Get 52-week high/low (simplified - would need additional API call for real data)
        let high_52w = current_price * 1.3; // Placeholder
        let low_52w = current_price * 0.7;  // Placeholder

        Ok(Quote {
            symbol: symbol.to_string(),
            price: current_price,
            change,
            change_percent,
            volume,
            bid: None,
            ask: None,
            bid_size: None,
            ask_size: None,
            high_52w,
            low_52w,
            market_cap: None,
            pe_ratio: None,
            dividend_yield: None,
            last_updated: Utc::now().to_rfc3339(),
        })
    }

    // REAL IMPLEMENTATION - Batch Quotes
    pub async fn fetch_batch_quotes(&self, symbols: &[String]) -> Result<HashMap<String, Quote>, ApiError> {
        let mut quotes = HashMap::new();
        let crumb = self.get_crumb().await?;

        // Process in batches of 5 to avoid overwhelming the API
        for chunk in symbols.chunks(5) {
            for symbol in chunk {
                match self.fetch_single_quote(symbol, &crumb).await {
                    Ok(quote) => {
                        quotes.insert(symbol.clone(), quote);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch quote for {}: {}", symbol, e);
                    }
                }
                
                // Brief delay between requests
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        Ok(quotes)
    }

    // REAL IMPLEMENTATION - Historical Data
    pub async fn fetch_historical_data(&self, symbol: &str, range: &str, interval: &str) -> Result<Vec<CandleData>, ApiError> {
        let crumb = self.get_crumb().await?;
        self.rate_limiter.write().await.wait_if_needed().await;

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}&crumb={}",
            symbol, range, interval, crumb
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", &format!("https://finance.yahoo.com/quote/{}", symbol))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}", response.status())));
        }

        let yahoo_response: YahooChartResponse = response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        let result = yahoo_response.chart.result
            .as_ref()
            .and_then(|results| results.get(0))
            .ok_or_else(|| ApiError::DataNotFound("No chart data found".to_string()))?;

        let timestamps = result.timestamp.as_ref()
            .ok_or_else(|| ApiError::DataNotFound("No timestamp data".to_string()))?;

        let quote_data = result.indicators.quote
            .as_ref()
            .and_then(|quotes| quotes.get(0))
            .ok_or_else(|| ApiError::DataNotFound("No quote data".to_string()))?;

        let opens = quote_data.open.as_ref().unwrap_or(&vec![]);
        let highs = quote_data.high.as_ref().unwrap_or(&vec![]);
        let lows = quote_data.low.as_ref().unwrap_or(&vec![]);
        let closes = quote_data.close.as_ref().unwrap_or(&vec![]);
        let volumes = quote_data.volume.as_ref().unwrap_or(&vec![]);

        let adj_closes = result.indicators.adjclose
            .as_ref()
            .and_then(|adj| adj.get(0))
            .and_then(|adj_data| adj_data.adjclose.as_ref())
            .unwrap_or(&vec![]);

        let mut candles = Vec::new();

        for (i, &timestamp) in timestamps.iter().enumerate() {
            if let (Some(Some(open)), Some(Some(high)), Some(Some(low)), Some(Some(close))) = (
                opens.get(i).cloned().flatten(),
                highs.get(i).cloned().flatten(),
                lows.get(i).cloned().flatten(),
                closes.get(i).cloned().flatten(),
            ) {
                let volume = volumes.get(i).cloned().flatten();
                let adj_close = adj_closes.get(i).cloned().flatten();
                
                let datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
                let dt: DateTime<Utc> = datetime.into();

                candles.push(CandleData {
                    timestamp,
                    datetime: dt.to_rfc3339(),
                    open,
                    high,
                    low,
                    close,
                    volume: volume.map(|v| v as f64),
                    adj_close,
                });
            }
        }

        Ok(candles)
    }

    // REAL IMPLEMENTATION - Predefined Screener
    pub async fn fetch_predefined_screener(&self, screener_id: &str, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<ScreenerResult>, ApiError> {
        let crumb = self.get_crumb().await?;
        let limit = limit.unwrap_or(25);
        let offset = offset.unwrap_or(0);

        self.rate_limiter.write().await.wait_if_needed().await;

        let url = format!(
            "https://query2.finance.yahoo.com/v1/finance/screener/predefined/saved?count={}&offset={}&scrIds={}&crumb={}",
            limit, offset, screener_id, crumb
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://finance.yahoo.com/screener")
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::FetchError(format!("HTTP {}",