'use client';
import {
  AVAILABLE_INDICATORS,
  INDICATOR_CATEGORIES,
  theme,
} from '@/data/throwaway';
import React, { useState, useEffect, useCallback } from 'react';
import { Eye } from 'lucide-react';

// Control Panel Component
export const ControlPanel = ({
  indicators,
  onAddIndicator,
  onToggleIndicator,
  onRemoveIndicator,
  onUpdateIndicator,
}) => {
  const [activeTab, setActiveTab] = useState('indicators');
  const [selectedCategory, setSelectedCategory] = useState('Moving Averages');
  const [showParameterModal, setShowParameterModal] = useState(false);
  const [selectedIndicatorType, setSelectedIndicatorType] = useState(null);
  const [tempParams, setTempParams] = useState({});

  const [wasm, setWasm] = useState(null);
  const [availableIndicators, setAvailableIndicators] = useState([]);

  useEffect(() => {
    const loadWasm = async () => {
      const module = await import('@/wasm/wasm/stox_wasm.js');
      await module.default();
      setWasm(module);

      // Get all indicators
      const allIndicators = module.get_indicators();
      setAvailableIndicators(allIndicators);
      console.log('WASM indicators:', allIndicators);
    };
    loadWasm();
  }, []);

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
            color: theme.colors.primary[1],
          },
          {
            key: 'macd_histogram',
            name: 'Histogram',
            color: theme.colors.primary[2],
          },
        ];
        break;
      case 'STOCHASTIC':
        config.isMultiLine = true;
        config.lines = [
          { key: 'stoch_k', name: '%K', color: config.color },
          { key: 'stoch_d', name: '%D', color: theme.colors.primary[1] },
        ];
        break;
      case 'ICHIMOKU':
        config.isMultiLine = true;
        config.lines = [
          { key: 'tenkan_sen', name: 'Tenkan-sen', color: config.color },
          {
            key: 'kijun_sen',
            name: 'Kijun-sen',
            color: theme.colors.primary[1],
          },
          {
            key: 'senkou_span_a',
            name: 'Senkou Span A',
            color: theme.colors.primary[2],
            dashed: true,
          },
          {
            key: 'senkou_span_b',
            name: 'Senkou Span B',
            color: theme.colors.primary[3],
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
    setShowParameterModal(true);
  };

  const addIndicatorWithParams = () => {
    const type = selectedIndicatorType;
    const params = { ...tempParams };

    const baseColor =
      theme.colors.primary[indicators.length % theme.colors.primary.length];
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
    setShowParameterModal(false);
    setSelectedIndicatorType(null);
    setTempParams({});
  };

  const renderParameterInputs = () => {
    if (!selectedIndicatorType) return null;

    const config = AVAILABLE_INDICATORS[selectedIndicatorType];

    return Object.entries(config.paramTypes).map(([paramKey, paramType]) => (
      <div key={paramKey} style={{ marginBottom: '12px' }}>
        <label
          style={{
            color: theme.colors.text,
            fontSize: '12px',
            display: 'block',
            marginBottom: '4px',
            textTransform: 'capitalize',
          }}
        >
          {paramKey.replace(/_/g, ' ')}
        </label>
        {paramType === 'number' ? (
          <input
            type="number"
            value={tempParams[paramKey] || ''}
            onChange={(e) =>
              setTempParams((prev) => ({
                ...prev,
                [paramKey]: parseFloat(e.target.value) || 0,
              }))
            }
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '6px 8px',
              fontSize: '12px',
              width: '100%',
            }}
          />
        ) : paramType === 'array' ? (
          <input
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
            style={{
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '6px 8px',
              fontSize: '12px',
              width: '100%',
            }}
          />
        ) : null}
      </div>
    ));
  };

  return (
    <div
      style={{
        backgroundColor: theme.colors.surface,
        border: `1px solid ${theme.colors.border}`,
        borderRadius: theme.borderRadius,
        padding: theme.spacing.md,
        height: '500px',
        overflowY: 'auto',
      }}
    >
      <h3 style={{ color: theme.colors.text, marginBottom: theme.spacing.md }}>
        Technical Indicators
      </h3>

      {/* Category Selection */}
      <div style={{ marginBottom: theme.spacing.md }}>
        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value)}
          style={{
            backgroundColor: theme.colors.background,
            color: theme.colors.text,
            border: `1px solid ${theme.colors.border}`,
            borderRadius: theme.borderRadius,
            padding: '6px 8px',
            fontSize: '12px',
            width: '100%',
            marginBottom: '8px',
          }}
        >
          {Object.keys(INDICATOR_CATEGORIES).map((category) => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </select>

        {/* Indicator Buttons */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: theme.spacing.xs,
          }}
        >
          {INDICATOR_CATEGORIES[selectedCategory].map((type) => (
            <button
              key={type}
              onClick={() => openParameterModal(type)}
              style={{
                backgroundColor: theme.colors.primary[0],
                color: 'white',
                border: 'none',
                padding: `${theme.spacing.xs}px ${theme.spacing.sm}px`,
                borderRadius: theme.borderRadius,
                fontSize: '10px',
                cursor: 'pointer',
                textAlign: 'center',
              }}
            >
              {type.replace(/_/g, ' ')}
            </button>
          ))}
        </div>
      </div>

      {/* Active Indicators */}
      <div>
        <h4
          style={{
            color: theme.colors.text,
            marginBottom: theme.spacing.sm,
            fontSize: '14px',
          }}
        >
          Active Indicators ({indicators.filter((i) => i.enabled).length})
        </h4>
        {indicators.map((indicator) => (
          <div
            key={indicator.id}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: theme.spacing.sm,
              backgroundColor: indicator.enabled
                ? theme.colors.background
                : theme.colors.surface,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              marginBottom: theme.spacing.xs,
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: theme.spacing.sm,
                flex: 1,
              }}
            >
              <div
                style={{
                  width: '12px',
                  height: '12px',
                  backgroundColor: indicator.color,
                  borderRadius: '50%',
                }}
              />
              <span style={{ color: theme.colors.text, fontSize: '10px' }}>
                {indicator.displayName || indicator.type}
              </span>
            </div>
            <div style={{ display: 'flex', gap: theme.spacing.xs }}>
              <button
                onClick={() => onToggleIndicator(indicator.id)}
                style={{
                  backgroundColor: 'transparent',
                  color: theme.colors.textSecondary,
                  border: 'none',
                  cursor: 'pointer',
                }}
              >
                <Eye
                  size={12}
                  style={{ opacity: indicator.enabled ? 1 : 0.5 }}
                />
              </button>
              <button
                onClick={() => onRemoveIndicator(indicator.id)}
                style={{
                  backgroundColor: 'transparent',
                  color: theme.colors.error,
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: '14px',
                }}
              >
                ×
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Parameter Modal */}
      {showParameterModal && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
        >
          <div
            style={{
              backgroundColor: theme.colors.surface,
              border: `1px solid ${theme.colors.border}`,
              borderRadius: theme.borderRadius,
              padding: '20px',
              width: '300px',
              maxHeight: '400px',
              overflowY: 'auto',
            }}
          >
            <h4 style={{ color: theme.colors.text, marginBottom: '16px' }}>
              Configure {selectedIndicatorType?.replace(/_/g, ' ')}
            </h4>

            {renderParameterInputs()}

            <div style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
              <button
                onClick={addIndicatorWithParams}
                style={{
                  backgroundColor: theme.colors.accent,
                  color: 'white',
                  border: 'none',
                  borderRadius: theme.borderRadius,
                  padding: '8px 12px',
                  fontSize: '12px',
                  cursor: 'pointer',
                  flex: 1,
                }}
              >
                Add Indicator
              </button>
              <button
                onClick={() => {
                  setShowParameterModal(false);
                  setSelectedIndicatorType(null);
                  setTempParams({});
                }}
                style={{
                  backgroundColor: theme.colors.surface,
                  color: theme.colors.text,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  padding: '8px 12px',
                  fontSize: '12px',
                  cursor: 'pointer',
                  flex: 1,
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
