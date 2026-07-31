[English](binance.md) | [한국어](binance.ko.md)

# Binance

Spot and USD-margined perpetual futures. One `BinanceAdapter` talks to one
venue, chosen at construction.

```rust
use maxt::{Client, adapters::BinanceAdapter};

let spot = Client::new(BinanceAdapter::spot());
let perp = Client::new(BinanceAdapter::usd_m_futures());
```

## Venues

`BinanceAdapter::default()` is spot. `BTCUSDT` exists on both venues at
different prices. A `Market` of the wrong kind is `Error::InvalidRequest`,
raised before the request reaches the network; `supports` answers about the
adapter, not about a market, so it does not catch this.

| Subject | Spot | USD-M futures |
| --- | --- | --- |
| `Market` constructor | `Market::spot` | `Market::perpetual` |
| Positions, margin, funding, reduce-only | `Error::Unsupported` | yes |
| Candle intervals | all fifteen: the [baseline](../common-api.md#intervals) ten, plus `Sec1`, `Hour2`, `Hour8`, `Hour12` and `Day3` | fourteen: the same list without `Sec1` |
| Candles per response | 1 000 | 1 500 |
| REST book depths | 5, 10, 20, 50, 100, 500, 1 000, 5 000 | the same list without 5 000 |
| Live trades | `@trade`, every fill | `@trade`, every fill, plus a zero-priced frame per [spent trade id](#streams) that `maxt` drops |
| Book snapshot time | read time | Binance's stamp |
| Market order sizing | `Size::Base` or `Size::Quote` | `Size::Base` |
| Post-only | `LIMIT_MAKER` order type | `GTX` time in force |
| Weight budget | 6 000 per IP per minute | 2 400 per IP per minute |

One interval mapping serves REST and `Feed::Candles`, so an interval is
reachable on both or on neither. `Sec1` is the whole difference between the
venues: on USD-M it is `Error::Unsupported` naming `Feature::Candles`.

The `limit` and window checks below run before the interval lookup, so `Sec1`
on USD-M with a `limit` of 0, an inverted window, or a span past the paging
ceiling is `Error::InvalidRequest` naming that field, not `Unsupported`. Match
both, or validate the request before branching on the interval. This is the
shipped case behind
[a `true` from `supports` still needing a check at the call site](../common-api.md#a-true-still-has-to-be-checked-at-the-call).

## Limits

Checked before the request is built. Anything outside the range is
`Error::InvalidRequest` naming the field.

| Call | Accepted range |
| --- | --- |
| `trades` | `limit` 1 to 1 000 |
| `order_book` | a depth listed in the venue table. An unlisted depth is refused, not rounded up |
| `candles` | any `limit`, paged for you over at most a hundred calls. The ceiling is a hundred pages of whatever the venue serves per call: `limit` 0 names `limit`, and a window past 100 000 candles on spot or 150 000 on USD-M names `from`, both before the first call |
| `funding_rates`, `funding_payments` | `limit` 1 to 1 000, default 100 |
| `HistoryRequest` with no `limit` | 100 rows, which is a page and not the end of the history. Follow `Page::next` until `None` |

## Order precision and minimum size

Per symbol, on the `exchangeInfo` filters.

| Filter | Carries |
| --- | --- |
| `PRICE_FILTER` | tick size |
| `LOT_SIZE` | quantity step |
| `NOTIONAL` | smallest accepted price times quantity |

`maxt` neither checks an order against them nor rounds to them, so a price off
the tick is a rejection from Binance rather than an `Error::InvalidRequest`
from here. On spot, [`spot_symbol_filters`](#binance-only-calls) reads them.
Nothing in `maxt` carries them for USD-M; read
[`exchangeInfo`](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information)
yourself.

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::OrderBook` | `depth20@100ms`: 20 levels, 100 ms, both venues, not configurable |
| Book events | full snapshots, not diffs. Overwrite your copy |
| Deeper books | `Client::order_book` over REST, 5 000 levels on spot and 1 000 on USD-M |
| `Feed::Trades` | `@trade` on both venues: one event per fill |
| `Trade::id` | Binance's fill id, the same number on both venues and both transports |
| A USD-M `@trade` frame priced and sized at zero | a trade id Binance spent without publishing a fill. `maxt` drops it, so no `Trade` ever reaches you at a price of zero |
| USD-M endpoints | two, split by feed. See [the two USD-M entry points](#the-two-usd-m-entry-points) |

### Reconciling REST against the stream

A set of `Trade::id` deduplicates a REST backfill against a live subscription
on both venues. There is no tuple to fall back on and none is needed.

### Why not `@aggTrade`

`@aggTrade` collapses the fills one taker order swept at one price into a
single message. `maxt` carries it on the `/market` entry point below and
subscribes to `@trade` instead.

| Obstacle | Effect |
| --- | --- |
| A second id space | an `aggTrade` id numbers aggregates, not fills, and no REST call on either venue answers with one. Holding both transports, there is nothing to match on |
| No fallback tuple | its quantity is the sum of the fills it collapsed, so `(timestamp, price, quantity, taker_side)` does not reconcile it against REST either |

### The two USD-M entry points

Binance serves USD-M market data from two entry points on one host. A socket
naming neither is served as if it had named `/public`.

| Entry point | Streams | `maxt` subscriptions |
| --- | --- | --- |
| `wss://fstream.binance.com/public/stream` | what the matching engine pushes on change: `@trade`, `@depth*`, `@bookTicker` | `Feed::Trades`, `Feed::OrderBook` |
| `wss://fstream.binance.com/market/stream` | what an aggregator produces: `@aggTrade`, `@kline_*`, `@ticker`, `@miniTicker`, `@markPrice`, `@forceOrder`, `@compositeIndex`, `!contractInfo`, `!assetIndex@arr` | `Feed::Ticker`, `Feed::Candles` |
| `wss://fstream.binance.com/private/ws` | the account: `ORDER_TRADE_UPDATE`, `ACCOUNT_UPDATE`, `listenKeyExpired` | `subscribe_account` |

Nothing rejects a mismatch.

| Subscription | Answer |
| --- | --- |
| a stream belonging to the other entry point | `{"result": null, "id": 1}`, and then no frame for it, ever |
| a stream name Binance does not publish at all | the same acknowledgement, so it says nothing about whether data will follow |
| feeds from both entry points | two sockets merged into one `MarketStream`. Each reconnects on its own, so `MarketEvent::Reconnected` arrives once per socket that comes back rather than once per outage |

Spot market data is not split: `wss://stream.binance.com:9443/stream` carries
every feed.

`subscribe_account` on USD-M opens
`wss://fstream.binance.com/private/ws?listenKey=<key>&events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired`.

**`events` is an allow list, not a hint.** A socket receives the events its
filter names and nothing else.

| `events` filter | `listenKeyExpired` |
| --- | --- |
| absent | received |
| naming it, which is what `maxt` sends | received |
| `ORDER_TRADE_UPDATE/ACCOUNT_UPDATE` | **not** received |
| any filter on `wss://fstream.binance.com/ws/<key>` | **not** received. That path carries no user data |

Every event USD-M publishes, and whether the filter names it:

| Event | In the filter | Delivered as |
| --- | --- | --- |
| `ORDER_TRADE_UPDATE` | yes | `AccountEvent::Order` |
| `ACCOUNT_UPDATE` | yes | `AccountEvent::Balance` |
| `listenKeyExpired` | yes | `Error::Exchange`, so a stream with nothing left to say stops being waited on |
| `TRADE_LITE` | no | the same fill `ORDER_TRADE_UPDATE` already carries, sooner and with fewer fields |
| `MARGIN_CALL` | no | |
| `ACCOUNT_CONFIG_UPDATE` | no | |
| `CONDITIONAL_ORDER_TRIGGER_REJECT` | no | |
| `STRATEGY_UPDATE` | no | |
| `GRID_UPDATE` | no | |
| `ALGO_UPDATE` | no | |

An event the filter does not name never arrives, and `maxt` models none of the
seven. `eventStreamTerminated` is absent from the table because USD-M does not
publish it: it ends a WebSocket API session, and only the spot socket has one.

`ACCOUNT_UPDATE` arrives when a balance or a position actually changes, and a
resting order changes neither. Its locked margin rides on the `b` field of
`ORDER_TRADE_UPDATE`, the reserved bid notional, which `maxt` does not surface.
Read `balances()`.

### The spot account stream

Spot has no listen key. `subscribe_account` on a spot adapter opens
`wss://ws-api.binance.com:443/ws-api/v3` and sends one signed
`userDataStream.subscribe.signature`. The socket is opened unauthenticated and
the frame names the account, so nothing secret is in the URL and there is no
key to keep alive. Nothing filters events on this side, so a spot socket
receives everything the account produces and `maxt` drops what it does not
model.

| Method | HMAC-SHA-256 key | Ed25519 key |
| --- | --- | --- |
| `userDataStream.subscribe.signature`, signing every request, which is what `maxt` sends | works | works |
| `session.logon` then `userDataStream.subscribe` | `-2028 HMAC-SHA-256 API key is not supported`, then `-1193 WebSocket session not authenticated` | works |

**A reconnect signs a new frame.** The subscribe frame signs the millisecond
clock it was built at, so one signature subscribes one socket and no more:
Binance answers a replay past `recvWindow` with `-1021`. `maxt` signs again per
handshake and sends `recvWindow` 60 000 ms, Binance's documented maximum, which
is the ceiling and not a way past it.

A subscription Binance refuses is `Error::Exchange` on the stream carrying
Binance's own code, not a socket that stays open and quiet. Subscribe again
when you see one.

### A stream that yields nothing

A feed that delivers no events and no errors is indistinguishable from a market
where nothing is happening. `maxt` has no timer, no subscription
acknowledgement and no per-feed liveness signal.

| Suspect | Check |
| --- | --- |
| The feed | ask the same subject over REST. `Client::ticker` and `Client::candles` answer for every USD-M market, so a REST answer alongside a silent stream means the stream, not the market |
| The socket | set `StreamConfig::idle_timeout_ms`. A socket that has sent nothing for that long is torn down and rebuilt, and the rebuild arrives as `MarketEvent::Reconnected` |
| The entry point | subscribe to the same stream name on the other entry point with a raw socket and count frames |

## Quotas

Binance budgets **weight per IP per minute**, not requests, counted separately
per venue. A deep order book costs far more than a ticker. Every response
carries the running total in `X-MBX-USED-WEIGHT-1M`. Each venue publishes its
own ceiling in the `rateLimits` array of its `exchangeInfo` response, as a
`REQUEST_WEIGHT` entry over a one-minute interval, which is where the venue
table's 6 000 and 2 400 come from.

`maxt` does not throttle and does not read that header. Exceeding the budget is
HTTP 429, reported by `Error::is_rate_limited()`. Ignoring a 429 earns an
automated IP ban that scales from two minutes to three days, so back off on the
first one.

## Phantom positions

`/fapi/v3/positionRisk` opens a zero-amount row for any symbol that merely has
an order resting on it. `positions()` and `positions_on(&market)` return open
positions, and a row with no size is not one, so `maxt` drops it.

| Subject | Behaviour |
| --- | --- |
| Trigger | an open order on the symbol, and nothing else. An empty account never shows the row |
| Where the drop happens | on the common API, not in this adapter. Every flat row is dropped whichever adapter is underneath, so the guarantee is one filter rather than one per venue. Hyperliquid matches it at the venue, leaving a closed position out of `assetPositions` |
| The dropped row | preserved nowhere. Beyond the mark price, which is public, it said an order rests on that symbol, and `open_orders()` says so outright |
| A row `maxt` cannot parse | still `Error::Decode`. Only rows that parse and are flat are dropped |

## Surprises

| Field or call | Behaviour |
| --- | --- |
| `Ticker::last_trade_time` | always `None`; Binance never says when the last price traded |
| `Ticker::timestamp` | end of the 24-hour window, not a trade time |
| Spot book timestamp | read time; Binance publishes no clock on spot depth |
| `Position::leverage`, `margin_mode` | `None`; `/fapi/v3/positionRisk` carries neither. A symbol's configured leverage and margin mode are on `/fapi/v1/symbolConfig` instead, at the same weight |
| A symbol carrying only a resting order | not a position. Binance reports one; `maxt` drops it. See [phantom positions](#phantom-positions) |
| `FundingPayment::rate` | `None`; the ledger records the charge, not the rate |
| `MarginSummary::equity` | `totalMarginBalance`: wallet plus unrealized PnL |
| `MarginSummary::margin_balance` | `totalInitialMargin`: margin already consumed by open positions and orders. A cost, not a budget |
| `MarginSummary::available_balance` | `availableBalance`: what is free to open with, and the only one of the three that governs headroom |
| All three margin figures | denominated in `USDT` |
| USD-M `Balance::locked` | wallet minus available, floored at zero |
| Streamed order `created_at` | spot dates it from `O`, the creation time; USD-M from `T`, because `ORDER_TRADE_UPDATE` publishes no creation time. On a USD-M order that rests and later fills, that is the fill |
| `cancel_order`, `spot_order` | Binance's numeric order id only; a client order id is `Error::InvalidRequest` |
| `set_margin` | up to two calls, one per field, not atomic; leverage must be a whole multiplier at least 1 |
| Dated futures | dropped from `markets()`; reporting them as perpetuals would misprice them |
| Unknown quote asset on a stream frame | `Error::Decode`, never a wrong market |
| No credentials | `Error::Auth`, not `Error::Unsupported` |
| A credential Binance refused | `Error::Exchange`, not `Error::Auth`: HTTP 400 `-1022` for a bad signature, HTTP 401 `-2015` for a bad or unpermitted key, HTTP 401 `-2014` for no key |
| `-1021`, a timestamp outside `recvWindow` | `ExchangeErrorKind::Rejected`, so `is_retryable()` is `false`. Fix the clock, or build the request again and send it once |
| Spot account stream | no listen key; one signed request on the WebSocket API, signed again per reconnect. See [the spot account stream](#the-spot-account-stream) |
| A spot subscription Binance ends | `Error::Exchange` coded `eventStreamTerminated`. Spot has no listen key, so `listenKeyExpired` reaches only USD-M |
| The USD-M listen key | extended over REST every thirty minutes, and a failure to extend reaches the stream as Binance's own verdict. Left unresolved it stops carrying account changes within the hour |

## Binance-only calls

Through `Client::adapter()`.

| Method | Result | Absent on |
| --- | --- | --- |
| `spot_symbol_filters(&market)` | tick size, price and quantity bounds, lot step, minimum notional | USD-M |
| `spot_order(&market, id)` | one order by id, including filled and cancelled | USD-M |
| `usd_m_create_listen_key()` | a USD-M user data stream key | spot |
| `usd_m_keepalive_listen_key(&key)` | extends it another 60 minutes | spot |
| `usd_m_close_listen_key(&key)` | closes it | spot |

The filters keep Binance's shape. A field is `None` when the symbol carries no
filter of that kind, common on newly listed pairs.

`subscribe_account` runs the USD-M listen-key lifecycle, so the three key calls
are for driving the socket yourself, sharing one key between consumers, or
holding a key across a restart. `BinanceListenKey` goes into the socket URL, so
it is a bearer secret and its `Debug` output is redacted.

```rust
use maxt::{Client, Exchange, Market, adapters::BinanceAdapter};

async fn tick_size() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let filters = client
        .adapter()
        .spot_symbol_filters(&Market::spot(Exchange::Binance, "BTC", "USDT"))
        .await?;

    if let Some(tick) = filters.tick_size {
        println!("prices move in steps of {tick}");
    }
    Ok(())
}
```

## Credentials

An API key and a secret key. A key restricted to spot is rejected by the
futures API and the other way round.

```rust
use maxt::{Client, adapters::BinanceAdapter};

fn client() -> Client<BinanceAdapter> {
    let api_key = std::env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY");
    let secret = std::env::var("BINANCE_SECRET_KEY").expect("BINANCE_SECRET_KEY");
    Client::new(BinanceAdapter::usd_m_futures().with_credentials(api_key, secret))
}
```

The secret never leaves the process. It signs each request's query string with
HMAC-SHA256, and only the signature is sent.

This adapter carries no testnet host. Test against a key with trading disabled.

## Examples

`cargo run --example public_rest -- binance BTC USDT`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Binance's own docs

| Subject | Pages |
| --- | --- |
| Quotas | [spot limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits) · [USD-M general info](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info) |
| Order rules | [USD-M `exchangeInfo`](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information) |
| Spot | [REST](https://developers.binance.com/docs/binance-spot-api-docs/rest-api) · [WebSocket streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams) |
| Spot account stream | [user data streams](https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream) · [the subscribe requests](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/user-data-stream-requests) · [how a request is signed](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/request-security) |
| USD-M futures | [REST](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api) · [WebSocket streams](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams) |

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
