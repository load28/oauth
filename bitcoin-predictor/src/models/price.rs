use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 비트코인 가격 데이터 (OHLCV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// 과거 가격 데이터 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPrices {
    pub symbol: String,
    pub prices: Vec<PriceData>,
}

impl HistoricalPrices {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            prices: Vec::new(),
        }
    }

    /// 종가 배열 반환
    pub fn closes(&self) -> Vec<f64> {
        self.prices.iter().map(|p| p.close).collect()
    }

    /// 고가 배열 반환
    pub fn highs(&self) -> Vec<f64> {
        self.prices.iter().map(|p| p.high).collect()
    }

    /// 저가 배열 반환
    pub fn lows(&self) -> Vec<f64> {
        self.prices.iter().map(|p| p.low).collect()
    }

    /// 거래량 배열 반환
    pub fn volumes(&self) -> Vec<f64> {
        self.prices.iter().map(|p| p.volume).collect()
    }

    /// 최신 가격 반환
    pub fn latest_price(&self) -> Option<f64> {
        self.prices.last().map(|p| p.close)
    }
}

/// CoinGecko API 응답 구조
#[derive(Debug, Deserialize)]
pub struct CoinGeckoMarketChart {
    pub prices: Vec<Vec<f64>>,      // [timestamp, price]
    pub market_caps: Vec<Vec<f64>>, // [timestamp, market_cap]
    pub total_volumes: Vec<Vec<f64>>, // [timestamp, volume]
}

/// CoinGecko OHLC 응답 (각 요소: [timestamp, open, high, low, close])
pub type CoinGeckoOhlc = Vec<Vec<f64>>;
