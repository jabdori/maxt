[English](bithumb.md) | [한국어](bithumb.ko.md)

# Bithumb

The Bithumb adapter is spot-only. It provides public REST market data and public trade, order-book, and ticker streams; derivatives and a candle stream are not available.

## Construction and scope

```rust
use maxt::{Client, adapters::BithumbAdapter};

let public = Client::new(BithumbAdapter::new());
let access_key = "your access key";
let secret_key = "your secret key";
let authenticated = Client::new(
    BithumbAdapter::new().with_credentials(access_key, secret_key),
);
```

`BithumbAdapter::new()` needs no credentials. `with_credentials(access_key, secret_key)` adds the access key and secret key required by account, order, and private-stream calls; it does not change the spot-only scope.

Use `Market::spot(Exchange::Bithumb, "BTC", "KRW")`. Bithumb's native symbol for that market is `KRW-BTC`.

## Public REST

| Call | Bithumb behavior and limits |
| --- | --- |
| `markets(MarketKind::Spot)` | Returns the listed spot markets; other market kinds return an empty list |
| `trades` | `limit` must be `1..=500`; when omitted, Bithumb's default is 1 |
| `order_book` | `depth` must be above 0. `maxt` makes a single-market request, removes zero-quantity slots, sorts both sides best-first, and returns up to 30 valid levels per side, further truncated to `depth` |
| `ticker` | Returns one snapshot for the requested market |
| `candles` | Bithumb caps every supported candle response at 200. `maxt` pages at most 100 calls, so one request can assemble at most 20,000 candles; a wider `limit` or time window is rejected |

Supported candle intervals are `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, and `Month1`.

## Public streams

| Feed | Behavior |
| --- | --- |
| `Feed::Trades` | Public trade events |
| `Feed::OrderBook` | Full snapshots with up to 15 levels per side after zero-quantity slots are removed; the wire timestamp is in microseconds |
| `Feed::Ticker` | Public ticker snapshots and real-time updates |
| `Feed::Candles(_)` | `Error::Unsupported` before the socket opens; Bithumb publishes no public candle stream and `maxt` does not synthesize one |

## Candle ranges and grid

`CandleRequest::from` is inclusive and `CandleRequest::to` is exclusive. The Bithumb `to` parameter is a KST wall-clock value and is also exclusive; `maxt` converts the caller's UTC `Timestamp` and preserves subsecond exclusivity. Results are returned oldest-first.

| Interval | Candle opens in UTC |
| --- | --- |
| `Min1` through `Hour1` | Normal UTC unit boundaries |
| `Hour4` | 03:00, 07:00, 11:00, 15:00, 19:00, and 23:00 |
| `Day1` | 15:00, which is 00:00 KST on the following date |
| `Week1` | Sunday 15:00, which is Monday 00:00 KST |
| `Month1` | 15:00 on the last UTC day of the previous month, which is 00:00 KST on the first day |

## Bithumb-only market flags

These methods are available through `Client::adapter()` and describe different Bithumb designations.

| Method | Meaning |
| --- | --- |
| `market_warnings()` | Reads `/v1/market/all?isDetails=true` and returns every listed market with its raw `market_warning` label (`NONE` or `CAUTION`). `CAUTION` means an investment-warning market (유의 종목); it still trades and maps to `MarketStatus::Unknown` |
| `market_alerts()` | Reads the alert-system endpoint (경보제) and returns only active alert rows, one per market and criterion, with the criterion, severity step, and KST expiry converted to UTC. Markets without an alert are absent, and alerts do not change `MarketStatus` |

The word `CAUTION` therefore has two contexts: a `market_warning` value means 유의 종목, while `BithumbAlertStep::Caution` is the mildest alert-system step (주의).

## Credentials and current private limitations

Credentials enable balances, open orders, order placement and cancellation, and the private account stream. Missing credentials produce `Error::Auth` before a private request is built.

Bithumb's current API supports time-in-force (TIF) values such as IOC, FOK, and Post-Only for eligible orders. The current `maxt` Bithumb adapter does not expose that capability: any `OrderRequest::time_in_force` is rejected as `Error::InvalidRequest`.

## Rate limits

| Official scope | Limit |
| --- | --- |
| Public REST API | At most 150 requests per second |
| Private REST API | At most 140 requests per second |
| Order-related REST API | Additional restriction above 10 requests per second |
| WebSocket connection requests | At most 10 per second per IP, the same for Public and Private; excess requests receive HTTP 429, and sustained excess can block WebSocket use for 10 minutes |

Bithumb may lower REST limits without prior notice during excessive traffic.

## Errors

Local range and request-shape failures are `Error::InvalidRequest`. A Bithumb `{"error": ...}` envelope is `Error::Exchange` even when the HTTP status is 2xx; a numeric `error.name` is preserved as its string code. Non-2xx Bithumb failures are also `Error::Exchange`.

## Verification

On 2026-07-31, a representative BTC/KRW public smoke test covered market discovery, ticker, order book, recent trades, candles, and the public Trades, OrderBook, and Ticker streams. Private live account and order operations were not verified.

Run the public REST example with:

```text
cargo run --example public_rest -- bithumb BTC KRW
```

## Official documentation

| Subject | Current Bithumb pages |
| --- | --- |
| Index and limits | [documentation index](https://apidocs.bithumb.com/llms.txt) · [API request limits](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내.md) |
| Public REST | [markets](https://apidocs.bithumb.com/reference/거래-대상-목록-조회.md) · [alerts](https://apidocs.bithumb.com/reference/경보제-조회.md) · [recent trades](https://apidocs.bithumb.com/reference/체결-내역-조회.md) · [ticker](https://apidocs.bithumb.com/reference/현재가-조회.md) · [order book](https://apidocs.bithumb.com/reference/호가-조회.md) |
| Candles | [minute](https://apidocs.bithumb.com/reference/분minute-캔들-조회.md) · [day](https://apidocs.bithumb.com/reference/일day-캔들-조회.md) · [week](https://apidocs.bithumb.com/reference/주week-캔들-조회.md) · [month](https://apidocs.bithumb.com/reference/월month-캔들-조회.md) |
| Public WebSocket | [basics and connection limits](https://apidocs.bithumb.com/reference/기본-정보.md) · [ticker](https://apidocs.bithumb.com/reference/현재가-ticker.md) · [trades](https://apidocs.bithumb.com/reference/체결-trade.md) · [order book](https://apidocs.bithumb.com/reference/호가-orderbook.md) |
| Time in force | [order request](https://apidocs.bithumb.com/reference/주문-요청.md) |

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
