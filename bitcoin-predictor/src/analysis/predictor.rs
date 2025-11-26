use anyhow::Result;
use chrono::Utc;
use tracing::info;

use crate::analysis::{SentimentAnalyzer, TechnicalAnalyzer, QuantEngine, QuantPrediction};
use crate::data::{NewsFetcher, PriceFetcher};
use crate::models::{HistoricalPrices, NewsList, PredictionResult, PriceDirection, SentimentResult, TechnicalIndicators};

/// 비트코인 가격 예측 엔진
/// 퀀트 수준의 다중 요소 모델과 몬테카를로 시뮬레이션 기반 예측
pub struct PredictionEngine {
    price_fetcher: PriceFetcher,
    news_fetcher: NewsFetcher,
    technical_analyzer: TechnicalAnalyzer,
    sentiment_analyzer: SentimentAnalyzer,
    quant_engine: QuantEngine,
    /// 기술적 분석 가중치 (0.0 ~ 1.0)
    technical_weight: f64,
    /// 감성 분석 가중치 (0.0 ~ 1.0)
    sentiment_weight: f64,
}

impl PredictionEngine {
    pub fn new() -> Self {
        Self {
            price_fetcher: PriceFetcher::new(),
            news_fetcher: NewsFetcher::from_env(),
            technical_analyzer: TechnicalAnalyzer::new(),
            sentiment_analyzer: SentimentAnalyzer::new(),
            quant_engine: QuantEngine::new(),
            technical_weight: 0.7,  // 기술적 분석에 70% 가중치
            sentiment_weight: 0.3,  // 감성 분석에 30% 가중치
        }
    }

    /// 가중치 설정
    pub fn with_weights(mut self, technical: f64, sentiment: f64) -> Self {
        let total = technical + sentiment;
        self.technical_weight = technical / total;
        self.sentiment_weight = sentiment / total;
        self
    }

    /// 퀀트 예측 수행 (메인 함수)
    pub async fn predict(&self) -> Result<(PredictionResult, QuantPrediction)> {
        info!("Starting Quant-level Bitcoin price prediction...");
        info!("Running Monte Carlo simulation with 10,000 paths...");

        // 1. 데이터 수집 (병렬)
        let (prices_result, news_result, current_price_result) = tokio::join!(
            self.price_fetcher.fetch_comprehensive_data(90),
            self.news_fetcher.fetch_bitcoin_news(7),
            self.price_fetcher.fetch_current_price()
        );

        let prices = prices_result?;
        let news = news_result?;
        let current_price = current_price_result?;

        // 2. 기술적 분석
        let indicators = self.technical_analyzer.calculate_indicators(&prices);
        let technical_score = indicators.calculate_signal(current_price);

        // 3. 감성 분석
        let sentiment_result = self.sentiment_analyzer.analyze(&news);
        let sentiment_score = sentiment_result.score;

        // 4. 퀀트 엔진으로 종합 예측
        let quant_prediction = self.quant_engine.predict(
            &prices,
            &indicators,
            sentiment_score,
            current_price,
        );

        // 5. 기존 형식 호환을 위한 PredictionResult 생성
        let combined_score = quant_prediction.signal_strength;
        let direction = PriceDirection::from_score(combined_score);

        let result = PredictionResult {
            predicted_price: quant_prediction.expected_price,
            current_price,
            price_change_percent: quant_prediction.expected_return,
            direction,
            confidence: quant_prediction.upside_probability / 100.0,
            technical_score,
            sentiment_score,
            combined_score,
            prediction_date: Utc::now(),
            indicators,
        };

        info!(
            "Quant prediction complete: ${:.2} -> ${:.2} ({:+.2}%)",
            current_price, quant_prediction.expected_price, quant_prediction.expected_return
        );
        info!(
            "Upside probability: {:.1}%, Kelly position: {:.1}%",
            quant_prediction.upside_probability, quant_prediction.kelly_position * 100.0
        );

        Ok((result, quant_prediction))
    }

    /// 수동 예측 (데이터를 직접 제공)
    pub fn predict_with_data(
        &self,
        prices: &HistoricalPrices,
        news: &NewsList,
        current_price: f64,
    ) -> PredictionResult {
        // 기술적 분석
        let indicators = self.technical_analyzer.calculate_indicators(prices);
        let technical_score = indicators.calculate_signal(current_price);

        // 감성 분석
        let sentiment_result = self.sentiment_analyzer.analyze(news);
        let sentiment_score = sentiment_result.score;

        // 종합 점수
        let combined_score = self.calculate_combined_score(
            technical_score,
            sentiment_score,
            &sentiment_result,
        );

        // 가격 예측
        let (predicted_price, confidence) = self.calculate_predicted_price(
            current_price,
            combined_score,
            &indicators,
            &sentiment_result,
            prices,
        );

        let price_change_percent = (predicted_price - current_price) / current_price * 100.0;
        let direction = PriceDirection::from_score(combined_score);

        PredictionResult {
            predicted_price,
            current_price,
            price_change_percent,
            direction,
            confidence,
            technical_score,
            sentiment_score,
            combined_score,
            prediction_date: Utc::now(),
            indicators,
        }
    }

    /// 종합 점수 계산
    fn calculate_combined_score(
        &self,
        technical_score: f64,
        sentiment_score: f64,
        sentiment_result: &SentimentResult,
    ) -> f64 {
        // 감성 분석 신뢰도에 따른 동적 가중치 조정
        let adjusted_sentiment_weight = self.sentiment_weight * sentiment_result.confidence;
        let adjusted_technical_weight = self.technical_weight + (self.sentiment_weight - adjusted_sentiment_weight);

        let combined = technical_score * adjusted_technical_weight
            + sentiment_score * adjusted_sentiment_weight;

        combined.clamp(-1.0, 1.0)
    }

    /// 예측 가격 계산
    fn calculate_predicted_price(
        &self,
        current_price: f64,
        combined_score: f64,
        indicators: &TechnicalIndicators,
        sentiment: &SentimentResult,
        prices: &HistoricalPrices,
    ) -> (f64, f64) {
        // 변동성 기반 최대 예상 변동폭 계산
        let volatility_factor = (indicators.volatility / 100.0).clamp(0.01, 0.1);

        // 추세 분석
        let trend = self.technical_analyzer.calculate_trend(&prices.closes());

        // 예상 변동률 계산
        // combined_score (-1 ~ 1) * 변동성 기반 조정
        let base_change = combined_score * volatility_factor * 100.0;

        // 추세 보정
        let trend_adjustment = trend * volatility_factor * 50.0;

        // 최종 예상 변동률 (일일 변동률은 보통 제한적)
        let expected_change_percent = (base_change + trend_adjustment).clamp(-10.0, 10.0);

        let predicted_price = current_price * (1.0 + expected_change_percent / 100.0);

        // 신뢰도 계산
        let confidence = self.calculate_confidence(indicators, sentiment, prices);

        (predicted_price, confidence)
    }

    /// 예측 신뢰도 계산
    fn calculate_confidence(
        &self,
        indicators: &TechnicalIndicators,
        sentiment: &SentimentResult,
        prices: &HistoricalPrices,
    ) -> f64 {
        let mut confidence = 0.5; // 기본 신뢰도

        // 1. 데이터 양 기반 신뢰도
        let data_confidence = (prices.prices.len() as f64 / 90.0).min(1.0);
        confidence += data_confidence * 0.1;

        // 2. 지표 일관성 기반 신뢰도
        let technical_signal = indicators.calculate_signal(prices.latest_price().unwrap_or(0.0));
        let signal_strength = technical_signal.abs();
        confidence += signal_strength * 0.15;

        // 3. 감성 분석 신뢰도
        confidence += sentiment.confidence * 0.15;

        // 4. 변동성 기반 조정 (높은 변동성 = 낮은 신뢰도)
        let volatility_penalty = (indicators.volatility / 10.0).min(0.2);
        confidence -= volatility_penalty;

        // 5. RSI 극단값 신뢰도 보너스 (과매수/과매도는 예측하기 쉬움)
        if indicators.rsi_14 < 25.0 || indicators.rsi_14 > 75.0 {
            confidence += 0.1;
        }

        confidence.clamp(0.1, 0.95)
    }

    /// 퀀트 스타일 종합 리포트 생성
    pub fn generate_quant_report(&self, result: &PredictionResult, quant: &QuantPrediction) -> String {
        let mut report = String::new();

        report.push_str("=".repeat(70).as_str());
        report.push_str("\n         BITCOIN QUANTITATIVE ANALYSIS REPORT\n");
        report.push_str("         Multi-Factor Model + Monte Carlo Simulation\n");
        report.push_str("=".repeat(70).as_str());
        report.push('\n');

        report.push_str(&format!(
            "\nAnalysis Date: {}\n",
            result.prediction_date.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // 핵심 예측 결과
        report.push_str("\n");
        report.push_str("=".repeat(70).as_str());
        report.push_str("\n                    PREDICTION SUMMARY\n");
        report.push_str("=".repeat(70).as_str());
        report.push('\n');

        report.push_str(&format!("\n  Current Price:      ${:>12.2}\n", quant.current_price));
        report.push_str(&format!("  Expected Price:     ${:>12.2}  ({:+.2}%)\n",
            quant.expected_price, quant.expected_return));
        report.push_str(&format!("  95% Confidence:     ${:.2} ~ ${:.2}\n",
            quant.price_confidence_interval.0, quant.price_confidence_interval.1));
        report.push_str(&format!("\n  Upside Probability: {:>12.1}%\n", quant.upside_probability));
        report.push_str(&format!("  Signal Strength:    {:>+12.3}  ({})\n",
            quant.signal_strength, self.signal_interpretation(quant.signal_strength)));
        report.push_str(&format!("  Kelly Position:     {:>+12.1}%\n", quant.kelly_position * 100.0));

        // 다중 요소 분석
        report.push_str("\n");
        report.push_str("-".repeat(70).as_str());
        report.push_str("\n                    MULTI-FACTOR ANALYSIS\n");
        report.push_str("-".repeat(70).as_str());
        report.push('\n');

        let factors = &quant.factor_scores;
        report.push_str("\n  Factor              Score      Weight    Contribution\n");
        report.push_str("  ─────────────────────────────────────────────────────\n");
        report.push_str(&format!("  Trend             {:>+7.3}     {:>5.1}%     {:>+7.3}\n",
            factors.trend_factor, factors.weights.trend * 100.0,
            factors.trend_factor * factors.weights.trend));
        report.push_str(&format!("  Momentum          {:>+7.3}     {:>5.1}%     {:>+7.3}\n",
            factors.momentum_factor, factors.weights.momentum * 100.0,
            factors.momentum_factor * factors.weights.momentum));
        report.push_str(&format!("  Mean Reversion    {:>+7.3}     {:>5.1}%     {:>+7.3}\n",
            factors.mean_reversion_factor, factors.weights.mean_reversion * 100.0,
            factors.mean_reversion_factor * factors.weights.mean_reversion));
        report.push_str(&format!("  Volatility        {:>+7.3}     {:>5.1}%     {:>+7.3}\n",
            factors.volatility_factor, factors.weights.volatility * 100.0,
            factors.volatility_factor * factors.weights.volatility));
        report.push_str(&format!("  Sentiment         {:>+7.3}     {:>5.1}%     {:>+7.3}\n",
            factors.sentiment_factor, factors.weights.sentiment * 100.0,
            factors.sentiment_factor * factors.weights.sentiment));
        report.push_str("  ─────────────────────────────────────────────────────\n");
        report.push_str(&format!("  Combined Signal   {:>+7.3}\n", quant.signal_strength));

        // 리스크 메트릭스
        report.push_str("\n");
        report.push_str("-".repeat(70).as_str());
        report.push_str("\n                    RISK METRICS\n");
        report.push_str("-".repeat(70).as_str());
        report.push('\n');

        let risk = &quant.risk_metrics;
        report.push_str(&format!("\n  Daily Volatility:       {:>8.2}%\n", risk.daily_volatility));
        report.push_str(&format!("  Annualized Volatility:  {:>8.2}%\n", risk.annualized_volatility));
        report.push_str(&format!("  VaR (95%, 1-day):       {:>8.2}%\n", risk.var_95));
        report.push_str(&format!("  VaR (99%, 1-day):       {:>8.2}%\n", risk.var_99));
        report.push_str(&format!("  Max Expected Drawdown:  {:>8.2}%\n", risk.max_drawdown_expected));
        report.push_str(&format!("\n  Sharpe Ratio:           {:>8.2}\n", risk.sharpe_ratio));
        report.push_str(&format!("  Sortino Ratio:          {:>8.2}\n", risk.sortino_ratio));

        // 몬테카를로 시뮬레이션 결과
        report.push_str("\n");
        report.push_str("-".repeat(70).as_str());
        report.push_str("\n                    MONTE CARLO SIMULATION\n");
        report.push_str("-".repeat(70).as_str());
        report.push('\n');

        let sim = &quant.simulation_stats;
        report.push_str(&format!("\n  Simulations:  {:>10}\n", sim.num_simulations));
        report.push_str(&format!("  Mean Price:   ${:>12.2}\n", sim.mean_price));
        report.push_str(&format!("  Median Price: ${:>12.2}\n", sim.median_price));
        report.push_str(&format!("  Std Dev:      ${:>12.2}\n", sim.std_dev));
        report.push_str("\n  Price Distribution (7-day forecast):\n");
        report.push_str(&format!("    5th percentile:   ${:>12.2}\n", sim.percentile_5));
        report.push_str(&format!("    25th percentile:  ${:>12.2}\n", sim.percentile_25));
        report.push_str(&format!("    50th percentile:  ${:>12.2}\n", sim.median_price));
        report.push_str(&format!("    75th percentile:  ${:>12.2}\n", sim.percentile_75));
        report.push_str(&format!("    95th percentile:  ${:>12.2}\n", sim.percentile_95));

        // 기술적 지표 요약
        report.push_str("\n");
        report.push_str("-".repeat(70).as_str());
        report.push_str("\n                    TECHNICAL INDICATORS\n");
        report.push_str("-".repeat(70).as_str());
        report.push('\n');

        let ind = &result.indicators;
        report.push_str(&format!("\n  Moving Averages:\n"));
        report.push_str(&format!("    SMA 7/14/30:  ${:.2} / ${:.2} / ${:.2}\n",
            ind.sma_7, ind.sma_14, ind.sma_30));
        report.push_str(&format!("    EMA 12/26:    ${:.2} / ${:.2}\n",
            ind.ema_12, ind.ema_26));
        report.push_str(&format!("\n  Oscillators:\n"));
        report.push_str(&format!("    RSI (14):     {:.2}  {}\n",
            ind.rsi_14, self.rsi_interpretation(ind.rsi_14)));
        report.push_str(&format!("    MACD:         {:.2} (Signal: {:.2}, Hist: {:+.2})\n",
            ind.macd, ind.macd_signal, ind.macd_histogram));
        report.push_str(&format!("\n  Bollinger Bands:\n"));
        report.push_str(&format!("    Upper/Mid/Lower: ${:.2} / ${:.2} / ${:.2}\n",
            ind.bollinger_upper, ind.bollinger_middle, ind.bollinger_lower));

        // 투자 권고
        report.push_str("\n");
        report.push_str("=".repeat(70).as_str());
        report.push_str("\n                    INVESTMENT RECOMMENDATION\n");
        report.push_str("=".repeat(70).as_str());
        report.push('\n');

        let recommendation = self.generate_recommendation(quant);
        report.push_str(&format!("\n{}\n", recommendation));

        report.push_str("\n");
        report.push_str("=".repeat(70).as_str());
        report.push_str("\n  DISCLAIMER: This analysis is for educational purposes only.\n");
        report.push_str("  Past performance does not guarantee future results.\n");
        report.push_str("  Always do your own research before making investment decisions.\n");
        report.push_str("=".repeat(70).as_str());
        report.push('\n');

        report
    }

    /// 투자 권고 생성
    fn generate_recommendation(&self, quant: &QuantPrediction) -> String {
        let signal = quant.signal_strength;
        let kelly = quant.kelly_position;
        let upside_prob = quant.upside_probability;

        let mut rec = String::new();

        // 방향 판단
        let direction = if signal > 0.3 {
            "BULLISH (Strong Buy Signal)"
        } else if signal > 0.1 {
            "MODERATELY BULLISH (Buy Signal)"
        } else if signal < -0.3 {
            "BEARISH (Strong Sell Signal)"
        } else if signal < -0.1 {
            "MODERATELY BEARISH (Sell Signal)"
        } else {
            "NEUTRAL (Hold)"
        };

        rec.push_str(&format!("  Direction: {}\n\n", direction));

        // 포지션 권고
        rec.push_str("  Position Sizing (Kelly Criterion):\n");
        if kelly > 0.0 {
            rec.push_str(&format!("    - Recommended Long: {:.1}% of portfolio\n", kelly.abs() * 100.0));
        } else if kelly < 0.0 {
            rec.push_str(&format!("    - Recommended Short: {:.1}% of portfolio\n", kelly.abs() * 100.0));
        } else {
            rec.push_str("    - Recommended: Stay flat (no position)\n");
        }

        // 확률 기반 조언
        rec.push_str(&format!("\n  Probability Assessment:\n"));
        rec.push_str(&format!("    - {:.1}% chance of price increase\n", upside_prob));
        rec.push_str(&format!("    - {:.1}% chance of price decrease\n", 100.0 - upside_prob));

        // 리스크 경고
        let var = quant.risk_metrics.var_95;
        rec.push_str(&format!("\n  Risk Warning:\n"));
        rec.push_str(&format!("    - Potential 1-day loss (95% conf): {:.2}%\n", var));
        if quant.risk_metrics.annualized_volatility > 50.0 {
            rec.push_str("    - HIGH VOLATILITY: Reduce position size\n");
        }

        rec
    }

    fn signal_interpretation(&self, signal: f64) -> &str {
        if signal > 0.5 { "Strong Bullish" }
        else if signal > 0.2 { "Bullish" }
        else if signal > 0.0 { "Slightly Bullish" }
        else if signal > -0.2 { "Slightly Bearish" }
        else if signal > -0.5 { "Bearish" }
        else { "Strong Bearish" }
    }

    fn rsi_interpretation(&self, rsi: f64) -> &str {
        if rsi > 70.0 { "(Overbought)" }
        else if rsi < 30.0 { "(Oversold)" }
        else { "(Neutral)" }
    }

    /// 기존 형식 리포트 생성 (하위 호환)
    pub fn generate_report(&self, result: &PredictionResult) -> String {
        let mut report = String::new();

        report.push_str("=".repeat(60).as_str());
        report.push_str("\n           BITCOIN PRICE PREDICTION REPORT\n");
        report.push_str("=".repeat(60).as_str());
        report.push('\n');

        report.push_str(&format!(
            "\nPrediction Date: {}\n",
            result.prediction_date.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        report.push_str("\n--- PRICE PREDICTION ---\n");
        report.push_str(&format!("Current Price:   ${:>12.2}\n", result.current_price));
        report.push_str(&format!("Predicted Price: ${:>12.2}\n", result.predicted_price));
        report.push_str(&format!(
            "Expected Change: {:>+12.2}%\n",
            result.price_change_percent
        ));
        report.push_str(&format!(
            "Direction:       {:>12} ({})\n",
            result.direction.description(),
            result.direction.emoji()
        ));
        report.push_str(&format!(
            "Confidence:      {:>12.1}%\n",
            result.confidence * 100.0
        ));

        report.push_str("\n--- ANALYSIS SCORES ---\n");
        report.push_str(&format!(
            "Technical Score: {:>+12.3}\n",
            result.technical_score
        ));
        report.push_str(&format!(
            "Sentiment Score: {:>+12.3}\n",
            result.sentiment_score
        ));
        report.push_str(&format!(
            "Combined Score:  {:>+12.3}\n",
            result.combined_score
        ));

        report.push_str("\n--- TECHNICAL INDICATORS ---\n");
        let ind = &result.indicators;
        report.push_str(&format!("SMA (7/14/30):   ${:.2} / ${:.2} / ${:.2}\n",
            ind.sma_7, ind.sma_14, ind.sma_30));
        report.push_str(&format!("EMA (12/26):     ${:.2} / ${:.2}\n",
            ind.ema_12, ind.ema_26));
        report.push_str(&format!("RSI (14):        {:.2}\n", ind.rsi_14));
        report.push_str(&format!("MACD:            {:.2} (Signal: {:.2})\n",
            ind.macd, ind.macd_signal));
        report.push_str(&format!("Bollinger Bands: ${:.2} / ${:.2} / ${:.2}\n",
            ind.bollinger_upper, ind.bollinger_middle, ind.bollinger_lower));
        report.push_str(&format!("Momentum:        {:.2}\n", ind.momentum));
        report.push_str(&format!("Volatility:      {:.2}%\n", ind.volatility));

        report.push_str("\n--- INTERPRETATION ---\n");
        report.push_str(&self.interpret_indicators(result));

        report.push_str("\n");
        report.push_str("=".repeat(60).as_str());
        report.push_str("\nDISCLAIMER: This is for educational purposes only.\n");
        report.push_str("Do not use for actual trading decisions.\n");
        report.push_str("=".repeat(60).as_str());
        report.push('\n');

        report
    }

    /// 지표 해석
    fn interpret_indicators(&self, result: &PredictionResult) -> String {
        let mut interpretation = String::new();
        let ind = &result.indicators;
        let price = result.current_price;

        // RSI 해석
        if ind.rsi_14 < 30.0 {
            interpretation.push_str("- RSI indicates OVERSOLD condition (potential bounce)\n");
        } else if ind.rsi_14 > 70.0 {
            interpretation.push_str("- RSI indicates OVERBOUGHT condition (potential pullback)\n");
        } else {
            interpretation.push_str("- RSI is in neutral territory\n");
        }

        // 이동평균 해석
        if price > ind.sma_7 && ind.sma_7 > ind.sma_14 {
            interpretation.push_str("- Moving averages show BULLISH alignment\n");
        } else if price < ind.sma_7 && ind.sma_7 < ind.sma_14 {
            interpretation.push_str("- Moving averages show BEARISH alignment\n");
        } else {
            interpretation.push_str("- Moving averages show mixed signals\n");
        }

        // MACD 해석
        if ind.macd > ind.macd_signal {
            interpretation.push_str("- MACD is above signal line (bullish momentum)\n");
        } else {
            interpretation.push_str("- MACD is below signal line (bearish momentum)\n");
        }

        // 볼린저 밴드 해석
        if price > ind.bollinger_upper {
            interpretation.push_str("- Price is above upper Bollinger Band (extended)\n");
        } else if price < ind.bollinger_lower {
            interpretation.push_str("- Price is below lower Bollinger Band (extended)\n");
        } else {
            let position = (price - ind.bollinger_lower) / (ind.bollinger_upper - ind.bollinger_lower) * 100.0;
            interpretation.push_str(&format!("- Price is at {:.0}% of Bollinger Band range\n", position));
        }

        // 감성 분석 해석
        if result.sentiment_score > 0.3 {
            interpretation.push_str("- News sentiment is POSITIVE\n");
        } else if result.sentiment_score < -0.3 {
            interpretation.push_str("- News sentiment is NEGATIVE\n");
        } else {
            interpretation.push_str("- News sentiment is NEUTRAL\n");
        }

        interpretation
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NewsArticle, PriceData};
    use chrono::Duration;

    fn create_test_prices() -> HistoricalPrices {
        let mut prices = HistoricalPrices::new("BTC/USD");
        let base_price = 50000.0;
        let now = Utc::now();

        for i in 0..100 {
            let variation = (i as f64 * 0.1).sin() * 1000.0;
            let price = base_price + variation + (i as f64 * 10.0);

            prices.prices.push(PriceData {
                timestamp: now - Duration::days(100 - i),
                open: price - 50.0,
                high: price + 100.0,
                low: price - 100.0,
                close: price,
                volume: 1000000.0,
            });
        }

        prices
    }

    fn create_test_news() -> NewsList {
        let now = Utc::now();
        NewsList {
            articles: vec![
                NewsArticle {
                    title: "Bitcoin surges as institutional investors buy".to_string(),
                    description: Some("Major gains expected".to_string()),
                    source: "Test".to_string(),
                    published_at: now - Duration::days(1),
                    url: "https://example.com/1".to_string(),
                },
                NewsArticle {
                    title: "Crypto market shows bullish momentum".to_string(),
                    description: Some("Rally continues".to_string()),
                    source: "Test".to_string(),
                    published_at: now,
                    url: "https://example.com/2".to_string(),
                },
            ],
            query: "bitcoin".to_string(),
            from_date: now - Duration::days(7),
            to_date: now,
        }
    }

    #[test]
    fn test_prediction_with_data() {
        let engine = PredictionEngine::new();
        let prices = create_test_prices();
        let news = create_test_news();
        let current_price = 51000.0;

        let result = engine.predict_with_data(&prices, &news, current_price);

        assert!(result.predicted_price > 0.0);
        assert!(result.confidence > 0.0 && result.confidence <= 1.0);
        assert!(result.technical_score >= -1.0 && result.technical_score <= 1.0);
    }

    #[test]
    fn test_report_generation() {
        let engine = PredictionEngine::new();
        let prices = create_test_prices();
        let news = create_test_news();

        let result = engine.predict_with_data(&prices, &news, 51000.0);
        let report = engine.generate_report(&result);

        assert!(report.contains("BITCOIN PRICE PREDICTION"));
        assert!(report.contains("Current Price"));
        assert!(report.contains("RSI"));
    }
}
