// Available Technical Indicators Configuration
export const AVAILABLE_INDICATORS = {
  // Moving Averages
  SMA: {
    name: 'Simple Moving Average',
    defaultParams: { period: 20 },
    paramTypes: { period: 'number' },
  },
  EMA: {
    name: 'Exponential Moving Average',
    defaultParams: { period: 20 },
    paramTypes: { period: 'number' },
  },
  WMA: {
    name: 'Weighted Moving Average',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  HMA: {
    name: 'Hull Moving Average',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  TEMA: {
    name: 'Triple EMA',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  DEMA: {
    name: 'Double EMA',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  KAMA: {
    name: 'Kaufman Adaptive MA',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  FRAMA: {
    name: 'Fractal Adaptive MA',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },

  // Oscillators
  RSI: {
    name: 'Relative Strength Index',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },
  STOCHASTIC: {
    name: 'Stochastic Oscillator',
    defaultParams: { k_period: 14, d_period: 3 },
    paramTypes: { k_period: 'number', d_period: 'number' },
  },
  CCI: {
    name: 'Commodity Channel Index',
    defaultParams: { period: 20 },
    paramTypes: { period: 'number' },
  },
  WILLIAMS_R: {
    name: 'Williams %R',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },
  MFI: {
    name: 'Money Flow Index',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },
  ULTIMATE_OSCILLATOR: {
    name: 'Ultimate Oscillator',
    defaultParams: { short_period: 7, mid_period: 14, long_period: 28 },
    paramTypes: {
      short_period: 'number',
      mid_period: 'number',
      long_period: 'number',
    },
  },
  DETRENDED_PRICE_OSC: {
    name: 'Detrended Price Oscillator',
    defaultParams: { period: 20 },
    paramTypes: { period: 'number' },
  },
  RATE_OF_CHANGE: {
    name: 'Rate of Change',
    defaultParams: { period: 12 },
    paramTypes: { period: 'number' },
  },
  MOMENTUM: {
    name: 'Momentum',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  TRIX: {
    name: 'TRIX',
    defaultParams: { period: 15 },
    paramTypes: { period: 'number' },
  },

  // Bands and Channels
  BOLLINGER_BANDS: {
    name: 'Bollinger Bands',
    defaultParams: { period: 20, k: 2.0 },
    paramTypes: { period: 'number', k: 'number' },
  },
  PERCENT_B: {
    name: 'Percent B',
    defaultParams: { period: 20, std_dev_mult: 2.0 },
    paramTypes: { period: 'number', std_dev_mult: 'number' },
  },

  // Trend Following
  MACD: {
    name: 'MACD',
    defaultParams: { fast_period: 12, slow_period: 26 },
    paramTypes: { fast_period: 'number', slow_period: 'number' },
  },
  ADX: {
    name: 'Average Directional Index',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },
  PARABOLIC_SAR: {
    name: 'Parabolic SAR',
    defaultParams: { step: 0.02, max_step: 0.2 },
    paramTypes: { step: 'number', max_step: 'number' },
  },
  CHANDELIER_EXIT: {
    name: 'Chandelier Exit',
    defaultParams: { period: 22, atr_multiplier: 3.0 },
    paramTypes: { period: 'number', atr_multiplier: 'number' },
  },
  SCHAFF_TREND_CYCLE: {
    name: 'Schaff Trend Cycle',
    defaultParams: {
      cycle_period: 10,
      fast_k: 23,
      fast_d: 50,
      short_period: 50,
      long_period: 50,
    },
    paramTypes: {
      cycle_period: 'number',
      fast_k: 'number',
      fast_d: 'number',
      short_period: 'number',
      long_period: 'number',
    },
  },

  // Volume Indicators
  VWAP: {
    name: 'Volume Weighted Average Price',
    defaultParams: {},
    paramTypes: {},
  },
  OBV: { name: 'On Balance Volume', defaultParams: {}, paramTypes: {} },
  CMF: {
    name: 'Chaikin Money Flow',
    defaultParams: { period: 20 },
    paramTypes: { period: 'number' },
  },
  FORCE_INDEX: {
    name: 'Force Index',
    defaultParams: { period: 13 },
    paramTypes: { period: 'number' },
  },
  EASE_OF_MOVEMENT: {
    name: 'Ease of Movement',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },
  ACCUM_DIST_LINE: {
    name: 'Accumulation/Distribution Line',
    defaultParams: {},
    paramTypes: {},
  },
  PRICE_VOLUME_TREND: {
    name: 'Price Volume Trend',
    defaultParams: {},
    paramTypes: {},
  },
  VOLUME_OSCILLATOR: {
    name: 'Volume Oscillator',
    defaultParams: { short_period: 14, long_period: 28 },
    paramTypes: { short_period: 'number', long_period: 'number' },
  },

  // Volatility
  ATR: {
    name: 'Average True Range',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },

  // Complex Indicators
  ICHIMOKU: {
    name: 'Ichimoku Cloud',
    defaultParams: {
      conversion_period: 9,
      base_period: 26,
      leading_span_b_period: 52,
      displacement: 26,
    },
    paramTypes: {
      conversion_period: 'number',
      base_period: 'number',
      leading_span_b_period: 'number',
      displacement: 'number',
    },
  },
  GMMA: {
    name: 'Guppy Multiple Moving Average',
    defaultParams: {
      short_periods: [3, 5, 8, 10, 12, 15],
      long_periods: [30, 35, 40, 45, 50, 60],
    },
    paramTypes: { short_periods: 'array', long_periods: 'array' },
  },
  FIBONACCI_RETRACEMENT: {
    name: 'Fibonacci Retracement',
    defaultParams: { period: 14 },
    paramTypes: { period: 'number' },
  },

  // Advanced
  KALMAN_FILTER: {
    name: 'Kalman Filter Smoother',
    defaultParams: { measurement_variance: 1.0, process_variance: 1.0 },
    paramTypes: { measurement_variance: 'number', process_variance: 'number' },
  },
  HEIKIN_ASHI_SLOPE: {
    name: 'Heikin Ashi Slope',
    defaultParams: { period: 10 },
    paramTypes: { period: 'number' },
  },
  Z_SCORE: {
    name: 'Z-Score',
    defaultParams: { period: 20 },
    paramTypes: { period: 'number' },
  },
};

// Indicator categories for better organization
export const INDICATOR_CATEGORIES = {
  'Moving Averages': [
    'SMA',
    'EMA',
    'WMA',
    'HMA',
    'TEMA',
    'DEMA',
    'KAMA',
    'FRAMA',
  ],
  Oscillators: [
    'RSI',
    'STOCHASTIC',
    'CCI',
    'WILLIAMS_R',
    'MFI',
    'ULTIMATE_OSCILLATOR',
    'DETRENDED_PRICE_OSC',
    'RATE_OF_CHANGE',
    'MOMENTUM',
    'TRIX',
  ],
  'Trend Following': [
    'MACD',
    'ADX',
    'PARABOLIC_SAR',
    'CHANDELIER_EXIT',
    'SCHAFF_TREND_CYCLE',
  ],
  Volume: [
    'VWAP',
    'OBV',
    'CMF',
    'FORCE_INDEX',
    'EASE_OF_MOVEMENT',
    'ACCUM_DIST_LINE',
    'PRICE_VOLUME_TREND',
    'VOLUME_OSCILLATOR',
  ],
  Volatility: ['ATR', 'BOLLINGER_BANDS', 'PERCENT_B'],
  Complex: ['ICHIMOKU', 'GMMA', 'FIBONACCI_RETRACEMENT'],
  Advanced: ['KALMAN_FILTER', 'HEIKIN_ASHI_SLOPE', 'Z_SCORE'],
};

export // Theme configuration
const theme = {
  colors: {
    primary: ['#00d4ff', '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#feca57'],
    background: '#0a0e1a',
    surface: '#151b2d',
    text: '#e4e7eb',
    textSecondary: '#9aa0ac',
    grid: '#1e2a3e',
    border: '#2c3e50',
    accent: '#00d4ff',
    success: '#2ed573',
    warning: '#ffa502',
    error: '#ff4757',
  },
  spacing: { xs: 4, sm: 8, md: 16, lg: 24 },
  borderRadius: 8,
};
