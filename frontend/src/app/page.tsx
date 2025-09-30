'use client';
import React, { useState, useEffect, useCallback } from 'react';
import { useAccumDist } from '@/hooks/use-wasm';
import {
  TrendingUp,
  TrendingDown,
  Activity,
  Settings,
  Download,
  Maximize2,
  RefreshCw,
  PlayCircle,
  PauseCircle,
  Eye,
  Search,
  Calendar1,
  Newspaper,
  InfoIcon,
  Inspect,
  BrainCircuit,
  BrainCog,
  ScreenShare,
  BarChart3,
  PieChart,
  Wallet,
  Globe,
  Filter,
  Star,
  AlertCircle,
  Calculator,
  Target,
  Briefcase,
  LineChart,
  DollarSign,
  Building,
  Coins,
} from 'lucide-react';
import { Calendar } from '@/components/ui/calendar';
import { StockChart } from '@/components/widgets/stock_chart';
import { ControlPanel } from '@/components/widgets/control_panel';
import { OptionsChain } from '@/components/widgets/options-chain';
import { OptionsPnLCalculator } from '@/components/widgets/options-pnl';
import { QuoteSummary } from '@/components/widgets/quote-summary';
import { NewsComponent } from '@/components/widgets/news';
import { FinancialReports } from '@/components/widgets/financial_reports';
import { formatDate } from '@/utils/formatters';
import { theme } from '@/data/throwaway';
import { api } from '@/lib/api';

// Market Screeners Component
const MarketScreeners = () => {
  const [selectedMarket, setSelectedMarket] = useState('nasdaq');
  const [screenerType, setScreenerType] = useState('most-active');

  const markets = [
    { id: 'dow', name: 'Dow Jones', icon: Building },
    { id: 'nasdaq', name: 'NASDAQ', icon: BarChart3 },
    { id: 'sp500', name: 'S&P 500', icon: TrendingUp },
    { id: 'russell', name: 'Russell 2000', icon: Activity },
    { id: 'ftse', name: 'FTSE 100', icon: Globe },
    { id: 'nikkei', name: 'Nikkei 225', icon: Globe },
    { id: 'crypto', name: 'Cryptocurrency', icon: Coins },
    { id: 'forex', name: 'Forex', icon: DollarSign },
  ];

  const screeners = [
    { id: 'most-active', name: 'Most Active', icon: Activity },
    { id: 'gainers', name: 'Biggest Gainers', icon: TrendingUp },
    { id: 'losers', name: 'Biggest Losers', icon: TrendingDown },
    { id: 'unusual-volume', name: 'Unusual Volume', icon: BarChart3 },
    { id: 'near-highs', name: 'Near 52W High', icon: Target },
    { id: 'near-lows', name: 'Near 52W Low', icon: AlertCircle },
    { id: 'high-iv', name: 'High IV Options', icon: BrainCog },
    { id: 'earnings', name: 'Earnings Today', icon: Calendar1 },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      {/* Market Selection */}
      <div>
        <h3
          style={{
            color: theme.colors.text,
            marginBottom: '16px',
            fontSize: '18px',
          }}
        >
          Markets & Exchanges
        </h3>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
            gap: '12px',
          }}
        >
          {markets.map((market) => {
            const Icon = market.icon;
            return (
              <button
                key={market.id}
                onClick={() => setSelectedMarket(market.id)}
                style={{
                  backgroundColor:
                    selectedMarket === market.id
                      ? theme.colors.accent
                      : theme.colors.surface,
                  color:
                    selectedMarket === market.id ? 'white' : theme.colors.text,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '16px',
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: '8px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
              >
                <Icon size={24} />
                <span style={{ fontSize: '14px', fontWeight: '500' }}>
                  {market.name}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Screener Selection */}
      <div>
        <h3
          style={{
            color: theme.colors.text,
            marginBottom: '16px',
            fontSize: '18px',
          }}
        >
          Screeners
        </h3>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
            gap: '12px',
          }}
        >
          {screeners.map((screener) => {
            const Icon = screener.icon;
            return (
              <button
                key={screener.id}
                onClick={() => setScreenerType(screener.id)}
                style={{
                  backgroundColor:
                    screenerType === screener.id
                      ? theme.colors.success
                      : theme.colors.surface,
                  color:
                    screenerType === screener.id ? 'white' : theme.colors.text,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '12px 16px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
              >
                <Icon size={20} />
                <span style={{ fontSize: '14px', fontWeight: '500' }}>
                  {screener.name}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Results Table Placeholder */}
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '24px',
        }}
      >
        <h4 style={{ color: theme.colors.text, marginBottom: '16px' }}>
          {screeners.find((s) => s.id === screenerType)?.name} -{' '}
          {markets.find((m) => m.id === selectedMarket)?.name}
        </h4>
        <div
          style={{
            color: theme.colors.textSecondary,
            textAlign: 'center',
            padding: '40px',
          }}
        >
          Screener results would appear here
          <br />
          <small>Connect to market data API to populate live results</small>
        </div>
      </div>
    </div>
  );
};

// Portfolio Management Component
const PortfolioManager = () => {
  const [activePortfolio, setActivePortfolio] = useState('main');
  const [portfolioView, setPortfolioView] = useState('holdings');

  const portfolios = [
    { id: 'main', name: 'Main Portfolio', value: 125420.5, change: 2340.12 },
    {
      id: 'retirement',
      name: 'Retirement Fund',
      value: 78900.25,
      change: -890.45,
    },
    { id: 'trading', name: 'Active Trading', value: 45200.8, change: 1250.33 },
    {
      id: 'simulation',
      name: 'Paper Trading',
      value: 100000.0,
      change: 3420.55,
    },
  ];

  const portfolioViews = [
    { id: 'holdings', name: 'Holdings', icon: Briefcase },
    { id: 'performance', name: 'Performance', icon: LineChart },
    { id: 'analysis', name: 'Risk Analysis', icon: Calculator },
    { id: 'options', name: 'Options Positions', icon: Target },
    { id: 'allocation', name: 'Asset Allocation', icon: PieChart },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      {/* Portfolio Selection */}
      <div>
        <h3
          style={{
            color: theme.colors.text,
            marginBottom: '16px',
            fontSize: '18px',
          }}
        >
          Portfolio Selection
        </h3>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: '12px',
          }}
        >
          {portfolios.map((portfolio) => (
            <button
              key={portfolio.id}
              onClick={() => setActivePortfolio(portfolio.id)}
              style={{
                backgroundColor:
                  activePortfolio === portfolio.id
                    ? theme.colors.accent
                    : theme.colors.surface,
                color:
                  activePortfolio === portfolio.id
                    ? 'white'
                    : theme.colors.text,
                border: `1px solid ${theme.colors.border}`,
                borderRadius: theme.borderRadius,
                padding: '16px',
                textAlign: 'left',
                cursor: 'pointer',
                transition: 'all 0.2s',
              }}
            >
              <div style={{ fontWeight: '600', marginBottom: '8px' }}>
                {portfolio.name}
              </div>
              <div
                style={{
                  fontSize: '20px',
                  fontWeight: '700',
                  marginBottom: '4px',
                }}
              >
                ${portfolio.value.toLocaleString()}
              </div>
              <div
                style={{
                  fontSize: '14px',
                  color:
                    portfolio.change >= 0
                      ? theme.colors.success
                      : theme.colors.error,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                }}
              >
                {portfolio.change >= 0 ? (
                  <TrendingUp size={14} />
                ) : (
                  <TrendingDown size={14} />
                )}
                {portfolio.change >= 0 ? '+' : ''}${portfolio.change.toFixed(2)}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Portfolio View Options */}
      <div>
        <h3
          style={{
            color: theme.colors.text,
            marginBottom: '16px',
            fontSize: '18px',
          }}
        >
          Analysis Views
        </h3>
        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
          {portfolioViews.map((view) => {
            const Icon = view.icon;
            return (
              <button
                key={view.id}
                onClick={() => setPortfolioView(view.id)}
                style={{
                  backgroundColor:
                    portfolioView === view.id
                      ? theme.colors.success
                      : theme.colors.surface,
                  color:
                    portfolioView === view.id ? 'white' : theme.colors.text,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '12px 16px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
              >
                <Icon size={16} />
                {view.name}
              </button>
            );
          })}
        </div>
      </div>

      {/* Portfolio Content */}
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '24px',
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: '24px',
          }}
        >
          <h4 style={{ color: theme.colors.text, margin: 0 }}>
            {portfolios.find((p) => p.id === activePortfolio)?.name} -{' '}
            {portfolioViews.find((v) => v.id === portfolioView)?.name}
          </h4>
          <div style={{ display: 'flex', gap: '12px' }}>
            <button
              style={{
                backgroundColor: theme.colors.accent,
                color: 'white',
                border: 'none',
                borderRadius: theme.borderRadius,
                padding: '8px 16px',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
              }}
            >
              <Calculator size={16} />
              Mathematical Analysis
            </button>
            <button
              style={{
                backgroundColor: theme.colors.surface,
                color: theme.colors.text,
                border: `1px solid ${theme.colors.border}`,
                borderRadius: theme.borderRadius,
                padding: '8px 16px',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
              }}
            >
              <Download size={16} />
              Export
            </button>
          </div>
        </div>

        {/* Content based on selected view */}
        {portfolioView === 'holdings' && (
          <div>
            <div
              style={{
                color: theme.colors.textSecondary,
                marginBottom: '16px',
              }}
            >
              Holdings overview with position sizes, P&L, and allocation
              percentages
            </div>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
                gap: '16px',
              }}
            >
              {['AAPL', 'TSLA', 'NVDA', 'SPY'].map((symbol) => (
                <div
                  key={symbol}
                  style={{
                    backgroundColor: theme.colors.background,
                    border: `1px solid ${theme.colors.border}`,
                    borderRadius: theme.borderRadius,
                    padding: '16px',
                  }}
                >
                  <div style={{ fontWeight: '600', marginBottom: '8px' }}>
                    {symbol}
                  </div>
                  <div
                    style={{
                      fontSize: '14px',
                      color: theme.colors.textSecondary,
                    }}
                  >
                    100 shares • $15,420 value
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {portfolioView === 'analysis' && (
          <div>
            <div
              style={{
                color: theme.colors.textSecondary,
                marginBottom: '16px',
              }}
            >
              Mathematical risk metrics: Sharpe ratio, beta, VaR, correlation
              analysis
            </div>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
                gap: '16px',
              }}
            >
              {[
                { metric: 'Sharpe Ratio', value: '1.45', status: 'good' },
                { metric: 'Portfolio Beta', value: '1.12', status: 'neutral' },
                { metric: 'Max Drawdown', value: '-8.3%', status: 'warning' },
                { metric: '95% VaR (1d)', value: '-$2,840', status: 'warning' },
                {
                  metric: 'Volatility (30d)',
                  value: '18.7%',
                  status: 'neutral',
                },
                {
                  metric: 'Correlation to SPY',
                  value: '0.89',
                  status: 'neutral',
                },
              ].map((item) => (
                <div
                  key={item.metric}
                  style={{
                    backgroundColor: theme.colors.background,
                    border: `1px solid ${theme.colors.border}`,
                    borderRadius: theme.borderRadius,
                    padding: '16px',
                  }}
                >
                  <div
                    style={{
                      fontSize: '14px',
                      color: theme.colors.textSecondary,
                      marginBottom: '4px',
                    }}
                  >
                    {item.metric}
                  </div>
                  <div
                    style={{
                      fontSize: '20px',
                      fontWeight: '700',
                      color:
                        item.status === 'good'
                          ? theme.colors.success
                          : item.status === 'warning'
                            ? theme.colors.error
                            : theme.colors.text,
                    }}
                  >
                    {item.value}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {portfolioView !== 'holdings' && portfolioView !== 'analysis' && (
          <div
            style={{
              color: theme.colors.textSecondary,
              textAlign: 'center',
              padding: '40px',
            }}
          >
            {portfolioViews.find((v) => v.id === portfolioView)?.name} view
            would appear here
            <br />
            <small>Advanced portfolio analytics and visualizations</small>
          </div>
        )}
      </div>
    </div>
  );
};

// Main Enhanced Trading Platform Component
const EnhancedTradingPlatform = () => {
  const [stockData, setStockData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [ticker, setTicker] = useState('AAPL');
  const [range, setRange] = useState('1mo');
  const [indicators, setIndicators] = useState([]);
  const [quotes, setQuotes] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [activeSection, setActiveSection] = useState('markets');
  const [activeTab, setActiveTab] = useState('chart');

  const today = new Date();
  const [date, setDate] = React.useState<Date | undefined>(today);

  const accumDist = useAccumDist();
  const [output, setOutput] = useState<number[]>([]);

  // Main navigation sections
  const mainSections = [
    {
      id: 'markets',
      name: 'Markets & Screeners',
      icon: Globe,
      description: 'Market overviews, screeners, and cross-market analysis',
    },
    {
      id: 'portfolios',
      name: 'Portfolio Management',
      icon: Briefcase,
      description: 'Portfolio tracking, analysis, and mathematical evaluation',
    },
    {
      id: 'analysis',
      name: 'Individual Analysis',
      icon: Activity,
      description: 'Deep dive analysis of individual securities',
    },
  ];

  const handleCompute = () => {
    if (accumDist && stockData?.candles) {
      const result = accumDist(stockData.candles);
      setOutput(result);
      console.log('AccumDistLine:', result);
    }
  };

  // Fetch historical data with indicators
  const fetchHistoricalData = useCallback(async () => {
    if (!ticker) return;

    setLoading(true);
    setError(null);

    try {
      // Prepare indicator configuration for the API request
      const indicatorConfigs = indicators
        .filter((ind) => ind.enabled)
        .map((ind) => ({
          name: ind.backendKey,
          params: ind.params,
        }));

      const response = await api.get('/historical', {
        params: {
          tickers: ticker,
          range: range,
          interval: range === '1d' ? '5m' : '1d',
          include_indicators: indicatorConfigs.length > 0,
          indicators: JSON.stringify(indicatorConfigs),
        },
      });

      console.log('API Response:', response);

      if (response && response.data && response.data[ticker]) {
        setStockData(response.data[ticker]);
      } else if (response && response[ticker]) {
        setStockData(response[ticker]);
      } else {
        setError('No data received for ticker');
        console.error('Unexpected response structure:', response);
      }
    } catch (err) {
      setError(err.message);
      console.error('Error fetching historical data:', err);
    } finally {
      setLoading(false);
    }
  }, [ticker, range, indicators]);

  // Fetch quotes
  const fetchQuotes = useCallback(async () => {
    if (!ticker) return;

    try {
      const response = await api.get('/quotes', {
        params: { tickers: ticker },
      });

      console.log('Quotes Response:', response);

      if (response && response.quotes) {
        setQuotes(response.quotes);
      } else if (response && response[ticker]) {
        setQuotes({ [ticker]: response[ticker] });
      } else {
        setQuotes(response);
      }
    } catch (err) {
      console.error('Error fetching quotes:', err);
    }
  }, [ticker]);

  // Auto-refresh functionality
  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      fetchQuotes();
    }, 30000);

    return () => clearInterval(interval);
  }, [autoRefresh, fetchQuotes]);

  // Initial data fetch for analysis section
  useEffect(() => {
    if (activeSection === 'analysis') {
      fetchHistoricalData();
      fetchQuotes();
    }
  }, [fetchHistoricalData, fetchQuotes, activeSection]);

  // Indicator management
  const addIndicator = (indicator) => {
    setIndicators((prev) => [...prev, indicator]);
    setTimeout(() => fetchHistoricalData(), 100);
  };

  const toggleIndicator = (id) => {
    setIndicators((prev) =>
      prev.map((ind) =>
        ind.id === id ? { ...ind, enabled: !ind.enabled } : ind
      )
    );
    setTimeout(() => fetchHistoricalData(), 100);
  };

  const removeIndicator = (id) => {
    setIndicators((prev) => prev.filter((ind) => ind.id !== id));
    setTimeout(() => fetchHistoricalData(), 100);
  };

  const updateIndicator = (id, updates) => {
    setIndicators((prev) =>
      prev.map((ind) => (ind.id === id ? { ...ind, ...updates } : ind))
    );
    setTimeout(() => fetchHistoricalData(), 100);
  };

  // Get current underlying price for options
  const getCurrentPrice = () => {
    if (quotes && quotes[ticker]) {
      return quotes[ticker].price || quotes[ticker].regularMarketPrice;
    }
    if (stockData && stockData.candles && stockData.candles.length > 0) {
      return stockData.candles[stockData.candles.length - 1].close;
    }
    return null;
  };

  const underlyingPrice = getCurrentPrice();

  return (
    <div
      style={{
        backgroundColor: theme.colors.background,
        minHeight: '100vh',
        padding: '24px',
      }}
    >
      {/* Header */}
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '24px',
          marginBottom: '24px',
        }}
      >
        <h1
          style={{
            color: theme.colors.text,
            fontSize: '28px',
            fontWeight: 700,
            margin: 0,
            marginBottom: '8px',
          }}
        >
          Yeast; Grow your Bread!
        </h1>
        <p
          style={{
            color: theme.colors.textSecondary,
            fontSize: '16px',
            margin: 0,
            marginBottom: '24px',
          }}
        >
          Comprehensive trading platform with market analysis, portfolio
          management, and mathematical rigor
        </p>

        {/* Main Section Navigation */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
            gap: '16px',
          }}
        >
          {mainSections.map((section) => {
            const Icon = section.icon;
            return (
              <button
                key={section.id}
                onClick={() => setActiveSection(section.id)}
                style={{
                  backgroundColor:
                    activeSection === section.id
                      ? theme.colors.accent
                      : theme.colors.surface,
                  color:
                    activeSection === section.id ? 'white' : theme.colors.text,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '20px',
                  textAlign: 'left',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                    marginBottom: '8px',
                  }}
                >
                  <Icon size={24} />
                  <span style={{ fontSize: '18px', fontWeight: '600' }}>
                    {section.name}
                  </span>
                </div>
                <p
                  style={{
                    margin: 0,
                    fontSize: '14px',
                    opacity: 0.8,
                    color:
                      activeSection === section.id
                        ? 'rgba(255,255,255,0.8)'
                        : theme.colors.textSecondary,
                  }}
                >
                  {section.description}
                </p>
              </button>
            );
          })}
        </div>
      </div>

      {/* Section Content */}
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '24px',
        }}
      >
        {activeSection === 'markets' && <MarketScreeners />}

        {activeSection === 'portfolios' && <PortfolioManager />}

        {activeSection === 'analysis' && (
          <>
            {/* Analysis Controls */}
            <div style={{ marginBottom: '24px' }}>
              <h3
                style={{
                  color: theme.colors.text,
                  marginBottom: '16px',
                  fontSize: '18px',
                }}
              >
                Individual Security Analysis
              </h3>

              <div
                style={{
                  display: 'flex',
                  gap: '16px',
                  alignItems: 'center',
                  flexWrap: 'wrap',
                  marginBottom: '20px',
                }}
              >
                <div>
                  <label
                    style={{
                      color: theme.colors.textSecondary,
                      fontSize: '12px',
                      display: 'block',
                      marginBottom: '4px',
                    }}
                  >
                    Stock Symbol
                  </label>
                  <input
                    type="text"
                    value={ticker}
                    onChange={(e) => setTicker(e.target.value.toUpperCase())}
                    style={{
                      backgroundColor: theme.colors.background,
                      color: theme.colors.text,
                      border: `1px solid ${theme.colors.border}`,
                      borderRadius: theme.borderRadius,
                      padding: '8px 12px',
                      fontSize: '14px',
                      width: '120px',
                    }}
                    placeholder="AAPL"
                  />
                </div>

                <div>
                  <label
                    style={{
                      color: theme.colors.textSecondary,
                      fontSize: '12px',
                      display: 'block',
                      marginBottom: '4px',
                    }}
                  >
                    Time Range
                  </label>
                  <select
                    value={range}
                    onChange={(e) => setRange(e.target.value)}
                    style={{
                      backgroundColor: theme.colors.background,
                      color: theme.colors.text,
                      border: `1px solid ${theme.colors.border}`,
                      borderRadius: theme.borderRadius,
                      padding: '8px 12px',
                      fontSize: '14px',
                    }}
                  >
                    <option value="1d">1 Day</option>
                    <option value="5d">5 Days</option>
                    <option value="1mo">1 Month</option>
                    <option value="3mo">3 Months</option>
                    <option value="6mo">6 Months</option>
                    <option value="1y">1 Year</option>
                    <option value="2y">2 Years</option>
                    <option value="5y">5 Years</option>
                  </select>
                </div>

                <button
                  onClick={fetchHistoricalData}
                  disabled={loading}
                  style={{
                    backgroundColor: theme.colors.accent,
                    color: 'white',
                    border: 'none',
                    borderRadius: theme.borderRadius,
                    padding: '8px 16px',
                    fontSize: '14px',
                    cursor: loading ? 'not-allowed' : 'pointer',
                    opacity: loading ? 0.6 : 1,
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                  }}
                >
                  <RefreshCw
                    size={16}
                    style={{
                      animation: loading ? 'spin 1s linear infinite' : 'none',
                    }}
                  />
                  {loading ? 'Loading...' : 'Fetch Data'}
                </button>

                <button
                  onClick={() => setAutoRefresh(!autoRefresh)}
                  style={{
                    backgroundColor: autoRefresh
                      ? theme.colors.success
                      : theme.colors.surface,
                    color: autoRefresh ? 'white' : theme.colors.text,
                    border: `1px solid ${theme.colors.border}`,
                    borderRadius: theme.borderRadius,
                    padding: '8px 16px',
                    fontSize: '14px',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                  }}
                >
                  {autoRefresh ? (
                    <PauseCircle size={16} />
                  ) : (
                    <PlayCircle size={16} />
                  )}
                  {autoRefresh ? 'Live' : 'Static'}
                </button>
              </div>

              {/* Quote display */}
              {quotes && quotes[ticker] && (
                <div
                  style={{
                    marginBottom: '16px',
                    display: 'flex',
                    gap: '24px',
                    alignItems: 'center',
                  }}
                >
                  <div
                    style={{
                      color: theme.colors.text,
                      fontSize: '24px',
                      fontWeight: 600,
                    }}
                  >
                    ${underlyingPrice?.toFixed(2) || 'N/A'}
                  </div>
                  <div
                    style={{
                      color:
                        (quotes[ticker].change ||
                          quotes[ticker].regularMarketChange ||
                          0) >= 0
                          ? theme.colors.success
                          : theme.colors.error,
                      fontSize: '16px',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '4px',
                    }}
                  >
                    {(quotes[ticker].change ||
                      quotes[ticker].regularMarketChange ||
                      0) >= 0 ? (
                      <TrendingUp size={16} />
                    ) : (
                      <TrendingDown size={16} />
                    )}
                    {(
                      quotes[ticker].change ||
                      quotes[ticker].regularMarketChange ||
                      0
                    )?.toFixed(2) || '0.00'}
                    (
                    {(
                      quotes[ticker].change_percent ||
                      quotes[ticker].regularMarketChangePercent ||
                      0
                    )?.toFixed(2) || '0.00'}
                    %)
                  </div>
                  {output.length > 0 && (
                    <div
                      style={{
                        backgroundColor: theme.colors.background,
                        border: `1px solid ${theme.colors.border}`,
                        borderRadius: theme.borderRadius,
                        padding: '12px',
                        fontSize: '12px',
                      }}
                    >
                      <div
                        style={{
                          color: theme.colors.textSecondary,
                          marginBottom: '4px',
                        }}
                      >
                        Accumulation/Distribution (latest 5 values):
                      </div>
                      <div
                        style={{
                          color: theme.colors.text,
                          fontFamily: 'monospace',
                        }}
                      >
                        {output
                          .slice(-5)
                          .map((val) => val.toFixed(2))
                          .join(', ')}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* Tab Navigation */}
              <div
                style={{
                  display: 'flex',
                  gap: '8px',
                  borderBottom: `1px solid ${theme.colors.border}`,
                }}
              >
                {[
                  { id: 'chart', label: 'Chart', icon: Activity },
                  { id: 'options', label: 'Options Chain', icon: Search },
                  { id: 'pnl', label: 'P&L Calculator', icon: TrendingUp },
                  { id: 'info', label: 'Info', icon: InfoIcon },
                  { id: 'news', label: 'News', icon: Newspaper },
                  { id: 'calendar', label: 'Calendar', icon: Calendar1 },
                  { id: 'reports', label: 'Reports', icon: Inspect },
                  { id: 'predictions', label: 'Predictions', icon: BrainCog },
                ].map((tab) => {
                  const Icon = tab.icon;
                  return (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      style={{
                        backgroundColor:
                          activeTab === tab.id
                            ? theme.colors.accent
                            : 'transparent',
                        color:
                          activeTab === tab.id ? 'white' : theme.colors.text,
                        border: 'none',
                        borderRadius: `${theme.borderRadius}px ${theme.borderRadius}px 0 0`,
                        padding: '12px 16px',
                        fontSize: '14px',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        borderBottom:
                          activeTab === tab.id
                            ? 'none'
                            : `1px solid transparent`,
                      }}
                    >
                      <Icon size={16} />
                      {tab.label}
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Tab Content */}
            <div style={{ marginTop: '24px' }}>
              {activeTab === 'chart' && (
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: '1fr 300px',
                    gap: '24px',
                  }}
                >
                  {/* Chart Area */}
                  <div>
                    {error && (
                      <div
                        style={{
                          backgroundColor: theme.colors.error,
                          color: 'white',
                          padding: '16px',
                          borderRadius: theme.borderRadius,
                          marginBottom: '16px',
                        }}
                      >
                        Error: {error}
                      </div>
                    )}

                    <StockChart
                      data={stockData}
                      indicators={indicators}
                      height={500}
                    />
                  </div>

                  {/* Control Panel */}
                  <ControlPanel
                    indicators={indicators}
                    onAddIndicator={addIndicator}
                    onToggleIndicator={toggleIndicator}
                    onRemoveIndicator={removeIndicator}
                    onUpdateIndicator={updateIndicator}
                  />
                </div>
              )}

              {activeTab === 'options' && (
                <div style={{ marginTop: '0' }}>
                  <OptionsChain
                    ticker={ticker}
                    underlyingPrice={underlyingPrice}
                  />
                </div>
              )}

              {activeTab === 'pnl' && (
                <div style={{ marginTop: '0' }}>
                  <OptionsPnLCalculator
                    ticker={ticker}
                    underlyingPrice={underlyingPrice}
                  />
                </div>
              )}

              {activeTab === 'info' && (
                <div style={{ marginTop: '0' }}>
                  <QuoteSummary ticker={ticker} />
                </div>
              )}

              {activeTab === 'news' && (
                <div style={{ marginTop: '0' }}>
                  <NewsComponent ticker={ticker} />
                </div>
              )}

              {activeTab === 'calendar' && (
                <div className="p-4">
                  <h2 className="text-lg font-semibold mb-2">Select a Date</h2>
                  <Calendar
                    mode="single"
                    selected={date} // highlights today
                    onSelect={setDate} // updates selected date
                    className="rounded-md border"
                  />
                </div>
              )}

              {activeTab === 'reports' && (
                <div style={{ marginTop: '0' }}>
                  <FinancialReports ticker={ticker} />
                </div>
              )}

              {activeTab === 'predictions' && (
                <div
                  style={{
                    backgroundColor: theme.colors.background,
                    border: `1px solid ${theme.colors.border}`,
                    borderRadius: theme.borderRadius,
                    padding: '24px',
                    textAlign: 'center',
                  }}
                >
                  <BrainCog
                    size={48}
                    style={{ color: theme.colors.accent, marginBottom: '16px' }}
                  />
                  <h4 style={{ color: theme.colors.text, marginBottom: '8px' }}>
                    AI-Powered Predictions
                  </h4>
                  <p style={{ color: theme.colors.textSecondary }}>
                    Machine learning models for price prediction, pattern
                    recognition, and risk assessment
                  </p>
                </div>
              )}
            </div>

            {/* Data Info - only show on chart tab */}
            {activeTab === 'chart' && stockData && stockData.candles && (
              <div
                style={{
                  marginTop: '24px',
                  backgroundColor: theme.colors.background,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '16px',
                }}
              >
                <h3 style={{ color: theme.colors.text, marginBottom: '8px' }}>
                  Data Summary
                </h3>
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
                    gap: '16px',
                    fontSize: '14px',
                  }}
                >
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Data Points:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      {stockData.candles.length}
                    </span>
                  </div>
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Latest Price:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      $
                      {stockData.candles[
                        stockData.candles.length - 1
                      ]?.close?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Date Range:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      {formatDate(stockData.candles[0]?.timestamp)} to{' '}
                      {formatDate(
                        stockData.candles[stockData.candles.length - 1]
                          ?.timestamp
                      )}
                    </span>
                  </div>
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Symbol:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      {stockData.symbol}
                    </span>
                  </div>
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Exchange:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      {stockData.meta?.exchange}
                    </span>
                  </div>
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Currency:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      {stockData.meta?.currency}
                    </span>
                  </div>
                  <div>
                    <span style={{ color: theme.colors.textSecondary }}>
                      Active Indicators:{' '}
                    </span>
                    <span style={{ color: theme.colors.text }}>
                      {indicators.filter((ind) => ind.enabled).length}
                    </span>
                  </div>
                </div>
              </div>
            )}
          </>
        )}
      </div>

      <style jsx>{`
        @keyframes spin {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </div>
  );
};

export default EnhancedTradingPlatform;
