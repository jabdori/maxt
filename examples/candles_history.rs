//! Read recent Binance Spot candles without credentials.
//!
//! ```text
//! cargo run --example candles_history
//! ```
//!
//! The same `CandleRequest` shape is used by every common adapter. Its `from`
//! boundary is inclusive, its `to` boundary is exclusive, and results are
//! oldest first.

use maxt::adapters::BinanceAdapter;
use maxt::{CandleRequest, Client, Exchange, HistoryRequest, Interval, Market};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    let client = Client::new(BinanceAdapter::spot());

    let candles = client
        .candles(&CandleRequest::new(market.clone(), Interval::Min1).limit(5))
        .await?;
    for candle in candles {
        println!(
            "{} open={} high={} low={} close={} volume={}",
            candle.open_time, candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }

    // Private history uses the same pagination idea. Pass `page.next` back as
    // `HistoryRequest::cursor` only when the previous response supplies one.
    let next_page = HistoryRequest::new(market).limit(100);
    println!("private history request prepared: {next_page:?}");
    Ok(())
}
