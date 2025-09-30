'use client';
import { theme } from '@/data/throwaway';
import { api } from '@/lib/api';
import React, { useState, useEffect, useCallback } from 'react';

// Enhanced Options Chain Component with Proper Coloring
export const OptionsChain = ({ ticker, underlyingPrice }) => {
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

  // Enhanced coloring functions
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
      'deep-itm': isCall ? '#c8e6c9' : '#ffcdd2', // Green for ITM calls, Red for ITM puts
      itm: isCall ? '#e8f5e9' : '#ffebee', // Light green/red
      atm: '#fff3e0', // Orange tint for at-the-money
      otm: '#f3e5f5', // Purple tint for OTM
      'deep-otm': '#e8eaf6', // Blue tint for deep OTM
    };
    return colors[moneyness] || 'transparent';
  };

  const getDeltaColor = (delta, optionType) => {
    const absDelta = Math.abs(delta);
    if (optionType === 'call') {
      if (delta > 0.8) return { bg: '#1b5e20', text: '#ffffff' }; // Dark green - deep ITM call
      if (delta > 0.6) return { bg: '#2e7d32', text: '#ffffff' }; // Medium green
      if (delta > 0.4) return { bg: '#388e3c', text: '#ffffff' }; // Light green
      if (delta > 0.2) return { bg: '#66bb6a', text: '#000000' }; // Very light green
      return { bg: '#c8e6c9', text: '#000000' }; // Pale green
    } else {
      if (delta < -0.8) return { bg: '#b71c1c', text: '#ffffff' }; // Dark red - deep ITM put
      if (delta < -0.6) return { bg: '#c62828', text: '#ffffff' }; // Medium red
      if (delta < -0.4) return { bg: '#d32f2f', text: '#ffffff' }; // Light red
      if (delta < -0.2) return { bg: '#f44336', text: '#ffffff' }; // Very light red
      return { bg: '#ffcdd2', text: '#000000' }; // Pale red
    }
  };

  const getGammaColor = (gamma) => {
    if (gamma > 0.1) return { bg: '#1976d2', text: '#ffffff' }; // High gamma - blue
    if (gamma > 0.05) return { bg: '#42a5f5', text: '#ffffff' }; // Medium gamma - light blue
    if (gamma > 0.01) return { bg: '#90caf9', text: '#000000' }; // Low gamma - pale blue
    return { bg: 'transparent', text: '#000000' };
  };

  const getThetaColor = (theta) => {
    const absTheta = Math.abs(theta);
    if (absTheta > 0.1) return { bg: '#d32f2f', text: '#ffffff' }; // High time decay - red
    if (absTheta > 0.05) return { bg: '#f57c00', text: '#ffffff' }; // Medium time decay - orange
    if (absTheta > 0.01) return { bg: '#ff9800', text: '#000000' }; // Low time decay - light orange
    return { bg: 'transparent', text: '#000000' };
  };

  const getVegaColor = (vega) => {
    if (vega > 0.5) return { bg: '#388e3c', text: '#ffffff' }; // High vega sensitivity - green
    if (vega > 0.2) return { bg: '#66bb6a', text: '#000000' }; // Medium vega - light green
    if (vega > 0.1) return { bg: '#c8e6c9', text: '#000000' }; // Low vega - pale green
    return { bg: 'transparent', text: '#000000' };
  };

  const getIVColor = (iv) => {
    if (iv > 1.5) return { bg: '#b71c1c', text: '#ffffff' }; // Extremely high IV - dark red
    if (iv > 1.0) return { bg: '#d32f2f', text: '#ffffff' }; // Very high IV - red
    if (iv > 0.8) return { bg: '#f57c00', text: '#ffffff' }; // High IV - dark orange
    if (iv > 0.6) return { bg: '#ff9800', text: '#000000' }; // Elevated IV - orange
    if (iv > 0.4) return { bg: '#ffb74d', text: '#000000' }; // Normal IV - light orange
    if (iv > 0.2) return { bg: '#c8e6c9', text: '#000000' }; // Low IV - light green
    return { bg: '#a5d6a7', text: '#000000' }; // Very low IV - green
  };

  const getVolumeColor = (volume) => {
    if (volume > 10000) return { bg: '#2e7d32', text: '#ffffff' }; // Very high volume - dark green
    if (volume > 5000) return { bg: '#388e3c', text: '#ffffff' }; // High volume - green
    if (volume > 1000) return { bg: '#66bb6a', text: '#000000' }; // Medium volume - light green
    if (volume > 100) return { bg: '#c8e6c9', text: '#000000' }; // Low volume - pale green
    return { bg: 'transparent', text: '#000000' }; // Very low volume
  };

  const getOpenInterestColor = (oi) => {
    if (oi > 50000) return { bg: '#4527a0', text: '#ffffff' }; // Very high OI - dark purple
    if (oi > 10000) return { bg: '#5e35b1', text: '#ffffff' }; // High OI - purple
    if (oi > 5000) return { bg: '#7e57c2', text: '#ffffff' }; // Medium OI - light purple
    if (oi > 1000) return { bg: '#b39ddb', text: '#000000' }; // Low OI - pale purple
    return { bg: 'transparent', text: '#000000' }; // Very low OI
  };

  const getBidAskSpreadColor = (bid, ask) => {
    if (bid === 0 || ask === 0) return { bg: '#d32f2f', text: '#ffffff' }; // No bid/ask - red

    const spread = ask - bid;
    const midPrice = (bid + ask) / 2;
    const spreadPct = midPrice > 0 ? (spread / midPrice) * 100 : 100;

    if (spreadPct > 50) return { bg: '#c62828', text: '#ffffff' }; // Very wide spread - dark red
    if (spreadPct > 20) return { bg: '#f57c00', text: '#ffffff' }; // Wide spread - dark orange
    if (spreadPct > 10) return { bg: '#ff9800', text: '#000000' }; // Medium spread - orange
    if (spreadPct > 5) return { bg: '#ffb74d', text: '#000000' }; // Narrow spread - light orange
    return { bg: '#c8e6c9', text: '#000000' }; // Very tight spread - green
  };

  const isNearMoney = (strike, underlyingPrice) => {
    if (!underlyingPrice) return false;
    return Math.abs(strike - underlyingPrice) < underlyingPrice * 0.02; // Within 2%
  };

  const renderOptionsTable = (contracts, type) => (
    <div style={{ marginBottom: '20px' }}>
      <h4 style={{ color: theme.colors.text, marginBottom: '12px' }}>
        {type.toUpperCase()}S
      </h4>
      <div style={{ overflowX: 'auto' }}>
        <table
          style={{
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: '11px',
          }}
        >
          <thead>
            <tr style={{ backgroundColor: theme.colors.background }}>
              <th style={tableHeaderStyle}>Strike</th>
              <th style={tableHeaderStyle}>Bid</th>
              <th style={tableHeaderStyle}>Ask</th>
              <th style={tableHeaderStyle}>Last</th>
              <th style={tableHeaderStyle}>Volume</th>
              <th style={tableHeaderStyle}>OI</th>
              {includeGreeks && (
                <>
                  <th style={tableHeaderStyle}>Delta</th>
                  <th style={tableHeaderStyle}>Gamma</th>
                  <th style={tableHeaderStyle}>Theta</th>
                  <th style={tableHeaderStyle}>Vega</th>
                  <th style={tableHeaderStyle}>IV</th>
                </>
              )}
            </tr>
          </thead>
          <tbody>
            {contracts.map((contract, idx) => {
              const moneyness = getMoneyness(
                contract.strike,
                underlyingPrice,
                type
              );
              const rowBgColor = getMoneynessColor(moneyness, type === 'call');
              const isAtm = isNearMoney(contract.strike, underlyingPrice);

              return (
                <tr
                  key={idx}
                  style={{
                    borderBottom: `1px solid ${theme.colors.border}`,
                    backgroundColor: isAtm ? '#fff3e0' : rowBgColor,
                    border: isAtm ? '2px solid #ff9800' : 'none',
                  }}
                >
                  <td
                    style={{
                      ...tableCellStyle,
                      fontWeight: isAtm ? 'bold' : 'normal',
                      backgroundColor: isAtm ? '#ff6f00' : 'transparent',
                      color: isAtm ? '#ffffff' : '#000000',
                    }}
                  >
                    ${contract.strike}
                  </td>

                  <td
                    style={{
                      ...tableCellStyle,
                      backgroundColor: getBidAskSpreadColor(
                        contract.bid,
                        contract.ask
                      ).bg,
                      color: getBidAskSpreadColor(contract.bid, contract.ask)
                        .text,
                    }}
                  >
                    ${contract.bid.toFixed(2)}
                  </td>

                  <td
                    style={{
                      ...tableCellStyle,
                      backgroundColor: getBidAskSpreadColor(
                        contract.bid,
                        contract.ask
                      ).bg,
                      color: getBidAskSpreadColor(contract.bid, contract.ask)
                        .text,
                    }}
                  >
                    ${contract.ask.toFixed(2)}
                  </td>

                  <td
                    style={{
                      ...tableCellStyle,
                      fontWeight: isAtm ? 'bold' : 'normal',
                      backgroundColor: isAtm ? '#ff6f00' : 'transparent',
                      color: isAtm ? '#ffffff' : '#000000',
                    }}
                  >
                    ${contract.last.toFixed(2)}
                  </td>

                  <td
                    style={{
                      ...tableCellStyle,
                      backgroundColor: getVolumeColor(contract.volume).bg,
                      color: getVolumeColor(contract.volume).text,
                      fontWeight: contract.volume > 5000 ? 'bold' : 'normal',
                    }}
                  >
                    {contract.volume.toLocaleString()}
                  </td>

                  <td
                    style={{
                      ...tableCellStyle,
                      backgroundColor: getOpenInterestColor(
                        contract.open_interest
                      ).bg,
                      color: getOpenInterestColor(contract.open_interest).text,
                      fontWeight:
                        contract.open_interest > 10000 ? 'bold' : 'normal',
                    }}
                  >
                    {contract.open_interest.toLocaleString()}
                  </td>

                  {includeGreeks && contract.greeks && (
                    <>
                      <td
                        style={{
                          ...tableCellStyle,
                          backgroundColor: getDeltaColor(
                            contract.greeks.delta,
                            type
                          ).bg,
                          color: getDeltaColor(contract.greeks.delta, type)
                            .text,
                          fontWeight:
                            Math.abs(contract.greeks.delta) > 0.7
                              ? 'bold'
                              : 'normal',
                        }}
                      >
                        {contract.greeks.delta.toFixed(3)}
                      </td>

                      <td
                        style={{
                          ...tableCellStyle,
                          backgroundColor: getGammaColor(contract.greeks.gamma)
                            .bg,
                          color: getGammaColor(contract.greeks.gamma).text,
                        }}
                      >
                        {contract.greeks.gamma.toFixed(4)}
                      </td>

                      <td
                        style={{
                          ...tableCellStyle,
                          backgroundColor: getThetaColor(contract.greeks.theta)
                            .bg,
                          color: getThetaColor(contract.greeks.theta).text,
                        }}
                      >
                        {contract.greeks.theta.toFixed(3)}
                      </td>

                      <td
                        style={{
                          ...tableCellStyle,
                          backgroundColor: getVegaColor(contract.greeks.vega)
                            .bg,
                          color: getVegaColor(contract.greeks.vega).text,
                        }}
                      >
                        {contract.greeks.vega.toFixed(3)}
                      </td>

                      <td
                        style={{
                          ...tableCellStyle,
                          backgroundColor: getIVColor(
                            contract.implied_volatility
                          ).bg,
                          color: getIVColor(contract.implied_volatility).text,
                          fontWeight:
                            contract.implied_volatility > 1.2
                              ? 'bold'
                              : 'normal',
                        }}
                      >
                        {(contract.implied_volatility * 100).toFixed(1)}%
                      </td>
                    </>
                  )}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Legend */}
      <div
        style={{
          marginTop: '12px',
          fontSize: '10px',
          color: theme.colors.textSecondary,
        }}
      >
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: '8px',
          }}
        >
          <div>
            <span
              style={{
                backgroundColor: '#2e7d32',
                color: 'white',
                padding: '2px 4px',
                marginRight: '4px',
              }}
            >
              ■
            </span>
            High Volume
          </div>
          <div>
            <span
              style={{
                backgroundColor: '#4527a0',
                color: 'white',
                padding: '2px 4px',
                marginRight: '4px',
              }}
            >
              ■
            </span>
            High Open Interest
          </div>
          <div>
            <span
              style={{
                backgroundColor: '#ff6f00',
                color: 'white',
                padding: '2px 4px',
                marginRight: '4px',
              }}
            >
              ■
            </span>
            At-the-Money
          </div>
          <div>
            <span
              style={{
                backgroundColor: '#b71c1c',
                color: 'white',
                padding: '2px 4px',
                marginRight: '4px',
              }}
            >
              ■
            </span>
            High IV/Risk
          </div>
          <div>
            <span
              style={{
                backgroundColor: '#1976d2',
                color: 'white',
                padding: '2px 4px',
                marginRight: '4px',
              }}
            >
              ■
            </span>
            High Gamma
          </div>
          <div>
            <span
              style={{
                backgroundColor: '#c62828',
                color: 'white',
                padding: '2px 4px',
                marginRight: '4px',
              }}
            >
              ■
            </span>
            Wide Spread
          </div>
        </div>
      </div>
    </div>
  );

  const tableHeaderStyle = {
    color: theme.colors.text,
    padding: '8px',
    textAlign: 'left',
    fontSize: '12px',
    fontWeight: 'bold',
    position: 'sticky',
    top: 0,
    backgroundColor: theme.colors.background,
    zIndex: 1,
  };

  const tableCellStyle = {
    color: theme.colors.text,
    padding: '6px 8px',
    fontSize: '11px',
    textAlign: 'right',
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
        <h3 style={{ color: theme.colors.text, margin: 0 }}>Options Chain</h3>
        <button
          onClick={fetchOptionsChain}
          disabled={loading}
          style={{
            backgroundColor: theme.colors.accent,
            color: 'white',
            border: 'none',
            borderRadius: theme.borderRadius,
            padding: '6px 12px',
            fontSize: '12px',
            cursor: 'pointer',
          }}
        >
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {/* Controls - same as before */}
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
            Expiration Date
          </label>
          <select
            value={selectedExpiry}
            onChange={(e) => setSelectedExpiry(e.target.value)}
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '6px',
              fontSize: '11px',
              width: '100%',
            }}
          >
            {optionsData?.expirations &&
              Object.keys(optionsData.expirations).map((date) => (
                <option key={date} value={date}>
                  {date} (
                  {optionsData.expirations[date].days_to_expiry.toFixed(0)}{' '}
                  days)
                </option>
              ))}
          </select>
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
            Option Type
          </label>
          <select
            value={optionType}
            onChange={(e) => setOptionType(e.target.value)}
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '6px',
              fontSize: '11px',
              width: '100%',
            }}
          >
            <option value="both">Both</option>
            <option value="call">Calls Only</option>
            <option value="put">Puts Only</option>
          </select>
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
            Min Strike
          </label>
          <input
            type="number"
            value={minStrike}
            onChange={(e) => setMinStrike(e.target.value)}
            placeholder="Min"
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
            Max Strike
          </label>
          <input
            type="number"
            value={maxStrike}
            onChange={(e) => setMaxStrike(e.target.value)}
            placeholder="Max"
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

      <div style={{ marginBottom: '12px' }}>
        <label
          style={{
            display: 'flex',
            alignItems: 'center',
            color: theme.colors.text,
            fontSize: '12px',
          }}
        >
          <input
            type="checkbox"
            checked={includeGreeks}
            onChange={(e) => setIncludeGreeks(e.target.checked)}
            style={{ marginRight: '8px' }}
          />
          Include Greeks
        </label>
      </div>

      {/* Options Data */}
      {optionsData &&
        selectedExpiry &&
        optionsData.expirations[selectedExpiry] && (
          <div>
            <div
              style={{
                marginBottom: '16px',
                padding: '12px',
                backgroundColor: theme.colors.background,
                borderRadius: theme.borderRadius,
              }}
            >
              <div
                style={{
                  color: theme.colors.text,
                  fontSize: '14px',
                  fontWeight: 'bold',
                }}
              >
                {ticker} @ $
                {underlyingPrice?.toFixed(2) ||
                  optionsData.underlying_price?.toFixed(2)}
              </div>
              <div
                style={{ color: theme.colors.textSecondary, fontSize: '12px' }}
              >
                {selectedExpiry} •{' '}
                {optionsData.expirations[selectedExpiry].days_to_expiry.toFixed(
                  0
                )}{' '}
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
    </div>
  );
};
