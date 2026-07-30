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

`BTCUSDT` exists on both at different prices. Passing a `Market` of the wrong
kind is `Error::InvalidRequest`, raised before anything reaches the network.
`supports` answers about the adapter, not about a market, so it cannot catch
this.

| What differs | Spot | USD-M futures |
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
| Streamed order `created_at` | `O`, when the order was created | `T`, the event's own time |
| Weight budget | 6 000 per IP per minute | 2 400 per IP per minute |

One mapping serves both directions, so an interval is either reachable over
REST *and* on `Feed::Candles`, or on neither. `Sec1` is the whole difference
between the venues: USD-M refuses it with `Error::Unsupported` naming
`Feature::Candles`, and serves every other interval `Interval` can name.

**But only if the request is otherwise valid.** The `limit` and window checks
below run before the interval lookup, so `Sec1` on USD-M with a `limit` of 0, an
inverted window, or a span past the paging ceiling comes back as
`Error::InvalidRequest` naming that field, not as `Unsupported`. Code that
matches on `Unsupported` alone to fall back to another venue falls through.
Match both, or validate the request before you branch on the interval. Binance
USD-M's `Sec1` is the one shipped case the common API cites for
[why a `true` from `supports` still needs re-checking at the call site](../common-api.md#a-true-still-has-to-be-checked-at-the-call),
so it is the one most likely to be reached by a router.

`BinanceAdapter::default()` is spot.

## Limits

Checked before the request is built.

| Call | Accepts | Outside it |
| --- | --- | --- |
| `trades` | `limit` 1 to 1 000 | `Error::InvalidRequest` |
| `order_book` | a depth from the table above | `Error::InvalidRequest` |
| `candles` | any `limit`; paged for you, over at most a hundred calls | `limit` 0, or a window past 100 000 candles on spot and 150 000 on USD-M, is `Error::InvalidRequest` |
| `funding_rates`, `funding_payments` | `limit` 1 to 1 000, **default 100** | `Error::InvalidRequest` |

A depth Binance does not serve is refused, not rounded up.

The candle ceiling is a hundred pages of whatever the venue serves per call.
A `from` far enough back to need more, with no `limit` to bound it, is
`Error::InvalidRequest` naming `from`, raised before the first call rather
than discovered a hundred round trips in.

**The history default bites.** `HistoryRequest` with no `limit` asks for 100.
A loop that reads a page, counts 100, and concludes it has caught up has
concluded nothing. Follow `Page::next` until `None`.

## Order precision and minimum size

Binance answers this per symbol through the filters on its `exchangeInfo`
listing: `PRICE_FILTER` carries the tick size, `LOT_SIZE` the quantity step,
`NOTIONAL` the smallest price times quantity it will accept. `maxt` neither
checks an order against them nor rounds to them, so a price off the tick comes
back as a rejection from Binance rather than an `Error::InvalidRequest` from
here.

On spot, [`spot_symbol_filters`](#binance-only-calls) reads them for you. On
USD-M there is no such call and nothing else in `maxt` carries the numbers, so
read
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

`Trade::id` is Binance's own fill identifier, so a set of ids deduplicates a
REST backfill against a live subscription on both venues. There is no tuple to
fall back on and none is needed.

Measured on 2026-07-30, sixty seconds of `btcusdt@trade` off
`wss://fstream.binance.com/stream` checked against
`https://fapi.binance.com/fapi/v1/trades?symbol=BTCUSDT&limit=1000`:

| Measured | Count |
| --- | --- |
| Streamed fills falling inside the REST window | 1 000 |
| Of those, id present in REST with the same price, quantity, time and taker side | 1 000 |
| Of those, id absent from REST | 0 |
| Duplicate ids on the stream | 0 |
| Spent-id frames in the same minute, none of them listed by REST | 14 of 1 186 |

### Why not `@aggTrade`

`@aggTrade` collapses the fills one taker order swept at one price into a
single message, which over the same 1 000 fills was 319 messages instead of
1 000. That is its whole advantage, and `maxt` does not take it.

| Against it | |
| --- | --- |
| A second id space | An `aggTrade` id numbers aggregates, not fills, and no REST call on either venue answers with one. Holding both transports, you have nothing to match on |
| No tuple to fall back on | Its quantity is the sum of the fills it collapsed, so `(timestamp, price, quantity, taker_side)` does not reconcile it against REST either. Over those same 1 000 fills, 141 of the 319 aggregates matched no individual REST trade, because 39% of them covered more than one fill |

It is carried, and on the `/market` entry point below it delivers: 117 frames
over 25 seconds on `BTCUSDT`. The reason against it is the id space, not
availability.

### The two USD-M entry points

Binance serves USD-M market data from two entry points on one host, and
decommissioned the unrouted `/stream` and `/ws` paths on 2026-04-23. A socket
naming neither is served as if it had named `/public`.

| Entry point | Carries | `maxt` sends it |
| --- | --- | --- |
| `wss://fstream.binance.com/public/stream` | what the matching engine pushes on change: `@trade`, `@depth*`, `@bookTicker` | `Feed::Trades`, `Feed::OrderBook` |
| `wss://fstream.binance.com/market/stream` | what an aggregator produces: `@aggTrade`, `@kline_*`, `@ticker`, `@miniTicker`, `@markPrice`, `@forceOrder`, `@compositeIndex`, `!contractInfo`, `!assetIndex@arr` | `Feed::Ticker`, `Feed::Candles` |
| `wss://fstream.binance.com/private/ws` | the account: `ORDER_TRADE_UPDATE`, `ACCOUNT_UPDATE`, `listenKeyExpired` | `subscribe_account` |

Nothing rejects a mismatch. A socket on one entry point accepts a `SUBSCRIBE`
naming the other's streams, acknowledges it with `{"result": null, "id": 1}`,
and then never sends a frame for it. The same acknowledgement comes back for a
stream name Binance does not publish at all, so it says nothing about whether
data will follow. That is why a socket on the decommissioned path delivers
trades and books and stays silent on candles and tickers forever, with no error
and no close.

Measured 2026-07-30 over 25 seconds on `BTCUSDT`, one `SUBSCRIBE` frame naming
all seven streams on each endpoint, counting frames by their own `stream` name:

| Endpoint | `@trade` | `@depth20@100ms` | `@bookTicker` | `@aggTrade` | `@kline_1m` | `@ticker` | `@markPrice@1s` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `/stream`, the decommissioned path | 141 | 229 | 896 | 0 | 0 | 0 | 0 |
| `/public/stream` | 149 | 229 | 1 993 | 0 | 0 | 0 | 0 |
| `/market/stream` | 0 | 0 | 0 | 117 | 47 | 12 | 25 |

Spot market data is not split. `wss://stream.binance.com:9443/stream` carries
every feed. Binance's spot market data documentation names no entry points and
carries no decommission notice, so only USD-M's market data moved. Do not
assume both venues did.

The USD-M account socket moved with the rest: `subscribe_account` now opens
`wss://fstream.binance.com/private/ws?listenKey=<key>&events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired`.

**`events` is an allow list, not a hint.** A socket receives the events its
filter names and nothing else, so an event `maxt` acts on and does not name is
an event `maxt` can never receive. Measured 2026-07-31 on four sockets sharing
one listen key, by sending `DELETE /fapi/v1/listenKey` to make the server push
the expiry:

| The socket's `events` | `listenKeyExpired` |
| --- | --- |
| no `events` parameter | received |
| `listenKeyExpired` | received |
| `ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired`, what `maxt` sends | received |
| `ORDER_TRADE_UPDATE/ACCOUNT_UPDATE` | **not** received |
| the decommissioned `wss://fstream.binance.com/ws/<key>`, no filter | **not** received |

Every event USD-M publishes, and whether `maxt` asks for it:

| Event | Asked for | Why |
| --- | --- | --- |
| `ORDER_TRADE_UPDATE` | yes | an order change, read into `AccountEvent::Order` |
| `ACCOUNT_UPDATE` | yes | a balance change, read into `AccountEvent::Balance` |
| `listenKeyExpired` | yes | the key behind the socket lapsed, raised as `Error::Exchange` so you stop waiting on a stream that has nothing left to say |
| `TRADE_LITE` | no | the same fill `ORDER_TRADE_UPDATE` already carries, sooner and with fewer fields. Asking for it would report one fill twice |
| `MARGIN_CALL` | no | risk guidance, which Binance's own page says not to trade on. `AccountEvent` has no shape for it |
| `ACCOUNT_CONFIG_UPDATE` | no | leverage and multi-assets mode, and `maxt` reports neither: `leverage` and `margin_mode` on a Binance position are always `None` |
| `CONDITIONAL_ORDER_TRIGGER_REJECT` | no | a rejected TP/SL trigger, and `maxt` places no conditional orders |
| `STRATEGY_UPDATE` | no | Binance's own grid strategies, which `maxt` does not create |
| `GRID_UPDATE` | no | a sub-order of one of those strategies, and deprecated on Binance's page |
| `ALGO_UPDATE` | no | an algo order, and `maxt` places none |

The five `maxt` does not ask for are the five it would drop on arrival.

What that leaves measured and unmeasured on this URL:

| Claim | Standing |
| --- | --- |
| Binance publishes this `/private` form | quoted from the change notice's own worked example |
| The socket carries this account rather than merely opening | measured. Deleting the key pushed `listenKeyExpired` down it |
| The decommissioned unrouted path is dead for user data | measured. The same key on `wss://fstream.binance.com/ws/<key>` was told nothing across the same deletion |
| A lapsed key reaches the consumer | measured through `maxt`. `subscribe_account` yielded `Error::Exchange` with code `listenKeyExpired`; the same run before this filter was fixed yielded nothing at all |
| `ORDER_TRADE_UPDATE` arrives | measured. Placing a limit order pushed `x=NEW`, `X=NEW`; cancelling it pushed `x=CANCELED`, `X=CANCELED`, and the order id matched the REST placement response and the REST read |
| `ACCOUNT_UPDATE` arrives | **not measured.** Neither placing nor cancelling that order produced one. Binance pushes `ACCOUNT_UPDATE` when a balance or position actually changes, and a resting order changes neither. Its decoder is pinned against Binance's own published payload instead |

**Reserved margin does not arrive as `ACCOUNT_UPDATE`.** A resting order locks
margin without moving a balance or a position, so waiting for `ACCOUNT_UPDATE`
to learn about it waits forever. The figure rides on `ORDER_TRADE_UPDATE`'s `b`
field, the reserved bid notional: measured 2026-07-31, `b` read `"5"` while the
order rested and `"0"` after the cancel, against `Balance::locked` of
`0.25000000` and then `0.00000000`, which is that notional over 20x leverage.
`maxt` does not surface `b`; read `balances()` for the locked figure.

A lapsed key now reaches you twice over, and both routes are wanted. The
refresher extends the key over REST every thirty minutes and sends its own
failure down the stream, which covers a refresher that cannot reach Binance;
the event covers a key invalidated some other way, which no REST call of
`maxt`'s would notice.

### The spot account stream

Spot's user data stream does not use a listen key and no longer has one to use.
Binance deprecated `listenKey` on `wss://stream.binance.com:9443` on 2025-04-07
and removed the endpoints that minted one on 2026-02-20 07:00 UTC. Its spot page
now says only to subscribe via the WebSocket API using an API key.

| Removed on 2026-02-20 07:00 UTC | Replaced by |
| --- | --- |
| `POST`, `PUT`, `DELETE /api/v3/userDataStream` | nothing. There is no key to create, extend, or close |
| `userDataStream.start`, `.ping`, `.stop` | `userDataStream.subscribe.signature`, one signed request per socket |

`subscribe_account` on a spot adapter opens
`wss://ws-api.binance.com:443/ws-api/v3` and sends one signed
`userDataStream.subscribe.signature`. The socket is opened unauthenticated and
the frame names the account, so nothing secret is in the URL and there is no key
to keep alive.

Nothing filters events on this side. The subscribe frame carries no event list
and there is no `events` parameter to get wrong, so a spot socket receives
everything the account produces and `maxt` drops what it does not model after
the fact rather than before.

Two methods reach the same subscription, and which one is open to you depends on
your key type:

| Method | HMAC-SHA-256 key | Ed25519 key |
| --- | --- | --- |
| `userDataStream.subscribe.signature`, signing every request | works | works |
| `session.logon` then `userDataStream.subscribe` | `-2028 HMAC-SHA-256 API key is not supported`, then `-1193 WebSocket session not authenticated` | works |

`maxt` sends the first, which serves both key types, so no key of either kind
needs `session.logon`.

Measured 2026-07-31 with a live HMAC-SHA-256 key, spot account with no balances
and no open orders:

| Claim | Standing |
| --- | --- |
| `POST /api/v3/userDataStream` is gone | measured. `410 Gone`, an nginx error page, before any socket opens |
| A spot subscription is accepted | measured. `{"status":200,"result":{"subscriptionId":0}}` on the socket `maxt` opens |
| The socket really carries this account | measured. `userDataStream.unsubscribe` after 150 s of silence pushed `{"subscriptionId":0,"event":{"e":"eventStreamTerminated"}}`, which only reaches a socket whose subscription is live |
| A wrong signature is distinguishable from silence | measured through `maxt`. A stream opened with a bad secret yields `Error::Exchange` carrying `-1022`; the same run with the right secret yields nothing |
| The socket survives an idle account | measured. 200 s through `maxt` with no event, no error, and no reconnect |
| A balance or order event is decoded correctly | **not measured.** The account holds nothing and has no open orders, and no order was placed to make one. The decoders are pinned against Binance's own published payloads instead |

That fourth row is the one the old transport could not offer. A fabricated
listen key used to be accepted by the handshake and then answered with silence,
so a wrong stream and an idle account looked identical. A refused subscription
is now an error on the stream rather than a socket that stays open and says
nothing.

**A reconnect signs a new frame.** The subscribe frame signs the millisecond
clock it was built at, so one signature subscribes one socket and no more:
Binance refuses a replayed one once `recvWindow` has passed, and a reconnect
loop replaying it would open socket after socket that carries nothing. `maxt`
signs again per handshake, the same way it mints a fresh authorization header
per handshake on Upbit and Bithumb.

Measured 2026-07-31, driving `subscribe_account` through a local relay cut after
75 s so the reconnect was a real one against Binance:

| What the reconnect sent | Binance's answer |
| --- | --- |
| the frame the first socket subscribed with, replayed | `400 -1021 Timestamp for this request is outside of the recvWindow` |
| a frame signed for that handshake, which is what `maxt` sends | `{"status":200,"result":{"subscriptionId":0}}` |

`recvWindow` still matters, because it sets how long an outage a *single*
signature survives and so how much of a reconnect's own latency is covered.
`maxt` sends Binance's documented maximum of 60 000 ms. Measured 2026-07-31 on
one socket:

| Frame signed | `recvWindow` | Answer |
| --- | --- | --- |
| 50 s earlier | absent, so Binance's own 5 000 ms | `-1021 Timestamp for this request is outside of the recvWindow` |
| 50 s earlier | 60 000, what `maxt` sends | `{"status":200,"result":{"subscriptionId":0}}` |
| 90 s earlier | 60 000 | `-1021`, because 60 000 is the ceiling and not a way past it |

A resubscription Binance refuses for any other reason reports `Error::Exchange`
carrying that reason's code rather than leaving the socket open and quiet.
Subscribe again when you see one.

A USD-M subscription naming feeds from both entry points is opened as two
sockets and merged into one `MarketStream`. Each reconnects on its own, so such
a subscription reports `MarketEvent::Reconnected` once per socket that comes
back rather than once per outage.

### A stream that yields nothing

A feed that delivers no events and no errors is indistinguishable from a market
where nothing is happening. `maxt` reports no timer, no subscription
acknowledgement and no per-feed liveness signal, so nothing in the API tells
the two apart.

| To tell them apart | Do this |
| --- | --- |
| Is the feed carried at all? | ask the same subject over REST. `Client::ticker` and `Client::candles` answer for every USD-M market, so a REST answer alongside a silent stream means the stream, not the market, is the problem |
| Is the socket alive? | set `StreamConfig::idle_timeout_ms`. A socket that has sent nothing for that long is torn down and rebuilt, and the rebuild arrives as `MarketEvent::Reconnected` |
| Is it this endpoint? | subscribe to the same stream name on the other entry point with a raw socket and count frames, as the table above does |

Do that before filing a silent feed here.

## Quotas

Binance budgets **weight per IP per minute**, not requests, counted separately
per venue. A deep order book costs far more than a ticker. Every response
carries the running total in `X-MBX-USED-WEIGHT-1M`.

Binance documents the mechanism rather than the ceiling: each venue publishes
its own in the `rateLimits` array of its `exchangeInfo` response, as a
`REQUEST_WEIGHT` entry over a one-minute interval. That is where the two
figures in the venue table come from, spot at 6 000 and USD-M at 2 400, and
reading them back beats trusting a number written down here.

`maxt` does not throttle and does not read that header. Exceeding the budget is
HTTP 429, reported by `Error::is_rate_limited()`. Ignoring a 429 earns an
automated IP ban that scales from two minutes to three days, so back off on the
first one.

## Phantom positions

`/fapi/v3/positionRisk` opens a zero-amount row for any symbol that merely has
an order resting on it. `positions()` returns open positions, and a row with no
size is not one, so `maxt` drops it.

Measured 2026-07-31 on a funded USD-M account with no position at all:

| Account state | Raw endpoint | `positions()` | `open_orders()` | `Balance::locked` |
| --- | --- | --- | --- | --- |
| one resting XRPUSDT limit order | 1 row, `positionAmt` `0.0` | 0 | 1 | 0.25000000 |
| that order cancelled | `[]` | 0 | 0 | 0.00000000 |

The middle column is what changed. Before the filter, the resting order gave
`positions()` one `Position` with `quantity: 0` and `side: None` on a market the
account had never traded.

| Question | Answer |
| --- | --- |
| What triggers the row | an open order on the symbol, and nothing else. An empty account never shows it, which is why it went unnoticed for seven review rounds |
| What `positions_on(&market)` does | the same. A market carrying only an order answers with an empty list, matching Hyperliquid, which answers an empty list for a market it holds no position on |
| Whether the zero row is preserved anywhere | no. Beyond the mark price, which is public, it said an order rests on that symbol, and `open_orders()` says so outright |
| Whether a row `maxt` cannot parse is dropped too | no. Only rows that parse and are flat are dropped; a malformed row is still reported as `Error::Decode` |

## Surprises

| Field or call | What to expect |
| --- | --- |
| `Ticker::last_trade_time` | always `None`; Binance never says when the last price traded |
| `Ticker::timestamp` | end of the 24-hour window, not a trade time |
| Spot book timestamp | read time; Binance publishes no clock on spot depth |
| `Position::leverage`, `margin_mode` | `None`; `/fapi/v3/positionRisk` carries neither. Binance keeps a symbol's configured leverage and margin mode on `/fapi/v1/symbolConfig` instead, at the same weight, so a caller who needs them reads that |
| A symbol carrying only a resting order | not a position. Binance reports one; `maxt` drops it. See [phantom positions](#phantom-positions) |
| `FundingPayment::rate` | `None`; the ledger records the charge, not the rate |
| `MarginSummary::equity` | `totalMarginBalance`: wallet plus unrealized PnL |
| `MarginSummary::margin_balance` | `totalInitialMargin`: margin already consumed by open positions and orders. A cost, not a budget |
| `MarginSummary::available_balance` | `availableBalance`: what is free to open with, and the only one of the three that governs headroom |
| All three margin figures | denominated in `USDT` |
| USD-M `Balance::locked` | derived as wallet minus available, floored at zero |
| Streamed order `created_at` | spot dates it from `O`, the creation time; USD-M from `T`, because `ORDER_TRADE_UPDATE` publishes no creation time. On a USD-M order that rests and later fills, that is the fill |
| `cancel_order`, `spot_order` | Binance's numeric order id only; a client order id is `Error::InvalidRequest` |
| `set_margin` | up to two calls, one per field, not atomic; leverage must be a whole multiplier at least 1 |
| Dated futures | dropped from `markets()`; reporting them as perpetuals would misprice them |
| Unknown quote asset on a stream frame | `Error::Decode`, never a wrong market |
| No credentials | `Error::Auth`, not `Error::Unsupported` |
| A credential Binance refused | `Error::Exchange`, not `Error::Auth`: HTTP 400 `-1022` for a bad signature, HTTP 401 `-2015` for a bad or unpermitted key, HTTP 401 `-2014` for no key. Measured 2026-07-31 |
| `-1021`, a timestamp outside `recvWindow` | `ExchangeErrorKind::Rejected`, so `is_retryable()` is `false`. Fix the clock, or build the request again and send it once; a loop resolves neither |
| Spot account stream | no listen key; one signed request on the WebSocket API, signed again per reconnect. See [the spot account stream](#the-spot-account-stream) |
| A refused spot subscription | `Error::Exchange` on the stream carrying Binance's own code, not a socket that stays open and silent |
| A spot subscription Binance ends | `Error::Exchange` coded `listenKeyExpired` or `eventStreamTerminated`, the event name Binance pushed |
| A USD-M listen key that cannot be extended | Binance's own verdict, forwarded to the stream. Left unresolved it stops carrying account changes within the hour |

## Binance-only calls

Through `Client::adapter()`.

| Method | Gives you | Not on |
| --- | --- | --- |
| `spot_symbol_filters(&market)` | tick size, price and quantity bounds, lot step, minimum notional | USD-M |
| `spot_order(&market, id)` | one order by id, including filled and cancelled | USD-M |
| `usd_m_create_listen_key()` | a USD-M user data stream key | spot |
| `usd_m_keepalive_listen_key(&key)` | extends it another 60 minutes | spot |
| `usd_m_close_listen_key(&key)` | closes it | spot |

No two exchanges express order rules alike, which is why the filters keep
Binance's shape. Filter fields are `None` when the symbol carries no filter of
that kind, common on newly listed pairs.

Those three are USD-M only, and not because spot's are hidden elsewhere: spot
has no listen key at all since 2026-02-20. `subscribe_account` runs the USD-M
listen-key lifecycle for you. Reach for them only to drive the socket yourself,
share one key between consumers, or hold a key across a restart.
`BinanceListenKey` goes into the socket URL, so it is a bearer secret and its
`Debug` output is redacted.

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

Binance publishes no testnet host in this adapter. Test against a key with
trading disabled.

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
