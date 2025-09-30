'use client';
import { theme } from '@/data/throwaway';
import { api } from '@/lib/api';
import React, { useState, useEffect, useCallback } from 'react';
import { CartesianGrid, Line, LineChart, ReferenceLine, ResponsiveContainer, Tooltip as ReTooltip, XAxis, YAxis } from 'recharts';

// Options P&L Calculator Component
export const OptionsPnLCalculator = ({ ticker, underlyingPrice }) => {
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
      console.log(response);
      setPnlAnalysis(response);
    } catch (err) {
      console.error('Error calculating P&L:', err);
    } finally {
      setLoading(false);
    }
  };

  const inputStyle = {
    backgroundColor: theme.colors.surface,
    color: theme.colors.text,
    border: `1px solid ${theme.colors.border}`,
    borderRadius: theme.borderRadius,
    padding: '4px',
    fontSize: '11px',
    width: '100%',
  };

  const labelStyle = {
    color: theme.colors.textSecondary,
    fontSize: '10px',
    display: 'block',
    marginBottom: '4px',
  };

  return (
    <div
      style={{
        backgroundColor: theme.colors.surface,
        border: `1px solid ${theme.colors.border}`,
        borderRadius: theme.borderRadius,
        padding: '20px',
        height: '600px',
        overflowY: 'auto',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '16px',
        }}
      >
        <h3 style={{ color: theme.colors.text, margin: 0 }}>P&L Calculator</h3>
        <button
          onClick={addPosition}
          style={{
            backgroundColor: theme.colors.success,
            color: 'white',
            border: 'none',
            borderRadius: theme.borderRadius,
            padding: '6px 12px',
            fontSize: '12px',
            cursor: 'pointer',
          }}
        >
          Add Position
        </button>
      </div>

      {/* Price Range */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: '12px',
          marginBottom: '16px',
        }}
      >
        <div>
          <label
            style={{
              color: theme.colors.textSecondary,
              fontSize: '11px',
              display: 'block',
              marginBottom: '4px',
            }}
          >
            Min Price
          </label>
          <input
            type="number"
            value={priceRange.min}
            onChange={(e) =>
              setPriceRange((prev) => ({ ...prev, min: e.target.value }))
            }
            placeholder={(underlyingPrice * 0.7).toFixed(0)}
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '6px',
              fontSize: '11px',
              width: '100%',
            }}
          />
        </div>

        <div>
          <label
            style={{
              color: theme.colors.textSecondary,
              fontSize: '11px',
              display: 'block',
              marginBottom: '4px',
            }}
          >
            Max Price
          </label>
          <input
            type="number"
            value={priceRange.max}
            onChange={(e) =>
              setPriceRange((prev) => ({ ...prev, max: e.target.value }))
            }
            placeholder={(underlyingPrice * 1.3).toFixed(0)}
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '6px',
              fontSize: '11px',
              width: '100%',
            }}
          />
        </div>
      </div>

      {/* Positions */}
      <div style={{ marginBottom: '16px' }}>
        <h4
          style={{
            color: theme.colors.text,
            fontSize: '14px',
            marginBottom: '8px',
          }}
        >
          Positions
        </h4>
        {positions.map((position) => (
          <div
            key={position.id}
            style={{
              backgroundColor: theme.colors.background,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '12px',
              marginBottom: '8px',
            }}
          >
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: '1fr 1fr 1fr 1fr 1fr auto',
                gap: '8px',
                alignItems: 'center',
              }}
            >
              {/* Option Type */}
              <div>
                <label style={labelStyle}>Option</label>
                <select
                  value={position.option_type}
                  onChange={(e) =>
                    updatePosition(position.id, 'option_type', e.target.value)
                  }
                  style={inputStyle}
                >
                  <option value="call">Call</option>
                  <option value="put">Put</option>
                </select>
              </div>

              {/* Strike */}
              <div>
                <label style={labelStyle}>Strike</label>
                <input
                  type="number"
                  value={position.strike}
                  onChange={(e) =>
                    updatePosition(position.id, 'strike', e.target.value)
                  }
                  style={inputStyle}
                />
              </div>

              {/* Quantity */}
              <div>
                <label style={labelStyle}>Qty</label>
                <input
                  type="number"
                  value={position.quantity}
                  onChange={(e) =>
                    updatePosition(position.id, 'quantity', e.target.value)
                  }
                  style={inputStyle}
                />
              </div>

              {/* Entry Price */}
              <div>
                <label style={labelStyle}>Price</label>
                <input
                  type="number"
                  step="0.01"
                  value={position.entry_price}
                  onChange={(e) =>
                    updatePosition(position.id, 'entry_price', e.target.value)
                  }
                  style={inputStyle}
                />
              </div>

              {/* Days to Expiry */}
              <div>
                <label style={labelStyle}>DTE</label>
                <input
                  type="number"
                  value={position.days_to_expiry}
                  onChange={(e) =>
                    updatePosition(
                      position.id,
                      'days_to_expiry',
                      e.target.value
                    )
                  }
                  style={inputStyle}
                />
              </div>

              {/* Remove button */}
              <button
                onClick={() => removePosition(position.id)}
                style={{
                  backgroundColor: 'transparent',
                  color: theme.colors.error,
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: '16px',
                  marginTop: '18px', // aligns with input bottom
                }}
                aria-label="Remove position"
                title="Remove position"
              >
                ×
              </button>
            </div>
          </div>
        ))}
      </div>

      <button
        onClick={calculatePnL}
        disabled={loading || positions.length === 0}
        style={{
          backgroundColor: theme.colors.accent,
          color: 'white',
          border: 'none',
          borderRadius: theme.borderRadius,
          padding: '8px 16px',
          fontSize: '12px',
          cursor: 'pointer',
          width: '100%',
          marginBottom: '16px',
        }}
      >
        {loading ? 'Calculating...' : 'Calculate P&L'}
      </button>

      {/* P&L Chart */}
      {pnlAnalysis && (
        <div style={{ height: '300px', marginTop: '16px' }}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={pnlAnalysis.portfolio.total_pnl_curve}>
              <CartesianGrid strokeDasharray="3 3" stroke={theme.colors.grid} />
              <XAxis
                dataKey="underlying_price"
                stroke={theme.colors.text}
                fontSize={10}
                tickFormatter={(value) => `$${value.toFixed(0)}`}
              />
              <YAxis
                stroke={theme.colors.text}
                fontSize={10}
                tickFormatter={(value) => `$${value.toFixed(0)}`}
              />
              <ReTooltip
                contentStyle={{
                  backgroundColor: theme.colors.surface,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  color: theme.colors.text,
                }}
                formatter={(value) => [`$${value.toFixed(2)}`, 'P&L']}
                labelFormatter={(value) => `Price: $${value.toFixed(2)}`}
              />
              <ReferenceLine
                y={0}
                stroke={theme.colors.textSecondary}
                strokeDasharray="2 2"
              />
              <Line
                type="monotone"
                dataKey="pnl"
                stroke={theme.colors.accent}
                strokeWidth={2}
                dot={false}
                name="Portfolio P&L"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Portfolio Summary */}
      {pnlAnalysis && (
        <div
          style={{
            marginTop: '16px',
            padding: '12px',
            backgroundColor: theme.colors.background,
            borderRadius: theme.borderRadius,
          }}
        >
          <h4
            style={{
              color: theme.colors.text,
              margin: '0 0 8px 0',
              fontSize: '14px',
            }}
          >
            Portfolio Summary
          </h4>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '8px',
              fontSize: '12px',
            }}
          >
            <div>
              <span style={{ color: theme.colors.textSecondary }}>
                Max Profit:{' '}
              </span>
              <span style={{ color: theme.colors.success }}>
                {pnlAnalysis.portfolio.max_profit
                  ? `$${pnlAnalysis.portfolio.max_profit.toFixed(2)}`
                  : 'Unlimited'}
              </span>
            </div>
            <div>
              <span style={{ color: theme.colors.textSecondary }}>
                Max Loss:{' '}
              </span>
              <span style={{ color: theme.colors.error }}>
                {pnlAnalysis.portfolio.max_loss
                  ? `$${pnlAnalysis.portfolio.max_loss.toFixed(2)}`
                  : 'Unlimited'}
              </span>
            </div>
            <div>
              <span style={{ color: theme.colors.textSecondary }}>
                Break-even:{' '}
              </span>
              <span style={{ color: theme.colors.text }}>
                {pnlAnalysis.portfolio.break_even_points
                  .map((p) => `$${p.toFixed(2)}`)
                  .join(', ')}
              </span>
            </div>
            <div>
              <span style={{ color: theme.colors.textSecondary }}>
                Net Delta:{' '}
              </span>
              <span style={{ color: theme.colors.text }}>
                {pnlAnalysis.portfolio.total_greeks.delta.toFixed(3)}
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
