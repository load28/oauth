use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 기술적 분석 지표
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub sma_7: f64,         // 7일 단순이동평균
    pub sma_14: f64,        // 14일 단순이동평균
    pub sma_30: f64,        // 30일 단순이동평균
    pub ema_12: f64,        // 12일 지수이동평균
    pub ema_26: f64,        // 26일 지수이동평균
    pub rsi_14: f64,        // 14일 RSI (0-100)
    pub macd: f64,          // MACD 라인
    pub macd_signal: f64,   // MACD 시그널 라인
    pub macd_histogram: f64,// MACD 히스토그램
    pub bollinger_upper: f64, // 볼린저 밴드 상단
    pub bollinger_middle: f64,// 볼린저 밴드 중간
    pub bollinger_lower: f64, // 볼린저 밴드 하단
    pub momentum: f64,      // 모멘텀
    pub volatility: f64,    // 변동성 (표준편차)
}

impl TechnicalIndicators {
    /// 기술적 분석 기반 신호 점수 (-1.0 ~ 1.0)
    pub fn calculate_signal(&self, current_price: f64) -> f64 {
        let mut signal = 0.0;
        let mut weight_sum = 0.0;

        // 이동평균 크로스오버 신호 (가중치: 0.2)
        let ma_signal = if current_price > self.sma_7 && self.sma_7 > self.sma_14 {
            1.0 // 강한 상승 추세
        } else if current_price < self.sma_7 && self.sma_7 < self.sma_14 {
            -1.0 // 강한 하락 추세
        } else if current_price > self.sma_14 {
            0.3 // 약한 상승
        } else {
            -0.3 // 약한 하락
        };
        signal += ma_signal * 0.2;
        weight_sum += 0.2;

        // RSI 신호 (가중치: 0.2)
        let rsi_signal = if self.rsi_14 < 30.0 {
            1.0 // 과매도 -> 상승 기대
        } else if self.rsi_14 > 70.0 {
            -1.0 // 과매수 -> 하락 기대
        } else if self.rsi_14 < 40.0 {
            0.5
        } else if self.rsi_14 > 60.0 {
            -0.5
        } else {
            0.0 // 중립
        };
        signal += rsi_signal * 0.2;
        weight_sum += 0.2;

        // MACD 신호 (가중치: 0.25)
        let macd_signal_value = if self.macd > self.macd_signal && self.macd_histogram > 0.0 {
            1.0 // 상승 크로스
        } else if self.macd < self.macd_signal && self.macd_histogram < 0.0 {
            -1.0 // 하락 크로스
        } else if self.macd > 0.0 {
            0.3
        } else {
            -0.3
        };
        signal += macd_signal_value * 0.25;
        weight_sum += 0.25;

        // 볼린저 밴드 신호 (가중치: 0.2)
        let bb_signal = if current_price < self.bollinger_lower {
            1.0 // 하단 이탈 -> 반등 기대
        } else if current_price > self.bollinger_upper {
            -1.0 // 상단 이탈 -> 조정 기대
        } else {
            let bb_position = (current_price - self.bollinger_lower)
                / (self.bollinger_upper - self.bollinger_lower);
            (0.5 - bb_position) * 2.0 // -1 ~ 1 사이 값
        };
        signal += bb_signal * 0.2;
        weight_sum += 0.2;

        // 모멘텀 신호 (가중치: 0.15)
        let momentum_signal = if self.momentum > 0.0 {
            (self.momentum / current_price * 100.0).min(1.0)
        } else {
            (self.momentum / current_price * 100.0).max(-1.0)
        };
        signal += momentum_signal * 0.15;
        weight_sum += 0.15;

        signal / weight_sum
    }
}

/// 예측 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub predicted_price: f64,
    pub current_price: f64,
    pub price_change_percent: f64,
    pub direction: PriceDirection,
    pub confidence: f64,           // 0.0 ~ 1.0
    pub technical_score: f64,      // -1.0 ~ 1.0
    pub sentiment_score: f64,      // -1.0 ~ 1.0
    pub combined_score: f64,       // -1.0 ~ 1.0
    pub prediction_date: DateTime<Utc>,
    pub indicators: TechnicalIndicators,
}

/// 가격 방향
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PriceDirection {
    StrongUp,
    Up,
    Neutral,
    Down,
    StrongDown,
}

impl PriceDirection {
    pub fn from_score(score: f64) -> Self {
        if score > 0.5 {
            PriceDirection::StrongUp
        } else if score > 0.15 {
            PriceDirection::Up
        } else if score < -0.5 {
            PriceDirection::StrongDown
        } else if score < -0.15 {
            PriceDirection::Down
        } else {
            PriceDirection::Neutral
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            PriceDirection::StrongUp => "++",
            PriceDirection::Up => "+",
            PriceDirection::Neutral => "=",
            PriceDirection::Down => "-",
            PriceDirection::StrongDown => "--",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            PriceDirection::StrongUp => "Strong Bullish",
            PriceDirection::Up => "Bullish",
            PriceDirection::Neutral => "Neutral",
            PriceDirection::Down => "Bearish",
            PriceDirection::StrongDown => "Strong Bearish",
        }
    }
}
