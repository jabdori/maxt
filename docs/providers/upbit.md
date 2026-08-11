# Upbit

[English](upbit.md) | [한국어](upbit.ko.md)

## Venue and constructor

Spot only. An `UpbitAdapter` is fixed to one region; hosts, listings, books,
accounts, and credentials are not shared across regions.

| Region | Constructor | REST | WebSocket |
| --- | --- | --- | --- |
| Korea | `UpbitAdapter::new()` or `with_region(UpbitRegion::Korea)` | `https://api.upbit.com` | `wss://api.upbit.com/websocket/v1` |
| Singapore | `with_region(UpbitRegion::Singapore)` | `https://sg-api.upbit.com` | `wss://sg-api.upbit.com/websocket/v1` |
| Indonesia | `with_region(UpbitRegion::Indonesia)` | `https://id-api.upbit.com` | `wss://id-api.upbit.com/websocket/v1` |
| Thailand | `with_region(UpbitRegion::Thailand)` | `https://th-api.upbit.com` | `wss://th-api.upbit.com/websocket/v1` |

| Field | Value |
| --- | --- |
| `Market` | `Market::spot(Exchange::Upbit, "BTC", "KRW")` |
| `MarketInfo::native_symbol` | `KRW-BTC` |
| `base`, `quote` | `[A-Z0-9]+` |

## REST

| Call | Endpoint | Contract |
| --- | --- | --- |
| `markets(MarketKind::Spot)` | `/v1/market/all?is_details=true` | Listed Spot markets |
| `markets(MarketKind::Perpetual)` | — | `Ok(vec![])` |
| `trades(market, limit)` | `/v1/trades/ticks` | `limit: 1..=500`; newest-first |
| `order_book(market, depth)` | `/v1/orderbook` | Shared unaggregated view; `depth: 1..=30`; at most `depth` levels per side; `None -> 30` |
| `ticker(market)` | `/v1/ticker` | One market snapshot |

Derivative methods return `Error::Unsupported`.

## Candles

| Surface | Exposed `Interval` variants | No generic `Interval` |
| --- | --- | --- |
| REST | `Sec1`, `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` | `1y` |
| WebSocket | `Sec1`, `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4` | — |

Annual candles are available through the provider-specific
`year_candles(market, to, count)` method and return `UpbitYearCandle`; they do
not extend the shared `Interval`.

| Constraint | Value |
| --- | ---: |
| Provider page cap | 200 |
| Provider calls per request | `<= 100` |
| Preflight candle estimate | `<= 20_000` |
| `Sec1` retention | Latest three months |

## Streams

| Feed | Contract |
| --- | --- |
| `Feed::Trades` | One event per execution; `Trade::id = sequential_id` |
| `Feed::OrderBook` | Full snapshot; 30 levels per side; fixed depth |
| `Feed::Ticker` | Full snapshot |
| `Feed::Candles(interval)` | Forming updates and transition-based close events |

| Candle frame | Events |
| --- | --- |
| `SNAPSHOT && interval_end <= now` | One event with `closed == true` |
| `new.open_time == held.open_time` | Replace `held`; emit `closed == false` |
| `REALTIME && new.open_time > held.open_time` | `held(closed == true)`, then `new(closed == false)` |
| `new.open_time < held.open_time || new.open_time <= settled.open_time` | Drop frame |
| No later frame or reconnect | No synthetic close event |

## Private and provider-specific APIs

Configure private calls with `.with_credentials(access_key, secret_key)`. The
credentials must belong to `UpbitAdapter::region()`. Private features include
balances, order lookup and history, place/cancel order, and account streams.

| Common call | Endpoint | Contract |
| --- | --- | --- |
| `order_rules(market)` | `GET /v1/orders/chance` | Fees, supported side/type/TIF combinations, quote/base balances and average buy prices, and quote-denominated limits; deprecated fields are omitted |
| `open_orders*` | `GET /v1/orders/open` | One page, at most 100 orders |
| `order(market, order_id)` | `GET /v1/order?uuid=...` | Verifies the returned market |
| `order_by_client_id(market, client_id)` | `GET /v1/order?identifier=...` | Verifies the returned market |
| `orders_by_ids(request)` | `GET /v1/orders/uuids` | One to 100 UUIDs or identifiers; optional market; newest-first |
| `order_history(request)` | `GET /v1/orders/closed` | `limit: 1..=1_000`; at most seven days; newest-first; no cursor, so `next == None` |
| `cancel_orders(request)` | `DELETE /v1/orders/uuids` | One to 20 UUIDs or identifiers; one identifier namespace; partial failures stay in the result |

| Order input | Contract |
| --- | --- |
| Best buy | `Size::Quote` with `IOC` or `FOK` |
| Best sell | `Size::Base` with `IOC` or `FOK` |
| `client_id` | 1–64 RFC 3986 unreserved ASCII bytes; usable with `cancel_order_by_client_id` |
| Cancel methods | Return `()` after validating the provider response |

The common `Order` keeps normalized fields only. Upbit-specific self-match
prevention fields and the detailed `trades` array are not exposed yet.

Access the following provider-specific methods through `Client::adapter()`.

| Method | Contract | Rate-limit group |
| --- | --- | --- |
| `tickers(&[Market])` | `markets.len() >= 1`; one ticker per market | `ticker` |
| `tickers_by_quote(&[String])` | At least one quote currency; normalizes to uppercase; returns all matching ticker snapshots | `ticker` |
| `order_books(&[Market], depth)` | `markets.len() >= 1`; `depth: 1..=30` or `None` | `orderbook` |
| `order_books_at_level(&[Market], Decimal, depth)` | Upbit Korea only; `level >= 0`; use the current `supported_levels` metadata before a non-zero request | `orderbook` |
| `orderbook_instruments(&[Market])` | `markets.len() >= 1`; current price-band tick size and supported aggregation levels; levels are empty when the region does not return them | `orderbook` |
| `year_candles(market, to, count)` | `count: 1..=200` or `None`; optional ISO-8601 boundary; oldest-first; Korean open time is optional by region | `candle` |
| `market_events()` | Investment warning and caution criteria by market | `market` |
| `test_order(request)` | Validates an order without creating it; requires the order-placement permission; the returned `Order` is dry-run only, so its ID cannot be queried or cancelled and its status is not a live order | `order-test` |

| Market event | Mapping |
| --- | --- |
| `warning == true` | `MarketStatus::Unknown` |
| `cautions` non-empty | No `MarketStatus` change |
| `region != UpbitRegion::Korea` | `UpbitMarketEvent::cautions == []` |

`UpbitOrderBookInstrument::tick_size` is current metadata, not a permanent
market constant. Fetch it again when an intended order price crosses an Upbit
price band.

## Limits and official links

| Group | Limit | Scope |
| --- | --- | --- |
| `market`, `candle`, `trade`, `ticker`, `orderbook` | 10/s each | IP |
| `default` | 30/s | Korea: Pocket; Global: Account |
| `order`, `order-test` | 8/s each | Korea: Pocket; Global: Account |
| `order-cancel-all` | 1/2s | Korea: Pocket; Global: Account |
| WebSocket connections | 5/s | Unauthenticated: IP; authenticated: Pocket or Account |
| WebSocket messages | 5/s and 100/min | Connection |

`maxt` does not throttle requests. Read `Remaining-Req`; HTTP rate-limit errors
satisfy `Error::is_rate_limited() == true`.

- [Regions and endpoints](https://global-docs.upbit.com/reference/api-overview)
- [Public REST](https://global-docs.upbit.com/reference/list-trading-pairs)
- [Order books](https://global-docs.upbit.com/reference/list-orderbooks)
- [Candles](https://global-docs.upbit.com/reference/list-candles-minutes)
- [Annual candles](https://docs.upbit.com/kr/reference/list-candles-years)
- [Quote-currency tickers](https://docs.upbit.com/kr/reference/list-quote-tickers)
- [Orderbook instruments](https://docs.upbit.com/kr/reference/list-orderbook-instruments)
- [WebSocket](https://global-docs.upbit.com/reference/websocket-guide)
- [Rate limits](https://global-docs.upbit.com/reference/rate-limits)
- [Authentication](https://global-docs.upbit.com/reference/auth)
- [Test order](https://global-docs.upbit.com/reference/order-test)
- [Get order](https://global-docs.upbit.com/reference/get-order)
- [Closed orders](https://global-docs.upbit.com/reference/list-closed-orders)

[Common API](../common-api.md) · [Provider support](../providers.md)
