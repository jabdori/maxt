# Getting started

[English](getting-started.md) | [한국어](getting-started.ko.md)

Five steps, each runnable on its own. Only the last two need an API key.

## Install

`maxt` is not published to a package registry, so depend on the repository. Rust
1.85 or newer. `futures-util` supplies the `StreamExt` that step 3 uses to pull
events off a subscription.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 1. Pick an adapter

An adapter talks to one exchange. `Client` wraps it and gives you the common
API.

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
    // What each one can do is answered locally, before any request.
    assert!(hyperliquid.supports(Feature::FundingRates));
    assert!(!binance_spot.supports(Feature::FundingRates));
    assert!(!bithumb.supports(Feature::CandleStream));
}
```

Those five are separate types, so a variable holding one cannot later hold
another. To choose the exchange at runtime, box the adapter as
`Client<Box<dyn Adapter>>`, the way
[`examples/public_rest.rs`](../examples/public_rest.rs) does. When the exchange
is a runtime value the market's kind usually is too, and
`Market::new(exchange, kind, base, quote)` is the constructor for that case.
[Choosing an exchange](providers.md) covers what each one cannot do.

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

Three rules hold everywhere in `maxt`:

- A `Market` is an exchange, a kind, a base asset, and a quote asset. The
  adapter translates it into whatever the exchange calls the same instrument.
  Spot and perpetual on one pair are two markets, not one with a flag.
- Prices and quantities are `Decimal`, never `f64`.
- A field the exchange does not publish is `None`, not zero. A `None`
  `ticker.volume` means the exchange said nothing about volume.

## 3. Subscribe to a live feed

A subscription names markets and feeds, and becomes one connection however many
of each it names.

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

Match on the item, do not `?` it.

- An `Err` is a report the stream polls past: a frame that could not be read, or
  a reconnect that has stopped looking transient. Only `None` means nothing more
  is coming, so returning on the first `Err` abandons a subscription that was
  about to recover.
- Dropping the stream closes the connection.
- Not every exchange carries every feed. Bithumb publishes no candle stream, and
  a subscription that asks for one fails as a whole; the feed is not silently
  dropped. Ask `client.supports(Feature::CandleStream)` first when the answer
  should change what your program does.
- A `true` from `supports` is not a promise about every argument. Upbit claims
  `Feature::CandleStream` and still refuses `Feed::Candles(Interval::Day1)`,
  because it streams no daily candle. Handle `Error::Unsupported` at the call
  even after checking, and see
  [the common API](common-api.md#feature-and-clientsupports).

## 4. Add credentials and read the account

Credentials come from the environment, never from the source. Each adapter takes
them in the form its exchange issues. Upbit's is an access key and a secret key.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Feature};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));

    // `supports` answers for the adapter as configured. Without credentials
    // this is false, and the call is never made.
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

`client.subscribe_account()` has the same shape as step 3 and yields
`AccountEvent::Balance` and `AccountEvent::Order`. Its `Reconnected` is heavier
than the market one: fills may have happened during the gap, so re-read balances
and open orders over REST before trusting a local view again.

## 5. Place an order

This one needs a trading key, and of the four exchanges only Hyperliquid
publishes a test network. On Upbit the order below is a real order against real
money, which is why it is priced at the deepest bid the book carries and
cancelled straight away.

`OrderRequest::limit` takes four things in order: the market, the side, the size
as a `Size`, and the price. `Size::Base` and `Size::Quote` name the asset the
number is in, so a market buy sized in won cannot be confused with one sized in
bitcoin. Cancelling takes the exchange's own order id off the returned `Order`.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Decimal, Exchange, Market, OrderRequest, Side, Size, TimeInForce};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    // The price comes off the book, not out of this page. A figure written here
    // rests only while the market stays above it and fills as a taker the day it
    // does not, with nothing to announce the change. The deepest bid the exchange
    // returned is below every other bid by construction, so a buy at it cannot
    // cross the ask whatever the market is doing.
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

Tick size, lot step, and minimum order value are per-exchange. Two of the five
venue configurations expose them, and only one of those checks an order against
them before signing; see
[Order precision and minimum size](common-api.md#order-precision-and-minimum-size)
before sizing anything real. Hyperliquid has no market order type at all, and
quote-denominated sizing is not universal, so read the provider page for the
exchange you picked. Full details are in
[the common API](common-api.md#orders).

## Next

- [`examples/`](../examples/) holds the four programs behind these steps:
  [`public_rest.rs`](../examples/public_rest.rs),
  [`public_stream.rs`](../examples/public_stream.rs),
  [`private_account.rs`](../examples/private_account.rs),
  [`private_stream.rs`](../examples/private_stream.rs). Run one with `cargo run
  --example public_rest`. `public_stream.rs` is step 3 with a trade count and a
  deadline, so it exits on its own.
- [The common API](common-api.md): errors, `Decimal`, timestamps, subscriptions,
  paging, and reaching an exchange's own typed methods. Nothing above touches
  the derivatives half, which is
  [worked through there](common-api.md#a-worked-derivatives-read): positions,
  margin, funding, and leverage on a perpetual venue.
- [Choosing an exchange](providers.md), then the page for the one you picked:
  [Upbit](providers/upbit.md), [Bithumb](providers/bithumb.md),
  [Binance](providers/binance.md), [Hyperliquid](providers/hyperliquid.md).
