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
| `trades(market, limit)` | `recentTrades` | `limit: 1..=10`; no provider count parameter; `None -> provider page (<= 10)`; `Some(limit)` truncates locally; newest-first |
| `order_book(market, depth)` | `l2Book` | `depth: 1..=20`; at most `depth` levels per side; local truncation |
| `ticker(market)` | `metaAndAssetCtxs`, `spotMetaAndAssetCtxs` | Reference-price summary |
| `funding_rates(request)` | `fundingHistory` | Public; Perpetual only; provider page `<= 500` |
| `all_mids()` | `POST /info` with `{"type":"allMids"}` | Public; default perpetual DEX and first-DEX Spot mids; empty books use the last trade price as fallback |

| `HistoryRequest` field or state | Contract |
| --- | --- |
| `from`, `to` | `from <= timestamp < to` |
| `startTime` | `ceil_ms(from)` |
| `endTime` | `ceil_ms(to) - 1` |
| `cursor` | Opaque resume point; overrides `from` |
| `limit` | Local page-size target; one millisecond group is never split, so `items.len()` may be below or above `limit` |
| `limit == 0` | `Error::InvalidRequest` before credential checks or network I/O |
| Continuation | Set `cursor = page.next` until `page.next == None` |

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

## Account configuration and provider-specific APIs

Public market data and market streams need no account configuration.
Hyperliquid separates its public account-query address from its local signer:

| Configuration | Spot | Perpetual |
| --- | --- | --- |
| `.with_query_address(address)` | Balances, open orders, and account stream | Spot reads plus positions, margin summary, and funding payments |
| `.with_signer(private_key)` | Place and cancel orders | Spot actions plus margin configuration and reduce-only orders |
| `.with_wallet(address, private_key)` | Configures both rows | Configures both rows |

The address is sent to Hyperliquid's account-query APIs without a local
signature. The private key is used only for local signing and is redacted from
`Debug`. Values are validated by the first dependent call; `Client::supports`
checks whether the relevant address or signer was configured, not whether the
provider will accept the request.

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

Access the following provider-specific methods through `Client::adapter()`:

| Method | Contract |
| --- | --- |
| `asset_context(&market)` | Mid, mark, oracle, funding, open interest, and order precision |
| `basic_open_orders()` | Public-address-bound, unsigned `POST /info` `openOrders` read. This compact provider response is deliberately separate from the common open-order surface, which uses Hyperliquid's richer `frontendOpenOrders` response. Fixture-verified only |
| `order_status(reference)` | Public-address-bound, unsigned `POST /info` `orderStatus` read. `reference` accepts a numeric server `oid` or a `0x`-prefixed 32-hex-character client order ID. `unknownOid` is the normal `HyperliquidOrderStatusResponse::UnknownOrder` result; future top-level statuses retain their status string and raw JSON. Fixture-verified only |
| `historical_orders()` | Public-address-bound, unsigned `POST /info` `historicalOrders` read of up to the latest 2,000 orders. Detailed orders retain Hyperliquid trigger, time-in-force, reduce-only, client-ID, status, and raw JSON fields. Fixture-verified only |
| `user_fills(aggregate_by_time)` | Public-address-bound, unsigned `POST /info` `userFills` read. A configured query address is required; `aggregate_by_time` selects Hyperliquid's partial-fill aggregation. Preserves execution, account-position, fee, order, direction, and raw provider data instead of widening common `Trade`. Fixture-verified only |
| `user_fills_by_time(from, to, aggregate_by_time)` | Public-address-bound, unsigned `POST /info` `userFillsByTime` read with required `from` and optional `to`. Both provider millisecond boundaries are inclusive; caller boundaries are rounded so the returned range stays within the requested nanosecond range. Shares `aggregate_by_time` and raw-field preservation with `user_fills`. Fixture-verified only |
| `non_funding_ledger(from, to, cursor, limit)` | Deposits, withdrawals, transfers, and liquidations; funding excluded; configured public query address required, with no signature; provider page `<= 500` |
| `user_rate_limit()` | Public `POST /info` `userRateLimit` read for the configured address; cumulative volume plus current request use, cap, and surplus |
| `user_role()` | Public `userRole` read; recognized roles are typed and an unknown provider role is retained |
| `referral()` | Public `referral` read; stable balances are typed and provider-owned referral state remains JSON text |
| `user_fees()` | Public `userFees` read; current account rates and daily volumes are typed while the provider fee schedule remains JSON text |
| `portfolio()` | Public `portfolio` read; account-value and PnL histories by provider period |
| `sub_accounts()` | Public `subAccounts` read; a provider `null` means an empty list |
| `user_vault_equities()` | Public `userVaultEquities` read; current vault equity positions |

`all_mids()` is a public, read-only snapshot and is fixture-verified; live reads
have not been verified. The provider's default empty `dex` selects the first
perpetual DEX, and Spot mids are included only for that DEX. If a book is empty,
Hyperliquid uses the last trade price.

All address-scoped Info reads above, including the three order-query methods,
require a valid public address configured through `with_query_address(...)` or
`with_wallet(...)`. They use no API key, private key, or local signature; a
missing or invalid address fails before a network request. They are
fixture-verified only; live reads have not been verified. See Hyperliquid's
[official Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint).

| `non_funding_ledger` field or state | Contract |
| --- | --- |
| `from`, `to` | Provider millisecond range `from_ms <= time <= to_ms` |
| `cursor` | Opaque resume point; overrides `from` |
| `limit` | Local page-size target; one millisecond group is never split, so `items.len()` may be below or above `limit` |
| `limit == 0` | `Error::InvalidRequest` before credential checks or network I/O |
| Continuation | Set `cursor = page.next` until `page.next == None` |

Unknown ledger `type` strings are preserved as
`HyperliquidLedgerKind::Other(provider_name)` instead of being collapsed to a
generic value.

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
- [Account-scoped Info reads](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [User fills (`userFills`, `userFillsByTime`)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [All mids (`allMids`)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Perpetual info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Spot info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [Rate limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

[Common API](../common-api.md) · [Provider support](../providers.md)
