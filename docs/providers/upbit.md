[English](upbit.md) | [한국어](upbit.ko.md)

# Upbit

`UpbitAdapter` supports Upbit's four separate spot exchanges. Choose one region when constructing the adapter.

## Connect

```rust
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let korea = Client::new(UpbitAdapter::new());
let singapore = Client::new(UpbitAdapter::with_region(UpbitRegion::Singapore));
```

| Region | Value | REST base URL | Public WebSocket |
| --- | --- | --- | --- |
| Korea, default | `UpbitRegion::Korea` | `https://api.upbit.com` | `wss://api.upbit.com/websocket/v1` |
| Singapore | `UpbitRegion::Singapore` | `https://sg-api.upbit.com` | `wss://sg-api.upbit.com/websocket/v1` |
| Indonesia | `UpbitRegion::Indonesia` | `https://id-api.upbit.com` | `wss://id-api.upbit.com/websocket/v1` |
| Thailand | `UpbitRegion::Thailand` | `https://th-api.upbit.com` | `wss://th-api.upbit.com/websocket/v1` |

Listings, books, accounts, and credentials are isolated by region. A key issued in one region cannot authenticate another; `UpbitAdapter::region()` returns the selected region.

Upbit writes the quote asset first: `KRW-BTC`. In `maxt`, pass `Market::spot(Exchange::Upbit, "BTC", "KRW")`; `MarketInfo::native_symbol` returns the Upbit spelling.

```text
cargo run --example public_rest -- upbit BTC KRW
```

## Feature support

| Surface | Support |
| --- | --- |
| Public REST | `markets`, `trades`, `order_book`, `ticker`, and `candles`, without credentials |
| Public stream | trades, order books, tickers, and candles through `subscribe` or `subscribe_with` |
| Account and orders | balances, open orders, trading, and account streams after `with_credentials` |
| Derivatives | unsupported; Upbit lists spot markets only |
| `markets(MarketKind::Perpetual)` | empty list |

## REST limits and candle intervals

| Input | Accepted value |
| --- | --- |
| `trades` limit | 1–500 |
| `order_book` depth | 1–30 levels per side; omitted means Upbit's 30-level default |
| `candles` page | 200 candles per Upbit response; `maxt` walks at most 100 pages, or 20,000 candles |
| `candles` request | `limit > 0`, and `from < to` when both are present |
| Asset code | uppercase ASCII letters and digits |

| Surface | Upbit officially supports | Exposed by `maxt` |
| --- | --- | --- |
| REST candles | 1s; 1, 3, 5, 10, 15, 30, 60, 240m; 1d, 1w, 1M, 1y | 1s, 1m, 3m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M |
| WebSocket candles | 1s; 1, 3, 5, 10, 15, 30, 60, 240m | 1s, 1m, 3m, 5m, 15m, 30m, 1h, 4h |

`Interval` has no 10-minute or yearly variant, so those official Upbit intervals are not exposed. Second-candle REST history covers only the most recent three months. REST candles are returned oldest first by `maxt`.

## Streams

| Feed | Behaviour |
| --- | --- |
| `Feed::OrderBook` | full snapshots, 30 levels per side by default; `Subscription` cannot select a smaller depth |
| Trades | one event per execution; `Trade::id` is Upbit's `sequential_id` |
| Tickers | full snapshots; later events replace earlier state |
| Candles | repeated forming bars plus the completion rules below |

For streamed candles:

- A snapshot whose window has already ended is emitted immediately with `closed = true`.
- A snapshot or update for the current window has `closed = false` and replaces the held forming bar.
- When a later window arrives, the held window is emitted settled at most once, before the new forming bar.
- If no trade creates a candle, or no later frame arrives, there is no transition-based settled event.

REST candle completion is determined from the window end and the reading clock, including calendar-month boundaries.

## Rate limits

| Group | Limit | Scope |
| --- | --- | --- |
| `market` | 10 requests/second | IP |
| `candle` | 10 requests/second | IP |
| `trade` | 10 requests/second | IP |
| `ticker` | 10 requests/second | IP |
| `orderbook` | 10 requests/second | IP |
| WebSocket connections | 5/second | IP without authentication; account with authentication |
| WebSocket request messages | 5/second and 100/minute | connection |

The five public REST groups have independent counters. `maxt` does not throttle; use `Error::is_rate_limited()` and Upbit's `Remaining-Req` response header.

## Upbit-specific methods

Use these through `Client::adapter()` when one request should cover several markets.

| Method | Result | Rate-limit group |
| --- | --- | --- |
| `tickers(&[Market])` | one ticker per returned market | `ticker` |
| `order_books(&[Market], Option<u32>)` | one book per returned market, up to 30 levels per side | `orderbook` |
| `market_events()` | each market's warning and available caution criteria | `market` |

Upbit publishes no maximum batch market count; the practical bound is the accepted request-URL length.

### Warnings and cautions

| Designation | Common status | Detailed source |
| --- | --- | --- |
| Investment warning | `MarketStatus::Unknown` | `MarketInfo::status` and `market_events()` |
| Investment caution criteria | status remains `MarketStatus::Active` | `market_events()` |

Korea returns `market_event` with the warning and caution flags. Singapore, Indonesia, and Thailand return the older `market_warning`, so `UpbitMarketEvent::cautions` is empty there. `MarketStatus::Active` is not a guarantee that an order is currently accepted; check the current market and order policy before relying on availability.

## Credentials

```rust,no_run
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
let adapter = UpbitAdapter::with_region(UpbitRegion::Korea)
    .with_credentials(access_key, secret_key);
let client = Client::new(adapter);
```

Use a key pair from the same region and grant only the permissions required. The [`private_account`](../../examples/private_account.rs) and [`private_stream`](../../examples/private_stream.rs) examples are Upbit-specific and read-only.

## Verification scope

- On 2026-07-31, Korea received a representative public REST and WebSocket smoke test; the documented example command completed successfully.
- Singapore, Indonesia, and Thailand received public REST spot checks for listings, ticker, book, trades, and minute candles.
- Private live calls were not verified. Private behaviour is covered by offline tests and official documentation only.

## Official documentation

| Subject | Links |
| --- | --- |
| Regions and endpoints | [Global overview](https://global-docs.upbit.com/reference/api-overview) |
| Public REST | [pairs](https://global-docs.upbit.com/reference/list-trading-pairs) · [trades](https://global-docs.upbit.com/reference/list-pair-trades) · [tickers](https://global-docs.upbit.com/reference/list-tickers) · [order books](https://global-docs.upbit.com/reference/list-orderbooks) · [candles](https://global-docs.upbit.com/reference/list-candles-minutes) |
| WebSocket | [guide](https://global-docs.upbit.com/reference/websocket-guide) · [trades](https://global-docs.upbit.com/reference/websocket-trade) · [tickers](https://global-docs.upbit.com/reference/websocket-ticker) · [order books](https://global-docs.upbit.com/reference/websocket-orderbook) · [candles](https://global-docs.upbit.com/reference/websocket-candle) |
| Limits and authentication | [rate limits](https://global-docs.upbit.com/reference/rate-limits) · [authentication](https://global-docs.upbit.com/reference/auth) |

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
