'use client';
import { theme } from '@/data/throwaway';
import { api } from '@/lib/api';
import React, { useState, useEffect, useCallback } from 'react';

// News Component
export const NewsComponent = ({ ticker }) => {
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

  const formatNewsDate = (timestamp) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

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
        Loading news...
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
          Latest News - {ticker}
        </h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <select
            value={newsCount}
            onChange={(e) => setNewsCount(parseInt(e.target.value))}
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '4px 8px',
              fontSize: '12px',
            }}
          >
            <option value={10}>10 articles</option>
            <option value={20}>20 articles</option>
            <option value={50}>50 articles</option>
          </select>
          <button
            onClick={fetchNews}
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
      </div>

      {error && (
        <div
          style={{
            backgroundColor: theme.colors.error,
            color: 'white',
            padding: '12px',
            borderRadius: theme.borderRadius,
            marginBottom: '16px',
            fontSize: '14px',
          }}
        >
          Error: {error}
        </div>
      )}

      {newsData && newsData.stories && (
        <div>
          <div
            style={{
              color: theme.colors.textSecondary,
              fontSize: '12px',
              marginBottom: '16px',
            }}
          >
            {newsData.total_count} articles found
          </div>

          <div
            style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}
          >
            {newsData.stories.map((story, index) => (
              <div
                key={story.uuid || index}
                style={{
                  backgroundColor: theme.colors.background,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '16px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onClick={() => window.open(story.link, '_blank')}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = theme.colors.surface;
                  e.currentTarget.style.borderColor = theme.colors.accent;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor =
                    theme.colors.background;
                  e.currentTarget.style.borderColor = theme.colors.border;
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    gap: '12px',
                  }}
                >
                  {story.thumbnail && (
                    <img
                      src={story.thumbnail}
                      alt="News thumbnail"
                      style={{
                        width: '60px',
                        height: '60px',
                        objectFit: 'cover',
                        borderRadius: theme.borderRadius,
                        flexShrink: 0,
                      }}
                    />
                  )}
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        color: theme.colors.text,
                        fontSize: '14px',
                        fontWeight: '600',
                        lineHeight: '1.3',
                        marginBottom: '8px',
                      }}
                    >
                      {story.title}
                    </div>

                    {story.summary && (
                      <div
                        style={{
                          color: theme.colors.textSecondary,
                          fontSize: '12px',
                          lineHeight: '1.4',
                          marginBottom: '8px',
                        }}
                      >
                        {story.summary.substring(0, 150)}
                        {story.summary.length > 150 && '...'}
                      </div>
                    )}

                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        fontSize: '11px',
                        color: theme.colors.textSecondary,
                      }}
                    >
                      <div style={{ display: 'flex', gap: '12px' }}>
                        <span>{story.publisher}</span>
                        {story.author && <span>by {story.author}</span>}
                      </div>
                      <span>{getTimeAgo(story.publish_time)}</span>
                    </div>

                    {story.related_tickers &&
                      story.related_tickers.length > 0 && (
                        <div style={{ marginTop: '8px' }}>
                          {story.related_tickers.map((relatedTicker) => (
                            <span
                              key={relatedTicker}
                              style={{
                                backgroundColor: theme.colors.accent,
                                color: 'white',
                                padding: '2px 6px',
                                borderRadius: '3px',
                                fontSize: '10px',
                                marginRight: '4px',
                              }}
                            >
                              {relatedTicker}
                            </span>
                          ))}
                        </div>
                      )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {(!newsData || !newsData.stories || newsData.stories.length === 0) &&
        !loading &&
        !error && (
          <div
            style={{
              textAlign: 'center',
              color: theme.colors.textSecondary,
              padding: '40px',
            }}
          >
            No news available for {ticker}
          </div>
        )}
    </div>
  );
};
