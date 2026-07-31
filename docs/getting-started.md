# Getting started

[English](getting-started.md) | [한국어](getting-started.ko.md)

The last two steps need an API key.

## Install

`maxt` is not published to a package registry. Rust 1.85 or newer.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 1. Pick an adapter

```rust
use maxt::adapters::{
    BinanceAdapter, BinanceMarket, BithumbAdapter, HyperliquidAdapter, UpbitAdapter,
};
use maxt::{Client, Exchange, Feature};

fn adapters() {
    let upbit = Client::new(UpbitAdapter::new());
    let bithumb = Client::new(BithumbAdapter::new());
    let binance_spot = Client::new(BinanceAdapter::spot());
    let binance_perp = Client::new(BinanceAdapter::usd_m_futures());
    let hyperliquid = Client::new(HyperliquidAdapter::new());

    assert_eq!(upbit.exchange(), Exchange::Upbit);
    // One exchange, two venues, fixed at construction.
    assert_eq!(binance_spot.adapter().venue(), BinanceMarket::Spot);
    assert_eq!(binance_perp.adapter().venue(), BinanceMarket::UsdMFutures);
    // Answered locally, before any request.
    assert!(hyperliquid.supports(Feature::FundingRates));
    assert!(!binance_spot.supports(Feature::FundingRates));
    assert!(!bithumb.supports(Feature::CandleStream));
}
```

Four adapter types, five venue configurations: `BinanceAdapter` covers both
Binance venues. Across exchanges the types differ, so one variable cannot hold
two of them. For a runtime choice, box the adapter as `Client<Box<dyn Adapter>>`,
the way [`examples/public_rest.rs`](../examples/public_rest.rs) does, and build
markets with `Market::new(exchange, kind, base, quote)`.

## 2. Read public market data

No credentials.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    let markets = client.markets(MarketKind::Spot).await?;
    println!("{} lists {} spot markets", client.exchange(), markets.len());

    let ticker = client.ticker(&market).await?;
    println!("{market} last {}", ticker.last_price);

    let book = client.order_book(&market, Some(5)).await?;
    println!("spread {:?}", book.spread());
    Ok(())
}
```

- A `Market` is an exchange, a kind, a base asset, and a quote asset. Spot and
  perpetual on one pair are two markets, not one with a flag.
- Prices and quantities are `Decimal`, never `f64`.
- A field the exchange does not publish is `None`, not zero.

## 3. Subscribe to a live feed

One subscription is one connection, however many markets and feeds it names.

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
    while let Some(event) = stream.next().await {
        match event {
            Ok(MarketEvent::Trade(t)) => println!("{} {} {:?}", t.price, t.quantity, t.taker_side),
            // The socket dropped and came back; what it published in between was missed.
            Ok(MarketEvent::Reconnected) => println!("reconnected; there is a gap behind us"),
            Ok(other) => println!("{other:?}"),
            // The stream carries on after reporting this.
            Err(error) => eprintln!("reported, still subscribed: {error}"),
        }
    }
    Ok(())
}
```

- Only `None` ends a stream. Match on the item, do not `?` it.
- Dropping the stream closes the connection.
- Bithumb publishes no candle stream, and a subscription that asks for one fails
  as a whole. Ask `client.supports(Feature::CandleStream)` first.
- `supports` answers per feature, not per argument: Upbit claims
  `Feature::CandleStream` and still refuses `Feed::Candles(Interval::Day1)` with
  `Error::Unsupported`
  ([the common API](common-api.md#feature-and-clientsupports)).

## 4. Add credentials and read the account

Credentials come from the environment, never from the source.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Feature};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));

    // Without credentials this is false, and the call is never made.
    if !client.supports(Feature::Balances) {
        return Ok(());
    }

    for balance in client.balances().await? {
        if !balance.total().is_zero() {
            println!("{} {} available", balance.asset, balance.available);
        }
    }
    for order in client.open_orders().await? {
        println!("{} {:?} {:?} (id {})", order.market, order.side, order.status, order.id);
    }
    Ok(())
}
```

A read-only key is enough for the above.

`client.subscribe_account()` streams `AccountEvent::Balance` and
`AccountEvent::Order`. After `AccountEvent::Reconnected`, re-read balances and
open orders over REST.

## 5. Place an order

Needs a trading key. Only Hyperliquid publishes a test network; on Upbit the
order below is real money.

`OrderRequest::limit` takes the market, the side, a `Size`, and the price, in
that order. Cancelling takes the exchange's own order id off the returned
`Order`.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Decimal, Exchange, Market, OrderRequest, Side, Size, TimeInForce};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    // The price comes off the book, not out of this page: a figure written here
    // fills as a taker the day the market drops to it. The deepest bid returned
    // is below every other bid, so a buy at it cannot cross the ask.
    let book = client.order_book(&market, None).await?;
    let Some(deepest_bid) = book.bids.last() else {
        println!("no bids on the book");
        return Ok(());
    };

    let order = client
        .place_order(
            &OrderRequest::limit(
                market.clone(),
                Side::Buy,
                // 0.001 BTC. `Size::Quote` here would have meant 0.001 KRW.
                Size::Base(Decimal::new(1, 3)),
                deepest_bid.price,
            )
            // Rejected outright rather than filled as a taker if the book moved
            // between that read and this call.
            .time_in_force(TimeInForce::PostOnly),
        )
        .await?;

    println!("{} is {:?}", order.id, order.status);

    // `is_live` is the test for "can still fill". A cancel races the book, so
    // trust the order that comes back, not the one that went in.
    if order.status.is_live() {
        let cancelled = client.cancel_order(&market, &order.id).await?;
        println!("{} filled, {} withdrawn", cancelled.filled_quantity, cancelled.remaining_quantity);
    }
    Ok(())
}
```

Tick size, lot step, and minimum order value are per-exchange, and only
Hyperliquid checks an order against them before signing:
[Order precision and minimum size](common-api.md#order-precision-and-minimum-size).
Hyperliquid has no market order type, and quote-denominated sizing is not
universal. Full order reference: [the common API](common-api.md#orders).

## Next

- `cargo run --example` [`public_rest`](../examples/public_rest.rs),
  [`public_stream`](../examples/public_stream.rs),
  [`private_account`](../examples/private_account.rs),
  [`private_stream`](../examples/private_stream.rs)
- [The common API](common-api.md), including
  [derivatives](common-api.md#a-worked-derivatives-read)
- [Choosing an exchange](providers.md): [Upbit](providers/upbit.md),
  [Bithumb](providers/bithumb.md), [Binance](providers/binance.md),
  [Hyperliquid](providers/hyperliquid.md)
