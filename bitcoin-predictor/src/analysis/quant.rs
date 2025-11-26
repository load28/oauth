//! 퀀트 분석 모듈
//!
//! 전문 퀀트 펀드에서 사용하는 수학적 방법론 기반 종합 예측 시스템
//! - 다중 요소 모델 (Multi-Factor Model)
//! - Z-Score 정규화
//! - 기하 브라운 운동 (GBM)
//! - 몬테카를로 시뮬레이션
//! - 켈리 기준 (Kelly Criterion)
//! - 리스크 메트릭스 (VaR, Sharpe Ratio)

use crate::models::{HistoricalPrices, TechnicalIndicators};
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};
use std::f64::consts::PI;

/// 퀀트 분석 엔진
pub struct QuantEngine {
    /// 몬테카를로 시뮬레이션 횟수
    simulation_count: usize,
    /// 예측 기간 (일)
    forecast_days: usize,
    /// 신뢰 수준 (VaR 계산용)
    confidence_level: f64,
}

/// 퀀트 예측 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantPrediction {
    /// 예측 가격 (기대값)
    pub expected_price: f64,
    /// 현재 가격
    pub current_price: f64,
    /// 예상 수익률 (%)
    pub expected_return: f64,
    /// 상승 확률 (%)
    pub upside_probability: f64,
    /// 가격 신뢰구간 (하한, 상한)
    pub price_confidence_interval: (f64, f64),
    /// 종합 신호 강도 (-1.0 ~ 1.0)
    pub signal_strength: f64,
    /// 개별 요소 점수
    pub factor_scores: FactorScores,
    /// 리스크 메트릭스
    pub risk_metrics: RiskMetrics,
    /// 권장 포지션 (켈리 기준)
    pub kelly_position: f64,
    /// 시뮬레이션 통계
    pub simulation_stats: SimulationStats,
}

/// 개별 요소 점수 (정규화된 Z-Score)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorScores {
    /// 추세 요소 (이동평균 기반)
    pub trend_factor: f64,
    /// 모멘텀 요소 (RSI, MACD 기반)
    pub momentum_factor: f64,
    /// 평균회귀 요소 (볼린저밴드 기반)
    pub mean_reversion_factor: f64,
    /// 변동성 요소
    pub volatility_factor: f64,
    /// 감성 요소
    pub sentiment_factor: f64,
    /// 각 요소 가중치
    pub weights: FactorWeights,
}

/// 요소 가중치 (IC 기반)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorWeights {
    pub trend: f64,
    pub momentum: f64,
    pub mean_reversion: f64,
    pub volatility: f64,
    pub sentiment: f64,
}

/// 리스크 메트릭스
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// 일일 변동성 (%)
    pub daily_volatility: f64,
    /// 연율화 변동성 (%)
    pub annualized_volatility: f64,
    /// VaR (Value at Risk) - 95%
    pub var_95: f64,
    /// VaR (Value at Risk) - 99%
    pub var_99: f64,
    /// 최대 예상 손실 (%)
    pub max_drawdown_expected: f64,
    /// 샤프 비율 (무위험 수익률 4% 가정)
    pub sharpe_ratio: f64,
    /// 소르티노 비율
    pub sortino_ratio: f64,
}

/// 시뮬레이션 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStats {
    /// 시뮬레이션 횟수
    pub num_simulations: usize,
    /// 평균 최종 가격
    pub mean_price: f64,
    /// 가격 중앙값
    pub median_price: f64,
    /// 가격 표준편차
    pub std_dev: f64,
    /// 5% 분위수 가격
    pub percentile_5: f64,
    /// 25% 분위수 가격
    pub percentile_25: f64,
    /// 75% 분위수 가격
    pub percentile_75: f64,
    /// 95% 분위수 가격
    pub percentile_95: f64,
}

impl QuantEngine {
    pub fn new() -> Self {
        Self {
            simulation_count: 10000,
            forecast_days: 7,
            confidence_level: 0.95,
        }
    }

    pub fn with_simulations(mut self, count: usize) -> Self {
        self.simulation_count = count;
        self
    }

    pub fn with_forecast_days(mut self, days: usize) -> Self {
        self.forecast_days = days;
        self
    }

    /// 종합 퀀트 예측 수행
    pub fn predict(
        &self,
        prices: &HistoricalPrices,
        indicators: &TechnicalIndicators,
        sentiment_score: f64,
        current_price: f64,
    ) -> QuantPrediction {
        let closes = prices.closes();
        let returns = self.calculate_returns(&closes);

        // 1. GBM 파라미터 추정
        let (mu, sigma) = self.estimate_gbm_params(&returns);

        // 2. 다중 요소 분석
        let factor_scores = self.calculate_factor_scores(
            indicators,
            current_price,
            sentiment_score,
            &closes,
        );

        // 3. 시그널 기반 드리프트 조정
        let adjusted_mu = self.adjust_drift(mu, &factor_scores);

        // 4. 몬테카를로 시뮬레이션
        let simulated_prices = self.monte_carlo_simulation(
            current_price,
            adjusted_mu,
            sigma,
        );

        // 5. 시뮬레이션 통계 계산
        let simulation_stats = self.calculate_simulation_stats(&simulated_prices);

        // 6. 리스크 메트릭스 계산
        let risk_metrics = self.calculate_risk_metrics(&returns, adjusted_mu, sigma);

        // 7. 켈리 기준 포지션 계산
        let kelly_position = self.calculate_kelly_position(
            &factor_scores,
            &risk_metrics,
            adjusted_mu,
        );

        // 8. 상승 확률 계산
        let upside_probability = self.calculate_upside_probability(&simulated_prices, current_price);

        // 9. 신뢰구간 계산
        let price_confidence_interval = (
            simulation_stats.percentile_5,
            simulation_stats.percentile_95,
        );

        let expected_return = (simulation_stats.mean_price - current_price) / current_price * 100.0;

        QuantPrediction {
            expected_price: simulation_stats.mean_price,
            current_price,
            expected_return,
            upside_probability,
            price_confidence_interval,
            signal_strength: factor_scores.combined_signal(),
            factor_scores,
            risk_metrics,
            kelly_position,
            simulation_stats,
        }
    }

    /// 일별 수익률 계산
    fn calculate_returns(&self, prices: &[f64]) -> Vec<f64> {
        prices
            .windows(2)
            .map(|w| (w[1] / w[0]).ln())
            .collect()
    }

    /// GBM 파라미터 추정 (최대 우도법)
    fn estimate_gbm_params(&self, returns: &[f64]) -> (f64, f64) {
        if returns.is_empty() {
            return (0.0, 0.02); // 기본값
        }

        let n = returns.len() as f64;

        // 드리프트 (연율화)
        let mu: f64 = returns.iter().sum::<f64>() / n * 365.0;

        // 변동성 (연율화)
        let mean = returns.iter().sum::<f64>() / n;
        let variance: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
        let sigma = variance.sqrt() * (365.0_f64).sqrt();

        (mu, sigma.max(0.01)) // 최소 변동성 보장
    }

    /// 다중 요소 점수 계산
    fn calculate_factor_scores(
        &self,
        indicators: &TechnicalIndicators,
        current_price: f64,
        sentiment_score: f64,
        prices: &[f64],
    ) -> FactorScores {
        // 1. 추세 요소 (Z-Score 정규화된 이동평균 위치)
        let trend_factor = self.calculate_trend_factor(indicators, current_price);

        // 2. 모멘텀 요소 (RSI, MACD 결합)
        let momentum_factor = self.calculate_momentum_factor(indicators);

        // 3. 평균회귀 요소 (볼린저밴드 위치)
        let mean_reversion_factor = self.calculate_mean_reversion_factor(indicators, current_price);

        // 4. 변동성 요소 (VIX-like)
        let volatility_factor = self.calculate_volatility_factor(indicators, prices);

        // 5. 감성 요소
        let sentiment_factor = sentiment_score;

        // IC 기반 가중치 (실제로는 과거 데이터로 회귀분석해야 함)
        // 여기서는 학술적 연구 기반 기본 가중치 사용
        let weights = FactorWeights {
            trend: 0.30,
            momentum: 0.25,
            mean_reversion: 0.20,
            volatility: 0.10,
            sentiment: 0.15,
        };

        FactorScores {
            trend_factor,
            momentum_factor,
            mean_reversion_factor,
            volatility_factor,
            sentiment_factor,
            weights,
        }
    }

    /// 추세 요소 계산
    fn calculate_trend_factor(&self, indicators: &TechnicalIndicators, current_price: f64) -> f64 {
        // 골든크로스/데드크로스 분석
        let short_term_trend = if current_price > indicators.sma_7 { 1.0 } else { -1.0 };
        let medium_term_trend = if indicators.sma_7 > indicators.sma_14 { 1.0 } else { -1.0 };
        let long_term_trend = if indicators.sma_14 > indicators.sma_30 { 1.0 } else { -1.0 };

        // EMA 추세
        let ema_trend = if indicators.ema_12 > indicators.ema_26 { 1.0 } else { -1.0 };

        // 가격과 이동평균 간 거리 (정규화)
        let price_distance = (current_price - indicators.sma_30) / indicators.sma_30;
        let distance_signal = (price_distance * 10.0).tanh(); // -1 ~ 1로 정규화

        // 가중 평균
        let trend = short_term_trend * 0.3
            + medium_term_trend * 0.25
            + long_term_trend * 0.15
            + ema_trend * 0.2
            + distance_signal * 0.1;

        trend.clamp(-1.0, 1.0)
    }

    /// 모멘텀 요소 계산
    fn calculate_momentum_factor(&self, indicators: &TechnicalIndicators) -> f64 {
        // RSI 신호 (비선형 변환)
        let rsi_normalized = (indicators.rsi_14 - 50.0) / 50.0; // -1 ~ 1
        let rsi_signal = if indicators.rsi_14 < 30.0 {
            // 과매도 - 강한 상승 신호
            1.0 - (indicators.rsi_14 / 30.0)
        } else if indicators.rsi_14 > 70.0 {
            // 과매수 - 강한 하락 신호
            -1.0 + ((100.0 - indicators.rsi_14) / 30.0)
        } else {
            rsi_normalized * 0.5
        };

        // MACD 신호
        let macd_signal = if indicators.macd_histogram.abs() > 0.0 {
            (indicators.macd_histogram / indicators.bollinger_middle.abs().max(1.0) * 100.0)
                .tanh()
        } else {
            0.0
        };

        // MACD 히스토그램 방향 변화 (가속도)
        let macd_direction = if indicators.macd > indicators.macd_signal {
            0.5
        } else {
            -0.5
        };

        // 모멘텀 지표
        let momentum_signal = (indicators.momentum / indicators.bollinger_middle.abs().max(1.0) * 10.0)
            .tanh();

        // 가중 결합
        let momentum = rsi_signal * 0.35
            + macd_signal * 0.30
            + macd_direction * 0.15
            + momentum_signal * 0.20;

        momentum.clamp(-1.0, 1.0)
    }

    /// 평균회귀 요소 계산
    fn calculate_mean_reversion_factor(&self, indicators: &TechnicalIndicators, current_price: f64) -> f64 {
        let bb_range = indicators.bollinger_upper - indicators.bollinger_lower;
        if bb_range <= 0.0 {
            return 0.0;
        }

        // 볼린저 밴드 내 위치 (0 ~ 1)
        let bb_position = (current_price - indicators.bollinger_lower) / bb_range;

        // 평균회귀 신호: 극단에서 중심으로 회귀 예상
        // 0.5가 중심, 0이나 1에 가까울수록 회귀 신호 강함
        let mean_reversion = if bb_position < 0.2 {
            // 하단 근처 - 상승 회귀 예상
            1.0 - (bb_position / 0.2)
        } else if bb_position > 0.8 {
            // 상단 근처 - 하락 회귀 예상
            -1.0 + ((1.0 - bb_position) / 0.2)
        } else {
            // 중간 영역 - 약한 신호
            (0.5 - bb_position) * 0.5
        };

        mean_reversion.clamp(-1.0, 1.0)
    }

    /// 변동성 요소 계산
    fn calculate_volatility_factor(&self, indicators: &TechnicalIndicators, prices: &[f64]) -> f64 {
        // 현재 변동성과 역사적 평균 비교
        let current_vol = indicators.volatility;

        // 볼린저 밴드 폭으로 변동성 체제 판단
        let bb_width = (indicators.bollinger_upper - indicators.bollinger_lower)
            / indicators.bollinger_middle * 100.0;

        // 변동성 수축 = 큰 움직임 예상 (방향은 다른 요소로 결정)
        // 변동성 확대 = 추세 지속 또는 반전 가능

        // 변동성 레짐: 낮은 변동성에서 높은 기대수익
        let vol_signal = if bb_width < 5.0 {
            0.3 // 변동성 수축 - 돌파 가능
        } else if bb_width > 15.0 {
            -0.2 // 변동성 확대 - 조심
        } else {
            0.0
        };

        // 변동성 추세
        let prices_len = prices.len();
        if prices_len > 20 {
            let recent_vol = self.calculate_volatility(&prices[prices_len-10..]);
            let older_vol = self.calculate_volatility(&prices[prices_len-20..prices_len-10]);

            if recent_vol > older_vol * 1.2 {
                return (vol_signal - 0.3).clamp(-1.0, 1.0);
            } else if recent_vol < older_vol * 0.8 {
                return (vol_signal + 0.3).clamp(-1.0, 1.0);
            }
        }

        vol_signal
    }

    fn calculate_volatility(&self, prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1] - w[0]) / w[0] * 100.0)
            .collect();

        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        variance.sqrt()
    }

    /// 드리프트 조정 (신호 기반)
    fn adjust_drift(&self, base_mu: f64, factors: &FactorScores) -> f64 {
        let signal = factors.combined_signal();

        // 신호 강도에 따라 드리프트 조정
        // 강한 신호일수록 드리프트 조정폭 증가
        let adjustment = signal * 0.3; // 최대 ±30% 연율화 수익률 조정

        base_mu + adjustment
    }

    /// 몬테카를로 시뮬레이션 (GBM 기반)
    fn monte_carlo_simulation(
        &self,
        initial_price: f64,
        mu: f64,
        sigma: f64,
    ) -> Vec<f64> {
        let dt = 1.0 / 365.0; // 일별 시간 단위
        let sqrt_dt = dt.sqrt();

        let mut final_prices = Vec::with_capacity(self.simulation_count);

        // 간단한 난수 생성기 (Box-Muller 변환)
        let mut seed = 42u64;

        for _ in 0..self.simulation_count {
            let mut price = initial_price;

            for _ in 0..self.forecast_days {
                // Box-Muller 변환으로 정규 난수 생성
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let u1 = (seed as f64 / u64::MAX as f64).max(1e-10);
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let u2 = seed as f64 / u64::MAX as f64;

                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();

                // GBM: dS = μS dt + σS dW
                let drift = (mu - 0.5 * sigma * sigma) * dt;
                let diffusion = sigma * sqrt_dt * z;

                price *= (drift + diffusion).exp();
            }

            final_prices.push(price);
        }

        // 정렬
        final_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
        final_prices
    }

    /// 시뮬레이션 통계 계산
    fn calculate_simulation_stats(&self, prices: &[f64]) -> SimulationStats {
        let n = prices.len();
        if n == 0 {
            return SimulationStats::default();
        }

        let mean_price: f64 = prices.iter().sum::<f64>() / n as f64;
        let median_price = prices[n / 2];

        let variance: f64 = prices.iter().map(|p| (p - mean_price).powi(2)).sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();

        SimulationStats {
            num_simulations: n,
            mean_price,
            median_price,
            std_dev,
            percentile_5: prices[(n as f64 * 0.05) as usize],
            percentile_25: prices[(n as f64 * 0.25) as usize],
            percentile_75: prices[(n as f64 * 0.75) as usize],
            percentile_95: prices[(n as f64 * 0.95) as usize],
        }
    }

    /// 리스크 메트릭스 계산
    fn calculate_risk_metrics(&self, returns: &[f64], mu: f64, sigma: f64) -> RiskMetrics {
        let daily_vol = sigma / (365.0_f64).sqrt() * 100.0;
        let annual_vol = sigma * 100.0;

        // VaR 계산 (파라메트릭 방법)
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_95 = normal.inverse_cdf(0.05);
        let z_99 = normal.inverse_cdf(0.01);

        let var_95 = -(mu / 365.0 + z_95 * sigma / (365.0_f64).sqrt()) * 100.0;
        let var_99 = -(mu / 365.0 + z_99 * sigma / (365.0_f64).sqrt()) * 100.0;

        // 최대 예상 손실 (Cornish-Fisher 확장)
        let max_drawdown = var_99 * 2.0; // 근사값

        // 샤프 비율 (무위험 수익률 4% 가정)
        let risk_free_rate = 0.04;
        let sharpe = if sigma > 0.0 {
            (mu - risk_free_rate) / sigma
        } else {
            0.0
        };

        // 소르티노 비율 (하방 변동성만 사용)
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();

        let downside_vol = if downside_returns.len() > 1 {
            let mean: f64 = downside_returns.iter().sum::<f64>() / downside_returns.len() as f64;
            let var: f64 = downside_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                / downside_returns.len() as f64;
            var.sqrt() * (365.0_f64).sqrt()
        } else {
            sigma
        };

        let sortino = if downside_vol > 0.0 {
            (mu - risk_free_rate) / downside_vol
        } else {
            0.0
        };

        RiskMetrics {
            daily_volatility: daily_vol,
            annualized_volatility: annual_vol,
            var_95,
            var_99,
            max_drawdown_expected: max_drawdown.abs(),
            sharpe_ratio: sharpe,
            sortino_ratio: sortino,
        }
    }

    /// 켈리 기준 포지션 계산
    fn calculate_kelly_position(
        &self,
        factors: &FactorScores,
        risk: &RiskMetrics,
        expected_return: f64,
    ) -> f64 {
        // Kelly Criterion: f* = (bp - q) / b
        // b = 승리 시 배당률, p = 승리 확률, q = 패배 확률

        let signal = factors.combined_signal();

        // 신호 기반 승리 확률 추정
        let win_prob = 0.5 + signal * 0.2; // 0.3 ~ 0.7
        let lose_prob = 1.0 - win_prob;

        // 예상 수익률 기반 배당률
        let win_return = expected_return.abs() / 100.0 + 0.01;
        let lose_return = risk.var_95 / 100.0;

        if lose_return <= 0.0 {
            return 0.0;
        }

        // Kelly 공식
        let kelly = (win_prob * win_return - lose_prob * lose_return)
            / (win_return * lose_return);

        // Half Kelly (리스크 관리)
        let half_kelly = kelly * 0.5;

        // -1 ~ 1 범위로 제한 (숏 포지션 허용)
        half_kelly.clamp(-1.0, 1.0)
    }

    /// 상승 확률 계산
    fn calculate_upside_probability(&self, simulated_prices: &[f64], current_price: f64) -> f64 {
        let upside_count = simulated_prices.iter()
            .filter(|&&p| p > current_price)
            .count();

        upside_count as f64 / simulated_prices.len() as f64 * 100.0
    }
}

impl Default for QuantEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FactorScores {
    /// 종합 신호 계산 (가중 평균)
    pub fn combined_signal(&self) -> f64 {
        let signal = self.trend_factor * self.weights.trend
            + self.momentum_factor * self.weights.momentum
            + self.mean_reversion_factor * self.weights.mean_reversion
            + self.volatility_factor * self.weights.volatility
            + self.sentiment_factor * self.weights.sentiment;

        signal.clamp(-1.0, 1.0)
    }
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self {
            num_simulations: 0,
            mean_price: 0.0,
            median_price: 0.0,
            std_dev: 0.0,
            percentile_5: 0.0,
            percentile_25: 0.0,
            percentile_75: 0.0,
            percentile_95: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_indicators() -> TechnicalIndicators {
        TechnicalIndicators {
            sma_7: 50500.0,
            sma_14: 50000.0,
            sma_30: 49000.0,
            ema_12: 50300.0,
            ema_26: 49800.0,
            rsi_14: 55.0,
            macd: 500.0,
            macd_signal: 400.0,
            macd_histogram: 100.0,
            bollinger_upper: 52000.0,
            bollinger_middle: 50000.0,
            bollinger_lower: 48000.0,
            momentum: 1000.0,
            volatility: 3.5,
        }
    }

    #[test]
    fn test_quant_prediction() {
        let engine = QuantEngine::new().with_simulations(1000);
        let indicators = create_test_indicators();

        let mut prices = HistoricalPrices::new("BTC/USD");
        for i in 0..100 {
            prices.prices.push(crate::models::PriceData {
                timestamp: chrono::Utc::now(),
                open: 49000.0 + i as f64 * 10.0,
                high: 49500.0 + i as f64 * 10.0,
                low: 48500.0 + i as f64 * 10.0,
                close: 49000.0 + i as f64 * 15.0,
                volume: 1000000.0,
            });
        }

        let prediction = engine.predict(&prices, &indicators, 0.3, 51000.0);

        assert!(prediction.expected_price > 0.0);
        assert!(prediction.upside_probability >= 0.0 && prediction.upside_probability <= 100.0);
        assert!(prediction.signal_strength >= -1.0 && prediction.signal_strength <= 1.0);
    }

    #[test]
    fn test_factor_scores() {
        let engine = QuantEngine::new();
        let indicators = create_test_indicators();

        let prices: Vec<f64> = (0..100).map(|i| 49000.0 + i as f64 * 20.0).collect();

        let factors = engine.calculate_factor_scores(&indicators, 51000.0, 0.2, &prices);

        assert!(factors.trend_factor >= -1.0 && factors.trend_factor <= 1.0);
        assert!(factors.momentum_factor >= -1.0 && factors.momentum_factor <= 1.0);
    }
}
