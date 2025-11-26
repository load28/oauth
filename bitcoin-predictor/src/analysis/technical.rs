use crate::models::{HistoricalPrices, TechnicalIndicators};
use statrs::statistics::Statistics;
use tracing::info;

/// 기술적 분석기 - 퀀트 수준의 수학적 지표 계산
pub struct TechnicalAnalyzer;

impl TechnicalAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// 모든 기술적 지표 계산
    pub fn calculate_indicators(&self, prices: &HistoricalPrices) -> TechnicalIndicators {
        let closes = prices.closes();

        if closes.len() < 30 {
            return self.default_indicators();
        }

        info!("Calculating technical indicators for {} data points", closes.len());

        // 이동평균 계산
        let sma_7 = self.calculate_sma(&closes, 7);
        let sma_14 = self.calculate_sma(&closes, 14);
        let sma_30 = self.calculate_sma(&closes, 30);

        // 지수이동평균 계산
        let ema_12 = self.calculate_ema(&closes, 12);
        let ema_26 = self.calculate_ema(&closes, 26);

        // RSI 계산
        let rsi_14 = self.calculate_rsi(&closes, 14);

        // MACD 계산
        let (macd, macd_signal, macd_histogram) = self.calculate_macd(&closes);

        // 볼린저 밴드 계산
        let (bb_upper, bb_middle, bb_lower) = self.calculate_bollinger_bands(&closes, 20, 2.0);

        // 모멘텀 계산
        let momentum = self.calculate_momentum(&closes, 10);

        // 변동성 계산
        let volatility = self.calculate_volatility(&closes, 20);

        TechnicalIndicators {
            sma_7,
            sma_14,
            sma_30,
            ema_12,
            ema_26,
            rsi_14,
            macd,
            macd_signal,
            macd_histogram,
            bollinger_upper: bb_upper,
            bollinger_middle: bb_middle,
            bollinger_lower: bb_lower,
            momentum,
            volatility,
        }
    }

    /// 단순이동평균 (Simple Moving Average)
    fn calculate_sma(&self, data: &[f64], period: usize) -> f64 {
        if data.len() < period {
            return data.last().copied().unwrap_or(0.0);
        }

        let slice = &data[data.len() - period..];
        slice.iter().sum::<f64>() / period as f64
    }

    /// 지수이동평균 (Exponential Moving Average)
    fn calculate_ema(&self, data: &[f64], period: usize) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        if data.len() < period {
            return self.calculate_sma(data, data.len());
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = self.calculate_sma(&data[..period], period);

        for price in &data[period..] {
            ema = (price - ema) * multiplier + ema;
        }

        ema
    }

    /// 상대강도지수 (Relative Strength Index)
    fn calculate_rsi(&self, data: &[f64], period: usize) -> f64 {
        if data.len() < period + 1 {
            return 50.0; // 중립값
        }

        let mut gains: Vec<f64> = Vec::new();
        let mut losses: Vec<f64> = Vec::new();

        for i in 1..data.len() {
            let change = data[i] - data[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        // 최근 period 기간의 평균 상승/하락 계산
        let recent_gains = &gains[gains.len().saturating_sub(period)..];
        let recent_losses = &losses[losses.len().saturating_sub(period)..];

        let avg_gain = recent_gains.iter().sum::<f64>() / period as f64;
        let avg_loss = recent_losses.iter().sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// MACD (Moving Average Convergence Divergence)
    fn calculate_macd(&self, data: &[f64]) -> (f64, f64, f64) {
        let ema_12 = self.calculate_ema(data, 12);
        let ema_26 = self.calculate_ema(data, 26);

        let macd_line = ema_12 - ema_26;

        // MACD 라인의 EMA (시그널 라인)
        // 간단한 구현을 위해 최근 데이터로 근사
        let mut macd_values: Vec<f64> = Vec::new();
        for i in 26..data.len() {
            let slice = &data[..=i];
            let ema12 = self.calculate_ema(slice, 12);
            let ema26 = self.calculate_ema(slice, 26);
            macd_values.push(ema12 - ema26);
        }

        let signal_line = if macd_values.len() >= 9 {
            self.calculate_ema(&macd_values, 9)
        } else {
            macd_line
        };

        let histogram = macd_line - signal_line;

        (macd_line, signal_line, histogram)
    }

    /// 볼린저 밴드
    fn calculate_bollinger_bands(&self, data: &[f64], period: usize, std_dev: f64) -> (f64, f64, f64) {
        if data.len() < period {
            let middle = data.last().copied().unwrap_or(0.0);
            return (middle, middle, middle);
        }

        let slice = &data[data.len() - period..];
        let middle = slice.iter().sum::<f64>() / period as f64;

        let variance = slice.iter().map(|x| (x - middle).powi(2)).sum::<f64>() / period as f64;
        let std = variance.sqrt();

        let upper = middle + (std_dev * std);
        let lower = middle - (std_dev * std);

        (upper, middle, lower)
    }

    /// 모멘텀 (현재 가격 - N일 전 가격)
    fn calculate_momentum(&self, data: &[f64], period: usize) -> f64 {
        if data.len() < period + 1 {
            return 0.0;
        }

        let current = data[data.len() - 1];
        let past = data[data.len() - 1 - period];

        current - past
    }

    /// 변동성 (표준편차 기반)
    fn calculate_volatility(&self, data: &[f64], period: usize) -> f64 {
        if data.len() < period {
            return 0.0;
        }

        let slice = &data[data.len() - period..];

        // 일별 수익률 계산
        let returns: Vec<f64> = slice
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0] * 100.0)
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        returns.clone().std_dev()
    }

    /// 기본 지표 (데이터 부족 시)
    fn default_indicators(&self) -> TechnicalIndicators {
        TechnicalIndicators {
            sma_7: 0.0,
            sma_14: 0.0,
            sma_30: 0.0,
            ema_12: 0.0,
            ema_26: 0.0,
            rsi_14: 50.0,
            macd: 0.0,
            macd_signal: 0.0,
            macd_histogram: 0.0,
            bollinger_upper: 0.0,
            bollinger_middle: 0.0,
            bollinger_lower: 0.0,
            momentum: 0.0,
            volatility: 0.0,
        }
    }

    /// 가격 추세 분석 (선형 회귀)
    pub fn calculate_trend(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let n = data.len() as f64;
        let x_sum: f64 = (0..data.len()).map(|i| i as f64).sum();
        let y_sum: f64 = data.iter().sum();
        let xy_sum: f64 = data.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let x_sq_sum: f64 = (0..data.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_sq_sum - x_sum.powi(2));

        // 정규화된 추세 값 반환 (-1 ~ 1)
        let normalized_slope = slope / data.last().unwrap_or(&1.0) * 100.0;
        normalized_slope.clamp(-1.0, 1.0)
    }
}

impl Default for TechnicalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let analyzer = TechnicalAnalyzer::new();
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let sma = analyzer.calculate_sma(&data, 3);
        assert!((sma - 40.0).abs() < 0.01); // (30 + 40 + 50) / 3 = 40
    }

    #[test]
    fn test_rsi() {
        let analyzer = TechnicalAnalyzer::new();
        let data = vec![44.0, 44.25, 44.5, 43.75, 44.5, 44.25, 44.0, 43.5, 44.0, 44.5, 44.25, 44.0, 43.75, 44.0, 44.25];
        let rsi = analyzer.calculate_rsi(&data, 14);
        assert!(rsi >= 0.0 && rsi <= 100.0);
    }

    #[test]
    fn test_bollinger_bands() {
        let analyzer = TechnicalAnalyzer::new();
        let data: Vec<f64> = (1..=20).map(|x| x as f64 * 100.0).collect();
        let (upper, middle, lower) = analyzer.calculate_bollinger_bands(&data, 20, 2.0);
        assert!(upper > middle);
        assert!(middle > lower);
    }
}
