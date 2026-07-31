[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

# Hyperliquid

Spot and perpetual markets on one chain-settled venue. One
`HyperliquidAdapter` serves both, and the distinction rides on `Market.kind`.

```rust
use maxt::{Client, Feature, adapters::HyperliquidAdapter};

let client = Client::new(HyperliquidAdapter::new());
let testnet = Client::new(HyperliquidAdapter::testnet());

assert!(client.supports(Feature::TradeStream)); // live: every trade
assert!(client.supports(Feature::Trades));      // REST: the last ten
```

## Venues

| Subject | Spot | Perpetual |
| --- | --- | --- |
| `Market` constructor | `Market::spot`, resolved through the token table | `Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC")`, a bare coin name settled in USDC |
| Native symbol | `@107`, or a legacy slash form such as `PURR/USDC`. Both resolve | the coin name |
| Pair resolution | a token table the adapter loads on its first call and keeps for its lifetime, so a market listed later needs a fresh adapter. Discover through `markets(MarketKind::Spot)`; `MarketInfo::native_symbol` reconciles against Hyperliquid's UI | none needed |
| Balances | `balances()` | `margin_summary()`, in USDC |
| `funding_rates`, `funding_payments` | `Error::Unsupported` naming the feature | yes |
| `set_margin` | `Error::Unsupported` naming `Feature::MarginConfig` | yes |
| `reduce_only` on an order | `Error::Unsupported` naming `Feature::ReduceOnlyOrders` | yes |
| `positions_on(&market)` | `Ok(vec![])`, not an error: it reads the perpetual account and filters. Indistinguishable by return value from a flat perpetual, so branch on `Market::kind` | the position, if any |
| `positions_on` on an unlisted market | `Error::InvalidRequest` on `market` | `Error::InvalidRequest` on `market` |

## Limits

Checked before the request is built.

| Call | Range | Outside it |
| --- | --- | --- |
| `trades` | `limit` up to 10, or unset for all 10. `recentTrades` takes no count, so 10 is the whole endpoint and a wider gap is recoverable only from `Feed::Trades` | above 10, `Error::InvalidRequest` on `limit` |
| `order_book` | `depth` 1 to 20 | `Error::InvalidRequest` |
| `candles` | any `limit`; 5 000 per call, paged for you over at most a hundred calls | a window past 500 000 candles is `Error::InvalidRequest` |
| Candle intervals | fourteen: the [baseline](../common-api.md#intervals) ten plus `Hour2`, `Hour8`, `Hour12` and `Day3` | `Interval::Sec1` is `Error::Unsupported`, on `candles` and `Feed::Candles` alike |
| `Sec1` together with a bad `limit` or window | `limit` and window are checked first | `Error::InvalidRequest` on that field, not `Unsupported` |
| Order price | at most the asset's price decimals, and at most 5 significant figures once it has a fractional part | `Error::InvalidRequest` naming the field |
| Order size | at most the asset's own decimal count | `Error::InvalidRequest` naming the field |
| `funding_rates`, `funding_payments`, `non_funding_ledger` | 500 entries per page | follow `Page::next` until `None` |

## Order precision and minimum size

Both rules are enforced before signing, and the error message carries the
permitted count. Both counts come off `HyperliquidAssetContext`, so an order can
be built to fit:

```rust
use maxt::{Client, Exchange, Market, adapters::HyperliquidAdapter};

async fn precision() -> maxt::Result<()> {
    let client = Client::new(HyperliquidAdapter::new());
    let market = Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC");
    let context = client.adapter().asset_context(&market).await?;

    println!(
        "sizes take {} decimals, prices {}",
        context.size_decimals, context.price_decimals
    );
    Ok(())
}
```

| Field | Value |
| --- | --- |
| `price_decimals` | 8 on spot, 6 on perpetuals, less that asset's size decimals. It cannot express the five-significant-figure rule, which applies on top of it to any price with a fractional part |
| `size_decimals` | the asset's own decimal count |
| Minimum order value | Hyperliquid's own rule. `maxt` does not check it |

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::OrderBook` | `l2Book` without Hyperliquid's `fast` flag: 20 levels per side, the REST depth, not configurable |
| Book events | full snapshots, not diffs, overwrite your copy |
| `nSigFigs`, `mantissa` | Hyperliquid's price aggregation. The common API has no field for it |
| `Candle::closed` | `true` on one emission per window, sent when Hyperliquid opens the next one. See below |
| Candle events per window | many with `closed` false, then exactly one with it `true`. The settled one carries the figures of that window's own last frame |
| A reconnect | drops the window being held, so the window a `MarketEvent::Reconnected` interrupts gets no `closed` emission |
| `subscribe_account` | spot balances, as `balances()` does |
| Keepalive | `{"method":"ping"}` every 15 seconds |

### `Candle::closed` on the stream

**Hyperliquid stops publishing a window about two seconds before that window's
own close time, so the payload's `T` is still in the future when the last frame
of the window arrives.**

| Frame, `candle` `1m` `BTC`, 2026-07-30 | `t` | `T` | Received |
| --- | --- | --- | --- |
| last one the 07:45 window got | 07:45:00.000 | 07:45:59.999 | 07:45:57.557 |
| first one of the next window | 07:46:00.000 | 07:46:59.999 | 07:46:01.416 |

Over 210 seconds and five windows, the last frame of a window arrived 1.7, 2.1
and 2.4 seconds before its own `T`, and 0.3 seconds after it once. So `maxt`
holds the most recent frame of each market and interval and emits it with
`closed` set when a frame opens a later window. The settled emission arrives one
frame late, and a window with no successor never settles.

## Quotas

Hyperliquid's
[published limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)
are per IP and per address. The address budget has no counterpart on the other
three exchanges.

| Budget | Allowance |
| --- | --- |
| Per IP | 1 200 weight a minute across all REST requests. Weight is per endpoint, not per call |
| Per address | one *action* per USDC traded cumulatively since the address was created, on top of an opening buffer of 10 000. Past that, one every 10 seconds. Cancels get a larger cumulative allowance |
| WebSocket | 10 connections, 30 new connections a minute, 1 000 subscriptions, 2 000 outbound messages a minute, 100 inflight post messages |

| Weight | Charged by |
| --- | --- |
| 2 | `l2Book`, `allMids`, `clearinghouseState`, `orderStatus`, `spotClearinghouseState`, `exchangeStatus` |
| 20 | every other documented info request |
| 60 | `userRole` |
| 1 per 20 items returned | the paged endpoints, on top of their own weight |
| 1 per 60 candles | the candle snapshot, on top of its own weight |

The address budget charges actions, not reads. Hyperliquid's page says it
verbatim: the address limit "only applies to actions, not info requests".
Everything readable goes to `POST /info`, everything that changes state to
`POST /exchange`.

| Charged to the address budget | Not charged |
| --- | --- |
| `place_order`, `cancel_order`, `set_margin` | `balances`, `positions`, `positions_on`, `open_orders`, `open_orders_on`, `margin_summary`, `funding_rates`, `funding_payments`, `non_funding_ledger`, and every public read |

Batched orders count as one request against the IP budget and `n` against the
address budget; `maxt` sends one order per action. `maxt` does not
[throttle](../common-api.md#rate-limits) against any of these, and its keepalive
spends 4 of the 2 000 outbound messages a minute per connection.

## Surprises

| Field or call | Behaviour |
| --- | --- |
| HTTP 200 | can carry a rejection. Order, cancel and leverage failures are reported inside a success-status body, and a per-order rejection can hide inside an envelope that already said `ok`. The adapter reads both and raises `Error::Exchange` |
| `Ticker::timestamp` | the moment `maxt` read it; asset contexts carry no clock |
| `Ticker::last_trade_time` | `None`, for the same reason |
| `Ticker::high`, `low` | `None`; the asset context carries no session high or low |
| `Interval::Month1` | a 30-day bucket on a fixed grid measured from the Unix epoch, not a calendar month. Opens run 2026-05-07, 2026-06-06, 2026-07-06, every one at 00:00 UTC and shared by every market: there is no June bucket and none closes on 1 July. `maxt` reports Hyperliquid's own `open_time` |
| `Interval::Week1` | a 7-day bucket from the same epoch, so it opens **Thursday** 00:00 UTC, not Monday as Upbit's and Binance's do. 1 January 1970 was a Thursday |
| `Interval::Day3` | a 3-day bucket from the same epoch, at 00:00 UTC. Binance's `Day3` is one day ahead of it |
| `Interval::Day1`, `Hour12` and shorter | the same UTC grid as Upbit's and Binance's. Only `Day3`, `Week1` and `Month1` are measured from the epoch |
| `candleSnapshot` `endTime` | two different meanings across Hyperliquid's history. A bucket from before roughly mid-2023 is served once `endTime` reaches that bucket's close, a newer one as soon as `endTime` reaches its open. `maxt` asks one interval further on and cuts the answer back, so a `from` with a `limit` returns the count it asked for on either side of that line |
| `balances()` | spot only. Perpetual collateral is in `margin_summary()`, in USDC |
| `MarginSummary::available_balance` | the figure that governs headroom, and the one to size a new position against. `margin_balance` is margin already posted, and `equity` includes unrealized PnL |
| Order types | limit only, `Size::Base` only, price required. No market order: send an immediate-or-cancel limit priced through the book. No fill-or-kill either |
| Place-order acknowledgement | says filled or rested, not how much filled: a fill reads as complete, a rest as untouched |
| Cancel acknowledgement | the verdict and nothing else; the returned `Order` carries placeholder side, size and price. Read the order back with `open_orders` |
| `open_orders_on`, `positions_on` | filter client-side. Hyperliquid answers both for the whole account and takes no market |
| A closed position | absent from `clearinghouseState` rather than reported at size zero |
| `set_margin` | one action, so leverage and margin mode must both be given; one alone is `Error::InvalidRequest`. Leverage is a whole number within the asset's own cap, and some assets take isolated margin only |
| Builder-deployed perpetuals | the ones whose name contains a colon. Separate universe, separate asset numbering, absent from `markets()` |
| Wallet shape | checked by the first call that needs it, not by `with_wallet`, which stays infallible |
| No wallet | `Error::Auth`, and `Client::supports` answers `false` for every account feature |
| A signature Hyperliquid refused | `Error::Exchange`, not `Error::Auth`. HTTP 200 with `{"status":"err","response":"User or API Wallet 0x… does not exist."}`, where the address is whatever the bad signature recovered to and so differs per request. There is no code to branch on. **Documented, not measured** |

## Hyperliquid-only calls

Through `Client::adapter()`.

| Method | Returns |
| --- | --- |
| `non_funding_ledger(from, to, cursor, limit)` | deposits, withdrawals, transfers between wallets and subaccounts, vault movements, liquidations. Amounts are unsigned magnitudes and the direction is the entry's `kind`; a liquidation carries no single amount. Pages like `funding_payments`: pass `Page::next` back as `cursor` until it is `None` |
| `asset_context(&market)` | mark, mid and oracle price, open interest, and the funding rate currently accruing, which is not what `FundingRate` reports: that records funding already charged |

```rust
use maxt::{Client, Exchange, Market, adapters::HyperliquidAdapter};

async fn accruing_funding() -> maxt::Result<()> {
    let client = Client::new(HyperliquidAdapter::new());
    let context = client
        .adapter()
        .asset_context(&Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"))
        .await?;

    if let Some(rate) = context.funding_rate {
        println!("hourly funding is running at {rate}");
    }
    Ok(())
}
```

## Credentials

A wallet address and a hex private key, not an API key.

```rust
use maxt::{Client, adapters::HyperliquidAdapter};

fn client() -> Client<HyperliquidAdapter> {
    let address = std::env::var("HYPERLIQUID_ADDRESS").expect("HYPERLIQUID_ADDRESS");
    let key = std::env::var("HYPERLIQUID_PRIVATE_KEY").expect("HYPERLIQUID_PRIVATE_KEY");

    Client::new(HyperliquidAdapter::new().with_wallet(address, key))
}
```

| Subject | Rule |
| --- | --- |
| The key | signs, and nothing else. Each private request is encoded, hashed and signed locally with secp256k1 over an EIP-712 digest. The wire carries the action, a nonce, the signature, and, for reads, the account address. The key itself is never sent |
| `Debug` | redacts it, so a `{:?}` on the adapter cannot put it in a log |
| Address | 20 bytes of hex with a `0x` prefix, lowercased before it is sent: Hyperliquid matches `user` fields literally and a checksummed address reads back as an empty account with no error |
| Private key | 32 bytes of hex, with or without a `0x` prefix. Anything else is `Error::Auth` from the first call that needs it |
| Which key | prefer an approved API wallet key over the account's own. Both sign the actions this adapter sends, and an API wallet key cannot withdraw. The address you pass is the account being acted on either way |
| Public market data | needs no wallet at all |

## Examples

`cargo run --example public_rest -- hyperliquid HYPE USDC`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Hyperliquid's own docs

- [Rate limits and user limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Info endpoint: perpetuals](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Info endpoint: spot](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [Exchange endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
