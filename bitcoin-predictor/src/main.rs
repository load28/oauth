use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use bitcoin_predictor::analysis::PredictionEngine;

#[derive(Parser)]
#[command(name = "bitcoin-predictor")]
#[command(author = "Bitcoin Predictor Team")]
#[command(version = "0.1.0")]
#[command(about = "Bitcoin price prediction using quantitative analysis and news sentiment")]
#[command(long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Technical analysis weight (0.0 - 1.0)
    #[arg(short, long, default_value_t = 0.7)]
    technical_weight: f64,

    /// Sentiment analysis weight (0.0 - 1.0)
    #[arg(short, long, default_value_t = 0.3)]
    sentiment_weight: f64,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run full prediction analysis
    Predict {
        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show only technical indicators
    Technical,

    /// Show current Bitcoin price
    Price,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load environment variables
    let _ = dotenvy::dotenv();

    info!("Bitcoin Price Predictor v0.1.0");

    match &cli.command {
        Some(Commands::Predict { format }) => {
            run_prediction(&cli, format).await?;
        }
        Some(Commands::Technical) => {
            run_technical_analysis().await?;
        }
        Some(Commands::Price) => {
            show_current_price().await?;
        }
        None => {
            // Default: run prediction
            run_prediction(&cli, "text").await?;
        }
    }

    Ok(())
}

async fn run_prediction(cli: &Cli, format: &str) -> Result<()> {
    info!("Running prediction analysis...");

    let engine = PredictionEngine::new()
        .with_weights(cli.technical_weight, cli.sentiment_weight);

    let result = engine.predict().await?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{}", json);
        }
        _ => {
            let report = engine.generate_report(&result);
            println!("{}", report);
        }
    }

    Ok(())
}

async fn run_technical_analysis() -> Result<()> {
    use bitcoin_predictor::analysis::TechnicalAnalyzer;
    use bitcoin_predictor::data::PriceFetcher;

    info!("Fetching price data...");

    let fetcher = PriceFetcher::new();
    let prices = fetcher.fetch_comprehensive_data(90).await?;
    let current_price = fetcher.fetch_current_price().await?;

    let analyzer = TechnicalAnalyzer::new();
    let indicators = analyzer.calculate_indicators(&prices);

    println!("\n=== TECHNICAL INDICATORS ===\n");
    println!("Current Price: ${:.2}\n", current_price);
    println!("Moving Averages:");
    println!("  SMA  7:  ${:.2}", indicators.sma_7);
    println!("  SMA 14:  ${:.2}", indicators.sma_14);
    println!("  SMA 30:  ${:.2}", indicators.sma_30);
    println!("  EMA 12:  ${:.2}", indicators.ema_12);
    println!("  EMA 26:  ${:.2}", indicators.ema_26);
    println!("\nOscillators:");
    println!("  RSI (14): {:.2}", indicators.rsi_14);
    println!("  MACD:     {:.2}", indicators.macd);
    println!("  Signal:   {:.2}", indicators.macd_signal);
    println!("  Histogram:{:.2}", indicators.macd_histogram);
    println!("\nBollinger Bands:");
    println!("  Upper:  ${:.2}", indicators.bollinger_upper);
    println!("  Middle: ${:.2}", indicators.bollinger_middle);
    println!("  Lower:  ${:.2}", indicators.bollinger_lower);
    println!("\nOther:");
    println!("  Momentum:   {:.2}", indicators.momentum);
    println!("  Volatility: {:.2}%", indicators.volatility);

    let signal = indicators.calculate_signal(current_price);
    println!("\nTechnical Signal: {:+.3}", signal);

    if signal > 0.3 {
        println!("Interpretation: BULLISH");
    } else if signal < -0.3 {
        println!("Interpretation: BEARISH");
    } else {
        println!("Interpretation: NEUTRAL");
    }

    Ok(())
}

async fn show_current_price() -> Result<()> {
    use bitcoin_predictor::data::PriceFetcher;

    let fetcher = PriceFetcher::new();
    let price = fetcher.fetch_current_price().await?;

    println!("\nBitcoin (BTC) Current Price");
    println!("===========================");
    println!("${:.2} USD\n", price);

    Ok(())
}
