//! Read Hyperliquid public market data and optionally address-scoped Info data.
//!
//! ```text
//! cargo run --example hyperliquid_provider
//! HYPERLIQUID_ADDRESS=0x... cargo run --example hyperliquid_provider
//! ```

use maxt::adapters::HyperliquidAdapter;
use maxt::{Exchange, Market};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let market = Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC");
    let adapter = HyperliquidAdapter::new();
    let mids = adapter.all_mids().await?;
    let book = adapter.l2_book(&market).await?;
    let trades = adapter.recent_trades(&market).await?;
    println!(
        "{} mid prices; {} book levels; {} recent trades",
        mids.len(),
        book.bids.len() + book.asks.len(),
        trades.len()
    );

    if let Ok(address) = std::env::var("HYPERLIQUID_ADDRESS") {
        let account = HyperliquidAdapter::new().with_query_address(address);
        let orders = account.basic_open_orders().await?;
        println!("{} address-scoped open orders", orders.len());
    } else {
        println!(
            "set HYPERLIQUID_ADDRESS to read an address-scoped Info response without a private key"
        );
    }
    Ok(())
}
