//! Read a public Binance Spot price with no account configuration.
//!
//! ```text
//! cargo run --example binance_first_read
//! ```

use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    let client = Client::new(BinanceAdapter::spot());

    let ticker = client.ticker(&market).await?;
    let average = client.adapter().spot_average_price(&market).await?;
    println!("{}: last={}", ticker.market, ticker.last_price);
    println!(
        "Binance {}-minute average={}",
        average.minutes, average.price
    );
    Ok(())
}
