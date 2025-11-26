//! Bitcoin Price Predictor
//!
//! A quantitative analysis tool for Bitcoin price prediction
//! using technical indicators and news sentiment analysis.
//!
//! # Features
//!
//! - **Technical Analysis**: SMA, EMA, RSI, MACD, Bollinger Bands, Momentum
//! - **Sentiment Analysis**: News-based sentiment scoring
//! - **Combined Prediction**: Weighted integration of technical and sentiment signals
//!
//! # Example
//!
//! ```rust,ignore
//! use bitcoin_predictor::analysis::PredictionEngine;
//!
//! #[tokio::main]
//! async fn main() {
//!     let engine = PredictionEngine::new();
//!     let result = engine.predict().await.unwrap();
//!     println!("Predicted price: ${:.2}", result.predicted_price);
//! }
//! ```

pub mod analysis;
pub mod data;
pub mod models;

pub use analysis::PredictionEngine;
