# CORE including
- fetching historical stock data
- fetching stock options
- calculating technical indicators
- Black-Scholes analysis on options


| **Indicator**                       | **Category**   | **What It Measures / Purpose**                                                                                |
| ----------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------- |
| **accum\_dist\_line.rs**            | Volume         | Accumulation/Distribution Line – Measures the cumulative flow of money into/out of a security.                |
| **adx.rs**                          | Trend          | Average Directional Index – Measures the strength of a trend (not direction).                                 |
| **atr.rs**                          | Volatility     | Average True Range – Measures market volatility based on recent price ranges.                                 |
| **bollinger\_bands.rs**             | Volatility     | Bollinger Bands – Measures volatility and identifies overbought/oversold conditions using standard deviation. |
| **cci.rs**                          | Momentum       | Commodity Channel Index – Identifies cyclical trends and overbought/oversold conditions.                      |
| **chandelier\_exit.rs**             | Volatility     | Uses ATR to determine trailing stop-loss points for trade exits.                                              |
| **cmf.rs**                          | Volume         | Chaikin Money Flow – Measures buying and selling pressure based on price and volume.                          |
| **dema.rs**                         | Trend          | Double Exponential Moving Average – Reduces lag compared to traditional EMAs.                                 |
| **detrended\_price\_oscillator.rs** | Momentum       | Removes long-term trends to focus on short-term cycles.                                                       |
| **ease\_of\_movement.rs**           | Volume         | Ease of Movement – Shows the relationship between price and volume changes.                                   |
| **ema.rs**                          | Trend          | Exponential Moving Average – Weighted moving average giving more importance to recent prices.                 |
| **fibonacci\_retracement.rs**       | Support/Res    | Uses Fibonacci levels to identify potential support and resistance zones.                                     |
| **force\_index.rs**                 | Volume         | Combines price and volume to indicate buying/selling strength.                                                |
| **frama.rs**                        | Trend          | Fractal Adaptive Moving Average – Adjusts sensitivity based on market volatility.                             |
| **gmma.rs**                         | Trend          | Guppy Multiple Moving Averages – Identifies trend strength using short- and long-term MAs.                    |
| **heikin\_ashi\_slope.rs**          | Trend          | Smoothed candlestick indicator to identify trend direction more clearly.                                      |
| **hma.rs**                          | Trend          | Hull Moving Average – Reduces lag and improves responsiveness compared to EMA and SMA.                        |
| **ichimoku.rs**                     | Trend          | Ichimoku Cloud – Provides trend direction, momentum, and support/resistance.                                  |
| **kama.rs**                         | Trend          | Kaufman’s Adaptive Moving Average – Adapts to market volatility for smoothing.                                |
| **kalman\_filter\_smoother.rs**     | Trend          | Advanced smoothing algorithm to reduce noise in price data.                                                   |
| **macd.rs**                         | Momentum       | Moving Average Convergence Divergence – Measures momentum and trend changes.                                  |
| **mfi.rs**                          | Volume         | Money Flow Index – Volume-weighted RSI for identifying overbought/oversold conditions.                        |
| **momentum.rs**                     | Momentum       | Simple momentum indicator based on price differences over time.                                               |
| **obv.rs**                          | Volume         | On-Balance Volume – Measures cumulative buying and selling pressure.                                          |
| **parabolic\_sar.rs**               | Trend          | Parabolic Stop and Reverse – Provides trailing stops for trend following strategies.                          |
| **percent\_b.rs**                   | Volatility     | Measures price relative to Bollinger Bands (0 to 1 range).                                                    |
| **price\_volume\_trend.rs**         | Volume         | Combines price changes and volume to identify buying/selling pressure.                                        |
| **roc.rs**                          | Momentum       | Rate of Change – Measures speed of price movement.                                                            |
| **rsi.rs**                          | Momentum       | Relative Strength Index – Identifies overbought/oversold conditions.                                          |
| **schaff\_trend\_cycle.rs**         | Trend/Momentum | Combines MACD and cycles for trend timing signals.                                                            |
| **sma.rs**                          | Trend          | Simple Moving Average – Basic trend smoothing indicator.                                                      |
| **stochastic.rs**                   | Momentum       | Stochastic Oscillator – Compares closing price to price range to find momentum.                               |
| **tema.rs**                         | Trend          | Triple Exponential Moving Average – Reduces lag compared to EMA.                                              |
| **trix.rs**                         | Momentum       | Triple Smoothed EMA to show trend direction and momentum.                                                     |
| **ultimate\_oscillator.rs**         | Momentum       | Combines multiple timeframes to reduce false momentum signals.                                                |
| **volume\_oscillator.rs**           | Volume         | Compares two volume moving averages to measure volume trends.                                                 |
| **vwap.rs**                         | Volume/Price   | Volume Weighted Average Price – Shows average price weighted by volume.                                       |
| **williams\_r.rs**                  | Momentum       | Williams %R – Identifies overbought/oversold levels similar to Stochastic Oscillator.                         |
| **wma.rs**                          | Trend          | Weighted Moving Average – Similar to SMA but with weights favoring recent prices.                             |
| **z\_score.rs**                     | Statistical    | Measures how far the price is from its mean in standard deviations (useful for mean reversion).               |

# To run:
```bash
cargo run
```