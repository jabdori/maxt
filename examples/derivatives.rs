//! Read public Binance USD-M perpetual market data.
//!
//! ```text
//! cargo run --example derivatives
//! ```

use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, HistoryRequest, Market};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
    let adapter = BinanceAdapter::usd_m_futures();
    let mark = adapter.mark_price(&market).await?;
    let interest = adapter.open_interest(&market).await?;
    println!(
        "mark={} next funding={:?}",
        mark.mark_price, mark.next_funding_time
    );
    println!(
        "open interest={} at {}",
        interest.open_interest, interest.time
    );

    let client = Client::new(adapter);
    let funding = client
        .funding_rates(&HistoryRequest::new(market).limit(5))
        .await?;
    println!(
        "{} funding-rate rows; next={:?}",
        funding.items.len(),
        funding.next
    );
    Ok(())
}
