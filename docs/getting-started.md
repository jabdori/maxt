# Getting started

[English](getting-started.md) | [한국어](getting-started.ko.md)

This guide makes public REST calls and opens one public stream. Neither step
needs an exchange account.

## Install

`maxt` requires Rust 1.85 or newer and is installed from its Git repository.
`futures-util` supplies `StreamExt` for reading subscription events.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## Read public market data

An adapter selects the provider. `Client` exposes the common API, and `Market`
identifies the exchange, market kind, base asset, and quote asset.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    let listed = client.markets(MarketKind::Spot).await?;
    let ticker = client.ticker(&market).await?;
    let book = client.order_book(&market, Some(5)).await?;

    println!("{} spot markets", listed.len());
    println!("{market}: {}", ticker.last_price);
    println!("spread: {:?}", book.spread());
    Ok(())
}
```

The common data rules are:

- trades are newest-first; candles are oldest-first; order-book sides are best-first;
- prices, quantities, and amounts use `maxt::Decimal`, never `f64`;
- a value the provider does not publish is `None`, not zero;
- provider limits and timestamp details remain provider-specific.

See the [common API reference](common-api.md) before using fields for accounting
or execution decisions.

## Subscribe to a public stream

A `Subscription` is one logical stream. It applies every requested feed to every
requested market. Most adapters use one socket; Binance USD-M may split feeds
across multiple sockets and merge them into the returned stream.

```rust,no_run
use futures_util::StreamExt;
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Feed, Market, MarketEvent, Subscription};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let subscription = Subscription::new()
        .market(Market::spot(Exchange::Upbit, "BTC", "KRW"))
        .feed(Feed::Trades);

    let mut stream = client.subscribe(&subscription).await?;
    while let Some(item) = stream.next().await {
        match item {
            Ok(MarketEvent::Trade(trade)) => {
                println!("{} {}", trade.price, trade.quantity)
            }
            Ok(MarketEvent::Reconnected) => {
                println!("reconnected; events during the gap were missed")
            }
            Ok(_) => {}
            Err(error) => eprintln!("stream report: {error}"),
        }
    }

    Ok(())
}
```

An `Err` item reports a problem but does not end the stream. Only `None` means
the stream has ended. Dropping the stream closes its underlying connection or
connections. After an account-stream reconnect, re-read balances and open orders
over REST before trusting local state.

## Choose another provider

```rust
use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::Client;

fn clients() {
    let _upbit = Client::new(UpbitAdapter::new());
    let _bithumb = Client::new(BithumbAdapter::new());
    let _binance_spot = Client::new(BinanceAdapter::spot());
    let _binance_usd_m = Client::new(BinanceAdapter::usd_m_futures());
    let _hyperliquid = Client::new(HyperliquidAdapter::new());
}
```

Read [Choosing a provider](providers.md) for venue selection, supported candle
intervals, order shapes, and credentials.

## Before private calls

Configure credentials on the adapter before wrapping it in `Client`. Private
features return `false` from `Client::supports` until credentials are
configured, and a call without them returns `Error::Auth`. Private account and
trading paths have not been included in the live conformance check, so begin
with read-only permissions and verify provider-specific constraints directly.

Continue with the [common API reference](common-api.md) and the runnable
[`examples/`](../examples/).
