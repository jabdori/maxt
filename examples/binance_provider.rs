//! Use Binance provider-specific public reads alongside the common client.
//!
//! ```text
//! cargo run --example binance_provider
//! ```

use maxt::adapters::BinanceAdapter;
use maxt::{Exchange, Market};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let adapter = BinanceAdapter::spot();
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    let average = adapter.spot_average_price(&market).await?;
    let filters = adapter.spot_symbol_filters(&market).await?;
    let exchange = adapter.spot_exchange_info().await?;

    println!("{}-minute average: {}", average.minutes, average.price);
    println!("{} tick size: {:?}", filters.symbol, filters.tick_size);
    println!(
        "{} symbols in the provider response",
        exchange.symbols.len()
    );
    Ok(())
}
