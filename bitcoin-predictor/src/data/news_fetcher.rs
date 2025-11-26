use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use tracing::{info, warn};

use crate::models::{NewsApiArticle, NewsApiResponse, NewsArticle, NewsList};

/// 뉴스 데이터 수집기 (NewsAPI 또는 대체 소스 사용)
pub struct NewsFetcher {
    client: Client,
    api_key: Option<String>,
}

impl NewsFetcher {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// 환경변수에서 API 키를 읽어서 생성
    pub fn from_env() -> Self {
        let api_key = std::env::var("NEWS_API_KEY").ok();
        Self::new(api_key)
    }

    /// 비트코인/암호화폐 관련 미국 뉴스 가져오기 (지난 7일)
    pub async fn fetch_bitcoin_news(&self, days: i64) -> Result<NewsList> {
        let to_date = Utc::now();
        let from_date = to_date - Duration::days(days);

        // NewsAPI 키가 있으면 NewsAPI 사용
        if let Some(ref api_key) = self.api_key {
            return self
                .fetch_from_newsapi(api_key, from_date, to_date)
                .await;
        }

        // API 키가 없으면 대체 데이터 사용
        warn!("No NEWS_API_KEY found, using simulated news data");
        Ok(self.generate_simulated_news(from_date, to_date))
    }

    /// NewsAPI에서 뉴스 가져오기
    async fn fetch_from_newsapi(
        &self,
        api_key: &str,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<NewsList> {
        let from_str = from_date.format("%Y-%m-%d").to_string();
        let to_str = to_date.format("%Y-%m-%d").to_string();

        let url = format!(
            "https://newsapi.org/v2/everything?\
            q=bitcoin OR cryptocurrency OR crypto OR BTC&\
            language=en&\
            sortBy=publishedAt&\
            from={}&to={}&\
            apiKey={}",
            from_str, to_str, api_key
        );

        info!("Fetching news from NewsAPI ({} to {})", from_str, to_str);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "BitcoinPredictor/1.0")
            .send()
            .await
            .context("Failed to fetch news from NewsAPI")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("NewsAPI error: {} - {}", status, body);
        }

        let api_response: NewsApiResponse = response
            .json()
            .await
            .context("Failed to parse NewsAPI response")?;

        let articles: Vec<NewsArticle> = api_response
            .articles
            .into_iter()
            .filter_map(|a| self.convert_article(a))
            .collect();

        info!("Fetched {} news articles", articles.len());

        Ok(NewsList {
            articles,
            query: "bitcoin cryptocurrency".to_string(),
            from_date,
            to_date,
        })
    }

    /// NewsAPI 기사를 내부 형식으로 변환
    fn convert_article(&self, article: NewsApiArticle) -> Option<NewsArticle> {
        let title = article.title?;
        let url = article.url?;
        let published_at = article.published_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })?;

        Some(NewsArticle {
            title,
            description: article.description,
            source: article.source.name.unwrap_or_else(|| "Unknown".to_string()),
            published_at,
            url,
        })
    }

    /// 시뮬레이션용 뉴스 데이터 생성 (API 키가 없을 때 사용)
    fn generate_simulated_news(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> NewsList {
        // 실제 시장 상황을 반영하는 샘플 뉴스 헤드라인
        let sample_headlines = vec![
            ("Bitcoin ETF sees record inflows as institutional demand grows", 0.8),
            ("Federal Reserve signals potential rate cuts in 2025", 0.6),
            ("Major bank announces crypto custody services", 0.7),
            ("Regulatory clarity emerges for cryptocurrency markets", 0.5),
            ("Bitcoin mining difficulty reaches new all-time high", 0.3),
            ("Cryptocurrency adoption grows in emerging markets", 0.4),
            ("Wall Street analysts raise Bitcoin price targets", 0.6),
            ("Blockchain technology sees enterprise adoption surge", 0.4),
            ("Central bank digital currency developments worldwide", 0.2),
            ("DeFi total value locked reaches new highs", 0.5),
            ("Bitcoin halving impact analysis by major institutions", 0.4),
            ("Crypto market volatility decreases as market matures", 0.3),
            ("New cryptocurrency regulations proposed in Congress", -0.2),
            ("Market uncertainty amid global economic concerns", -0.3),
        ];

        let duration = to_date - from_date;
        let interval = duration / sample_headlines.len() as i32;

        let articles: Vec<NewsArticle> = sample_headlines
            .iter()
            .enumerate()
            .map(|(i, (headline, _))| {
                let published_at = from_date + interval * i as i32;
                NewsArticle {
                    title: headline.to_string(),
                    description: Some(format!("Analysis and market impact of: {}", headline)),
                    source: "Market Analysis".to_string(),
                    published_at,
                    url: format!("https://example.com/news/{}", i),
                }
            })
            .collect();

        NewsList {
            articles,
            query: "bitcoin cryptocurrency (simulated)".to_string(),
            from_date,
            to_date,
        }
    }
}

impl Default for NewsFetcher {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulated_news_generation() {
        let fetcher = NewsFetcher::new(None);
        let to_date = Utc::now();
        let from_date = to_date - Duration::days(7);

        let news = fetcher.generate_simulated_news(from_date, to_date);
        assert!(!news.articles.is_empty());
    }
}
