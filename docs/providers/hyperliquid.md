# Hyperliquid

[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

## Venue and constructor

| Constructor | Network |
| --- | --- |
| `HyperliquidAdapter::new()` | Mainnet |
| `HyperliquidAdapter::testnet()` | Testnet |

One adapter exposes Spot and the default perpetual DEX.

| Market | Metadata | Support |
| --- | --- | --- |
| Spot | `spotMeta` | Included; non-USDC quote assets included |
| Default perpetual DEX | `meta` without `dex` | `MarketKind::Perpetual` |
| HIP-3 perpetual DEX | `perpDexs`, `meta` with `dex` | Not exposed |
| Outcome assets | `outcomeMeta` | Not exposed |

The adapter caches `meta` and `spotMeta` on first use. Create a new adapter to
reload listings. Spot metadata uses `spotMeta.universe[].name`; stream frames
may use that name or `@{index}`. Use the returned `MarketInfo::native_symbol`.

## REST

| Call | Hyperliquid request | Contract |
| --- | --- | --- |
| `markets(kind)` | `meta`, `spotMeta` | Markets in the support table |
| `trades(market, limit)` | `recentTrades` | `limit in 1..=10`; no provider count parameter; `None -> full provider window (<= 10)`; `Some(limit)` truncates locally; newest-first |
| `order_book(market, depth)` | `l2Book` | `depth in 1..=20`; local truncation |
| `ticker(market)` | `metaAndAssetCtxs`, `spotMetaAndAssetCtxs` | Reference-price summary |
| `funding_rates(request)` | `fundingHistory` | Public; Perpetual only; provider page `<= 500` |

| `HistoryRequest` field | Provider value |
| --- | --- |
| Range | `from <= time < to` |
| `startTime` | `ceil_ms(from)` |
| `endTime` | `ceil_ms(to) - 1` |
| `cursor` | Replaces `from` when present |
| Continuation | Read until `Page::next == None` |

Ticker mapping:

| `Ticker` field | Source |
| --- | --- |
| `last_price` | `midPx ?? markPx`; not the latest execution price |
| `last_trade_time` | `None` |
| `timestamp` | Local read time |
| `change` | `last_price - prevDayPx` |
| `change_rate` | `(last_price - prevDayPx) / prevDayPx`; `prevDayPx == 0 -> None` |

Use `trades` or `Feed::Trades` for execution prices and times.

## Candles

| Contract | Value |
| --- | --- |
| Exposed intervals | `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| `Sec1` | `Error::Unsupported` |
| Retention | Latest 5,000 candles per interval |
| `Month1` grid | Fixed 30 days |
| `quote_volume` | `None` |

## Streams

| Feed | Subscription | Contract |
| --- | --- | --- |
| `Feed::Trades` | `trades` | Executions with provider timestamps |
| `Feed::OrderBook` | `l2Book` | Full snapshot; up to 20 levels per side; no diffs |
| `Feed::Ticker` | `activeAssetCtx` | REST ticker mapping |
| `Feed::Candles(interval)` | `candle` | Forming and closed candles |

| Candle transition | Events |
| --- | --- |
| `new.open_time > held.open_time` | `held(closed = true)`, then `new(closed = false)` |
| `new.open_time == held.open_time` | Replace `held`; emit `new(closed = false)` |
| `new.open_time < held.open_time` | Drop frame |
| Reconnect | Drop `held`; no cross-connection close event |

The adapter sends `{"method":"ping"}` every 15 seconds. `l2Book.nSigFigs` and
`l2Book.mantissa` are not exposed.

## Private and provider-specific APIs

Configure private calls with `.with_wallet(address, private_key)`. Wallet values
are validated on the first private call; `Client::supports` treats the feature
as configured as soon as a wallet is present. The private key is used only for
local signing and is redacted from `Debug`.

| Market | Private features |
| --- | --- |
| Spot | Balances, open orders, place/cancel order, account stream |
| Perpetual | Spot features plus positions, margin summary/configuration, funding payments, and reduce-only orders |

`positions()` returns all open perpetual positions;
`positions_on(spot) == Ok(vec![])`.

| Order input | Contract |
| --- | --- |
| `order_type` | `Limit`; `Market -> Error::Unsupported` |
| `size` | `Size::Base`; `size > 0` |
| `price` | `price > 0`; max decimal places: Perpetual `6 - szDecimals`, Spot `8 - szDecimals`; non-integer prices: `significant_figures <= 5`; integer prices exempt |
| `time_in_force` | `GTC`, `IOC`, `PostOnly`; `FOK -> Error::Unsupported` |
| `reduce_only` | `MarketKind::Perpetual` only |
| Minimum notional | Provider validation; no `maxt` preflight validation |

| `set_margin` input | Contract |
| --- | --- |
| Fields | `leverage.is_some() && margin_mode.is_some()` |
| `leverage` | Positive integer; `leverage <= asset.max_leverage` |
| `margin_mode == Cross` | `Error::InvalidRequest` when `asset.only_isolated == true` |

Provider-specific methods are available through `Client::adapter()`:

| Method | Contract |
| --- | --- |
| `asset_context(&market)` | Mid, mark, oracle, funding, open interest, and order precision. |
| `non_funding_ledger(from, to, cursor, limit)` | Deposits, withdrawals, transfers, and liquidations; funding excluded; wallet required; provider page `<= 500` |

| `non_funding_ledger` field | Contract |
| --- | --- |
| Provider range | `from_ms <= time <= to_ms` |
| `cursor` | Replaces `from` when present |
| `limit` | Local target; a same-millisecond group may exceed it |

## Limits and official links

| Scope | Limit |
| --- | --- |
| REST | 1,200 aggregate request weight per minute per IP |
| WebSocket connections | 10 concurrent; 30 new per minute |
| WebSocket subscriptions | 1,000 |
| WebSocket messages sent | 2,000 per minute across all connections |

`l2Book` weight is 2. A generic `info` request weight is 20.
`candleSnapshot` adds weight per 60 returned items. `maxt` does not throttle
requests.

HIP-3, outcome assets, `Sec1`, market orders, `FOK`, `l2Book.nSigFigs`, and
`l2Book.mantissa` are not exposed.

- [API overview](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Perpetual info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Spot info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [Rate limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

[Common API](../common-api.md) · [Provider matrix](../providers.md)
