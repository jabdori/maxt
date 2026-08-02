//! A live trade subscription with no credentials, printing trades as they
//! arrive and stopping on its own.
//!
//! Run it with no arguments for Upbit's BTC/KRW, or name an exchange and a
//! pair.
//!
//! ```text
//! cargo run --example public_stream
//! cargo run --example public_stream -- binance BTC USDT
//! cargo run --example public_stream -- bithumb ETH KRW
//! cargo run --example public_stream -- hyperliquid HYPE USDC
//! ```
//!
//! No environment variables. It exits after 20 trades or 30 seconds, whichever
//! comes first, so it never needs to be interrupted.

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::{Adapter, Client, Feature, Feed, Market, MarketEvent, Subscription};

const TRADE_LIMIT: usize = 20;
const TIME_LIMIT: Duration = Duration::from_secs(30);

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| "upbit".to_string());
    let Some((client, home_quote)) = client_for(&name) else {
        eprintln!("unknown exchange {name:?}: try upbit, bithumb, binance, or hyperliquid");
        return Ok(());
    };
    let base = args.next().unwrap_or_else(|| "BTC".to_string());
    let quote = args.next().unwrap_or_else(|| home_quote.to_string());
    let market = Market::spot(client.exchange(), base, quote);

    if !client.supports(Feature::TradeStream) {
        println!(
            "{} does not support {}",
            client.exchange(),
            Feature::TradeStream
        );
        return Ok(());
    }

    // A subscription names the requested market/feed pairs. Socket layout is an
    // adapter detail; Binance USD-M may split mixed feeds across two sockets.
    let subscription = Subscription::new()
        .market(market.clone())
        .feed(Feed::Trades);
    let mut stream = client.subscribe(&subscription).await?;
    println!("streaming trades on {market}, stopping after {TRADE_LIMIT} or {TIME_LIMIT:?}");

    let started = Instant::now();
    let mut seen = 0usize;

    while seen < TRADE_LIMIT {
        let Some(left) = TIME_LIMIT.checked_sub(started.elapsed()) else {
            break;
        };
        let Ok(item) = tokio::time::timeout(left, stream.next()).await else {
            println!("no trade before the {TIME_LIMIT:?} deadline. A quiet market, not a failure");
            break;
        };
        // Only `None` ends the stream. Report an `Err` item and keep polling.
        let Some(event) = item else {
            break;
        };
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                println!("-- {error}");
                continue;
            }
        };

        match event {
            MarketEvent::Trade(trade) => {
                seen += 1;
                println!(
                    "{seen:>3}. {:>16} {:>18} {:?} at {}",
                    trade.price, trade.quantity, trade.taker_side, trade.timestamp
                );
            }
            // A reconnect leaves a gap in events even though the adapter restores
            // the subscription.
            MarketEvent::Reconnected => {
                println!("-- reconnected; trades during the gap were missed")
            }
            other => println!("-- {other:?}"),
        }
    }

    // Dropping the stream closes every connection it owns.
    println!("done after {} trades in {:?}", seen, started.elapsed());
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
