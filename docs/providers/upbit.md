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

The common `Order` keeps normalized fields only. Use `order_detail(request)`
when Upbit-specific fills, fees, locked amounts, self-match-prevention, or
time-in-force fields are needed.

`order_history(request)` remains the common normalized history API.
`closed_orders(request)` is its provider-specific complement: it keeps Upbit's
official closed-order summary fields, including fees, SMP, `identifier`, and
time-in-force, but does not return a `trades` list. Its optional `market` and
`state` filters are mutually exclusive with `states[]`; the
creation-time window is at most seven days, `limit` is at most 1,000, and the
request can choose ascending or descending ordering. Supplied `Timestamp`
values go to Upbit directly as milliseconds, unlike the common history API's
exclusive-end adaptation. The official endpoint documentation does not state
time-boundary inclusion, so this API makes no further boundary claim.
Fixture-verified only; maxt has not performed a live trade or read.

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
| `list_subscriptions(subscription)` | Queries the one active Upbit public connection matching `subscription`; call `subscribe` first, keep its returned stream running, and use the exact same selector. No match or multiple matches fail locally rather than querying a different socket. Preserves the ticket, data types, markets, and optional order-book level returned by `LIST_SUBSCRIPTIONS`. Fixture-verified only | WebSocket request |
| `test_order(request)` | Validates an order without creating it; requires the order-placement permission; the returned `Order` is dry-run only, so its ID cannot be queried or cancelled and its status is not a live order | `order-test` |
| `order_detail(request)` | Authenticated provider-specific `GET /v1/order`. The request supplies the expected `market` plus a UUID and/or `identifier`; one identifier is required and Upbit gives the UUID priority when both are set. Preserves detailed fills, fees, locked amounts, SMP, and time-in-force raw fields that the common `Order` does not carry. Reserved characters in `identifier` are safely percent-encoded while the original query text is used for the JWT query hash. Fixture-verified only | `default` |
| `closed_orders(request)` | Authenticated provider-specific `GET /v1/orders/closed` summary list. Optional `market`, `state`, or `states[]`; `state` and `states[]` are mutually exclusive. Creation-time window is at most seven days; `limit: 1..=1_000`; ascending or descending ordering. Timestamps are sent directly as milliseconds. Preserves fee, SMP, `identifier`, and time-in-force fields, but no individual `trades`. Fixture-verified only; no live trade or read | `default` |
| `deposit_info(asset, network)` | Requires `View Deposits`; returns availability, reason, minimum amount, confirmation count, and decimal precision. The response network is nullable and is preserved as returned. This metadata may be delayed by several minutes | `default` |
| `withdrawal_addresses()` | Authenticated `GET /v1/withdraws/coin_addresses` provider read. Preserves registered address, network, recipient, and wallet metadata for this account. Fixture-verified only | `default` |
| `travel_rule_vasps()` | Requires `View Deposits`; lists VASPs available for Travel Rule verification in Korea or Singapore. Indonesia and Thailand return `Error::Unsupported` before authentication or network I/O | `default` |
| `verify_travel_rule_by_uuid(...)`, `verify_travel_rule_by_txid(...)` | Korea or Singapore only; financial writes that request account-owner verification. Upbit enforces the per-deposit repeat limit. Fixture-verified only; maxt does not submit a live verification | `default` |
| `batch_cancel_open_orders(request)` | Requires order-placement permission. `UpbitBatchCancelScope::All` explicitly selects every eligible market, not an unbounded number of orders; Upbit applies the request count (default 20, maximum 300). Quote-currency and pair scopes are alternatives, with up to 20 excluded pairs. The result keeps both completed and failed cancellations. Fixture-verified only; no live cancellation is run by maxt | `order-cancel-all` |
| `cancel_and_new_order(request)` | Requires order-placement permission; JSON `POST /v1/orders/cancel_and_new` only. The replacement keeps the original market and side, and can change order type, size, price, TIF, and SMP. `post_only` cannot be combined with SMP. A successful HTTP response does not guarantee a new order: if the previous order fills before cancellation completes, `new_order_uuid` is absent. Fixture-verified only; no live order is submitted by maxt | `order` |
| `deposit_krw(request)` | Upbit Korea only; financial `POST /v1/deposits/krw`. `UpbitKrwTransferRequest` requires a positive amount and `Kakao`, `Naver`, or `Hana` second-factor type. Upbit verifies the registered transfer account and second factor; fixture-verified only, with no live transfer submitted by maxt | `default` |
| `withdraw_krw(request)` | Upbit Korea only; financial `POST /v1/withdraws/krw` with the same request. A withdrawal-enabled key can still be rejected while Upbit's withdrawal safety lock is enabled. Fixture-verified only; no live transfer is submitted by maxt | `default` |
| `api_keys()` | Upbit Korea only; authenticated `GET /v1/api_keys`. Returns access-key identifiers and expiration times, never secret-key material. Fixture-verified only | `default` |
| `list_pockets()` | Upbit Korea only; authenticated `GET /v1/pockets`. Lists the pockets visible to the configured key. Fixture-verified only | `default` |
| `list_pocket_api_keys(request)`, `sub_pocket_balances(pocket_uuid)` | Upbit Korea only; authenticated reads for pocket API-key groups and one sub-pocket's balances. The key needs the corresponding Upbit pocket-management permission. Fixture-verified only | `default` |
| `universal_transfer(request)`, `sub_pocket_transfer(request)` | Upbit Korea-only financial pocket transfers. Both current request contracts require `to`; the universal transfer may additionally set `from`. Fixture-verified only; no live transfer is submitted by maxt | `default` |
| `universal_transfers(query)`, `sub_pocket_transfers(query)` | Upbit Korea-only pocket-transfer history. Query UUID and identifier lists are capped at 20 entries, the time range is at most seven days, and `limit` is `1..=100`. Fixture-verified only | `default` |

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
- [Batch cancel orders](https://global-docs.upbit.com/reference/batch-cancel-orders)
- [Cancel and new order](https://global-docs.upbit.com/reference/cancel-and-new-order)
- [Deposit availability](https://global-docs.upbit.com/reference/available-deposit-information)
- [Registered withdrawal addresses](https://docs.upbit.com/kr/reference/list-withdrawal-addresses)
- [KRW deposit](https://docs.upbit.com/kr/reference/deposit-krw)
- [KRW withdrawal](https://docs.upbit.com/kr/reference/withdraw-krw)
- [API key list](https://docs.upbit.com/kr/reference/list-api-keys)
- [Pocket list](https://docs.upbit.com/kr/reference/list-pockets)
- [Pocket API keys](https://docs.upbit.com/kr/reference/list-pocket-api-keys)
- [Sub-pocket balance](https://docs.upbit.com/kr/reference/get-sub-pocket-balance)
- [Universal pocket transfer](https://docs.upbit.com/kr/reference/universal-transfer)
- [Universal pocket-transfer history](https://docs.upbit.com/kr/reference/list-universal-transfers)
- [Sub-pocket transfer](https://docs.upbit.com/kr/reference/transfer)
- [Sub-pocket-transfer history](https://docs.upbit.com/kr/reference/list-transfers)
- [Korea Travel Rule VASPs](https://docs.upbit.com/kr/reference/list-travelrule-vasps)
- [Singapore Travel Rule VASPs](https://global-docs.upbit.com/reference/list-travelrule-vasps)
- [Get order and provider order detail](https://global-docs.upbit.com/reference/get-order)
- [Closed orders — Korea](https://docs.upbit.com/kr/reference/list-closed-orders)
- [Closed orders — Global](https://global-docs.upbit.com/reference/list-closed-orders)

[Common API](../common-api.md) · [Provider support](../providers.md)
