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

| What differs | Spot | Perpetual |
| --- | --- | --- |
| `Market` constructor | `Market::spot`, resolved through the token table | `Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC")`, a bare coin name settled in USDC |
| Native symbol | `@107`, or a legacy slash form such as `PURR/USDC` | the coin name |
| Balances | `balances()` | `margin_summary()`, in USDC |
| `funding_rates`, `funding_payments` | `Error::Unsupported` naming the feature | yes |
| `set_margin` | `Error::Unsupported` naming `Feature::MarginConfig` | yes |
| `reduce_only` on an order | `Error::Unsupported` naming `Feature::ReduceOnlyOrders` | yes |
| `positions_on(&market)` | `Ok(vec![])` | the position, if any |

**`positions_on` on a spot market is not an error.** It resolves the market,
reads the perpetual account, and filters, which leaves an empty list. A listed
spot market is therefore indistinguishable from a flat perpetual one by return
value alone. Branch on `Market::kind` if what you meant to ask was "is this the
wrong kind of market", because `Unsupported` will not tell you. An *unlisted*
market is `Error::InvalidRequest` on `market`, from the same lookup.

Spot pairs are a table, not a formula. Nothing in `@107` says `HYPE`; base and
quote come from a token table the adapter loads on its first call and keeps for
its lifetime, so a market listed after your adapter connected will not resolve
and picking it up means a fresh adapter. Discover pairs through
`markets(MarketKind::Spot)` rather than guessing, and read
`MarketInfo::native_symbol` to reconcile against Hyperliquid's own UI. Both
spellings read back.

## Limits

Checked before the request is built.

| Call | Accepts | Outside it |
| --- | --- | --- |
| `trades` | `limit` up to 10, or unset for all 10 | above 10, `Error::InvalidRequest` on `limit` |
| `order_book` | `depth` 1 to 20 | `Error::InvalidRequest` |
| `candles` | any `limit`; 5 000 per call, paged for you over at most a hundred calls | a window past 500 000 candles is `Error::InvalidRequest` |
| Candle intervals | fourteen: the [baseline](../common-api.md#intervals) ten plus `Hour2`, `Hour8`, `Hour12` and `Day3` | `Interval::Sec1` is `Error::Unsupported`, on `candles` and `Feed::Candles` alike |
| Order price | at most the asset's price decimals, and at most 5 significant figures once it has a fractional part | `Error::InvalidRequest` naming the field |
| Order size | at most the asset's own decimal count | `Error::InvalidRequest` naming the field |
| `funding_rates`, `funding_payments`, `non_funding_ledger` | 500 entries per page | follow `Page::next` until `None` |

`trades` is a ten-trade window, not a backfill. `recentTrades` takes no count,
so ten is everything the endpoint offers and a larger `limit` cannot be served
by asking differently. A gap wider than the last ten trades is not recoverable
over REST here; `Feed::Trades` is where a continuous record comes from.

Those `limit` and window checks run before the interval is looked up, so a
`Sec1` request that also breaks one of them, a `limit` of 0 or a window past
500 000, is `Error::InvalidRequest` naming that field rather than the
`Unsupported` the interval alone would have raised. Match both if you branch on
the difference.

## Order precision and minimum size

Hyperliquid states an order's precision as decimal counts per asset, and `maxt`
enforces both before signing: a price carries at most the asset's price
decimals and, once it has a fractional part, at most 5 significant figures; a
size carries at most the asset's size decimals. Breaking either is
`Error::InvalidRequest` naming the field, with the permitted count in the
message, rather than a round trip spent on a rejection.

Both counts come off `HyperliquidAssetContext`, so an order can be built to fit
rather than rejected:

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

`price_decimals` is 8 on spot and 6 on perpetuals, less that asset's size
decimals. It cannot express the five-significant-figure rule, which applies on
top of it to any price with a fractional part.

Minimum order value is Hyperliquid's own rule and `maxt` does not check it.

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::OrderBook` | `l2Book` without Hyperliquid's `fast` flag: 20 levels per side, the REST depth, not configurable |
| Book events | full snapshots, not diffs, overwrite your copy |
| `nSigFigs`, `mantissa` | Hyperliquid's price aggregation, not reachable through the common API |
| `Candle::closed` | `true` on one emission per window, sent when Hyperliquid opens the next one. See below |
| Candle events per window | many with `closed` false, then exactly one with it `true`. The settled one carries the figures of that window's own last frame |
| A reconnect | drops the window being held rather than settling it across the gap, so the window a `MarketEvent::Reconnected` interrupts gets no `closed` emission |
| `subscribe_account` | spot balances, as `balances()` does |
| Keepalive | `{"method":"ping"}` every 15 seconds |

### `Candle::closed` on the stream

**Hyperliquid stops publishing a window about two seconds before that window's
own close time, so the payload's `T` is still in the future when the last frame
of the window arrives.** A frame read on its own cannot say the bar has settled,
and a clock reading of one says so only by luck.

| Frame on `candle` `1m` `BTC`, 2026-07-30 | `t` | `T` | received at |
| --- | --- | --- | --- |
| last one the 07:45 window got | 07:45:00.000 | 07:45:59.999 | 07:45:57.557 |
| first one of the next window | 07:46:00.000 | 07:46:59.999 | 07:46:01.416 |

Over 210 seconds and five windows, the last frame of a window arrived 1.7, 2.1
and 2.4 seconds before its own `T` on three of them, and 0.3 seconds after it on
one. So `maxt` holds the most recent frame of each market and interval and
emits it with `closed` set when a frame opens a later window, which is the only
announcement Hyperliquid ever makes that a window is over. The settled emission
therefore arrives one frame late, and a window with no successor never settles.

## Quotas

Hyperliquid's
[published limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)
have two halves, and the second has no counterpart on the other three
exchanges.

| Budget | Allowance |
| --- | --- |
| Per IP | 1 200 weight a minute across all REST requests. Weight is per endpoint, not per call |
| Per address | one *action* per USDC traded cumulatively since the address was created, on top of an opening buffer of 10 000. An account that has never traded has 10 000 actions, and past that is throttled to one every 10 seconds. Cancels get a larger cumulative allowance, so a throttled address can still clear its book |
| WebSocket | 10 connections, 30 new connections a minute, 1 000 subscriptions, 2 000 outbound messages a minute, 100 inflight post messages |

Weight is not flat, and the cheap tier is a short list rather than the common
case. `l2Book`, `allMids`, `clearinghouseState`, `orderStatus`,
`spotClearinghouseState` and `exchangeStatus` cost 2. Every other documented
info request costs 20, and `userRole` costs 60. The paged endpoints add weight
per 20 items returned, and the candle snapshot per 60, so a wide backfill costs
more than its call count suggests.

**The address budget charges actions, not reads.** Hyperliquid's page says it
verbatim: the address limit "only applies to actions, not info requests".
Everything readable goes to `POST /info` and everything that changes state goes
to `POST /exchange`, so of `maxt`'s calls exactly three are actions:

| Charged to the address budget | Not charged |
| --- | --- |
| `place_order`, `cancel_order`, `set_margin` | `balances`, `positions`, `positions_on`, `open_orders`, `open_orders_on`, `margin_summary`, `funding_rates`, `funding_payments`, `non_funding_ledger`, and every public read |

A read-only polling loop therefore spends nothing from the lifetime budget, only
IP weight. A program that places orders spends the whole of it on orders. Sizing
one against the other is how a trading loop runs out of budget it thought it had.
Batched orders count as one request for the IP budget and as `n` for the address
budget; `maxt` sends one order per action, so the two agree here.

`maxt` does not throttle against any of them. Its own keepalive spends 4 of the
2 000 outbound messages a minute per connection.

## Surprises

| Field or call | What to expect |
| --- | --- |
| HTTP 200 | can carry a rejection. Order, cancel, and leverage failures are reported inside a success-status body, and a per-order rejection can hide inside an envelope that already said `ok`. The adapter reads both and raises `Error::Exchange`. A status-code check alone would read a rejection as a success. |
| `Ticker::timestamp` | the moment `maxt` read it; asset contexts carry no clock |
| `Ticker::last_trade_time` | `None`, for the same reason |
| `Ticker::high`, `low` | `None`; the asset context carries no session high or low |
| `Interval::Month1` | a 30-day bucket on a fixed grid measured from the Unix epoch, not a calendar month. Opens run 2026-05-07, 2026-06-06, 2026-07-06, every one at 00:00 UTC and shared by every market: there is no June bucket and none closes on 1 July. `maxt` reports Hyperliquid's own `open_time`, so a monthly series from here does not line up with a monthly series from the other three. |
| `Interval::Week1` | a 7-day bucket measured from the same epoch, so it opens on a **Thursday** at 00:00 UTC, not on a Monday as Upbit's and Binance's do. 1 January 1970 was a Thursday. |
| `Interval::Day3` | a 3-day bucket measured from the same epoch, at 00:00 UTC. Binance's `Day3` is one day ahead of it, so those two do not line up either. |
| `Interval::Day1`, `Hour12` and shorter | on the same UTC grid as Upbit's and Binance's. Only `Day3`, `Week1` and `Month1` are measured from the epoch. |
| `candleSnapshot` `endTime` | means two different things across Hyperliquid's history. A bucket from before roughly mid-2023 is served only once `endTime` reaches that bucket's close, and a newer one as soon as `endTime` reaches its open. `maxt` asks one interval further on and cuts the answer back, so a `from` with a `limit` returns the count it asked for on either side of that line. |
| `balances()` | spot only. Perpetual collateral is in `margin_summary()`, denominated in USDC |
| `MarginSummary::available_balance` | the figure that governs headroom, and the one to size a new position against. `margin_balance` is margin already posted, and `equity` includes unrealized PnL |
| Order types | limit only, `Size::Base` only, price required. No market order: send an immediate-or-cancel limit priced through the book. No fill-or-kill either. |
| Place-order acknowledgement | says filled or rested, not how much filled: a fill reads as complete, a rest as untouched |
| Cancel acknowledgement | the verdict and nothing else; the returned `Order` carries placeholder side, size, and price. Read the order back with `open_orders`. |
| `open_orders_on`, `positions_on` | filter client-side. Hyperliquid answers both questions for the whole account and takes no market. |
| `set_margin` | one action, so leverage and margin mode must both be given; one alone is `Error::InvalidRequest`. Leverage is a whole number within the asset's own cap, and some assets take isolated margin only. |
| Builder-deployed perpetuals | the ones whose name contains a colon. Separate universe, separate asset numbering, absent from `markets()`. |
| Wallet shape | checked by the first call that needs it, not by `with_wallet`, which stays infallible |
| No wallet | `Error::Auth` |
| A signature Hyperliquid refused | `Error::Exchange`, not `Error::Auth`. Hyperliquid answers HTTP 200 with `{"status":"err","response":"User or API Wallet 0x… does not exist."}`, and the address in it is whatever the bad signature recovered to, so it differs per request. There is no code to branch on, which is why nothing here promotes it. **Documented, not measured:** no Hyperliquid wallet was available for this crate's verification |

## Hyperliquid-only calls

Through `Client::adapter()`.

| Method | Gives you |
| --- | --- |
| `non_funding_ledger(from, to, cursor, limit)` | deposits, withdrawals, transfers between wallets and subaccounts, vault movements, liquidations |
| `asset_context(&market)` | mark, mid, and oracle price, the funding rate currently accruing, open interest |

Ledger entries are not `FundingPayment`s. Funding is a periodic charge against a
position in one market, while these are cash movements that belong to no market
at all, and forcing a withdrawal into the funding shape would have to name a
market it never touched. Amounts are unsigned magnitudes and the direction is
the entry's `kind`; a liquidation carries no single amount at all. The ledger
pages exactly like `funding_payments`: pass `Page::next` back as `cursor` until
it is `None`.

Likewise `FundingRate` records what funding *was* charged, a different question
from what the next charge is running at, and neither open interest nor an
oracle price has a common counterpart to carry it.

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

| Subject | What to know |
| --- | --- |
| The key | signs, and nothing else. Each private request is encoded, hashed, and signed locally with secp256k1 over an EIP-712 digest. The wire carries the action, a nonce, the signature, and, for reads, the account address. The key itself is never sent. |
| `Debug` | redacts it, so a `{:?}` on the adapter cannot put it in a log |
| Address | 20 bytes of hex with a `0x` prefix, lowercased before it is sent, because Hyperliquid matches `user` fields literally and a checksummed address reads back as an empty account with no error |
| Private key | 32 bytes of hex, with or without a `0x` prefix. Anything else, or a missing wallet, is `Error::Auth` from the first call that needs it. |
| Which key | prefer an approved API wallet key over the account's own. Both sign the actions this adapter sends, and an API wallet key cannot withdraw. The address you pass is the account being acted on either way. |

Public market data needs no wallet at all. Until one is set, `Client::supports`
answers `false` for every account feature.

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
