[English](bithumb.md) | [한국어](bithumb.ko.md)

# Bithumb

Spot only, one venue, no candle stream. Choose it for KRW pairs, or for a second
Korean venue to compare against Upbit.

```rust
use maxt::{Client, Feature, adapters::BithumbAdapter};

let client = Client::new(BithumbAdapter::new());
assert!(client.supports(Feature::Candles));       // over REST
assert!(!client.supports(Feature::CandleStream)); // live, never
```

## What is supported

Bithumb writes the quote asset first, `KRW-BTC`. You pass
`Market::spot(Exchange::Bithumb, "BTC", "KRW")` and the adapter translates;
`MarketInfo::native_symbol` gives Bithumb's own spelling back, for reconciling
against their UI.

| Call | What it needs, or why it cannot work |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | no credentials |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | credentials |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | `Error::Unsupported`, always. Bithumb lists no derivatives |
| `reduce_only` on an order | `Error::Unsupported`. A spot order has no position to reduce |
| `markets(MarketKind::Perpetual)` | an empty list, not an error |

What is not `Unsupported` behaves as [the common API](../common-api.md) says.

## Limits

Checked before the request is built.

| Call | Accepts | Outside it |
| --- | --- | --- |
| `trades` | `limit` 1 to 500 | `Error::InvalidRequest` on `limit` |
| `order_book` | any `depth` above 0, meaning the best N levels. Bithumb takes no depth parameter, so `maxt` sorts both sides and truncates | `depth` of 0 is `Error::InvalidRequest` on `depth` |
| `candles` | any `limit`; 200 per Bithumb response, paged for you over at most a hundred calls | `limit` of 0, a `from` not earlier than `to`, or a window past 20 000 candles, is `Error::InvalidRequest` |
| `candles` intervals | 1m, 3m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M, and nothing else | `Error::Unsupported` naming `Feature::Candles` |
| Order ids | letters, digits, `-`, `.`, `_`. An id is the only value here `maxt` did not build itself, and an `&` inside one would append a parameter to a request you already signed | `Error::InvalidRequest` on `order_id` |

Those ten intervals are exactly the
[baseline](../common-api.md#intervals). Bithumb also serves ten-minute candles,
which `Interval` cannot name, and publishes no one-second endpoint at all. An
interval outside the ten is refused as one `maxt` maps no endpoint for, which
is what it is. There is no streaming column, because there is no candle stream
at any interval.

The `limit` and window checks run before the interval is looked up, so an
unmapped interval combined with a `limit` of 0, an inverted window, or a span
past the paging ceiling is reported as `Error::InvalidRequest` naming that
field rather than as `Unsupported`. Match both if you branch on the difference.

Because `depth` is applied here and not by Bithumb, asking for more levels than
Bithumb sent yields fewer levels and no error. Bithumb does not document how
many `/v1/orderbook` returns; a capture on 2026-07-30 answered 30 a side, twice
what the socket sends. Treat any particular count as an observation rather than
a contract, and read `OrderBook` rather than assuming a depth.

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::Trades`, `Feed::OrderBook`, `Feed::Ticker` | carried |
| `Feed::Candles(_)` | `Error::Unsupported`, failing the whole subscription before a socket is opened. Not silently dropped from the feed list, and nothing is synthesized from trades on your behalf |
| `Feed::OrderBook` depth | 15 levels per side, which is what Bithumb [publishes](https://apidocs.bithumb.com/reference/호가-orderbook.md) and what every frame carried across 40 markets on 2026-07-30. `Subscription` cannot ask for more or fewer |
| Book events | full snapshots, not diffs. Overwrite your copy on every one. Both the opening `SNAPSHOT` frame and the `REALTIME` ones that follow carry all 15 levels |
| Book event clock | microseconds on the wire, unlike every other Bithumb payload. See the clock rows under [Surprises](#surprises) |
| `subscribe_account` | the whole account, not a market list, one event per changed asset. Bithumb publishes several balances in one frame |

Two ways to a live chart: read `candles` over REST on an interval, or subscribe
to `Feed::Trades` and fold them yourself. The stream's 15 levels and REST's 30
are separate figures from two unrelated endpoints, so do not size one against
the other.

## Quotas

Bithumb counts requests per second.

| Group | Limit |
| --- | --- |
| Public REST | 150 a second |
| Private REST | 140 a second |
| Orders, on top of the private figure | 10 a second |

Bithumb states neither whether the REST figures are per IP or per account, nor
any WebSocket limit at all. Treat the REST figures as per IP, the stricter
reading, and do not assume the socket is unmetered just because no number is
published. `maxt` does not throttle: a call goes out when you make it,
`Error::is_rate_limited()` is how you learn you were too fast, and Bithumb
temporarily blocks an IP that keeps going.

## Orders

Bithumb encodes the market and limit distinction in the order type together with
which of price and quantity is present. Three combinations exist; `maxt` refuses
the rest before signing.

| Order | Size | Price |
| --- | --- | --- |
| Limit, either side | `Size::Base` | required |
| Market buy | `Size::Quote`, the amount to spend | none |
| Market sell | `Size::Base`, the quantity to offer | none |

A market buy sized in `Size::Base`, or a limit order sized in `Size::Quote`, is
`Error::InvalidRequest` on `size`, as is a zero or negative price or quantity.

Any `TimeInForce` at all fails with `Error::InvalidRequest` on `time_in_force`.
Bithumb's order endpoint was not confirmed to accept one, and a wrong guess at
an order parameter's spelling is a mis-sent order. Orders placed through this
adapter behave as Bithumb's default for their type.

## Order precision and minimum size

`maxt` does not expose either, and nothing in this adapter answers the
question. It checks that a price and a quantity are above zero and sends what
you gave it, so a price off Bithumb's tick or an order under their minimum
comes back as a rejection from Bithumb rather than an `Error::InvalidRequest`
from here. Read Bithumb's own order documentation before you place a first
order.

## Surprises

| Field or call | What to expect |
| --- | --- |
| `place_order` status | `OrderStatus::Accepted`, never a fill. Bithumb answers with an identifier and nothing else. Read `open_orders`, or watch the account stream |
| `Order::remaining_quantity` after a market buy | zero. The order is sized in KRW and the acknowledgement carries no base figure |
| `Order::side` after a cancel | `Side::Buy` when Bithumb's response omits the side. Do not read meaning into it |
| An investment warning, 유의 종목 | `MarketStatus::Unknown`. Such a market still trades, so it is not `Paused`, and it is not plainly healthy, so it is not `Active`. See [Warnings and alerts](#warnings-and-alerts) below |
| An investment caution, 주의 종목 | `MarketStatus::Active`, unchanged. It is on a different endpoint that the market list never carries; read it from `market_alerts` |
| `Trade::id` | Bithumb's `sequential_id`, kept as the digits it sent. Over REST that value is the fill's millisecond times ten thousand, so fills sharing a millisecond share an id. The stream sends a per-fill number instead. Do not use it as a key across the two |
| `OrderBook::timestamp` from `Feed::OrderBook` | read as microseconds, which is the unit Bithumb documents and sends for that one frame. Every other Bithumb clock is milliseconds |
| `Ticker::timestamp` and `Ticker::last_trade_time` from `ticker` | pulled back nine hours. `/v1/ticker` documents both as UTC milliseconds and stamps both with the Korean wall clock. `maxt` measures the gap against the `trade_date` and `trade_time` in the same payload rather than assuming it, so the correction lapses on its own if Bithumb repairs the fields, and a third value is `Error::Decode` |
| `Ticker::timestamp` against `last_trade_time` over REST | equal. `/v1/ticker` sends one number for both clocks even though it documents two. `Feed::Ticker` sends two |
| `trades` order | newest-first, sorted here. The sort is stable, so trades sharing one millisecond keep Bithumb's own order |
| `candles` order | oldest-first, though Bithumb answers newest-first |
| `candles` cursor | Bithumb's own is a bare wall-clock string read as Korean time. Pass a `Timestamp` and think in UTC |
| `Candle::closed` | read off a clock, because Bithumb republishes the forming candle and marks nothing finished. `true` once the candle's own interval has ended. There is no candle stream, so that clock is always the reading machine's |
| `Candle::open_time` at `Month1` | 15:00 UTC on the last day of the previous UTC month, not the 1st. Bithumb cuts months in Korean time and `open_time` is the same instant stated in UTC |
| `Interval::Hour4` | 03:00, 07:00, 11:00, 15:00, 19:00 and 23:00 UTC, because Bithumb cuts four-hour windows in Korean time and nine hours is not a multiple of four. Upbit, Binance and Hyperliquid all open theirs at 00:00, 04:00 and so on |
| `Interval::Day1` | 15:00 UTC, which is midnight in Korea. A daily candle here covers a Korean day, not a UTC one |
| `Interval::Week1` | Sunday 15:00 UTC, which is Monday midnight in Korea. Upbit's and Binance's open Monday 00:00 UTC |
| Every interval that divides nine hours | `Min1` through `Hour1` land on the same UTC grid as the other three, because Korea is a whole number of hours ahead. Only `Hour4` and the daily-and-longer intervals shift |
| `open_orders` | Bithumb's resting-order state, which is what "open" means in the common API |
| No credentials | `Error::Auth`, before any request is built, not `Error::Unsupported` |
| A credential Bithumb refused | `Error::Exchange`, not `Error::Auth`, carrying Bithumb's own name. Bithumb's published table lists HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_jwt`, `NotAllowIP`, and `out_of_scope`. Note that the last two differ from Upbit's, which spells them `no_authorization_ip` and puts `out_of_scope` on 403, so a rule written for one exchange is wrong on the other. **Documented, not measured:** no Bithumb key was available for this crate's verification |

`Month1` is settled on the Korean calendar, because that is where Bithumb cuts
it. Read against Bithumb's own `/v1/candles/months` for `KRW-BTC`:

| Korean month | `open_time` | `closed` turns `true` at |
| --- | --- | --- |
| March 2026 | `2026-02-28T15:00Z` | `2026-03-31T15:00Z` |
| April 2026 | `2026-03-31T15:00Z` | `2026-04-30T15:00Z` |

A UTC month on from the first of those is 28 March, three days before the bar
stops moving, so the step happens in Korean time. Five months in twelve differ
that way. Every other interval has a fixed length and is unaffected, and Upbit's
monthly candles open at midnight UTC on the 1st, so the two Korean venues do not
date the same month alike.

`Auth` and `Unsupported` are not interchangeable: one is fixed by supplying a
key, the other is not, because `maxt` maps no endpoint there. Ask `client.supports(...)` first if you would rather branch than catch.

## Warnings and alerts

**Bithumb publishes two designations, on two endpoints, and they do not mean the
same thing.** `MarketStatus` has one value between them, so only the first
reaches it.

| Designation | What it is | `MarketStatus` | Where to read it |
| --- | --- | --- | --- |
| Warning, 유의 종목 | designated by hand and announced, while the market keeps trading. `/v1/market/all?isDetails=true` reports it in `market_warning` | `Unknown` | `MarketInfo::status`, or `market_warnings` |
| Caution, 주의 종목 | Bithumb's 경보제, raised and cleared automatically against published criteria, one row per criterion, each carrying a severity step and the moment it lapses | `Active` | `market_alerts` only |

**`market_warning` is spelled `CAUTION` and means 유의, the warning.** Bithumb
documents that field as 유의 종목 여부 and sends the reader elsewhere for 주의
종목, so the spelling is the one thing about it that misleads. `NONE` is the
only other value the enum holds. That designation is what Upbit also calls a
warning, so the two Korean adapters put the same concept in `Unknown`.

On 2026-07-30 Bithumb listed 486 markets: 15 carried the warning and 18 carried
at least one alert, 2 of them both. Reporting the alerts as `Unknown` too would
bury the 15 among them, and the alert set turns over through the day while the
warning set does not.

**The alert steps are ranked, and `BithumbAlertStep` compares in that order**, so
`step >= BithumbAlertStep::Warning` is the filter for "past the mildest".

| Step | Bithumb's word | What it is |
| --- | --- | --- |
| `BithumbAlertStep::Caution` | `CAUTION`, 주의 | the step raised first |
| `BithumbAlertStep::Warning` | `WARNING`, 경고 | the middle step, and the rarest |
| `BithumbAlertStep::Danger` | `DANGER`, 위험 | the gravest Bithumb documents, and the commonest in practice |
| `BithumbAlertStep::Unknown` | anything else | ranked above `Danger`, so a threshold surfaces a step Bithumb adds later instead of passing it |

`BithumbAlertStep::Caution` is 주의, the mildest alert step. The `CAUTION` that
`market_warnings` returns is 유의, the warning. The spelling collides; the
designations do not.

`BithumbMarketAlert::kind` is Bithumb's criterion, verbatim:

- `PRICE_SUDDEN_FLUCTUATION`, 가격 급등락
- `PRICE_DIFFERENCE_HIGH`, 글로벌 시세 차이
- `SPECIFIC_ACCOUNT_HIGH_TRANSACTION`, 소수계정 거래 집중
- `TRADING_VOLUME_SUDDEN_FLUCTUATION`, 거래량 급등
- `DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION`, 입금량 급등

Match on the ones you act on rather than on the list being non-empty. `kind` is
text, not an enum, because criteria are the part an exchange extends.

## Bithumb-only calls

Through `Client::adapter()`.

| Method | Gives you | Why it is not common |
| --- | --- | --- |
| `market_warnings()` | every listed market paired with its warning label, verbatim as Bithumb spells it, `"NONE"` where there is none | `MarketStatus` has no value meaning "trading, but flagged" |
| `market_alerts()` | one `BithumbMarketAlert` per raised alert, with the market, the criterion, the step and the expiry | `MarketStatus` carries no severity, no criterion and no expiry, and the caution never moves it off `Active` |

Bithumb labels markets it wants investors to be wary of while leaving them fully
tradable. `market_warnings()` returns every market, so it doubles as a market
list when the flag is what you filter on. `market_alerts()` does not: an
unalerted market is absent, and a market alerted on several criteria appears
once per criterion.

```rust
use maxt::{Client, adapters::{BithumbAdapter, BithumbAlertStep}};

async fn flagged() -> maxt::Result<()> {
    let client = Client::new(BithumbAdapter::new());
    for (market, label) in client.adapter().market_warnings().await? {
        println!("{market}: {label}");
    }
    for (market, alert) in client.adapter().market_alerts().await? {
        if alert.step >= BithumbAlertStep::Warning {
            println!("{market}: {} until {:?}", alert.kind, alert.ends_at);
        }
    }
    Ok(())
}
```

## Credentials

An access key and a secret key, issued together. They unlock
`Feature::Balances`, `Feature::OpenOrders`, `Feature::Trading`, and
`Feature::AccountStream`. Public data needs nothing.

```rust
use maxt::{Client, adapters::BithumbAdapter};

fn client() -> Client<BithumbAdapter> {
    let access_key = std::env::var("BITHUMB_ACCESS_KEY").expect("BITHUMB_ACCESS_KEY");
    let secret = std::env::var("BITHUMB_SECRET_KEY").expect("BITHUMB_SECRET_KEY");
    Client::new(BithumbAdapter::new().with_credentials(access_key, secret))
}
```

Every private call carries a JWT signed with HS256 over the secret key, naming
the access key, a fresh nonce, and a millisecond timestamp. A call with
parameters also names a SHA-512 hash of them, so a tampered query invalidates
the signature. The secret signs locally and never leaves the process. The
private WebSocket authenticates in the opening handshake, not in a frame, and
its token is minted afresh on every handshake, so a long-lived stream never
replays an ageing one.

**Because the token claims a timestamp, clock drift breaks credentials that are
otherwise correct.** Check the machine's clock first when working keys start
failing.

Keep keys out of source, and give them the narrowest permissions your program
actually uses.

## Examples

`cargo run --example public_rest`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Bithumb's own docs

| Subject | Pages |
| --- | --- |
| Quotas | [rate limits](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내) |
| Public REST | [markets](https://apidocs.bithumb.com/reference/거래-대상-목록-조회.md) · [alerts](https://apidocs.bithumb.com/reference/경보제-조회.md) · [tickers](https://apidocs.bithumb.com/reference/현재가-조회.md) · [order books](https://apidocs.bithumb.com/reference/호가-조회.md) · [trades](https://apidocs.bithumb.com/reference/체결-내역-조회.md) · [minute candles](https://apidocs.bithumb.com/reference/분minute-캔들-조회.md) |
| Private REST | [accounts](https://apidocs.bithumb.com/reference/전체-자산-조회.md) · [open orders](https://apidocs.bithumb.com/reference/대기-주문-목록-조회.md) · [cancel an order](https://apidocs.bithumb.com/reference/주문-취소-접수.md) |
| WebSocket | [trades](https://apidocs.bithumb.com/reference/체결-trade.md) · [order books](https://apidocs.bithumb.com/reference/호가-orderbook.md) · [account orders](https://apidocs.bithumb.com/reference/내-주문-및-체결-myorder.md) · [account assets](https://apidocs.bithumb.com/reference/내-자산-myasset.md) |

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
