# Bithumb

[English](bithumb.md) | [한국어](bithumb.ko.md)

## Venue and constructor

Spot only.

| Constructor | Features |
| --- | --- |
| `BithumbAdapter::new()` | Public REST and streams |
| `.with_credentials(access_key, secret_key)` | Account, order, and private-stream methods |

| Field | Value |
| --- | --- |
| `Market` | `Market::spot(Exchange::Bithumb, "BTC", "KRW")` |
| `MarketInfo::native_symbol` | `KRW-BTC` |

## REST

| Call | Endpoint | Contract |
| --- | --- | --- |
| `markets(MarketKind::Spot)` | `/v1/market/all?isDetails=true` | Listed Spot markets |
| `markets(MarketKind::Perpetual)` | — | `Ok(vec![])` |
| `trades(market, limit)` | `/v1/trades/ticks` | `limit: 1..=500`; `None -> 1`; newest-first |
| `order_book(market, depth)` | `/v1/orderbook` | `depth: 1..=30`; at most `depth` levels per side; `None -> 30`; remove `quantity == 0`, sort, then truncate locally |
| `ticker(market)` | `/v1/ticker` | One market snapshot |

`HTTP 2xx + {"error": ...} -> Error::Exchange`. Numeric `error.name` values
remain string codes.

## Candles

| Contract | Value |
| --- | --- |
| Exposed intervals | `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` |
| Provider page cap | 200 |
| Provider calls per request | `<= 100` |
| Preflight candle estimate | `<= 20_000` |
| Provider `to` | `format_kst(ceil_second(to))`; exclusive |

| Interval | UTC `open_time` grid |
| --- | --- |
| `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1` | UTC unit boundaries |
| `Hour4` | `03:00`, `07:00`, `11:00`, `15:00`, `19:00`, `23:00` |
| `Day1` | `15:00` |
| `Week1` | Sunday `15:00` |
| `Month1` | `15:00` on the final UTC day of the previous month |

## Streams

| Feed | Contract |
| --- | --- |
| `Feed::Trades` | Public execution events |
| `Feed::OrderBook` | Full snapshot; remove `quantity == 0`; up to 15 levels per side; raw provider `timestamp` unit: µs |
| `Feed::Ticker` | Snapshot and real-time updates |
| `Feed::Candles(_)` | `Error::Unsupported` before connection |

## Private and provider-specific APIs

Credentials enable balances, order lookup and history, place/cancel order, and
account streams.

| Common call | Endpoint | Contract |
| --- | --- | --- |
| `order_rules(market)` | `GET /v1/orders/chance` | Fees, supported order sides and types, buy/sell price units, quote/base balances and average buy prices, and quote-denominated limits |
| `open_orders*` | `GET /v1/orders` | One page, at most 100 orders |
| `order(market, order_id)` | `GET /v1/order?uuid=...` | Verifies the returned market |
| `order_by_client_id(market, client_id)` | `GET /v1/order?client_order_id=...` | Verifies the returned market |
| `orders_by_ids(request)` | `POST /v2/orders/search` | One to 100 order IDs or client IDs; unmatched IDs are omitted and duplicates collapse |
| `order_history(request)` | `GET /v2/orders/history` | `limit: 1..=1_000`; at most seven days; newest-first; `next_key` becomes the opaque `Page::next` cursor |
| `cancel_orders(request)` | `POST /v2/orders/cancel` | One to 30 order IDs or client IDs; each failure preserves its provider code and message |

| Order | Required `Size` |
| --- | --- |
| Limit buy or sell | `Size::Base` |
| Market buy | `Size::Quote` |
| Market sell | `Size::Base` |
| Best buy | `Size::Quote` |
| Best sell | `Size::Base` |

| Input | Result |
| --- | --- |
| Limit + `IOC`, `FOK`, or `PostOnly` | KRW markets only |
| Best + `IOC` or `FOK` | KRW markets only; policy required |
| `client_id` | 1–36 ASCII letters, digits, `-`, or `_` |
| `OrderRequest::reduce_only == true` | `Error::Unsupported` |
| `cancel_order(...)`, `cancel_order_by_client_id(...)` | Return `()` after validating the cancellation acknowledgement |

The common `Order` keeps normalized fields only. Bithumb-specific cancellation
and self-trade prevention fields and the detailed `trades` array are not exposed
yet.

Access the following provider-specific methods through `Client::adapter()`.

| Method | Contract |
| --- | --- |
| `market_warnings()` | One raw `NONE` or `CAUTION` value per listed market |
| `market_alerts()` | Active rows only; one row per market and criterion; `ends_at` converted from KST to UTC |
| `notices(count)` | `GET /v1/notices`; `count: 1..=20`; `None -> provider default 5`; newest-first; `published_at` and `modified_at` converted from KST to UTC |
| `transfer_fees(currency)` | `GET /v2/fee/inout/{currency}`; an asset symbol or `ALL`; per-network deposit fee/minimum and a fixed or rate-based withdrawal fee rule; not account-specific availability |
| `api_keys()` | Authenticated `GET /v1/api_keys`; each registered access-key identifier and its expiration time |
| `pending_orders(request)` | Authenticated `GET /v2/orders/pending`; optional market, `wait` or `watch` state, `1..=100` limit, ascending or descending order, and an opaque `next_key` cursor in `Page::next` |
| `batch_orders(request)` | Authenticated `POST /v2/orders/batch`; 1–20 orders. HTTP 200 can contain both accepted and rejected items, so inspect every `BithumbBatchOrderOutcome`; accepted items preserve provider `time_in_force` and `stp_type`, and rejected items preserve returned `time_in_force`. Fixture-verified only; no live order is submitted by maxt |

### TWAP

Bithumb's TWAP API is available for **KRW markets only** and requires JWT
credentials. `twap_orders(request)` is a read-only history query; it supports
the `progress`, `done`, and `cancel` states, optional TWAP IDs, a cursor, a
`1..=100` page size, and ascending or descending order.

```rust
let page = adapter
    .twap_orders(
        &BithumbTwapOrdersRequest::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .limit(20),
    )
    .await?;
```

`create_twap_order(...)` and `cancel_twap_order(...)` are financial writes.
Creation requires a 300–43,200 second duration, a 15/20/30/60/120 second
frequency, and `price` for a buy or `volume` for a sell. Cancelling stops
unsubmitted child orders; already executed orders remain executed. Do not run
either write method from a read-only verification.

| Provider state | Mapping |
| --- | --- |
| `market_warning == CAUTION` | `MarketStatus::Unknown` |
| `BithumbAlertStep::Caution` | Alert-system step `주의`; no `MarketStatus` change |

## Limits and official links

| Scope | Limit |
| --- | --- |
| Public REST | 150/s |
| Private REST | 140/s |
| Order REST | Additional throttling above 10/s |
| WebSocket connections | 10/s per IP; HTTP 429; repeated excess may block for 10 minutes |

`maxt` does not throttle requests. Derivatives, `MarketKind::Perpetual`, and
public candle streams are not supported.

- [Documentation index](https://apidocs.bithumb.com/llms.txt)
- [Request limits](https://apidocs.bithumb.com/docs/api-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9C-%EC%95%88%EB%82%B4.md)
- [Recent trades](https://apidocs.bithumb.com/reference/%EC%B2%B4%EA%B2%B0-%EB%82%B4%EC%97%AD-%EC%A1%B0%ED%9A%8C.md)
- [Notices](https://apidocs.bithumb.com/reference/%EA%B3%B5%EC%A7%80%EC%82%AC%ED%95%AD-%EC%A1%B0%ED%9A%8C.md)
- [Transfer fees](https://apidocs.bithumb.com/reference/%EC%9E%85%EC%B6%9C%EA%B8%88-%EC%88%98%EC%88%98%EB%A3%8C-%EC%A1%B0%ED%9A%8C.md)
- [API keys](https://apidocs.bithumb.com/reference/api-%ED%82%A4-%EB%A6%AC%EC%8A%A4%ED%8A%B8-%EC%A1%B0%ED%9A%8C.md)
- [Pending orders](https://apidocs.bithumb.com/reference/%EB%8C%80%EA%B8%B0-%EC%A3%BC%EB%AC%B8-%EB%AA%A9%EB%A1%9D-%EC%A1%B0%ED%9A%8C.md)
- [Batch order](https://apidocs.bithumb.com/reference/%EB%8B%A4%EA%B1%B4-%EC%A3%BC%EB%AC%B8-%EC%9A%94%EC%B2%AD)
- [Batch order cancellation](https://apidocs.bithumb.com/reference/%EB%8B%A4%EA%B1%B4-%EC%A3%BC%EB%AC%B8-%EC%B7%A8%EC%86%8C-%EC%A0%91%EC%88%98)
- [TWAP order history](https://apidocs.bithumb.com/reference/twap-%EC%A3%BC%EB%AC%B8%EB%82%B4%EC%97%AD-%EC%A1%B0%ED%9A%8C)
- [TWAP order request](https://apidocs.bithumb.com/reference/twap-%EC%A3%BC%EB%AC%B8-%EC%9A%94%EC%B2%AD)
- [TWAP order cancellation](https://apidocs.bithumb.com/reference/twap-%EC%A3%BC%EB%AC%B8-%EC%B7%A8%EC%86%8C)
- [Candles](https://apidocs.bithumb.com/reference/%EB%B6%84minute-%EC%BA%94%EB%93%A4-%EC%A1%B0%ED%9A%8C.md)
- [WebSocket](https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4.md)
- [Orders](https://apidocs.bithumb.com/reference/%EC%A3%BC%EB%AC%B8-%EC%9A%94%EC%B2%AD.md)
- [Get order](https://apidocs.bithumb.com/reference/%EA%B0%9C%EB%B3%84-%EC%A3%BC%EB%AC%B8-%EC%A1%B0%ED%9A%8C)
- [Closed orders](https://apidocs.bithumb.com/reference/%EC%A2%85%EB%A3%8C-%EC%A3%BC%EB%AC%B8-%EB%AA%A9%EB%A1%9D-%EC%A1%B0%ED%9A%8C)

[Common API](../common-api.md) · [Provider support](../providers.md)
