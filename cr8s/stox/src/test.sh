#!/bin/bash

BASE_URL="http://localhost:8080"

echo "▶️  Posting historical data request..."
curl -s -X POST "$BASE_URL/historical_data" \
  -H "Content-Type: application/json" \
  -d '{
    "symbols": ["AAPL", "TSLA"],
    "start": "2024-07-01",
    "end": "2024-07-22",
    "interval": "1d"
  }' | jq
echo ""

echo "▶️  Fetching options chain..."
curl -s "$BASE_URL/options_chain?symbol=AAPL&expiry=2024-07-26" | jq
echo ""

echo "▶️  Getting greeks for AAPL call option..."
curl -s "$BASE_URL/greeks?symbol=AAPL&expiry=2024-07-26&strike=200&option_type=call" | jq
echo ""

echo "▶️  Posting indicators request..."
curl -s -X POST "$BASE_URL/indicators" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "AAPL",
    "indicators": ["SMA", "RSI"],
    "start": "2024-07-01",
    "end": "2024-07-22"
  }' | jq
echo ""

echo "▶️  Calculating PnL for an option position..."
curl -s -X POST "$BASE_URL/pnl" \
  -H "Content-Type: application/json" \
  -d '{
    "entry_price": 2.50,
    "current_price": 3.90,
    "quantity": 5
  }' | jq
echo ""

echo "▶️  Listing available indicators..."
curl -s "$BASE_URL/indicators/list" | jq
echo ""
