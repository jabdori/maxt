[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

# Hyperliquid

`HyperliquidAdapter` exposes public spot and the default perpetual DEX through one API. Public calls need no wallet.

## Constructors

| Constructor | Use |
| --- | --- |
| `HyperliquidAdapter::new()` | Mainnet public client |
| `HyperliquidAdapter::testnet()` | Testnet public client |
| `.with_wallet(address, private_key)` | Adds credentials for account and trading calls; validation happens on the first private call |

## Market coverage

| Market | Source | Coverage |
| --- | --- | --- |
| Spot | `spotMeta` | Exposed, including pairs whose quote asset is not USDC |
| Default perpetual DEX | `meta` without `dex` | Exposed as `MarketKind::Perpetual` |
| HIP-3 perpetual DEXs | `perpDexs`, `meta` with `dex` | Not exposed |
| Outcome assets | `outcomeMeta` | Not exposed |

The adapter caches `meta` and `spotMeta` on first use. Create a new adapter to discover listings added later. Spot wire symbols are usually `@{index}`; use `markets(MarketKind::Spot)` instead of guessing them.

## Public REST

| Call | Hyperliquid request | Contract |
| --- | --- | --- |
| `markets(kind)` | `meta`, `spotMeta` | Lists the covered markets above |
| `trades(market, limit)` | `recentTrades` | Newest first; when set, `limit` must be `1..=10`; unset returns the endpoint's ten-trade window |
| `order_book(market, depth)` | `l2Book` | When set, `depth` must be `1..=20`; the adapter trims the full response locally |
| `ticker(market)` | `metaAndAssetCtxs`, `spotMetaAndAssetCtxs` | A reference-price summary, not a last-trade ticker; see below |
| `candles(request)` | `candleSnapshot` | Fourteen intervals from `1m` through `1M`; `Sec1` is unsupported |
| `funding_rates(request)` | `fundingHistory` | Public and perpetual-only; time-ranged responses are at most 500 entries |

Hyperliquid serves only the most recent 5,000 candles for an interval. This is a retention window, not a page size. A larger local `limit` cannot recover data the venue no longer serves.

Hyperliquid's `1M` candles use a fixed 30-day grid rather than calendar months.

For `HistoryRequest`, `from` is inclusive and `to` is exclusive. Hyperliquid's `endTime` is inclusive and has millisecond precision, so the adapter sends the last millisecond strictly before `to`. Follow `Page::next` until it is `None`.

## Ticker semantics

| Field | Meaning on Hyperliquid |
| --- | --- |
| `last_price` | `midPx`, falling back to `markPx`; despite the field name, this is not the most recent trade price |
| `last_trade_time` | `None`; asset contexts carry no trade timestamp |
| `timestamp` | When `maxt` read the context; the context has no timestamp |
| `change`, `change_rate` | The reference price above compared with `prevDayPx` |

Use `trades` or `Feed::Trades` when the exact recent execution price and time matter.

## Public streams

| Feed | Subscription | Behaviour |
| --- | --- | --- |
| `Feed::Trades` | `trades` | Executions with exchange timestamps |
| `Feed::OrderBook` | `l2Book` | Full snapshots with 20 levels per side, not diffs |
| `Feed::Ticker` | `activeAssetCtx` | The same reference-price semantics as REST ticker |
| `Feed::Candles(interval)` | `candle` | Forming updates plus one locally marked closed candle when the next window opens |
| Keepalive | `{"method":"ping"}` | Sent every 15 seconds; Hyperliquid closes an idle connection after 60 seconds |

The official `l2Book` aggregation options are `nSigFigs` and `mantissa`. The common API does not expose them and sends neither.

## Hyperliquid-only calls

| Method | Use |
| --- | --- |
| `asset_context(&market)` | Public mark, mid and oracle prices, current funding, open interest, and order precision |
| `non_funding_ledger(from, to, cursor, limit)` | Wallet-required deposits, withdrawals, transfers and liquidations that are not funding payments |

## Precision and minimum notional

| Rule | Value |
| --- | --- |
| Size decimals | The asset's `szDecimals` |
| Perpetual price decimals | `6 - szDecimals` |
| Spot price decimals | `8 - szDecimals` |
| Significant digits | A non-integer price may carry at most five significant digits |
| Minimum notional | Set by Hyperliquid and not pre-validated by `maxt` |

`asset_context` exposes the decimal-place limits. Both the decimal-place limit and significant-digit rule apply to an order price.

## Rate limits

| Scope | Official limit |
| --- | --- |
| REST | 1,200 aggregate request weight per minute per IP; endpoint weights differ |
| WebSocket connections | 10 concurrent and 30 new connections per minute |
| WebSocket subscriptions | 1,000 |
| WebSocket messages sent | 2,000 per minute across all connections |

`maxt` does not throttle requests. Budget by weight and back off on `Error::is_rate_limited()`.

## Wallet security

- Hyperliquid uses a wallet address and private key, not an API key.
- Prefer an approved API wallet key over the account key. It can trade but cannot withdraw.
- Signing is local. The private key is never sent and is redacted from `Debug` output.
- Public REST and streams work without `.with_wallet(...)`.

## Verification scope

On 2026-07-31, representative mainnet spot and default-perpetual markets passed public REST and `Trades`, `OrderBook`, `Ticker`, and `Candles` stream smoke checks. Private live calls were not verified.

## Examples

```text
cargo run --example public_rest -- hyperliquid HYPE USDC
cargo run --example public_stream -- hyperliquid HYPE USDC
```

## Official documentation

- [API overview](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Perpetual info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Spot info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [Asset IDs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/asset-ids)
- [WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [Timeouts and heartbeats](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/timeouts-and-heartbeats)
- [Rate limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
