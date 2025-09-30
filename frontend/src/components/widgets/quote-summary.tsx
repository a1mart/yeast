'use client';
import { theme } from '@/data/throwaway';
import { api } from '@/lib/api';
import React, { useState, useEffect, useCallback } from 'react';

// Quote Summary/Info Component
export const QuoteSummary = ({ ticker }) => {
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
    if (value >= 1e12) return `$${(value / 1e12).toFixed(2)}T`;
    if (value >= 1e9) return `$${(value / 1e9).toFixed(2)}B`;
    if (value >= 1e6) return `$${(value / 1e6).toFixed(2)}M`;
    if (value >= 1e3) return `$${(value / 1e3).toFixed(2)}K`;
    return `$${value.toFixed(2)}`;
  };

  const formatPercent = (value) => {
    if (value === null || value === undefined) return 'N/A';
    return `${(value * 100).toFixed(2)}%`;
  };

  const InfoCard = ({ title, children }) => (
    <div
      style={{
        backgroundColor: theme.colors.background,
        border: `1px solid ${theme.colors.border}`,
        borderRadius: theme.borderRadius,
        padding: '16px',
        marginBottom: '16px',
      }}
    >
      <h4
        style={{
          color: theme.colors.text,
          fontSize: '16px',
          fontWeight: '600',
          marginBottom: '12px',
          margin: 0,
        }}
      >
        {title}
      </h4>
      {children}
    </div>
  );

  const DataRow = ({ label, value, isGood, isBad }) => (
    <div
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '4px 0',
        borderBottom: `1px solid ${theme.colors.border}`,
        fontSize: '13px',
      }}
    >
      <span style={{ color: theme.colors.textSecondary }}>{label}</span>
      <span
        style={{
          color: isGood
            ? theme.colors.success
            : isBad
              ? theme.colors.error
              : theme.colors.text,
          fontWeight: isGood || isBad ? '600' : 'normal',
        }}
      >
        {value}
      </span>
    </div>
  );

  if (loading) {
    return (
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '40px',
          textAlign: 'center',
          color: theme.colors.textSecondary,
        }}
      >
        Loading company information...
      </div>
    );
  }

  if (error) {
    return (
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '20px',
        }}
      >
        <div
          style={{
            color: theme.colors.error,
            marginBottom: '12px',
            fontWeight: '600',
          }}
        >
          Error loading data: {error}
        </div>
        <button
          onClick={fetchQuoteSummary}
          style={{
            backgroundColor: theme.colors.accent,
            color: 'white',
            border: 'none',
            borderRadius: theme.borderRadius,
            padding: '8px 16px',
            fontSize: '12px',
            cursor: 'pointer',
          }}
        >
          Retry
        </button>
      </div>
    );
  }

  if (!summaryData) {
    return (
      <div
        style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          padding: '40px',
          textAlign: 'center',
          color: theme.colors.textSecondary,
        }}
      >
        No data available for {ticker}
      </div>
    );
  }

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
          marginBottom: '20px',
        }}
      >
        <h3 style={{ color: theme.colors.text, margin: 0 }}>
          Company Information - {ticker}
        </h3>
        <button
          onClick={fetchQuoteSummary}
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
          Refresh
        </button>
      </div>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: '20px',
        }}
      >
        {/* Company Profile */}
        {summaryData.asset_profile && (
          <InfoCard title="Company Profile">
            <DataRow label="Sector" value={summaryData.asset_profile.sector} />
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
              <div style={{ marginTop: '8px' }}>
                <a
                  href={summaryData.asset_profile.website}
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{
                    color: theme.colors.accent,
                    textDecoration: 'none',
                    fontSize: '12px',
                  }}
                >
                  Visit Website →
                </a>
              </div>
            )}
            {summaryData.asset_profile.long_business_summary && (
              <div style={{ marginTop: '12px' }}>
                <p
                  style={{
                    color: theme.colors.textSecondary,
                    fontSize: '12px',
                    lineHeight: '1.4',
                    margin: 0,
                  }}
                >
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
              value={summaryData.default_key_statistics.forward_pe?.toFixed(2)}
            />
            <DataRow
              label="Trailing P/E"
              value={summaryData.default_key_statistics.trailing_pe?.toFixed(2)}
            />
            <DataRow
              label="PEG Ratio"
              value={summaryData.default_key_statistics.peg_ratio?.toFixed(2)}
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
              value={formatCurrency(summaryData.financial_data.current_price)}
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
                summaryData.financial_data.recommendation_key === 'strong_buy'
              }
              isBad={
                summaryData.financial_data.recommendation_key === 'sell' ||
                summaryData.financial_data.recommendation_key === 'strong_sell'
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
              value={formatPercent(summaryData.financial_data.return_on_equity)}
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
              value={formatPercent(summaryData.financial_data.profit_margins)}
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
    </div>
  );
};
