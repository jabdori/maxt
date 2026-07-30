[English](upbit.md) | [한국어](upbit.ko.md)

# Upbit

Spot only, in four regional deployments. One `UpbitAdapter` talks to one
region, chosen at construction. Choose it for KRW pairs, or for Upbit prices to
compare against another venue.

```rust
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let korea = Client::new(UpbitAdapter::new());
let singapore = Client::new(UpbitAdapter::with_region(UpbitRegion::Singapore));
```

## What is supported

`UpbitRegion::Korea` is the default; `Singapore`, `Indonesia` and `Thailand`
are the others. They are separate exchanges rather than mirrors: a listing on
one is not a listing on another, and a credential issued for one does not work
on another. `UpbitAdapter::region()` reports which one an adapter points at.

Upbit writes the quote asset first, `KRW-BTC`. You pass
`Market::spot(Exchange::Upbit, "BTC", "KRW")` and the adapter translates;
`MarketInfo::native_symbol` gives Upbit's own spelling back, for reconciling
against their UI.

| Call | What it needs, or why it cannot work |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | no credentials |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | credentials |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | `Error::Unsupported`, always. Upbit lists no derivatives |
| `reduce_only` on an order | `Error::Unsupported`. A spot order has no position to reduce |
| `markets(MarketKind::Perpetual)` | an empty list, not an error |

What is not `Unsupported` behaves as [the common API](../common-api.md) says.

## Limits

Checked before the request is built.

| Call | Accepts | Outside it |
| --- | --- | --- |
| `trades` | `limit` 1 to 500 | `Error::InvalidRequest` on `limit` |
| `order_book` | `depth` 1 to 30 per side | `Error::InvalidRequest` on `depth` |
| `candles` | any `limit`; 200 per Upbit response, paged for you over at most a hundred calls | `limit` of 0, a `from` not earlier than `to`, or a window past 20 000 candles, is `Error::InvalidRequest` |
| Asset codes | uppercase ASCII letters and digits | `Error::InvalidRequest`, so nothing else can alter a signed request |

REST and the stream serve different interval sets.

| Interval | REST `candles` | `Feed::Candles` |
| --- | --- | --- |
| 1s | yes | yes |
| 1m, 3m, 5m, 15m, 30m, 1h, 4h | yes | yes |
| 1d, 1w, 1M | yes | no |

The eleven REST intervals are the [baseline](../common-api.md#intervals) ten
plus `Sec1`. Upbit also serves ten-minute and yearly candles that `Interval`
has no name for; that is a gap in `maxt`, not in Upbit, and the error text for
an unmapped interval says so rather than blaming the exchange.

The one-second endpoint is the exception on history: Upbit
[documents](https://global-docs.upbit.com/reference/list-candles-seconds) about
three months of it, against the full history on every other interval.

An interval outside a column is `Error::Unsupported` naming `Feature::Candles`
or `Feature::CandleStream`, **but only if the request is otherwise valid**. The
`limit`, window and paging checks in the table above run first, so an unmapped
interval asked for with a `limit` of 0, an inverted window, or a span past the
paging ceiling comes back as `Error::InvalidRequest` naming that field. Code
that matches on `Unsupported` alone to fall back to another venue falls
through. Match both, or validate the request before you branch on the interval.

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::OrderBook` | **30 levels a side**, Upbit's own default. `Subscription` cannot narrow it: Upbit's `{code}.{count}` suffix, which [accepts](https://global-docs.upbit.com/reference/websocket-orderbook) 1, 5, 15 or 30 units per side and falls back to 30, is not sent |
| Book events | full snapshots, not diffs. Overwrite your copy on every one |
| A depth you chose | `Client::order_book` over REST, 1 to 30 levels |
| `Candle::closed` | `true` on one emission per window, sent when Upbit opens the next one. See below |
| Candle events per window | many with `closed` false, then exactly one with it `true`. The settled one carries the figures of that window's own last frame |
| `subscribe_account` | the whole account, not a market list, one event per changed asset. Upbit publishes an entire wallet in one frame |

### `Candle::closed` on the stream

**No single Upbit candle frame says its window has ended, and no clock reading
of one frame can say it either.** Upbit stops publishing a window the instant
the next one opens, so a frame's own `timestamp` never reaches
`open_time + interval`.

| Frame on `candle.1m` `KRW-BTC`, 2026-07-30 | `candle_date_time_utc` | frame `timestamp` | window ends |
| --- | --- | --- | --- |
| last one 07:46 ever got | 07:46:00 | 07:46:58.309 | 07:47:00.000 |
| first one of the next window | 07:47:00 | 07:47:00.706 | 07:48:00.000 |

So `maxt` reads `closed` off the transition rather than off a clock. Each
subscription holds the last frame of each candle feed; when a frame opens a
later window, the held one is emitted with `closed` set, ahead of the new
forming one. A window is therefore never called finished before Upbit itself
stopped publishing it, and `last_settled_close` answers on Upbit's candle stream
as it does elsewhere.

| Situation | What arrives |
| --- | --- |
| A window is running | the forming bar, republished on every update, `closed` false |
| The next window opens | the settled bar first, then the new forming one |
| After a reconnect | the held bar is dropped, not settled, because the gap may have hidden later frames of its window. The first window to settle is the next one to open |
| A frame arrives behind one already seen | nothing. Its window has had its settled emission, and the held bar keeps its place so the window in between still settles with its own figures |
| The final window when you drop the subscription | never settled. Nothing opens after it |

Over REST the clock is the reading machine's, because a REST response is a set
of finished windows plus at most one running one, and `Month1` is settled by the
calendar like every other interval: the June candle closes on 1 July. A month
has no fixed length, but it has an end, and that is what the rule reads.

## Quotas

Upbit counts requests per second, which makes them easy to budget against.

| Group | Limit | Measured per |
| --- | --- | --- |
| Public quotation: `markets`, `candles`, `trades`, `ticker`, `order_book` | 10 a second | IP |
| Exchange default: `balances`, `open_orders`, `cancel_order` | 30 a second | account |
| Order placement: `place_order` | 8 a second | account |
| New WebSocket connections | 5 a second | IP unauthenticated, account authenticated |
| Frames sent on one WebSocket | 5 a second and 100 a minute | connection |

Cancelling is not in the order group. Upbit meters `place_order` at 8 a second
and leaves `cancel_order` in the 30-a-second default, so a cancel-and-replace
loop is bounded by the placements.

`maxt` does not throttle; pacing is yours, and `Error::is_rate_limited()` is how
you learn you were too fast. Ten public requests a second is the figure worth
designing around, and it is why the batched reads below exist.

## Orders

Upbit names an order type after how it is sized, not after how it matches. Three
combinations exist; `maxt` refuses the rest before signing.

| Order | Size | Price |
| --- | --- | --- |
| Limit, either side | `Size::Base` | required |
| Market buy | `Size::Quote`, the amount to spend | none |
| Market sell | `Size::Base`, the quantity to offer | none |

A market buy sized in `Size::Base`, or a limit order sized in `Size::Quote`, is
`Error::InvalidRequest` on `size`, as is a zero or negative price or quantity.

| `TimeInForce` on a limit order | Sent as |
| --- | --- |
| `GoodTilCancelled` | nothing at all, which is the default |
| `ImmediateOrCancel` | `ioc` |
| `FillOrKill` | `fok` |
| `PostOnly` | `post_only` |

A market order is immediate-or-cancel by construction and carries no field to
say so. `ImmediateOrCancel` on one is accepted and sent as nothing; anything
else fails on `time_in_force`.

## Order precision and minimum size

`maxt` does not expose either. It checks that a price and a quantity are above
zero and sends what you gave it, so a price off Upbit's tick or an order under
their minimum comes back as a rejection from Upbit rather than an
`Error::InvalidRequest` from here.

Upbit publishes tick sizes as a band table keyed by price range, and a minimum
order amount per quote asset, neither of which any type in `maxt` carries.
Read them from Upbit directly before you place a first order.

## Surprises

| Field or call | What to expect |
| --- | --- |
| `Trade::id` | Upbit's `sequential_id`, kept as the digits it sent. The same value on both paths, so a stream trade and a REST tick of one trade carry one id. Deduplicate on it |
| Ordering by `Trade::id` | do not. Upbit [documents](https://global-docs.upbit.com/reference/today-trades-history) it as a basis for uniqueness and says it does not guarantee the order of trades. Order by `Trade::timestamp` |
| `trades` order | newest-first, sorted here. The sort is stable, so trades sharing one millisecond keep Upbit's own order |
| `candles` order | oldest-first, though Upbit answers newest-first |
| An investment warning | `MarketStatus::Unknown`. Such a market is fully tradable. See [Warnings and cautions](#warnings-and-cautions) below |
| An investment caution | `MarketStatus::Active`, unchanged. A caution is not a warning; read it from `market_events` |
| `open_orders` | walks every page, 100 per page, asking for resting orders and orders waiting on a trigger. The endpoint's default would have dropped the second kind |
| `cancel_order` | takes the market and the order id. Upbit cancels by id alone; the market is checked so an id from another exchange cannot be sent by mistake |
| No credentials | `Error::Auth`, not `Error::Unsupported`, even though `supports(Feature::Balances)` is `false` |
| A credential Upbit refused | `Error::Exchange`, not `Error::Auth`, carrying Upbit's own name. Upbit's published table lists HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_access_key`, `nonce_used`, `no_authorization_ip`, `no_authorization_token`, and HTTP 403 `out_of_scope`. **Documented, not measured:** no Upbit key was available for this crate's verification |

That last pair is not a contradiction: the feature exists and the key does not.
Match on both if you handle the two separately.

## Warnings and cautions

**Upbit publishes two designations and they do not mean the same thing.**
`MarketStatus` has one value between them, so only the first reaches it.

| Designation | What it is | `MarketStatus` | Where to read it |
| --- | --- | --- | --- |
| Warning, 유의 종목 | designated by hand and announced. Upbit asks the project to resolve the cause and may end trading support if it is not resolved | `Unknown` | `MarketInfo::status`, or `market_events` |
| Caution, 주의 종목 | raised and cleared automatically against published criteria, one flag per criterion, describing how the market is trading now | `Active` | `market_events` only |

A caution is deliberately not folded into `Unknown`. On 2026-07-30 Upbit listed
800 markets: 11 carried a warning and 190 carried at least one caution, 175 of
those `GLOBAL_PRICE_DIFFERENCES` alone. The caution count moves through the day,
because the criteria are read continuously; the warning count does not.
Reporting both the same way would make
`Unknown` the answer for a quarter of the exchange and bury the 11 inside it.
Bithumb's `market_warning` is the same concept as the warning here, so the two
adapters agree. Bithumb publishes its caution separately, on its 경보제
endpoint, with a severity step and an expiry Upbit does not publish at all; see
[bithumb.md](bithumb.md).

**The four deployments do not send the same field.** Upbit Korea sends
`market_event`, carrying both designations. Singapore, Indonesia and Thailand
send the older `market_warning`, which reports the warning and has never carried
a caution. `maxt` reads whichever arrives, so `MarketInfo::status` means the
same thing on all four, and `UpbitMarketEvent::cautions` is empty outside Korea
because those payloads do not carry the criteria at all.

## Upbit-only calls

Through `Client::adapter()`. One Upbit call answers for many markets at once,
and the common API asks about one market per call, because that is what most
exchanges offer.

| Method | Gives you | Costs |
| --- | --- | --- |
| `tickers(&[Market])` | one ticker per market | one of the ten public requests a second |
| `order_books(&[Market], Option<u32>)` | one book per market, depth capped at 30 per side | the same one |
| `market_events()` | one `UpbitMarketEvent` per market: the warning flag and the caution criteria by name | the same one |

`Client::ticker` and `Client::order_book` are these two with a single-element
list. Watching thirty markets costs one request instead of three seconds' worth.

**Where the batch stops is undocumented on both sides.** Neither method caps the
number of market codes it joins into the comma-separated list, and Upbit
publishes no cap either, so the bound is whatever length of URL Upbit or a proxy
in front of it will accept. Past it the call comes back as `Error::Exchange`, not
as an `Error::InvalidRequest` from here. Thirty is well inside it; a few hundred
is worth testing against your own path to Upbit before you rely on it.

```rust
use maxt::{Client, Exchange, Market, adapters::UpbitAdapter};

async fn breadth(client: &Client<UpbitAdapter>) -> maxt::Result<()> {
    let markets = [Market::spot(Exchange::Upbit, "BTC", "KRW")];
    let _tickers = client.adapter().tickers(&markets).await?;
    let _books = client.adapter().order_books(&markets, Some(5)).await?;
    Ok(())
}
```

## Credentials

An access key and a secret key, issued together, from the same region as the
adapter. They unlock `Feature::Balances`, `Feature::OpenOrders`,
`Feature::Trading`, and `Feature::AccountStream`. Public data needs nothing.

```rust
use maxt::{Client, adapters::UpbitAdapter};

fn client() -> Client<UpbitAdapter> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    Client::new(UpbitAdapter::new().with_credentials(access_key, secret))
}
```

Upbit does not sign a request; it signs a statement about one. `maxt` mints a
JWT naming the access key, a fresh nonce, and, when the call carries parameters,
a SHA-512 hash of them. Upbit recomputes that hash from what it received, so a
token is good only for the call it was minted for. The secret signs the token
locally and never leaves the process.

The private WebSocket authenticates in the opening handshake, not in a frame.
Its token claims no expiry, so a stale one would in fact still open a socket
hours later, which is why `maxt` mints a fresh one per handshake instead:
every reconnect signs again, with a nonce no earlier connection used.

Keep keys out of source, and give them the narrowest permissions your program
actually uses: read-only keys if you never place an order.

## Examples

`cargo run --example public_rest`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Upbit's own docs

| Subject | Pages |
| --- | --- |
| Quotas | [rate limits](https://global-docs.upbit.com/reference/rate-limits) |
| Public REST | [markets](https://global-docs.upbit.com/reference/list-trading-pairs.md) · [tickers](https://global-docs.upbit.com/reference/list-tickers.md) · [order books](https://global-docs.upbit.com/reference/list-orderbooks.md) · [trades](https://global-docs.upbit.com/reference/list-pair-trades.md) · [minute candles](https://global-docs.upbit.com/reference/list-candles-minutes.md) · [second candles](https://global-docs.upbit.com/reference/list-candles-seconds) |
| Private REST | [accounts](https://global-docs.upbit.com/reference/get-balance.md) · [open orders](https://global-docs.upbit.com/reference/list-open-orders.md) |
| WebSocket | [trades](https://global-docs.upbit.com/reference/websocket-trade.md) · [order books](https://global-docs.upbit.com/reference/websocket-orderbook) · [candles](https://global-docs.upbit.com/reference/websocket-candle.md) · [account orders](https://global-docs.upbit.com/reference/websocket-myorder.md) · [account assets](https://global-docs.upbit.com/reference/websocket-myasset.md) |

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
