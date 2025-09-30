'use client';
import React, { useState, useEffect, useCallback } from 'react';
import { useAccumDist } from '@/hooks/use-wasm';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as ReTooltip,
  Legend,
  ResponsiveContainer,
  ComposedChart,
  ReferenceLine,
} from 'recharts';
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
  Plus,
  X,
  ExternalLink,
  Calendar as CalendarIcon,
} from 'lucide-react';

// Import shadcn/ui components
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Calendar } from '@/components/ui/calendar';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';

// API configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || '/api';

const api = {
  get: async (endpoint, options = {}) => {
    const url = new URL(`${API_BASE_URL}${endpoint}`, window.location.origin);

    if (options.params) {
      Object.keys(options.params).forEach((key) => {
        if (options.params[key] !== undefined && options.params[key] !== null) {
          url.searchParams.append(key, options.params[key]);
        }
      });
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  },

  post: async (endpoint, data, options = {}) => {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  },
};

// Available Technical Indicators Configuration
const AVAILABLE_INDICATORS = {
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
const INDICATOR_CATEGORIES = {
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

// Theme colors for indicators
const colors = {
  primary: ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899'],
};

// Utility function to format timestamp to readable date
const formatDate = (timestamp) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year:
      timestamp < Date.now() / 1000 - 365 * 24 * 60 * 60
        ? 'numeric'
        : undefined,
  });
};

// Stock Chart Component
const StockChart = ({ data, indicators = [], height = 400 }) => {
  const processedData = React.useMemo(() => {
    if (!data || !data.candles || data.candles.length === 0) return [];

    // Transform the API data structure to chart-friendly format
    let processed = data.candles.map((candle) => ({
      timestamp: candle.timestamp,
      date: formatDate(candle.timestamp),
      datetime: candle.datetime,
      open: candle.open,
      high: candle.high,
      low: candle.low,
      close: candle.close,
      volume: candle.volume || 0,
      adj_close: candle.adj_close,
    }));

    // Add indicator data from backend if available
    if (data.indicators) {
      indicators.forEach((indicator) => {
        if (!indicator.enabled) return;

        const indicatorKey = indicator.backendKey || indicator.id;
        if (data.indicators[indicatorKey]) {
          const indicatorData = data.indicators[indicatorKey];
          processed = processed.map((item, i) => ({
            ...item,
            [indicator.displayKey]: indicatorData[i],
          }));
        }
      });
    }

    return processed;
  }, [data, indicators]);

  if (!processedData || processedData.length === 0) {
    return (
      <Card className="h-96">
        <CardContent className="flex items-center justify-center h-full text-muted-foreground">
          No data available
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardContent className="p-6">
        <div style={{ height }}>
          <ResponsiveContainer width="100%" height="100%">
            <ComposedChart
              data={processedData}
              margin={{ top: 20, right: 30, left: 20, bottom: 20 }}
            >
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis
                dataKey="date"
                className="text-sm fill-muted-foreground"
                fontSize={12}
              />
              <YAxis className="text-sm fill-muted-foreground" fontSize={12} />
              <YAxis
                yAxisId="volume"
                orientation="right"
                className="text-sm fill-muted-foreground"
                fontSize={10}
              />
              <ReTooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--popover))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: 'var(--radius)',
                  color: 'hsl(var(--popover-foreground))',
                }}
                formatter={(value, name) => [
                  typeof value === 'number' ? value.toFixed(2) : value,
                  name,
                ]}
                labelFormatter={(label) => `Date: ${label}`}
              />

              {/* Volume bars */}
              <Bar
                dataKey="volume"
                fill="#ef4444"
                opacity={0.3}
                yAxisId="volume"
                name="Volume"
              />

              {/* Main price line */}
              <Line
                type="monotone"
                dataKey="close"
                stroke="#3b82f6"
                strokeWidth={2}
                dot={false}
                name="Close Price"
              />

              {/* Technical Indicators */}
              {indicators
                .map((indicator, index) => {
                  if (!indicator.enabled) return null;

                  // Handle different indicator types and their display
                  const lines = [];

                  // For most single-line indicators
                  if (indicator.displayKey && !indicator.isMultiLine) {
                    lines.push({
                      key: indicator.displayKey,
                      name: indicator.displayName,
                      color: indicator.color,
                      strokeWidth: indicator.type === 'VWAP' ? 2 : 1,
                    });
                  }

                  // For multi-line indicators like Bollinger Bands, MACD, etc.
                  if (indicator.isMultiLine && indicator.lines) {
                    indicator.lines.forEach((line) => {
                      lines.push({
                        key: line.key,
                        name: line.name,
                        color: line.color || indicator.color,
                        strokeWidth: line.strokeWidth || 1,
                        strokeDasharray: line.dashed ? '5 5' : undefined,
                      });
                    });
                  }

                  return lines.map((line) => (
                    <Line
                      key={`${indicator.id}-${line.key}`}
                      type="monotone"
                      dataKey={line.key}
                      stroke={line.color}
                      strokeWidth={line.strokeWidth}
                      strokeDasharray={line.strokeDasharray}
                      dot={false}
                      name={line.name}
                      connectNulls={false}
                    />
                  ));
                })
                .flat()}
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
};

// Control Panel Component
const ControlPanel = ({
  indicators,
  onAddIndicator,
  onToggleIndicator,
  onRemoveIndicator,
  onUpdateIndicator,
}) => {
  const [selectedCategory, setSelectedCategory] = useState('Moving Averages');
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedIndicatorType, setSelectedIndicatorType] = useState(null);
  const [tempParams, setTempParams] = useState({});

  const formatIndicatorName = (type, params) => {
    const config = AVAILABLE_INDICATORS[type];
    if (!config) return type;

    const paramStrings = Object.entries(params)
      .map(([key, value]) => {
        if (Array.isArray(value)) {
          return `${key}:[${value.join(',')}]`;
        }
        return `${key}:${value}`;
      })
      .filter(Boolean);

    return `${type}${paramStrings.length > 0 ? `(${paramStrings.join(',')})` : ''}`;
  };

  const generateBackendKey = (type, params) => {
    // Generate the key that matches your Rust backend format
    switch (type) {
      case 'SMA':
        return `SMA(${params.period || 20})`;
      case 'EMA':
        return `EMA(${params.period || 20})`;
      case 'RSI':
        return `RSI(${params.period || 14})`;
      case 'MACD':
        return `MACD(${params.fast_period || 12},${params.slow_period || 26})`;
      case 'BOLLINGER_BANDS':
        return `BollingerBands(${params.period || 20})`;
      case 'STOCHASTIC':
        return `Stochastic(${params.k_period || 14},${params.d_period || 3})`;
      case 'ULTIMATE_OSCILLATOR':
        return `UltimateOscillator(${params.short_period || 7},${params.mid_period || 14},${params.long_period || 28})`;
      case 'VOLUME_OSCILLATOR':
        return `VolumeOscillator(${params.short_period || 14},${params.long_period || 28})`;
      case 'CHANDELIER_EXIT':
        return `ChandelierExit(${params.period || 22}, ${params.atr_multiplier || 3.0})`;
      case 'PERCENT_B':
        return `PercentB(${params.period || 20}, ${params.std_dev_mult || 2.0})`;
      default:
        // For indicators without parameters or single parameter
        if (Object.keys(params).length === 0) {
          return type.charAt(0).toUpperCase() + type.slice(1).toLowerCase();
        }
        const firstParam = Object.values(params)[0];
        return `${type.charAt(0).toUpperCase() + type.slice(1).toLowerCase()}(${firstParam})`;
    }
  };

  const getIndicatorDisplayConfig = (type, params) => {
    const config = {
      isMultiLine: false,
      displayKey: type.toLowerCase(),
      displayName: formatIndicatorName(type, params),
      lines: [],
    };

    // Configure multi-line indicators
    switch (type) {
      case 'BOLLINGER_BANDS':
        config.isMultiLine = true;
        config.lines = [
          {
            key: 'bb_upper',
            name: 'BB Upper',
            color: config.color,
            dashed: true,
          },
          { key: 'bb_middle', name: 'BB Middle', color: config.color },
          {
            key: 'bb_lower',
            name: 'BB Lower',
            color: config.color,
            dashed: true,
          },
        ];
        break;
      case 'MACD':
        config.isMultiLine = true;
        config.lines = [
          { key: 'macd_line', name: 'MACD', color: config.color },
          {
            key: 'macd_signal',
            name: 'Signal',
            color: colors.primary[1],
          },
          {
            key: 'macd_histogram',
            name: 'Histogram',
            color: colors.primary[2],
          },
        ];
        break;
      case 'STOCHASTIC':
        config.isMultiLine = true;
        config.lines = [
          { key: 'stoch_k', name: '%K', color: config.color },
          { key: 'stoch_d', name: '%D', color: colors.primary[1] },
        ];
        break;
      case 'ICHIMOKU':
        config.isMultiLine = true;
        config.lines = [
          { key: 'tenkan_sen', name: 'Tenkan-sen', color: config.color },
          {
            key: 'kijun_sen',
            name: 'Kijun-sen',
            color: colors.primary[1],
          },
          {
            key: 'senkou_span_a',
            name: 'Senkou Span A',
            color: colors.primary[2],
            dashed: true,
          },
          {
            key: 'senkou_span_b',
            name: 'Senkou Span B',
            color: colors.primary[3],
            dashed: true,
          },
        ];
        break;
      default:
        config.displayKey = type.toLowerCase();
        config.displayName = formatIndicatorName(type, params);
    }

    return config;
  };

  const openParameterModal = (type) => {
    setSelectedIndicatorType(type);
    setTempParams({ ...AVAILABLE_INDICATORS[type].defaultParams });
    setIsDialogOpen(true);
  };

  const addIndicatorWithParams = () => {
    const type = selectedIndicatorType;
    const params = { ...tempParams };

    const baseColor = colors.primary[indicators.length % colors.primary.length];
    const displayConfig = getIndicatorDisplayConfig(type, params);

    const newIndicator = {
      id: `${type}_${Date.now()}`,
      type,
      params,
      backendKey: generateBackendKey(type, params),
      color: baseColor,
      enabled: true,
      ...displayConfig,
    };

    onAddIndicator(newIndicator);
    setIsDialogOpen(false);
    setSelectedIndicatorType(null);
    setTempParams({});
  };

  const renderParameterInputs = () => {
    if (!selectedIndicatorType) return null;

    const config = AVAILABLE_INDICATORS[selectedIndicatorType];

    return Object.entries(config.paramTypes).map(([paramKey, paramType]) => (
      <div key={paramKey} className="space-y-2">
        <Label className="text-sm capitalize">
          {paramKey.replace(/_/g, ' ')}
        </Label>
        {paramType === 'number' ? (
          <Input
            type="number"
            value={tempParams[paramKey] || ''}
            onChange={(e) =>
              setTempParams((prev) => ({
                ...prev,
                [paramKey]: parseFloat(e.target.value) || 0,
              }))
            }
            className="w-full"
          />
        ) : paramType === 'array' ? (
          <Input
            type="text"
            value={
              Array.isArray(tempParams[paramKey])
                ? tempParams[paramKey].join(',')
                : ''
            }
            onChange={(e) =>
              setTempParams((prev) => ({
                ...prev,
                [paramKey]: e.target.value
                  .split(',')
                  .map((v) => parseInt(v.trim()))
                  .filter((v) => !isNaN(v)),
              }))
            }
            placeholder="e.g., 3,5,8,10,12,15"
            className="w-full"
          />
        ) : null}
      </div>
    ));
  };

  return (
    <Card className="h-[500px]">
      <CardHeader>
        <CardTitle>Technical Indicators</CardTitle>
        <CardDescription>
          Add and configure technical indicators
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Category Selection */}
        <div className="space-y-2">
          <Label>Category</Label>
          <Select value={selectedCategory} onValueChange={setSelectedCategory}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {Object.keys(INDICATOR_CATEGORIES).map((category) => (
                <SelectItem key={category} value={category}>
                  {category}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Indicator Buttons */}
        <div className="grid grid-cols-2 gap-2">
          {INDICATOR_CATEGORIES[selectedCategory].map((type) => (
            <Dialog
              key={type}
              open={isDialogOpen && selectedIndicatorType === type}
              onOpenChange={(open) => {
                if (!open) {
                  setIsDialogOpen(false);
                  setSelectedIndicatorType(null);
                  setTempParams({});
                }
              }}
            >
              <DialogTrigger asChild>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => openParameterModal(type)}
                  className="text-xs"
                >
                  {type.replace(/_/g, ' ')}
                </Button>
              </DialogTrigger>
              <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                  <DialogTitle>
                    Configure {selectedIndicatorType?.replace(/_/g, ' ')}
                  </DialogTitle>
                  <DialogDescription>
                    Set the parameters for this indicator
                  </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">{renderParameterInputs()}</div>
                <DialogFooter>
                  <Button
                    variant="outline"
                    onClick={() => setIsDialogOpen(false)}
                  >
                    Cancel
                  </Button>
                  <Button onClick={addIndicatorWithParams}>
                    Add Indicator
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          ))}
        </div>

        {/* Active Indicators */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-sm font-medium">
              Active Indicators ({indicators.filter((i) => i.enabled).length})
            </Label>
          </div>

          <ScrollArea className="h-60">
            <div className="space-y-2">
              {indicators.map((indicator) => (
                <div
                  key={indicator.id}
                  className="flex items-center justify-between p-3 border rounded-lg"
                >
                  <div className="flex items-center space-x-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: indicator.color }}
                    />
                    <span className="text-xs font-medium">
                      {indicator.displayName || indicator.type}
                    </span>
                  </div>
                  <div className="flex items-center space-x-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onToggleIndicator(indicator.id)}
                    >
                      <Eye
                        className={`h-3 w-3 ${
                          indicator.enabled ? 'opacity-100' : 'opacity-50'
                        }`}
                      />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onRemoveIndicator(indicator.id)}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>
        </div>
      </CardContent>
    </Card>
  );
};

// Options Chain Component
const OptionsChain = ({ ticker, underlyingPrice }) => {
  const [optionsData, setOptionsData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [selectedExpiry, setSelectedExpiry] = useState('');
  const [includeGreeks, setIncludeGreeks] = useState(true);
  const [optionType, setOptionType] = useState('both');
  const [minStrike, setMinStrike] = useState('');
  const [maxStrike, setMaxStrike] = useState('');

  const fetchOptionsChain = async () => {
    if (!ticker) return;

    setLoading(true);
    try {
      const params = {
        ticker,
        include_greeks: includeGreeks,
        option_type: optionType,
      };

      if (selectedExpiry) params.expiration_dates = [selectedExpiry];
      if (minStrike) params.min_strike = parseFloat(minStrike);
      if (maxStrike) params.max_strike = parseFloat(maxStrike);

      const response = await api.get('/options', { params });
      setOptionsData(response);
      if (
        response.expirations &&
        Object.keys(response.expirations).length > 0 &&
        !selectedExpiry
      ) {
        setSelectedExpiry(Object.keys(response.expirations)[0]);
      }
    } catch (err) {
      console.error('Error fetching options chain:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchOptionsChain();
  }, [ticker, includeGreeks, optionType]);

  const getMoneyness = (strike, underlyingPrice, optionType) => {
    if (!underlyingPrice) return 'atm';

    const diff =
      optionType === 'call'
        ? strike - underlyingPrice
        : underlyingPrice - strike;
    const pct = Math.abs(diff) / underlyingPrice;

    if (diff < -underlyingPrice * 0.05) return 'deep-itm';
    if (diff < 0) return 'itm';
    if (pct < 0.02) return 'atm';
    if (diff < underlyingPrice * 0.05) return 'otm';
    return 'deep-otm';
  };

  const getMoneynessColor = (moneyness, isCall) => {
    const colors = {
      'deep-itm': isCall
        ? 'bg-green-100 dark:bg-green-900'
        : 'bg-red-100 dark:bg-red-900',
      itm: isCall
        ? 'bg-green-50 dark:bg-green-900/50'
        : 'bg-red-50 dark:bg-red-900/50',
      atm: 'bg-orange-50 dark:bg-orange-900/50',
      otm: 'bg-purple-50 dark:bg-purple-900/50',
      'deep-otm': 'bg-blue-50 dark:bg-blue-900/50',
    };
    return colors[moneyness] || '';
  };

  const isNearMoney = (strike, underlyingPrice) => {
    if (!underlyingPrice) return false;
    return Math.abs(strike - underlyingPrice) < underlyingPrice * 0.02;
  };

  const renderOptionsTable = (contracts, type) => (
    <Card className="mb-4">
      <CardHeader>
        <CardTitle className="text-lg">{type.toUpperCase()}S</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Strike</TableHead>
                <TableHead>Bid</TableHead>
                <TableHead>Ask</TableHead>
                <TableHead>Last</TableHead>
                <TableHead>Volume</TableHead>
                <TableHead>OI</TableHead>
                {includeGreeks && (
                  <>
                    <TableHead>Delta</TableHead>
                    <TableHead>Gamma</TableHead>
                    <TableHead>Theta</TableHead>
                    <TableHead>Vega</TableHead>
                    <TableHead>IV</TableHead>
                  </>
                )}
              </TableRow>
            </TableHeader>
            <TableBody>
              {contracts.map((contract, idx) => {
                const moneyness = getMoneyness(
                  contract.strike,
                  underlyingPrice,
                  type
                );
                const rowBgColor = getMoneynessColor(
                  moneyness,
                  type === 'call'
                );
                const isAtm = isNearMoney(contract.strike, underlyingPrice);

                return (
                  <TableRow
                    key={idx}
                    className={`${isAtm ? 'bg-orange-100 dark:bg-orange-900/50 border-orange-400' : rowBgColor}`}
                  >
                    <TableCell
                      className={`font-medium ${isAtm ? 'bg-orange-500 text-white' : ''}`}
                    >
                      ${contract.strike}
                    </TableCell>
                    <TableCell>${contract.bid.toFixed(2)}</TableCell>
                    <TableCell>${contract.ask.toFixed(2)}</TableCell>
                    <TableCell
                      className={
                        isAtm ? 'bg-orange-500 text-white font-bold' : ''
                      }
                    >
                      ${contract.last.toFixed(2)}
                    </TableCell>
                    <TableCell
                      className={
                        contract.volume > 5000 ? 'font-bold text-green-600' : ''
                      }
                    >
                      {contract.volume.toLocaleString()}
                    </TableCell>
                    <TableCell
                      className={
                        contract.open_interest > 10000
                          ? 'font-bold text-purple-600'
                          : ''
                      }
                    >
                      {contract.open_interest.toLocaleString()}
                    </TableCell>

                    {includeGreeks && contract.greeks && (
                      <>
                        <TableCell
                          className={
                            Math.abs(contract.greeks.delta) > 0.7
                              ? 'font-bold'
                              : ''
                          }
                        >
                          {contract.greeks.delta.toFixed(3)}
                        </TableCell>
                        <TableCell>
                          {contract.greeks.gamma.toFixed(4)}
                        </TableCell>
                        <TableCell>
                          {contract.greeks.theta.toFixed(3)}
                        </TableCell>
                        <TableCell>{contract.greeks.vega.toFixed(3)}</TableCell>
                        <TableCell
                          className={
                            contract.implied_volatility > 1.2
                              ? 'font-bold text-red-600'
                              : ''
                          }
                        >
                          {(contract.implied_volatility * 100).toFixed(1)}%
                        </TableCell>
                      </>
                    )}
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>

        {/* Legend */}
        <div className="mt-4 grid grid-cols-2 md:grid-cols-3 gap-2 text-xs text-muted-foreground">
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-green-600"></div>
            <span>High Volume</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-purple-600"></div>
            <span>High Open Interest</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-orange-500"></div>
            <span>At-the-Money</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );

  return (
    <Card className="h-[600px]">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Options Chain</CardTitle>
          <Button onClick={fetchOptionsChain} disabled={loading} size="sm">
            <RefreshCw
              className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`}
            />
            {loading ? 'Loading...' : 'Refresh'}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label className="text-sm">Expiration Date</Label>
            <Select value={selectedExpiry} onValueChange={setSelectedExpiry}>
              <SelectTrigger>
                <SelectValue placeholder="Select expiry" />
              </SelectTrigger>
              <SelectContent>
                {optionsData?.expirations &&
                  Object.keys(optionsData.expirations).map((date) => (
                    <SelectItem key={date} value={date}>
                      {date} (
                      {optionsData.expirations[date].days_to_expiry.toFixed(0)}{' '}
                      days)
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label className="text-sm">Option Type</Label>
            <Select value={optionType} onValueChange={setOptionType}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="both">Both</SelectItem>
                <SelectItem value="call">Calls Only</SelectItem>
                <SelectItem value="put">Puts Only</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label className="text-sm">Min Strike</Label>
            <Input
              type="number"
              value={minStrike}
              onChange={(e) => setMinStrike(e.target.value)}
              placeholder="Min"
            />
          </div>

          <div className="space-y-2">
            <Label className="text-sm">Max Strike</Label>
            <Input
              type="number"
              value={maxStrike}
              onChange={(e) => setMaxStrike(e.target.value)}
              placeholder="Max"
            />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <Checkbox
            id="greeks"
            checked={includeGreeks}
            onCheckedChange={setIncludeGreeks}
          />
          <Label htmlFor="greeks" className="text-sm">
            Include Greeks
          </Label>
        </div>

        <ScrollArea className="h-96">
          {/* Options Data */}
          {optionsData &&
            selectedExpiry &&
            optionsData.expirations[selectedExpiry] && (
              <div>
                <div className="mb-4 p-4 bg-muted rounded-lg">
                  <div className="font-bold text-lg">
                    {ticker} @ $
                    {underlyingPrice?.toFixed(2) ||
                      optionsData.underlying_price?.toFixed(2)}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {selectedExpiry} •{' '}
                    {optionsData.expirations[
                      selectedExpiry
                    ].days_to_expiry.toFixed(0)}{' '}
                    days to expiry
                  </div>
                </div>

                {(optionType === 'both' || optionType === 'call') &&
                  renderOptionsTable(
                    optionsData.expirations[selectedExpiry].calls,
                    'call'
                  )}

                {(optionType === 'both' || optionType === 'put') &&
                  renderOptionsTable(
                    optionsData.expirations[selectedExpiry].puts,
                    'put'
                  )}
              </div>
            )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
};

// Options P&L Calculator Component
const OptionsPnLCalculator = ({ ticker, underlyingPrice }) => {
  const [positions, setPositions] = useState([]);
  const [pnlAnalysis, setPnlAnalysis] = useState(null);
  const [loading, setLoading] = useState(false);
  const [priceRange, setPriceRange] = useState({ min: '', max: '' });

  const addPosition = () => {
    setPositions((prev) => [
      ...prev,
      {
        id: Date.now(),
        option_type: 'call',
        strike: underlyingPrice || 100,
        quantity: 1,
        entry_price: 5.0,
        days_to_expiry: 30,
      },
    ]);
  };

  const updatePosition = (id, field, value) => {
    setPositions((prev) =>
      prev.map((pos) =>
        pos.id === id
          ? {
              ...pos,
              [field]:
                field === 'option_type'
                  ? value
                  : field === 'quantity'
                    ? parseInt(value) || 0
                    : parseFloat(value) || 0,
            }
          : pos
      )
    );
  };

  const removePosition = (id) => {
    setPositions((prev) => prev.filter((pos) => pos.id !== id));
  };

  const calculatePnL = async () => {
    if (positions.length === 0) return;

    setLoading(true);
    try {
      const basePrice = underlyingPrice || 100;
      const min = parseFloat(priceRange.min) || basePrice * 0.7;
      const max = parseFloat(priceRange.max) || basePrice * 1.3;
      const underlyingPrices = [];

      for (let i = 0; i <= 100; i++) {
        underlyingPrices.push(min + (max - min) * (i / 100));
      }

      const request = {
        positions: positions.map((pos) => ({
          option_type: pos.option_type,
          strike: pos.strike,
          quantity: pos.quantity,
          entry_price: pos.entry_price,
          days_to_expiry: pos.days_to_expiry,
        })),
        underlying_prices: underlyingPrices,
        volatility: 0.25,
        risk_free_rate: 0.01,
      };

      const response = await api.post('/options/pnl', request);
      setPnlAnalysis(response);
    } catch (err) {
      console.error('Error calculating P&L:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="h-[600px]">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>P&L Calculator</CardTitle>
          <Button onClick={addPosition} size="sm">
            <Plus className="w-4 h-4 mr-2" />
            Add Position
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Price Range */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label className="text-sm">Min Price</Label>
            <Input
              type="number"
              value={priceRange.min}
              onChange={(e) =>
                setPriceRange((prev) => ({ ...prev, min: e.target.value }))
              }
              placeholder={(underlyingPrice * 0.7).toFixed(0)}
            />
          </div>
          <div className="space-y-2">
            <Label className="text-sm">Max Price</Label>
            <Input
              type="number"
              value={priceRange.max}
              onChange={(e) =>
                setPriceRange((prev) => ({ ...prev, max: e.target.value }))
              }
              placeholder={(underlyingPrice * 1.3).toFixed(0)}
            />
          </div>
        </div>

        {/* Positions */}
        <div className="space-y-2">
          <Label className="text-sm font-medium">Positions</Label>
          <ScrollArea className="h-40">
            <div className="space-y-2">
              {positions.map((position) => (
                <Card key={position.id} className="p-3">
                  <div className="grid grid-cols-6 gap-2 items-center">
                    <div className="space-y-1">
                      <Label className="text-xs">Option</Label>
                      <Select
                        value={position.option_type}
                        onValueChange={(value) =>
                          updatePosition(position.id, 'option_type', value)
                        }
                      >
                        <SelectTrigger className="h-8">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="call">Call</SelectItem>
                          <SelectItem value="put">Put</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1">
                      <Label className="text-xs">Strike</Label>
                      <Input
                        type="number"
                        value={position.strike}
                        onChange={(e) =>
                          updatePosition(position.id, 'strike', e.target.value)
                        }
                        className="h-8"
                      />
                    </div>

                    <div className="space-y-1">
                      <Label className="text-xs">Qty</Label>
                      <Input
                        type="number"
                        value={position.quantity}
                        onChange={(e) =>
                          updatePosition(
                            position.id,
                            'quantity',
                            e.target.value
                          )
                        }
                        className="h-8"
                      />
                    </div>

                    <div className="space-y-1">
                      <Label className="text-xs">Price</Label>
                      <Input
                        type="number"
                        step="0.01"
                        value={position.entry_price}
                        onChange={(e) =>
                          updatePosition(
                            position.id,
                            'entry_price',
                            e.target.value
                          )
                        }
                        className="h-8"
                      />
                    </div>

                    <div className="space-y-1">
                      <Label className="text-xs">DTE</Label>
                      <Input
                        type="number"
                        value={position.days_to_expiry}
                        onChange={(e) =>
                          updatePosition(
                            position.id,
                            'days_to_expiry',
                            e.target.value
                          )
                        }
                        className="h-8"
                      />
                    </div>

                    <div className="flex justify-end items-end h-full">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => removePosition(position.id)}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          </ScrollArea>
        </div>

        <Button
          onClick={calculatePnL}
          disabled={loading || positions.length === 0}
          className="w-full"
        >
          {loading ? (
            <>
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              Calculating...
            </>
          ) : (
            'Calculate P&L'
          )}
        </Button>

        {/* P&L Chart */}
        {pnlAnalysis && (
          <div className="space-y-4">
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={pnlAnalysis.portfolio.total_pnl_curve}>
                  <CartesianGrid
                    strokeDasharray="3 3"
                    className="stroke-muted"
                  />
                  <XAxis
                    dataKey="underlying_price"
                    className="text-sm fill-muted-foreground"
                    fontSize={10}
                    tickFormatter={(value) => `${value.toFixed(0)}`}
                  />
                  <YAxis
                    className="text-sm fill-muted-foreground"
                    fontSize={10}
                    tickFormatter={(value) => `${value.toFixed(0)}`}
                  />
                  <ReTooltip
                    contentStyle={{
                      backgroundColor: 'hsl(var(--popover))',
                      border: '1px solid hsl(var(--border))',
                      borderRadius: 'var(--radius)',
                      color: 'hsl(var(--popover-foreground))',
                    }}
                    formatter={(value) => [`${value.toFixed(2)}`, 'P&L']}
                    labelFormatter={(value) => `Price: ${value.toFixed(2)}`}
                  />
                  <ReferenceLine
                    y={0}
                    stroke="hsl(var(--muted-foreground))"
                    strokeDasharray="2 2"
                  />
                  <Line
                    type="monotone"
                    dataKey="pnl"
                    stroke="#3b82f6"
                    strokeWidth={2}
                    dot={false}
                    name="Portfolio P&L"
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>

            {/* Portfolio Summary */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Portfolio Summary</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Max Profit:</span>
                    <span className="text-green-600 font-medium">
                      {pnlAnalysis.portfolio.max_profit
                        ? `${pnlAnalysis.portfolio.max_profit.toFixed(2)}`
                        : 'Unlimited'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Max Loss:</span>
                    <span className="text-red-600 font-medium">
                      {pnlAnalysis.portfolio.max_loss
                        ? `${pnlAnalysis.portfolio.max_loss.toFixed(2)}`
                        : 'Unlimited'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Break-even:</span>
                    <span className="font-medium">
                      {pnlAnalysis.portfolio.break_even_points
                        .map((p) => `${p.toFixed(2)}`)
                        .join(', ')}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Net Delta:</span>
                    <span className="font-medium">
                      {pnlAnalysis.portfolio.total_greeks.delta.toFixed(3)}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </CardContent>
    </Card>
  );
};

// Quote Summary/Info Component
const QuoteSummary = ({ ticker }) => {
  const [summaryData, setSummaryData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchQuoteSummary = async () => {
    if (!ticker) return;

    setLoading(true);
    setError(null);
    try {
      const response = await api.get('/quotesummary', {
        params: { ticker },
      });
      setSummaryData(response);
    } catch (err) {
      setError(err.message);
      console.error('Error fetching quote summary:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchQuoteSummary();
  }, [ticker]);

  const formatCurrency = (value) => {
    if (!value) return 'N/A';
    if (value >= 1e12) return `${(value / 1e12).toFixed(2)}T`;
    if (value >= 1e9) return `${(value / 1e9).toFixed(2)}B`;
    if (value >= 1e6) return `${(value / 1e6).toFixed(2)}M`;
    if (value >= 1e3) return `${(value / 1e3).toFixed(2)}K`;
    return `${value.toFixed(2)}`;
  };

  const formatPercent = (value) => {
    if (value === null || value === undefined) return 'N/A';
    return `${(value * 100).toFixed(2)}%`;
  };

  const InfoCard = ({ title, children }) => (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{title}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">{children}</CardContent>
    </Card>
  );

  const DataRow = ({ label, value, isGood, isBad }) => (
    <div className="flex justify-between items-center py-1 text-sm border-b border-border last:border-b-0">
      <span className="text-muted-foreground">{label}</span>
      <span
        className={`font-medium ${
          isGood ? 'text-green-600' : isBad ? 'text-red-600' : 'text-foreground'
        }`}
      >
        {value}
      </span>
    </div>
  );

  if (loading) {
    return (
      <Card className="h-[600px]">
        <CardContent className="flex items-center justify-center h-full">
          <div className="text-center space-y-2">
            <RefreshCw className="w-8 h-8 animate-spin mx-auto text-muted-foreground" />
            <p className="text-muted-foreground">
              Loading company information...
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="h-[600px]">
        <CardContent className="p-6">
          <Alert>
            <AlertDescription>Error loading data: {error}</AlertDescription>
          </Alert>
          <Button onClick={fetchQuoteSummary} className="mt-4">
            <RefreshCw className="w-4 h-4 mr-2" />
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  if (!summaryData) {
    return (
      <Card className="h-[600px]">
        <CardContent className="flex items-center justify-center h-full text-muted-foreground">
          No data available for {ticker}
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="h-[600px]">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Company Information - {ticker}</CardTitle>
          <Button onClick={fetchQuoteSummary} size="sm" variant="outline">
            <RefreshCw className="w-4 h-4 mr-2" />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[500px]">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Company Profile */}
            {summaryData.asset_profile && (
              <InfoCard title="Company Profile">
                <DataRow
                  label="Sector"
                  value={summaryData.asset_profile.sector}
                />
                <DataRow
                  label="Industry"
                  value={summaryData.asset_profile.industry}
                />
                <DataRow
                  label="Employees"
                  value={summaryData.asset_profile.full_time_employees?.toLocaleString()}
                />
                <DataRow
                  label="Country"
                  value={summaryData.asset_profile.country}
                />
                {summaryData.asset_profile.website && (
                  <div className="mt-2">
                    <a
                      href={summaryData.asset_profile.website}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary hover:underline text-sm flex items-center"
                    >
                      Visit Website <ExternalLink className="w-3 h-3 ml-1" />
                    </a>
                  </div>
                )}
                {summaryData.asset_profile.long_business_summary && (
                  <div className="mt-3">
                    <p className="text-sm text-muted-foreground leading-relaxed">
                      {summaryData.asset_profile.long_business_summary.substring(
                        0,
                        300
                      )}
                      {summaryData.asset_profile.long_business_summary.length >
                        300 && '...'}
                    </p>
                  </div>
                )}
              </InfoCard>
            )}

            {/* Key Statistics */}
            {summaryData.default_key_statistics && (
              <InfoCard title="Key Statistics">
                <DataRow
                  label="Forward P/E"
                  value={summaryData.default_key_statistics.forward_pe?.toFixed(
                    2
                  )}
                />
                <DataRow
                  label="Trailing P/E"
                  value={summaryData.default_key_statistics.trailing_pe?.toFixed(
                    2
                  )}
                />
                <DataRow
                  label="PEG Ratio"
                  value={summaryData.default_key_statistics.peg_ratio?.toFixed(
                    2
                  )}
                  isGood={
                    summaryData.default_key_statistics.peg_ratio &&
                    summaryData.default_key_statistics.peg_ratio < 1
                  }
                  isBad={
                    summaryData.default_key_statistics.peg_ratio &&
                    summaryData.default_key_statistics.peg_ratio > 2
                  }
                />
                <DataRow
                  label="Beta"
                  value={summaryData.default_key_statistics.beta?.toFixed(2)}
                  isGood={
                    summaryData.default_key_statistics.beta &&
                    summaryData.default_key_statistics.beta < 1
                  }
                  isBad={
                    summaryData.default_key_statistics.beta &&
                    summaryData.default_key_statistics.beta > 1.5
                  }
                />
                <DataRow
                  label="Book Value"
                  value={formatCurrency(
                    summaryData.default_key_statistics.book_value
                  )}
                />
                <DataRow
                  label="Shares Outstanding"
                  value={
                    summaryData.default_key_statistics.shares_outstanding
                      ? `${(summaryData.default_key_statistics.shares_outstanding / 1e6).toFixed(0)}M`
                      : 'N/A'
                  }
                />
              </InfoCard>
            )}

            {/* Financial Data */}
            {summaryData.financial_data && (
              <InfoCard title="Financial Health">
                <DataRow
                  label="Current Price"
                  value={formatCurrency(
                    summaryData.financial_data.current_price
                  )}
                />
                <DataRow
                  label="Target Price"
                  value={formatCurrency(
                    summaryData.financial_data.target_mean_price
                  )}
                />
                <DataRow
                  label="Recommendation"
                  value={summaryData.financial_data.recommendation_key}
                  isGood={
                    summaryData.financial_data.recommendation_key === 'buy' ||
                    summaryData.financial_data.recommendation_key ===
                      'strong_buy'
                  }
                  isBad={
                    summaryData.financial_data.recommendation_key === 'sell' ||
                    summaryData.financial_data.recommendation_key ===
                      'strong_sell'
                  }
                />
                <DataRow
                  label="Total Cash"
                  value={formatCurrency(summaryData.financial_data.total_cash)}
                />
                <DataRow
                  label="Total Debt"
                  value={formatCurrency(summaryData.financial_data.total_debt)}
                />
                <DataRow
                  label="Current Ratio"
                  value={summaryData.financial_data.current_ratio?.toFixed(2)}
                  isGood={
                    summaryData.financial_data.current_ratio &&
                    summaryData.financial_data.current_ratio > 1.5
                  }
                  isBad={
                    summaryData.financial_data.current_ratio &&
                    summaryData.financial_data.current_ratio < 1
                  }
                />
                <DataRow
                  label="ROE"
                  value={formatPercent(
                    summaryData.financial_data.return_on_equity
                  )}
                  isGood={
                    summaryData.financial_data.return_on_equity &&
                    summaryData.financial_data.return_on_equity > 0.15
                  }
                  isBad={
                    summaryData.financial_data.return_on_equity &&
                    summaryData.financial_data.return_on_equity < 0.05
                  }
                />
                <DataRow
                  label="Profit Margins"
                  value={formatPercent(
                    summaryData.financial_data.profit_margins
                  )}
                  isGood={
                    summaryData.financial_data.profit_margins &&
                    summaryData.financial_data.profit_margins > 0.2
                  }
                  isBad={
                    summaryData.financial_data.profit_margins &&
                    summaryData.financial_data.profit_margins < 0.05
                  }
                />
              </InfoCard>
            )}

            {/* Summary Detail */}
            {summaryData.summary_detail && (
              <InfoCard title="Market Data">
                <DataRow
                  label="Market Cap"
                  value={formatCurrency(summaryData.summary_detail.market_cap)}
                />
                <DataRow
                  label="52W High"
                  value={formatCurrency(
                    summaryData.summary_detail.fifty_two_week_high
                  )}
                />
                <DataRow
                  label="52W Low"
                  value={formatCurrency(
                    summaryData.summary_detail.fifty_two_week_low
                  )}
                />
                <DataRow
                  label="Dividend Yield"
                  value={formatPercent(
                    summaryData.summary_detail.trailing_annual_dividend_yield
                  )}
                />
                <DataRow
                  label="Beta"
                  value={summaryData.summary_detail.beta?.toFixed(2)}
                />
                <DataRow
                  label="Volume"
                  value={summaryData.summary_detail.volume?.toLocaleString()}
                />
              </InfoCard>
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
};

// News Component
const NewsComponent = ({ ticker }) => {
  const [newsData, setNewsData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [newsCount, setNewsCount] = useState(20);

  const fetchNews = async () => {
    if (!ticker) return;

    setLoading(true);
    setError(null);
    try {
      const response = await api.get('/news', {
        params: { ticker, count: newsCount },
      });
      setNewsData(response);
    } catch (err) {
      setError(err.message);
      console.error('Error fetching news:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchNews();
  }, [ticker, newsCount]);

  const getTimeAgo = (timestamp) => {
    const now = Date.now();
    const diff = now - timestamp * 1000;
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    return 'Just now';
  };

  if (loading) {
    return (
      <Card className="h-[600px]">
        <CardContent className="flex items-center justify-center h-full">
          <div className="text-center space-y-2">
            <RefreshCw className="w-8 h-8 animate-spin mx-auto text-muted-foreground" />
            <p className="text-muted-foreground">Loading news...</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="h-[600px]">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Latest News - {ticker}</CardTitle>
          <div className="flex items-center space-x-2">
            <Select
              value={newsCount.toString()}
              onValueChange={(value) => setNewsCount(parseInt(value))}
            >
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="10">10 articles</SelectItem>
                <SelectItem value="20">20 articles</SelectItem>
                <SelectItem value="50">50 articles</SelectItem>
              </SelectContent>
            </Select>
            <Button onClick={fetchNews} size="sm" variant="outline">
              <RefreshCw className="w-4 h-4 mr-2" />
              Refresh
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert className="mb-4">
            <AlertDescription>Error: {error}</AlertDescription>
          </Alert>
        )}

        {newsData && newsData.stories && (
          <div className="space-y-4">
            <div className="text-sm text-muted-foreground">
              {newsData.total_count} articles found
            </div>

            <ScrollArea className="h-[480px]">
              <div className="space-y-4">
                {newsData.stories.map((story, index) => (
                  <Card
                    key={story.uuid || index}
                    className="cursor-pointer transition-colors hover:bg-accent"
                    onClick={() => window.open(story.link, '_blank')}
                  >
                    <CardContent className="p-4">
                      <div className="flex gap-4">
                        {story.thumbnail && (
                          <img
                            src={story.thumbnail}
                            alt="News thumbnail"
                            className="w-16 h-16 object-cover rounded flex-shrink-0"
                          />
                        )}
                        <div className="flex-1 space-y-2">
                          <h4 className="font-semibold text-sm leading-tight">
                            {story.title}
                          </h4>

                          {story.summary && (
                            <p className="text-xs text-muted-foreground leading-relaxed">
                              {story.summary.substring(0, 150)}
                              {story.summary.length > 150 && '...'}
                            </p>
                          )}

                          <div className="flex justify-between items-center text-xs text-muted-foreground">
                            <div className="flex space-x-3">
                              <span>{story.publisher}</span>
                              {story.author && <span>by {story.author}</span>}
                            </div>
                            <span>{getTimeAgo(story.publish_time)}</span>
                          </div>

                          {story.related_tickers &&
                            story.related_tickers.length > 0 && (
                              <div className="flex flex-wrap gap-1">
                                {story.related_tickers.map((relatedTicker) => (
                                  <Badge
                                    key={relatedTicker}
                                    variant="secondary"
                                    className="text-xs"
                                  >
                                    {relatedTicker}
                                  </Badge>
                                ))}
                              </div>
                            )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </ScrollArea>
          </div>
        )}

        {(!newsData || !newsData.stories || newsData.stories.length === 0) &&
          !loading &&
          !error && (
            <div className="text-center text-muted-foreground py-8">
              No news available for {ticker}
            </div>
          )}
      </CardContent>
    </Card>
  );
};

// Financial Reports Component
const FinancialReports = ({ ticker }) => {
  const [reports, setReports] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchReports = async () => {
    if (!ticker) return;
    setLoading(true);
    setError(null);
    try {
      const response = await api.get('/reports', { params: { ticker } });
      setReports(response.financials);
    } catch (err) {
      setError(err.message || 'Failed to fetch reports');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchReports();
  }, [ticker]);

  if (loading) {
    return (
      <Card className="h-[600px]">
        <CardContent className="flex items-center justify-center h-full">
          <div className="text-center space-y-2">
            <RefreshCw className="w-8 h-8 animate-spin mx-auto text-muted-foreground" />
            <p className="text-muted-foreground">Loading reports...</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="h-[600px]">
        <CardContent className="p-6">
          <Alert>
            <AlertDescription>Error: {error}</AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  if (!reports) return null;

  const renderTable = (title, statements) => {
    if (!statements || statements.length === 0) return null;

    const allKeys = Array.from(
      new Set(statements.flatMap((stmt) => Object.keys(stmt.data)))
    );

    return (
      <Card className="mb-6">
        <CardHeader>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="w-full">
            <Table>
              <TableHeader>
                <TableRow>
                  {allKeys.map((key) => (
                    <TableHead key={key}>{key}</TableHead>
                  ))}
                  <TableHead>Date</TableHead>
                  <TableHead>Period</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {statements.map((stmt, idx) => (
                  <TableRow key={idx}>
                    {allKeys.map((key) => (
                      <TableCell key={key}>
                        {stmt.data[key] !== null && stmt.data[key] !== undefined
                          ? stmt.data[key].toLocaleString()
                          : '-'}
                      </TableCell>
                    ))}
                    <TableCell>{stmt.date}</TableCell>
                    <TableCell>{stmt.period_type}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </ScrollArea>
        </CardContent>
      </Card>
    );
  };

  return (
    <div className="space-y-6">
      {renderTable('Income Statement', reports.income_statement)}
      {renderTable('Balance Sheet', reports.balance_sheet)}
      {renderTable('Cash Flow', reports.cash_flow)}

      {reports.income_statement.length === 0 &&
        reports.balance_sheet.length === 0 &&
        reports.cash_flow.length === 0 && (
          <Card>
            <CardContent className="text-center text-muted-foreground py-8">
              No financial data available for {ticker}
            </CardContent>
          </Card>
        )}
    </div>
  );
};

//   // Main Stock Charting Component
//   const StockCharting = () => {
//     const [stockData, setStockData] = useState(null);
//     const [loading, setLoading] = useState(false);
//     const [error, setError] = useState(null);
//     const [ticker, setTicker] = useState('AAPL');
//     const [range, setRange] = useState('1mo');
//     const [indicators, setIndicators] = useState([]);
//     const [quotes, setQuotes] = useState(null);
//     const [autoRefresh, setAutoRefresh] = useState(false);
//     const [activeTab, setActiveTab] = useState('chart');

//     const today = new Date();
//     const [date, setDate] = React.useState(today);

//     const accumDist = useAccumDist();
//     const [output, setOutput] = useState([]);

//     const handleCompute = () => {
//       if (accumDist && stockData?.candles) {
//         const result = accumDist(stockData.candles);
//         setOutput(result);
//         console.log('AccumDistLine:', result);
//       }
//     };

//     // Fetch historical data with indicators
//     const fetchHistoricalData = useCallback(async () => {
//       if (!ticker) return;

//       setLoading(true);
//       setError(null);

//       try {
//         const indicatorConfigs = indicators
//           .filter((ind) => ind.enabled)
//           .map((ind) => ({
//             name: ind.backendKey,
//             params: ind.params,
//           }));

//         const response = await api.get('/historical', {
//           params: {
//             tickers: ticker,
//             range: range,
//             interval: range === '1d' ? '5m' : '1d',
//             include_indicators: indicatorConfigs.length > 0,
//             indicators: JSON.stringify(indicatorConfigs),
//           },
//         });

//         if (response && response.data && response.data[ticker]) {
//           setStockData(response.data[ticker]);
//         } else if (response && response[ticker]) {
//           setStockData(response[ticker]);
//         } else {
//           setError('No data received for ticker');
//           console.error('Unexpected response structure:', response);
//         }
//       } catch (err) {
//         setError(err.message);
//         console.error('Error fetching historical data:', err);
//       } finally {
//         setLoading(false);
//       }
//     }, [ticker, range, indicators]);

//     // Fetch quotes
//     const fetchQuotes = useCallback(async () => {
//       if (!ticker) return;

//       try {
//         const response = await api.get('/quotes', {
//           params: { tickers: ticker },
//         });

//         if (response && response.quotes) {
//           setQuotes(response.quotes);
//         } else if (response && response[ticker]) {
//           setQuotes({ [ticker]: response[ticker] });
//         } else {
//           setQuotes(response);
//         }
//       } catch (err) {
//         console.error('Error fetching quotes:', err);
//       }
//     }, [ticker]);

//     // Auto-refresh functionality
//     useEffect(() => {
//       if (!autoRefresh) return;

//       const interval = setInterval(() => {
//         fetchQuotes();
//       }, 30000);

//       return () => clearInterval(interval);
//     }, [autoRefresh, fetchQuotes]);

//     // Initial data fetch
//     useEffect(() => {
//       fetchHistoricalData();
//       fetchQuotes();
//     }, [fetchHistoricalData, fetchQuotes]);

//     // Indicator management
//     const addIndicator = (indicator) => {
//       setIndicators((prev) => [...prev, indicator]);
//       setTimeout(() => fetchHistoricalData(), 100);
//     };

//     const toggleIndicator = (id) => {
//       setIndicators((prev) =>
//         prev.map((ind) =>
//           ind.id === id ? { ...ind, enabled: !ind.enabled } : ind
//         )
//       );
//       setTimeout(() => fetchHistoricalData(), 100);
//     };

//     const removeIndicator = (id) => {
//       setIndicators((prev) => prev.filter((ind) => ind.id !== id));
//       setTimeout(() => fetchHistoricalData(), 100);
//     };

//     const updateIndicator = (id, updates) => {
//       setIndicators((prev) =>
//         prev.map((ind) => (ind.id === id ? { ...ind, ...updates } : ind))
//       );
//       setTimeout(() => fetchHistoricalData(), 100);
//     };

//     // Get current underlying price for options
//     const getCurrentPrice = () => {
//       if (quotes && quotes[ticker]) {
//         return quotes[ticker].price || quotes[ticker].regularMarketPrice;
//       }
//       if (stockData && stockData.candles && stockData.candles.length > 0) {
//         return stockData.candles[stockData.candles.length - 1].close;
//       }
//       return null;
//     };

//     const underlyingPrice = getCurrentPrice();

//     const tabItems = [
//       { id: 'chart', label: 'Chart', icon: Activity },
//       { id: 'options', label: 'Options Chain', icon: Search },
//       { id: 'pnl', label: 'P&L Calculator', icon: TrendingUp },
//       { id: 'info', label: 'Info', icon: InfoIcon },
//       { id: 'news', label: 'News', icon: Newspaper },
//       { id: 'calendar', label: 'Calendar', icon: CalendarIcon },
//       { id: 'reports', label: 'Reports', icon: Inspect },
//       { id: 'predictions', label: 'Predictions', icon: BrainCog },
//       { id: 'screener', label: 'Screener', icon: ScreenShare },
//     ];

//     return (
//       <div className="min-h-screen bg-background p-6">
//         {/* Header */}
//         <Card className="mb-6">
//           <CardHeader>
//             <CardTitle className="text-3xl font-bold mb-4">
//               Yeast; Grow your Bread!
//             </CardTitle>

//             <div className="flex flex-wrap gap-4 items-center">
//               <div className="space-y-2">
//                 <Label className="text-sm">Stock Symbol</Label>
//                 <Input
//                   type="text"
//                   value={ticker}
//                   onChange={(e) => setTicker(e.target.value.toUpperCase())}
//                   placeholder="AAPL"
//                   className="w-32"
//                 />
//               </div>

//               <div className="space-y-2">
//                 <Label className="text-sm">Time Range</Label>
//                 <Select value={range} onValueChange={setRange}>
//                   <SelectTrigger className="w-32">
//                     <SelectValue />
//                   </SelectTrigger>
//                   <SelectContent>
//                     <SelectItem value="1d">1 Day</SelectItem>
//                     <SelectItem value="5d">5 Days</SelectItem>
//                     <SelectItem value="1mo">1 Month</SelectItem>
//                     <SelectItem value="3mo">3 Months</SelectItem>
//                     <SelectItem value="6mo">6 Months</SelectItem>
//                     <SelectItem value="1y">1 Year</SelectItem>
//                     <SelectItem value="2y">2 Years</SelectItem>
//                     <SelectItem value="5y">5 Years</SelectItem>
//                   </SelectContent>
//                 </Select>
//               </div>

//               <div className="flex items-end space-x-2">
//                 <Button
//                   onClick={fetchHistoricalData}
//                   disabled={loading}
//                   variant="default"
//                 >
//                   <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
//                   {loading ? 'Loading...' : 'Fetch Data'}
//                 </Button>

//                 <Button
//                   onClick={() => setAutoRefresh(!autoRefresh)}
//                   variant={autoRefresh ? 'default' : 'outline'}
//                 >
//                   {autoRefresh ? <PauseCircle className="w-4 h-4 mr-2" /> : <PlayCircle className="w-4 h-4 mr-2" />}
//                   {autoRefresh ? 'Live' : 'Static'}
//                 </Button>

//                 <Button onClick={handleCompute} variant="outline">
//                   Compute AccumDist
//                 </Button>
//               </div>
//             </div>

//             {/* Quote display */}
//             {quotes && quotes[ticker] && (
//               <div className="mt-4 flex gap-6 items-center">
//                 <div className="text-2xl font-semibold">
//                   ${underlyingPrice?.toFixed(2) || 'N/A'}
//                 </div>
//                 <div className={`flex items-center gap-1 text-lg ${
//                   (quotes[ticker].change || quotes[ticker].regularMarketChange || 0) >= 0
//                     ? 'text-green-600' : 'text-red-600'
//                 }`}>
//                   {(quotes[ticker].change || quotes[ticker].regularMarketChange || 0) >= 0 ? (
//                     <TrendingUp className="w-5 h-5" />
//                   ) : (
//                     <TrendingDown className="w-5 h-5" />
//                   )}
//                   {(quotes[ticker].change || quotes[ticker].regularMarketChange || 0)?.toFixed(2) || '0.00'}
//                   ({(quotes[ticker].change_percent || quotes[ticker].regularMarketChangePercent || 0)?.toFixed(2) || '0.00'}%)
//                 </div>
//               </div>
//             )}

//             {output.length > 0 && (
//               <div className="mt-4">
//                 <h4 className="text-sm font-medium mb-2">Accumulation/Distribution Sample</h4>
//                 <pre className="text-xs bg-muted p-2 rounded overflow-x-auto">
//                   {JSON.stringify(output.slice(0, 10), null, 2)}
//                 </pre>
//               </div>
//             )}
//           </CardHeader>
//         </Card>

//         {/* Tab Navigation */}
//         <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
//           <TabsList className="grid w-full grid-cols-9">
//             {tabItems.map((tab) => {
//               const Icon = tab.icon;
//               return (
//                 <TabsTrigger key={tab.id} value={tab.id} className="flex items-center gap-2">
//                   <Icon className="w-4 h-4" />
//                   <span className="hidden sm:inline">{tab.label}</span>
//                 </TabsTrigger>
//               );
//             })}
//           </TabsList>

//           {/* Tab Content */}
//           <TabsContent value="chart" className="space-y-6">
//             {error && (
//               <Alert>
//                 <AlertDescription>Error: {error}</AlertDescription>
//               </Alert>
//             )}

//             <div className="grid grid-cols-1 xl:grid-cols-4 gap-6">
//               <div className="xl:col-span-3">
//                 <StockChart data={stockData} indicators={indicators} height={500} />
//               </div>
//               <div className="xl:col-span-1">
//                 <ControlPanel
//                   indicators={indicators}
//                   onAddIndicator={addIndicator}
//                   onToggleIndicator={toggleIndicator}
//                   onRemoveIndicator={removeIndicator}
//                   onUpdateIndicator={updateIndicator}
//                 />
//               </div>
//             </div>

//             {/* Data Info */}
//             {stockData && stockData.candles && (
//               <Card>
//                 <CardHeader>
//                   <CardTitle className="text-lg">Data Summary</CardTitle>
//                 </CardHeader>
//                 <CardContent>
//                   <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-7 gap-4 text-sm">
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Data Points</span>
//                       <span className="font-medium">{stockData.candles.length}</span>
//                     </div>
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Latest Price</span>
//                       <span className="font-medium">
//                         ${stockData.candles[stockData.candles.length - 1]?.close?.toFixed(2) || 'N/A'}
//                       </span>
//                     </div>
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Date Range</span>
//                       <span className="font-medium text-xs">
//                         {formatDate(stockData.candles[0]?.timestamp)} to{' '}
//                         {formatDate(stockData.candles[stockData.candles.length - 1]?.timestamp)}
//                       </span>
//                     </div>
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Symbol</span>
//                       <span className="font-medium">{stockData.symbol}</span>
//                     </div>
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Exchange</span>
//                       <span className="font-medium">{stockData.meta?.exchange}</span>
//                     </div>
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Currency</span>
//                       <span className="font-medium">{stockData.meta?.currency}</span>
//                     </div>
//                     <div className="flex flex-col">
//                       <span className="text-muted-foreground">Active Indicators</span>
//                       <span className="font-medium">{indicators.filter((ind) => ind.enabled).length}</span>
//                     </div>
//                   </div>
//                 </CardContent>
//               </Card>
//             )}
//           </TabsContent>

//           <TabsContent value="options">
//             <OptionsChain ticker={ticker} underlyingPrice={underlyingPrice} />
//           </TabsContent>

//           <TabsContent value="pnl">
//             <OptionsPnLCalculator ticker={ticker} underlyingPrice={underlyingPrice} />
//           </TabsContent>

//           <TabsContent value="info">
//             <QuoteSummary ticker={ticker} />
//           </TabsContent>

//           <TabsContent value="news">
//             <NewsComponent ticker={ticker} />
//           </TabsContent>

//           <TabsContent value="calendar">
//             <Card className="h-[600px]">
//               <CardHeader>
//                 <CardTitle>Calendar</CardTitle>
//                 <CardDescription>Select a date for analysis</CardDescription>
//               </CardHeader>
//               <CardContent className="flex justify-center">
//                 <Calendar
//                   mode="single"
//                   selected={date}
//                   onSelect={setDate}
//                   className="rounded-md border"
//                 />
//               </CardContent>
//             </Card>
//           </TabsContent>

//           <TabsContent value="reports">
//             <Card className="h-[600px]">
//               <CardHeader>
//                 <CardTitle>Financial Reports</CardTitle>
//               </CardHeader>
//               <CardContent>
//                 <ScrollArea className="h-[500px]">
//                   <FinancialReports ticker={ticker} />
//                 </ScrollArea>
//               </CardContent>
//             </Card>
//           </TabsContent>

//           <TabsContent value="predictions">
//             <Card className="h-[600px]">
//               <CardHeader>
//                 <CardTitle>AI Predictions</CardTitle>
//                 <CardDescription>Coming soon - AI-powered market predictions</CardDescription>
//               </CardHeader>
//               <CardContent className="flex items-center justify-center h-[500px]">
//                 <div className="text-center space-y-4">
//                   <BrainCog className="w-16 h-16 mx-auto text-muted-foreground" />
//                   <p className="text-muted-foreground">
//                     AI prediction features will be available soon
//                   </p>
//                 </div>
//               </CardContent>
//             </Card>
//           </TabsContent>

//           <TabsContent value="screener">
//             <Card className="h-[600px]">
//               <CardHeader>
//                 <CardTitle>Stock Screener</CardTitle>
//                 <CardDescription>Coming soon - Advanced stock screening tools</CardDescription>
//               </CardHeader>
//               <CardContent className="flex items-center justify-center h-[500px]">
//                 <div className="text-center space-y-4">
//                   <ScreenShare className="w-16 h-16 mx-auto text-mute'

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

// Main Stock Charting Component with Options Integration
const StockCharting = () => {
  const [stockData, setStockData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [ticker, setTicker] = useState('AAPL');
  const [range, setRange] = useState('1mo');
  const [indicators, setIndicators] = useState([]);
  const [quotes, setQuotes] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [activeTab, setActiveTab] = useState('chart');

  const today = new Date();
  const [date, setDate] = React.useState<Date | undefined>(today);

  const accumDist = useAccumDist();
  const [output, setOutput] = useState<number[]>([]);

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

  // Initial data fetch
  useEffect(() => {
    fetchHistoricalData();
    fetchQuotes();
  }, [fetchHistoricalData, fetchQuotes]);

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
            marginBottom: '16px',
          }}
        >
          Yeast; Grow your Bread!
        </h1>

        <div
          style={{
            display: 'flex',
            gap: '16px',
            alignItems: 'center',
            flexWrap: 'wrap',
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
            {/* /v6/finance/quote/lookup for autocomplete suggestions */}
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
            {autoRefresh ? <PauseCircle size={16} /> : <PlayCircle size={16} />}
            {autoRefresh ? 'Live' : 'Static'}
          </button>

          <button
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
            onClick={handleCompute}
          >
            Compute AccumDist
          </button>
          <div>
            <h4>Accumulation/Distribution</h4>
            {output.length > 0 && (
              <pre>{JSON.stringify(output.slice(0, 10), null, 2)}</pre>
            )}
          </div>
        </div>

        {/* Quote display */}
        {quotes && quotes[ticker] && (
          <div
            style={{
              marginTop: '16px',
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
          </div>
        )}

        {/* Tab Navigation */}
        <div
          style={{
            marginTop: '20px',
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
            { id: 'screener', label: 'Screener', icon: ScreenShare },
          ].map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                style={{
                  backgroundColor:
                    activeTab === tab.id ? theme.colors.accent : 'transparent',
                  color: activeTab === tab.id ? 'white' : theme.colors.text,
                  border: 'none',
                  borderRadius: `${theme.borderRadius}px ${theme.borderRadius}px 0 0`,
                  padding: '12px 16px',
                  fontSize: '14px',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  borderBottom:
                    activeTab === tab.id ? 'none' : `1px solid transparent`,
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

            <StockChart data={stockData} indicators={indicators} height={500} />
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
          <OptionsChain ticker={ticker} underlyingPrice={underlyingPrice} />
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

      {/* Data Info - only show on chart tab */}
      {activeTab === 'chart' && stockData && stockData.candles && (
        <div
          style={{
            marginTop: '24px',
            backgroundColor: theme.colors.surface,
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
                  stockData.candles[stockData.candles.length - 1]?.timestamp
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

export default StockCharting;
