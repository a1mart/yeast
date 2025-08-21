// options_math.rs
use std::f64::consts::E;
use std::thread;

#[derive(Debug, Clone, Copy)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug)]
pub struct OptionGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub price: f64,
}

/// Approximate the standard normal PDF
fn norm_pdf(x: f64) -> f64 {
    (1.0 / (2.0 * std::f64::consts::PI).sqrt()) * (-0.5 * x * x).exp()
}

/// Approximate the standard normal CDF (Abramowitz and Stegun formula 7.1.26)
fn norm_cdf(x: f64) -> f64 {
    let k = 1.0 / (1.0 + 0.2316419 * x.abs());
    let k_sum = k * (0.319381530 + k * (-0.356563782 + k * (1.781477937 + k * (-1.821255978 + 1.330274429 * k))));
    let cdf = 1.0 - norm_pdf(x) * k_sum;

    if x < 0.0 { 1.0 - cdf } else { cdf }
}

pub fn black_scholes_greeks(
    s: f64,      // underlying price
    k: f64,      // strike price
    t: f64,      // time to expiration in years
    r: f64,      // risk-free rate
    sigma: f64,  // volatility
    option_type: OptionType,
) -> OptionGreeks {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let price = match option_type {
        OptionType::Call => s * norm_cdf(d1) - k * E.powf(-r * t) * norm_cdf(d2),
        OptionType::Put => k * E.powf(-r * t) * norm_cdf(-d2) - s * norm_cdf(-d1),
    };

    let delta = match option_type {
        OptionType::Call => norm_cdf(d1),
        OptionType::Put => norm_cdf(d1) - 1.0,
    };

    let gamma = norm_pdf(d1) / (s * sigma * sqrt_t);

    let theta = match option_type {
        OptionType::Call => {
            -(s * norm_pdf(d1) * sigma) / (2.0 * sqrt_t)
            - r * k * E.powf(-r * t) * norm_cdf(d2)
        }
        OptionType::Put => {
            -(s * norm_pdf(d1) * sigma) / (2.0 * sqrt_t)
            + r * k * E.powf(-r * t) * norm_cdf(-d2)
        }
    };

    let vega = s * norm_pdf(d1) * sqrt_t;

    let rho = match option_type {
        OptionType::Call => k * t * E.powf(-r * t) * norm_cdf(d2),
        OptionType::Put => -k * t * E.powf(-r * t) * norm_cdf(-d2),
    };

    OptionGreeks {
        delta,
        gamma,
        theta,
        vega,
        rho,
        price,
    }
}

fn compute_greeks_parallel(
    underlying_price: f64,
    options: &[OptionData],
    time_to_expiry: f64,
    risk_free_rate: f64,
    volatility: f64,
) -> Vec<OptionGreeks> {
    let n_threads = 4; // number of threads you want, tune as needed
    let chunk_size = (options.len() + n_threads - 1) / n_threads; // ceiling division

    let mut handles = Vec::with_capacity(n_threads);

    for chunk in options.chunks(chunk_size) {
        let chunk = chunk.to_owned(); // clone chunk to move into thread
        let s = underlying_price;
        let t = time_to_expiry;
        let r = risk_free_rate;
        let sigma = volatility;

        let handle = thread::spawn(move || {
            chunk.into_iter().map(|opt| {
                black_scholes_greeks(
                    s,
                    opt.strike,
                    t,
                    r,
                    sigma,
                    opt.option_type,
                )
            }).collect::<Vec<_>>()
        });

        handles.push(handle);
    }

    // Collect results from all threads
    let mut results = Vec::with_capacity(options.len());
    for handle in handles {
        let res = handle.join().expect("Thread panicked");
        results.extend(res);
    }

    results
}

/// Simple PnL calculation: (new_price - old_price) * position_size
pub fn calculate_pnl(position_size: f64, old_price: f64, new_price: f64) -> f64 {
    (new_price - old_price) * position_size
}

#[derive(Debug, Clone)]
pub struct OptionData {
    pub strike: f64,
    pub option_type: OptionType,
    pub open_interest: u64,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
}
