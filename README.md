# yeast
Grow your bread. Stocks


# Running
```bash
cargo run --bin yeast -- --server & cd frontend && npm run dev

# kill backend process;
#lsof -i :8080
#kill -9 <PID>
```


## Notes
The Rust backend has grown messy as I experiment with its structure; eventually I'll clean it up and encapsulate a lot of the functionality covered in the Python notebooks and source code to produce a unifed backend. Backend currrently supports fetching historical data and options chains for individual stocks, as well as quote summaries, quotes, market summaries, news, calendar events, and reports; it will later be expanded to support portfolio analysis and management, machine learning models, strategy building and backtesting, market screening, and more.

![Frontend](https://raw.githubusercontent.com/a1mart/yeast/main/docs/assets/frontend.png)

## TODO
- Convert functions to use f64 instead of the custom types
- Bind wasm to run compute from static sites w/o server (only really worthwhile if the value is in visualizing the indicators rather than the mathematics... -> prioritize backtesting strategies using indicators as signals)
- Cache crumb... get crumb more reliably


### Market(s)
    - Stock list
    - Screeners (common like most active, ... or custom)

### Individual stock
    - Historical data
    - Options chain ... PNL
    - Info
    - News ... sentiment
    - Calendar (splits, dividends, reports)
    - Reports (earnings, cashflow, balance)
    - Analysts/experts

    - Technical indicators
    - *Strategies*

### Portfolio(s)
    - holdings of equities and options

