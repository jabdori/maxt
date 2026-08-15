//! Read several Upbit Korea quotation APIs in one program.
//!
//! ```text
//! cargo run --example upbit_provider
//! ```

use maxt::adapters::UpbitAdapter;
use maxt::{Exchange, Market};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let adapter = UpbitAdapter::new();
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    let tickers = adapter.tickers(std::slice::from_ref(&market)).await?;
    let instruments = adapter
        .orderbook_instruments(std::slice::from_ref(&market))
        .await?;
    println!(
        "region={:?}, ticker rows={}",
        adapter.region(),
        tickers.len()
    );
    println!("order-book instrument rows={}", instruments.len());
    Ok(())
}
