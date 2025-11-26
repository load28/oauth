use crate::models::{NewsList, SentimentResult};
use std::collections::HashMap;
use tracing::info;

/// 뉴스 감성 분석기
pub struct SentimentAnalyzer {
    positive_words: HashMap<&'static str, f64>,
    negative_words: HashMap<&'static str, f64>,
}

impl SentimentAnalyzer {
    pub fn new() -> Self {
        let mut positive_words = HashMap::new();
        let mut negative_words = HashMap::new();

        // 긍정적 키워드 (암호화폐/금융 관련)
        positive_words.insert("surge", 0.8);
        positive_words.insert("soar", 0.9);
        positive_words.insert("rally", 0.7);
        positive_words.insert("bullish", 0.9);
        positive_words.insert("gain", 0.6);
        positive_words.insert("gains", 0.6);
        positive_words.insert("rise", 0.5);
        positive_words.insert("rises", 0.5);
        positive_words.insert("rising", 0.5);
        positive_words.insert("jump", 0.7);
        positive_words.insert("jumps", 0.7);
        positive_words.insert("growth", 0.6);
        positive_words.insert("grow", 0.5);
        positive_words.insert("record", 0.6);
        positive_words.insert("high", 0.4);
        positive_words.insert("highs", 0.4);
        positive_words.insert("breakthrough", 0.8);
        positive_words.insert("adoption", 0.7);
        positive_words.insert("institutional", 0.5);
        positive_words.insert("investment", 0.5);
        positive_words.insert("invest", 0.5);
        positive_words.insert("investors", 0.4);
        positive_words.insert("inflow", 0.7);
        positive_words.insert("inflows", 0.7);
        positive_words.insert("positive", 0.5);
        positive_words.insert("optimistic", 0.7);
        positive_words.insert("optimism", 0.7);
        positive_words.insert("upgrade", 0.6);
        positive_words.insert("approval", 0.7);
        positive_words.insert("approved", 0.7);
        positive_words.insert("etf", 0.5);
        positive_words.insert("strong", 0.5);
        positive_words.insert("strength", 0.5);
        positive_words.insert("success", 0.6);
        positive_words.insert("successful", 0.6);
        positive_words.insert("profit", 0.6);
        positive_words.insert("profitable", 0.6);
        positive_words.insert("boost", 0.6);
        positive_words.insert("momentum", 0.5);
        positive_words.insert("recovery", 0.6);
        positive_words.insert("support", 0.4);
        positive_words.insert("demand", 0.5);

        // 부정적 키워드
        negative_words.insert("crash", 0.9);
        negative_words.insert("plunge", 0.9);
        negative_words.insert("plummet", 0.9);
        negative_words.insert("bearish", 0.9);
        negative_words.insert("fall", 0.5);
        negative_words.insert("falls", 0.5);
        negative_words.insert("falling", 0.5);
        negative_words.insert("drop", 0.6);
        negative_words.insert("drops", 0.6);
        negative_words.insert("decline", 0.6);
        negative_words.insert("declining", 0.6);
        negative_words.insert("loss", 0.6);
        negative_words.insert("losses", 0.6);
        negative_words.insert("low", 0.4);
        negative_words.insert("lows", 0.4);
        negative_words.insert("fear", 0.7);
        negative_words.insert("fears", 0.7);
        negative_words.insert("concern", 0.5);
        negative_words.insert("concerns", 0.5);
        negative_words.insert("worried", 0.5);
        negative_words.insert("worry", 0.5);
        negative_words.insert("risk", 0.4);
        negative_words.insert("risks", 0.4);
        negative_words.insert("risky", 0.5);
        negative_words.insert("volatile", 0.4);
        negative_words.insert("volatility", 0.3);
        negative_words.insert("ban", 0.8);
        negative_words.insert("banned", 0.8);
        negative_words.insert("regulation", 0.3);
        negative_words.insert("regulatory", 0.3);
        negative_words.insert("crackdown", 0.8);
        negative_words.insert("hack", 0.9);
        negative_words.insert("hacked", 0.9);
        negative_words.insert("scam", 0.9);
        negative_words.insert("fraud", 0.9);
        negative_words.insert("collapse", 0.9);
        negative_words.insert("outflow", 0.6);
        negative_words.insert("outflows", 0.6);
        negative_words.insert("sell", 0.4);
        negative_words.insert("selling", 0.5);
        negative_words.insert("selloff", 0.7);
        negative_words.insert("negative", 0.5);
        negative_words.insert("pessimistic", 0.7);
        negative_words.insert("uncertainty", 0.5);
        negative_words.insert("uncertain", 0.5);
        negative_words.insert("warning", 0.6);
        negative_words.insert("warn", 0.5);

        Self {
            positive_words,
            negative_words,
        }
    }

    /// 뉴스 목록 감성 분석
    pub fn analyze(&self, news: &NewsList) -> SentimentResult {
        if news.articles.is_empty() {
            return SentimentResult::new();
        }

        info!("Analyzing sentiment for {} articles", news.articles.len());

        let mut positive_count = 0u32;
        let mut negative_count = 0u32;
        let mut neutral_count = 0u32;
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        for (i, article) in news.articles.iter().enumerate() {
            let text = format!(
                "{} {}",
                article.title.to_lowercase(),
                article
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase())
                    .unwrap_or_default()
            );

            let (score, pos_matches, neg_matches) = self.analyze_text(&text);

            // 최근 뉴스에 더 높은 가중치 부여
            let recency_weight = 1.0 + (i as f64 / news.articles.len() as f64);

            if score > 0.1 {
                positive_count += 1;
            } else if score < -0.1 {
                negative_count += 1;
            } else {
                neutral_count += 1;
            }

            total_score += score * recency_weight;
            weight_sum += recency_weight;

            info!(
                "Article '{}...': score={:.2}, +{}, -{}",
                &article.title[..article.title.len().min(40)],
                score,
                pos_matches,
                neg_matches
            );
        }

        let avg_score = if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            0.0
        };

        // 신뢰도 계산 (분석된 기사 수와 일관성 기반)
        let total = positive_count + negative_count + neutral_count;
        let max_category = positive_count.max(negative_count).max(neutral_count);
        let consistency = max_category as f64 / total as f64;
        let sample_confidence = (total as f64 / 10.0).min(1.0); // 10개 이상이면 최대 신뢰도
        let confidence = consistency * 0.5 + sample_confidence * 0.5;

        let result = SentimentResult {
            score: avg_score.clamp(-1.0, 1.0),
            positive_count,
            negative_count,
            neutral_count,
            total_articles: total,
            confidence,
        };

        info!(
            "Sentiment analysis complete: score={:.2}, confidence={:.2}",
            result.score, result.confidence
        );

        result
    }

    /// 텍스트 감성 분석
    fn analyze_text(&self, text: &str) -> (f64, u32, u32) {
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .collect();

        let mut positive_score = 0.0;
        let mut negative_score = 0.0;
        let mut pos_matches = 0u32;
        let mut neg_matches = 0u32;

        for word in &words {
            if let Some(&score) = self.positive_words.get(word) {
                positive_score += score;
                pos_matches += 1;
            }
            if let Some(&score) = self.negative_words.get(word) {
                negative_score += score;
                neg_matches += 1;
            }
        }

        // 부정어 처리 (간단한 구현)
        let negation_words = ["not", "no", "never", "neither", "without", "lack"];
        let negation_count = words
            .iter()
            .filter(|w| negation_words.contains(w))
            .count() as f64;

        // 부정어가 있으면 감성 점수 일부 반전
        let negation_factor = 1.0 - (negation_count * 0.2).min(0.5);

        let raw_score = positive_score - negative_score;
        let normalized_score = if pos_matches + neg_matches > 0 {
            raw_score / (pos_matches + neg_matches) as f64
        } else {
            0.0
        };

        (normalized_score * negation_factor, pos_matches, neg_matches)
    }

    /// 단일 텍스트 감성 점수 반환
    pub fn analyze_single(&self, text: &str) -> f64 {
        let (score, _, _) = self.analyze_text(&text.to_lowercase());
        score
    }
}

impl Default for SentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let analyzer = SentimentAnalyzer::new();
        let score = analyzer.analyze_single("Bitcoin surges to new record highs as institutional investors buy");
        assert!(score > 0.0);
    }

    #[test]
    fn test_negative_sentiment() {
        let analyzer = SentimentAnalyzer::new();
        let score = analyzer.analyze_single("Bitcoin crashes amid fears of regulatory crackdown");
        assert!(score < 0.0);
    }

    #[test]
    fn test_neutral_sentiment() {
        let analyzer = SentimentAnalyzer::new();
        let score = analyzer.analyze_single("Bitcoin price remains stable");
        assert!(score.abs() < 0.5);
    }
}
