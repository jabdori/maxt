//! Public REST reads with no credentials: a market list, a ticker, an order book
//! with its spread, and the last few trades.
//!
//! Run it with no arguments for Binance's BTC/USDT, or name an exchange and a
//! pair.
//!
//! ```text
//! cargo run --example public_rest
//! cargo run --example public_rest -- upbit BTC KRW
//! cargo run --example public_rest -- bithumb ETH KRW
//! cargo run --example public_rest -- hyperliquid HYPE USDC
//! ```
//!
//! No environment variables. Every call below is a public endpoint.

use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::{Adapter, Client, Decimal, Market, MarketKind};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| "binance".to_string());
    let Some((client, home_quote)) = client_for(&name) else {
        eprintln!("unknown exchange {name:?}: try upbit, bithumb, binance, or hyperliquid");
        return Ok(());
    };
    let base = args.next().unwrap_or_else(|| "BTC".to_string());
    let quote = args.next().unwrap_or_else(|| home_quote.to_string());
    let market = Market::spot(client.exchange(), base, quote);

    let markets = client.markets(MarketKind::Spot).await?;
    println!(
        "\n{} lists {} spot markets:",
        client.exchange().display_name(),
        markets.len()
    );
    for info in markets.iter().take(5) {
        // `native_symbol` is the exchange's market identifier.
        println!("  {:<28} {}", info.market.to_string(), info.native_symbol);
    }

    let ticker = client.ticker(&market).await?;
    println!("\n{market}");
    println!("  price     {}", ticker.last_price);
    // Fields an exchange does not publish are None rather than zero, so print
    // only what actually arrived.
    if let Some(rate) = ticker.change_rate {
        println!("  change    {}%", (rate * Decimal::ONE_HUNDRED).round_dp(2));
    }
    if let (Some(high), Some(low)) = (ticker.high, ticker.low) {
        println!("  range     {low} .. {high}");
    }
    if let Some(volume) = ticker.volume {
        println!("  volume    {volume} {}", market.base);
    }

    let book = client.order_book(&market, Some(5)).await?;
    println!("\norder book, best five a side:");
    // Print asks in reverse so the best ask borders the bids.
    for level in book.asks.iter().take(5).rev() {
        println!("  ask {:>16} {:>18}", level.price, level.quantity);
    }
    for level in book.bids.iter().take(5) {
        println!("  bid {:>16} {:>18}", level.price, level.quantity);
    }
    match (book.spread(), book.mid_price()) {
        (Some(spread), Some(mid)) => println!("  spread {spread} around a mid of {mid}"),
        _ => println!("  one side of the book is empty, so there is no spread"),
    }

    // Five is within every adapter's per-call trade limit.
    println!("\nlast five trades:");
    for trade in client.trades(&market, Some(5)).await? {
        println!(
            "  {:>16} {:>18} {:?} at {}",
            trade.price, trade.quantity, trade.taker_side, trade.timestamp
        );
    }

    Ok(())
}

/// Selects a boxed adapter and its default quote asset.
fn client_for(name: &str) -> Option<(Client<Box<dyn Adapter>>, &'static str)> {
    Some(match name {
        "upbit" => (Client::new(Box::new(UpbitAdapter::new()) as _), "KRW"),
        "bithumb" => (Client::new(Box::new(BithumbAdapter::new()) as _), "KRW"),
        "binance" => (Client::new(Box::new(BinanceAdapter::spot()) as _), "USDT"),
        "hyperliquid" => (
            Client::new(Box::new(HyperliquidAdapter::new()) as _),
            "USDC",
        ),
        _ => return None,
    })
}
