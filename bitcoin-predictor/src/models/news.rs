use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 뉴스 기사
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub description: Option<String>,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub url: String,
}

/// 뉴스 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsList {
    pub articles: Vec<NewsArticle>,
    pub query: String,
    pub from_date: DateTime<Utc>,
    pub to_date: DateTime<Utc>,
}

/// NewsAPI 응답 구조
#[derive(Debug, Deserialize)]
pub struct NewsApiResponse {
    pub status: String,
    pub articles: Vec<NewsApiArticle>,
}

#[derive(Debug, Deserialize)]
pub struct NewsApiArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub source: NewsApiSource,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewsApiSource {
    pub name: Option<String>,
}

/// 감성 분석 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub score: f64,        // -1.0 (매우 부정) ~ 1.0 (매우 긍정)
    pub positive_count: u32,
    pub negative_count: u32,
    pub neutral_count: u32,
    pub total_articles: u32,
    pub confidence: f64,   // 0.0 ~ 1.0
}

impl SentimentResult {
    pub fn new() -> Self {
        Self {
            score: 0.0,
            positive_count: 0,
            negative_count: 0,
            neutral_count: 0,
            total_articles: 0,
            confidence: 0.0,
        }
    }
}

impl Default for SentimentResult {
    fn default() -> Self {
        Self::new()
    }
}
