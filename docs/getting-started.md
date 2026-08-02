# Getting started

[English](getting-started.md) | [한국어](getting-started.ko.md)

Public REST and market streams require no exchange account.

## Install

`maxt` requires Rust 1.85 or newer. The stream example uses
`futures_util::StreamExt`.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## Read market data

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    let markets = client.markets(MarketKind::Spot).await?;
    let ticker = client.ticker(&market).await?;
    let book = client.order_book(&market, Some(5)).await?;

    println!("{} spot markets", markets.len());
    println!("{market}: {}", ticker.last_price);
    println!("spread: {:?}", book.spread());
    Ok(())
}
```

- Common types and contracts: [Common API reference](common-api.md)
- Provider limits and field sources: [Provider support](providers.md)

## Open a market stream

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
            Err(error) => eprintln!("stream error: {error}"),
        }
    }

    Ok(())
}
```

See [Stream state and cleanup](common-api.md#state) for item errors,
reconnection, termination, and explicit cleanup.

## Next steps

- [Provider support](providers.md): constructors, credentials, and provider limits
- [Common API reference](common-api.md): requests, streams, errors, and private calls
- [Runnable examples](../examples/)
