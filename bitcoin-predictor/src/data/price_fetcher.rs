use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use tracing::{info, warn};

use crate::models::{CoinGeckoOhlc, HistoricalPrices, PriceData};

/// 비트코인 가격 데이터 수집기
pub struct PriceFetcher {
    client: Client,
    base_url: String,
}

impl PriceFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.coingecko.com/api/v3".to_string(),
        }
    }

    /// 과거 OHLC 가격 데이터 가져오기
    /// days: 1, 7, 14, 30, 90, 180, 365, max
    pub async fn fetch_ohlc(&self, days: u32) -> Result<HistoricalPrices> {
        let url = format!(
            "{}/coins/bitcoin/ohlc?vs_currency=usd&days={}",
            self.base_url, days
        );

        info!("Fetching OHLC data for {} days from CoinGecko", days);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
            .context("Failed to fetch OHLC data from CoinGecko")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("CoinGecko API error: {} - {}", status, body);
        }

        let ohlc_data: CoinGeckoOhlc = response
            .json()
            .await
            .context("Failed to parse OHLC response")?;

        let mut prices = HistoricalPrices::new("BTC/USD");

        for item in ohlc_data {
            if item.len() >= 5 {
                let timestamp_ms = item[0] as i64;
                let timestamp = Utc
                    .timestamp_millis_opt(timestamp_ms)
                    .single()
                    .unwrap_or_else(Utc::now);

                prices.prices.push(PriceData {
                    timestamp,
                    open: item[1],
                    high: item[2],
                    low: item[3],
                    close: item[4],
                    volume: 0.0, // OHLC 엔드포인트는 거래량을 제공하지 않음
                });
            }
        }

        info!("Fetched {} OHLC data points", prices.prices.len());
        Ok(prices)
    }

    /// 현재 비트코인 가격 가져오기
    pub async fn fetch_current_price(&self) -> Result<f64> {
        let url = format!(
            "{}/simple/price?ids=bitcoin&vs_currencies=usd",
            self.base_url
        );

        info!("Fetching current Bitcoin price from CoinGecko");

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
            .context("Failed to fetch current price")?;

        if !response.status().is_success() {
            anyhow::bail!("CoinGecko API error: {}", response.status());
        }

        #[derive(serde::Deserialize)]
        struct PriceResponse {
            bitcoin: BitcoinPrice,
        }

        #[derive(serde::Deserialize)]
        struct BitcoinPrice {
            usd: f64,
        }

        let data: PriceResponse = response.json().await.context("Failed to parse price")?;

        info!("Current Bitcoin price: ${:.2}", data.bitcoin.usd);
        Ok(data.bitcoin.usd)
    }

    /// 지난 N일간의 일별 가격 데이터 가져오기 (거래량 포함)
    pub async fn fetch_market_chart(&self, days: u32) -> Result<HistoricalPrices> {
        let url = format!(
            "{}/coins/bitcoin/market_chart?vs_currency=usd&days={}&interval=daily",
            self.base_url, days
        );

        info!("Fetching market chart data for {} days", days);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
            .context("Failed to fetch market chart data")?;

        if !response.status().is_success() {
            anyhow::bail!("CoinGecko API error: {}", response.status());
        }

        #[derive(serde::Deserialize)]
        struct MarketChartResponse {
            prices: Vec<Vec<f64>>,
            total_volumes: Vec<Vec<f64>>,
        }

        let data: MarketChartResponse = response
            .json()
            .await
            .context("Failed to parse market chart")?;

        let mut prices = HistoricalPrices::new("BTC/USD");

        for (i, price_point) in data.prices.iter().enumerate() {
            if price_point.len() >= 2 {
                let timestamp_ms = price_point[0] as i64;
                let timestamp = Utc
                    .timestamp_millis_opt(timestamp_ms)
                    .single()
                    .unwrap_or_else(Utc::now);

                let volume = data
                    .total_volumes
                    .get(i)
                    .and_then(|v| v.get(1))
                    .copied()
                    .unwrap_or(0.0);

                let price = price_point[1];

                prices.prices.push(PriceData {
                    timestamp,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                });
            }
        }

        info!("Fetched {} market chart data points", prices.prices.len());
        Ok(prices)
    }

    /// 가격 데이터에 OHLC 정보 보강 (두 데이터 소스 병합)
    pub async fn fetch_comprehensive_data(&self, days: u32) -> Result<HistoricalPrices> {
        // OHLC 데이터 먼저 가져오기
        let ohlc_result = self.fetch_ohlc(days).await;

        match ohlc_result {
            Ok(ohlc_data) if !ohlc_data.prices.is_empty() => {
                info!("Using OHLC data with {} points", ohlc_data.prices.len());
                Ok(ohlc_data)
            }
            Ok(_) | Err(_) => {
                warn!("OHLC data unavailable, falling back to market chart");
                self.fetch_market_chart(days).await
            }
        }
    }
}

impl Default for PriceFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // API 호출이 필요하므로 기본적으로 무시
    async fn test_fetch_current_price() {
        let fetcher = PriceFetcher::new();
        let price = fetcher.fetch_current_price().await.unwrap();
        assert!(price > 0.0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_ohlc() {
        let fetcher = PriceFetcher::new();
        let data = fetcher.fetch_ohlc(30).await.unwrap();
        assert!(!data.prices.is_empty());
    }
}
