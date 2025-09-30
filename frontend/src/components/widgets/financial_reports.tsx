'use client';
import { theme } from '@/data/throwaway';
import { api } from '@/lib/api';
import React, { useState, useEffect, useCallback } from 'react';

export const FinancialReports = ({ ticker }) => {
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

  if (loading)
    return (
      <div style={{ padding: '40px', textAlign: 'center' }}>
        Loading reports...
      </div>
    );
  if (error)
    return <div style={{ color: 'red', padding: '20px' }}>Error: {error}</div>;
  if (!reports) return null;

  const renderTable = (title, statements) => {
    if (!statements || statements.length === 0) return null;

    // Collect all keys across all statements to ensure consistent columns
    const allKeys = Array.from(
      new Set(statements.flatMap((stmt) => Object.keys(stmt.data)))
    );

    return (
      <div
        style={{
          marginBottom: '24px',
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          overflowX: 'auto',
        }}
      >
        <h3 style={{ padding: '16px', margin: 0, color: theme.colors.text }}>
          {title}
        </h3>
        <table
          style={{
            width: '100%',
            borderCollapse: 'collapse',
            color: theme.colors.text,
          }}
        >
          <thead>
            <tr style={{ backgroundColor: theme.colors.background }}>
              {allKeys.map((key) => (
                <th
                  key={key}
                  style={{
                    textAlign: 'left',
                    padding: '8px 12px',
                    borderBottom: `1px solid ${theme.colors.border}`,
                  }}
                >
                  {key}
                </th>
              ))}
              <th
                style={{
                  textAlign: 'left',
                  padding: '8px 12px',
                  borderBottom: `1px solid ${theme.colors.border}`,
                }}
              >
                Date
              </th>
              <th
                style={{
                  textAlign: 'left',
                  padding: '8px 12px',
                  borderBottom: `1px solid ${theme.colors.border}`,
                }}
              >
                Period
              </th>
            </tr>
          </thead>
          <tbody>
            {statements.map((stmt, idx) => (
              <tr
                key={idx}
                style={{ borderBottom: `1px solid ${theme.colors.border}` }}
              >
                {allKeys.map((key) => (
                  <td key={key} style={{ padding: '8px 12px' }}>
                    {stmt.data[key] !== null && stmt.data[key] !== undefined
                      ? stmt.data[key].toLocaleString()
                      : '-'}
                  </td>
                ))}
                <td style={{ padding: '8px 12px' }}>{stmt.date}</td>
                <td style={{ padding: '8px 12px' }}>{stmt.period_type}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  return (
    <div
      style={{
        padding: '24px',
        backgroundColor: theme.colors.background,
        minHeight: '100%',
      }}
    >
      {renderTable('Income Statement', reports.income_statement)}
      {renderTable('Balance Sheet', reports.balance_sheet)}
      {renderTable('Cash Flow', reports.cash_flow)}

      {reports.income_statement.length === 0 &&
        reports.balance_sheet.length === 0 &&
        reports.cash_flow.length === 0 && (
          <div
            style={{
              textAlign: 'center',
              color: theme.colors.textSecondary,
              padding: '40px',
            }}
          >
            No financial data available for {ticker}
          </div>
        )}
    </div>
  );
};
