'use client';
import { theme } from '@/data/throwaway';
import { formatDate } from '@/utils/formatters';
import React from 'react';
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

// Stock Chart Component
export const StockChart = ({ data, indicators = [], height = 400 }) => {
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
      <div
        style={{
          height,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: theme.colors.surface,
          borderRadius: theme.borderRadius,
          color: theme.colors.textSecondary,
        }}
      >
        No data available
      </div>
    );
  }

  return (
    <div
      style={{
        height,
        backgroundColor: theme.colors.surface,
        borderRadius: theme.borderRadius,
        border: `1px solid ${theme.colors.border}`,
        padding: theme.spacing.md,
      }}
    >
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart
          data={processedData}
          margin={{ top: 20, right: 30, left: 20, bottom: 20 }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke={theme.colors.grid} />
          <XAxis dataKey="date" stroke={theme.colors.text} fontSize={12} />
          <YAxis stroke={theme.colors.text} fontSize={12} />
          <YAxis
            yAxisId="volume"
            orientation="right"
            stroke={theme.colors.textSecondary}
            fontSize={10}
          />
          <ReTooltip
            contentStyle={{
              backgroundColor: theme.colors.surface,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              color: theme.colors.text,
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
            fill={theme.colors.primary[1]}
            opacity={0.3}
            yAxisId="volume"
            name="Volume"
          />

          {/* Main price line */}
          <Line
            type="monotone"
            dataKey="close"
            stroke={theme.colors.accent}
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
  );
};
