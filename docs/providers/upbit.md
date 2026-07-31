[English](upbit.md) | [한국어](upbit.ko.md)

# Upbit

Spot only, in four regional deployments, one per adapter, chosen at
construction.

```rust
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let korea = Client::new(UpbitAdapter::new());
let singapore = Client::new(UpbitAdapter::with_region(UpbitRegion::Singapore));
```

## What is supported

`UpbitRegion::Korea` is the default; `Singapore`, `Indonesia` and `Thailand`
are the others. They are separate exchanges, not mirrors: neither a listing nor
a credential carries across. `UpbitAdapter::region()` reports which.

Upbit writes the quote asset first, `KRW-BTC`. Pass
`Market::spot(Exchange::Upbit, "BTC", "KRW")`; `MarketInfo::native_symbol`
gives Upbit's own spelling back.

| Call | Condition |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | no credentials |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | credentials |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | `Error::Unsupported`, always. Upbit lists no derivatives |
| `reduce_only` on an order | `Error::Unsupported`. A spot order has no position to reduce |
| `markets(MarketKind::Perpetual)` | an empty list, not an error |

## Limits

Checked before the request is built.

| Call | Accepted range | Error |
| --- | --- | --- |
| `trades` | `limit` 1 to 500 | `Error::InvalidRequest` on `limit` |
| `order_book` | `depth` 1 to 30 per side | `Error::InvalidRequest` on `depth` |
| `candles` | any `limit`; 200 per Upbit response, paged for you over at most a hundred calls | `limit` of 0, a `from` not earlier than `to`, or a window past 20 000 candles, is `Error::InvalidRequest` |
| Asset codes | uppercase ASCII letters and digits | `Error::InvalidRequest` |

| Interval | REST `candles` | `Feed::Candles` |
| --- | --- | --- |
| 1s | yes | yes |
| 1m, 3m, 5m, 15m, 30m, 1h, 4h | yes | yes |
| 1d, 1w, 1M | yes | no |

The eleven REST intervals are the [baseline](../common-api.md#intervals) ten
plus `Sec1`. Upbit's ten-minute and yearly candles have no `Interval` name.

`Sec1` is the exception on history: about three months, against full history
elsewhere ([Upbit](https://global-docs.upbit.com/reference/list-candles-seconds)).

An interval outside a column is `Error::Unsupported` naming `Feature::Candles`
or `Feature::CandleStream`, **but only if the request is otherwise valid**. The
`limit`, window and paging checks above run first and name their own field
instead. Match both.

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::OrderBook` | **30 levels a side**, Upbit's own [default](https://global-docs.upbit.com/reference/websocket-orderbook). `Subscription` cannot narrow it |
| Book events | full snapshots, not diffs. Overwrite your copy on every one |
| A depth you chose | `Client::order_book` over REST, 1 to 30 levels |
| `Candle::closed` | `true` on one emission per window, sent when Upbit opens the next one |
| Candle events per window | many with `closed` false, then exactly one with it `true`. The settled one carries the figures of that window's own last frame |
| `subscribe_account` | the whole account, not a market list, one event per changed asset |

### `Candle::closed` on the stream

Upbit stops publishing a window the instant the next one opens, so a frame's
own `timestamp` never reaches `open_time + interval`. `closed` is read off the
transition to a later window, not off a clock.

| Frame on `candle.1m` `KRW-BTC`, 2026-07-30 | `candle_date_time_utc` | frame `timestamp` | Window end |
| --- | --- | --- | --- |
| last one 07:46 ever got | 07:46:00 | 07:46:58.309 | 07:47:00.000 |
| first one of the next window | 07:47:00 | 07:47:00.706 | 07:48:00.000 |

| Situation | Emission |
| --- | --- |
| A window is running | the forming bar, republished on every update, `closed` false |
| The next window opens | the settled bar first, then the new forming one |
| After a reconnect | the held bar is dropped, not settled. The first window to settle is the next one to open |
| A frame arrives behind one already seen | nothing |
| The final window when you drop the subscription | never settled |

Over REST, `closed` is decided against the reading machine's clock. `Month1`
is no exception, settling by the calendar: the June candle closes on 1 July.

## Quotas

| Group | Limit | Scope |
| --- | --- | --- |
| Public quotation: `markets`, `candles`, `trades`, `ticker`, `order_book` | 10 a second | IP |
| Exchange default: `balances`, `open_orders`, `cancel_order` | 30 a second | account |
| Order placement: `place_order` | 8 a second | account |
| New WebSocket connections | 5 a second | IP unauthenticated, account authenticated |
| Frames sent on one WebSocket | 5 a second and 100 a minute | connection |

A cancel-and-replace loop is bounded by `place_order`, not by `cancel_order`.

`maxt` does not throttle. Pacing is yours, and `Error::is_rate_limited()` is how
you learn you were too fast.

## Orders

| Order | Size | Price |
| --- | --- | --- |
| Limit, either side | `Size::Base` | required |
| Market buy | `Size::Quote`, the amount to spend | none |
| Market sell | `Size::Base`, the quantity to offer | none |

Any other pairing is `Error::InvalidRequest` on `size`, as is a zero or
negative price or quantity. `maxt` refuses it before signing.

| `TimeInForce` on a limit order | Wire value |
| --- | --- |
| `GoodTilCancelled` | nothing at all, which is the default |
| `ImmediateOrCancel` | `ioc` |
| `FillOrKill` | `fok` |
| `PostOnly` | `post_only` |

On a market order, `ImmediateOrCancel` is accepted and sent as nothing;
anything else fails on `time_in_force`.

## Order precision and minimum size

`maxt` checks only that a price and a quantity are above zero, so a price off
Upbit's tick or an order under their minimum is rejected by Upbit, not by
`Error::InvalidRequest` from here. Upbit publishes tick sizes as a band table
keyed by price range, and a minimum order amount per quote asset; no type in
`maxt` carries either. Read both from Upbit before your first order.

## Surprises

| Field or call | Behaviour |
| --- | --- |
| `Trade::id` | Upbit's `sequential_id`, kept as the digits it sent, the same value on the stream and over REST. Deduplicate on it |
| Ordering by `Trade::id` | do not. Upbit [documents](https://global-docs.upbit.com/reference/today-trades-history) it as a basis for uniqueness and does not guarantee the order of trades. Order by `Trade::timestamp` |
| `trades` order | newest-first, sorted here. The sort is stable, so trades sharing one millisecond keep Upbit's own order |
| `candles` order | oldest-first, though Upbit answers newest-first |
| An investment warning | `MarketStatus::Unknown`. Such a market is fully tradable. See [Warnings and cautions](#warnings-and-cautions) |
| An investment caution | `MarketStatus::Active`, unchanged. See [Warnings and cautions](#warnings-and-cautions) |
| `open_orders` | walks every page, 100 per page, asking for resting orders and orders waiting on a trigger |
| `cancel_order` | takes the market and the order id, though Upbit cancels by id alone |
| No credentials | `Error::Auth`, not `Error::Unsupported`, even though `supports(Feature::Balances)` is `false`. Match on both |
| A credential Upbit refused | `Error::Exchange`, not `Error::Auth`, carrying Upbit's own name: HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_access_key`, `nonce_used`, `no_authorization_ip`, `no_authorization_token`, and HTTP 403 `out_of_scope`. **Documented, not measured** |

## Warnings and cautions

`MarketStatus` has one value between Upbit's two designations, so only the
first reaches it.

| Designation | Meaning | `MarketStatus` | Source |
| --- | --- | --- | --- |
| Warning, 유의 종목 | designated by hand and announced. Upbit asks the project to resolve the cause and may end trading support if it is not resolved | `Unknown` | `MarketInfo::status`, or `market_events` |
| Caution, 주의 종목 | raised and cleared automatically against published criteria, one flag per criterion | `Active` | `market_events` only |

On 2026-07-30 Upbit listed 800 markets: 11 carried a warning and 190 carried at
least one caution, 175 of those `GLOBAL_PRICE_DIFFERENCES` alone. The caution
count moves through the day; the warning count does not. Bithumb's
`market_warning` is the same designation, and its caution sits on a
[separate endpoint](bithumb.md#warnings-and-alerts).

**The four deployments do not send the same field.** Korea sends
`market_event`, carrying both designations; the other three send the older
`market_warning`, the warning only. `MarketInfo::status` still means the same
thing on all four, and `UpbitMarketEvent::cautions` is empty outside Korea.

## Upbit-only calls

Through `Client::adapter()`. One call answers for many markets at once.

| Method | Returns | Cost |
| --- | --- | --- |
| `tickers(&[Market])` | one ticker per market | one of the ten public requests a second |
| `order_books(&[Market], Option<u32>)` | one book per market, depth capped at 30 per side | the same one |
| `market_events()` | one `UpbitMarketEvent` per market: the warning flag and the caution criteria by name | the same one |

`Client::ticker` and `Client::order_book` are the first two, with one market.

**Where the batch stops is undocumented on both sides.** Neither method caps
the market codes it joins, and Upbit publishes no cap, so the bound is whatever
URL length Upbit or a proxy in front of it accepts. Past it the call is
`Error::Exchange`, not `Error::InvalidRequest`. Thirty is well inside; test a
few hundred on your own path first.

```rust
use maxt::{Client, Exchange, Market, adapters::UpbitAdapter};

async fn breadth(client: &Client<UpbitAdapter>) -> maxt::Result<()> {
    let markets = [Market::spot(Exchange::Upbit, "BTC", "KRW")];
    let _tickers = client.adapter().tickers(&markets).await?;
    let _books = client.adapter().order_books(&markets, Some(5)).await?;
    Ok(())
}
```

## Credentials

An access key and a secret key, issued together, from the adapter's own region.
They unlock `Feature::Balances`, `Feature::OpenOrders`, `Feature::Trading` and
`Feature::AccountStream`.

```rust
use maxt::{Client, adapters::UpbitAdapter};

fn client() -> Client<UpbitAdapter> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    Client::new(UpbitAdapter::new().with_credentials(access_key, secret))
}
```

The secret never leaves the process. Keep keys out of source and give them the
narrowest permissions your program uses: read-only if you never place an order.

## Examples

`cargo run --example public_rest`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Upbit's own docs

| Subject | Pages |
| --- | --- |
| Quotas | [rate limits](https://global-docs.upbit.com/reference/rate-limits) |
| Public REST | [markets](https://global-docs.upbit.com/reference/list-trading-pairs.md) · [tickers](https://global-docs.upbit.com/reference/list-tickers.md) · [order books](https://global-docs.upbit.com/reference/list-orderbooks.md) · [trades](https://global-docs.upbit.com/reference/list-pair-trades.md) · [minute candles](https://global-docs.upbit.com/reference/list-candles-minutes.md) · [second candles](https://global-docs.upbit.com/reference/list-candles-seconds) |
| Private REST | [accounts](https://global-docs.upbit.com/reference/get-balance.md) · [open orders](https://global-docs.upbit.com/reference/list-open-orders.md) |
| WebSocket | [trades](https://global-docs.upbit.com/reference/websocket-trade.md) · [order books](https://global-docs.upbit.com/reference/websocket-orderbook) · [candles](https://global-docs.upbit.com/reference/websocket-candle.md) · [account orders](https://global-docs.upbit.com/reference/websocket-myorder.md) · [account assets](https://global-docs.upbit.com/reference/websocket-myasset.md) |

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
