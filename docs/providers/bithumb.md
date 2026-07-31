[English](bithumb.md) | [한국어](bithumb.ko.md)

# Bithumb

Spot only, one venue, no candle stream.

```rust
use maxt::{Client, Feature, adapters::BithumbAdapter};

let client = Client::new(BithumbAdapter::new());
assert!(client.supports(Feature::Candles));       // over REST
assert!(!client.supports(Feature::CandleStream)); // live, never
```

## What is supported

Markets are written quote asset first, `KRW-BTC`. Pass
`Market::spot(Exchange::Bithumb, "BTC", "KRW")`; `MarketInfo::native_symbol`
gives Bithumb's own spelling back.

| Call | Requirement |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | none |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | credentials |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | `Error::Unsupported`, always. Bithumb lists no derivatives |
| `reduce_only` on an order | `Error::Unsupported` |
| `markets(MarketKind::Perpetual)` | an empty list, not an error |

Everything not `Unsupported` behaves as [the common API](../common-api.md) says.

## Limits

Checked before the request is built.

| Call | Range | Outside it |
| --- | --- | --- |
| `trades` | `limit` 1 to 500 | `Error::InvalidRequest` on `limit` |
| `order_book` | any `depth` above 0, the best N levels. Bithumb takes no depth parameter, so `maxt` sorts both sides and truncates | `depth` of 0 is `Error::InvalidRequest` on `depth` |
| `order_book` `depth` above what Bithumb sent | fewer levels, no error. `/v1/orderbook` documents no count; a capture on 2026-07-30 gave 30 a side. Read `OrderBook` | none |
| `candles` | any `limit`; 200 per Bithumb response, paged for you over at most a hundred calls | `limit` of 0, a `from` not earlier than `to`, or a window past 20 000 candles: `Error::InvalidRequest` |
| `candles` intervals | the [baseline](../common-api.md#intervals) ten, and nothing else. Bithumb publishes no one-second endpoint | `Error::Unsupported` naming `Feature::Candles` |
| an unmapped interval together with a bad `limit` or window | `limit` and window are checked first | `Error::InvalidRequest` on that field, not `Unsupported` |
| Order ids | letters, digits, `-`, `.`, `_` | `Error::InvalidRequest` on `order_id` |

## Streams

| Subject | Behaviour |
| --- | --- |
| `Feed::Trades`, `Feed::OrderBook`, `Feed::Ticker` | carried |
| `Feed::Candles(_)` | `Error::Unsupported`, failing the whole subscription before a socket is opened. Nothing is synthesized from trades |
| `Feed::OrderBook` depth | 15 levels per side, which is what Bithumb [publishes](https://apidocs.bithumb.com/reference/호가-orderbook.md) and what every frame carried across 40 markets on 2026-07-30. `Subscription` cannot ask for more or fewer, and this is unrelated to REST's 30 |
| Book events | full snapshots, not diffs. Overwrite your copy on every one. Both the opening `SNAPSHOT` frame and the `REALTIME` ones carry all 15 levels |
| Book event clock | microseconds on the wire, unlike every other Bithumb payload |
| `subscribe_account` | the whole account, not a market list, one event per changed asset. Several balances may arrive in one frame |
| A live chart | `candles` over REST, or fold `Feed::Trades` yourself |

## Quotas

| Group | Limit |
| --- | --- |
| Public REST | 150 a second |
| Private REST | 140 a second |
| Orders, on top of the private figure | 10 a second |
| Scope of the REST figures | unstated by Bithumb. Treat as per IP |
| WebSocket | unstated by Bithumb. Not therefore unmetered |

`maxt` does not [throttle](../common-api.md#rate-limits).
`Error::is_rate_limited()` is how you learn you were too fast, and Bithumb
temporarily blocks an IP that keeps going.

## Orders

| Order | Size | Price |
| --- | --- | --- |
| Limit, either side | `Size::Base` | required |
| Market buy | `Size::Quote`, the amount to spend | none |
| Market sell | `Size::Base`, the quantity to offer | none |

Any other pairing is `Error::InvalidRequest` on `size`, as is a zero or negative
price or quantity. Any `TimeInForce` at all is `Error::InvalidRequest` on
`time_in_force`; orders get Bithumb's default for their type.

## Order precision and minimum size

[Not exposed](../common-api.md#order-precision-and-minimum-size). Price and
quantity are checked against zero and sent as given, so a price off Bithumb's
tick or an order under their minimum comes back as Bithumb's own rejection, not
as `Error::InvalidRequest` from here.

## Surprises

| Field or call | Behaviour |
| --- | --- |
| `place_order` status | `OrderStatus::Accepted`, never a fill. Bithumb answers with an identifier and nothing else. Read `open_orders`, or the account stream |
| `Order::remaining_quantity` after a market buy | zero. The acknowledgement carries no base figure |
| `Order::side` after a cancel | `Side::Buy` where Bithumb's response omits the side. It carries no meaning |
| An investment warning, 유의 종목 | `MarketStatus::Unknown`, still trading. See [Warnings and alerts](#warnings-and-alerts) |
| An investment caution, 주의 종목 | `MarketStatus::Active`, unchanged. Read it from `market_alerts`; the market list never carries it |
| `Trade::id` | Bithumb's `sequential_id`, as sent. Over REST it is the fill's millisecond times ten thousand, so fills sharing a millisecond share an id; the stream sends a per-fill number. Not a key across the two |
| `OrderBook::timestamp` from `Feed::OrderBook` | microseconds, the unit Bithumb documents and sends for that one frame. Every other Bithumb clock is milliseconds |
| `Ticker::timestamp` and `Ticker::last_trade_time` from `ticker` | pulled back nine hours. `/v1/ticker` documents both as UTC milliseconds and stamps both with the Korean wall clock. `maxt` measures the gap against the `trade_date` and `trade_time` in the same payload, so the correction lapses if Bithumb repairs the fields; a third value is `Error::Decode` |
| `Ticker::timestamp` against `last_trade_time` over REST | equal. `/v1/ticker` sends one number for both clocks. `Feed::Ticker` sends two |
| `trades` order | newest-first, sorted here. The sort is stable, so trades sharing a millisecond keep Bithumb's own order |
| `candles` order | oldest-first, though Bithumb answers newest-first |
| `candles` cursor | Bithumb's own is a bare wall-clock string read as Korean time. Pass a `Timestamp` and think in UTC |
| `Candle::closed` | `true` once the candle's own interval has ended, read off the local clock. Bithumb republishes the forming candle and marks nothing finished |
| `Candle::open_time` at `Month1` | 15:00 UTC on the last day of the previous UTC month, not the 1st |
| `Interval::Hour4` | 03:00, 07:00, 11:00, 15:00, 19:00 and 23:00 UTC: Bithumb cuts four-hour windows in Korean time and nine hours is not a multiple of four. Upbit, Binance and Hyperliquid open theirs at 00:00, 04:00 and so on |
| `Interval::Day1` | 15:00 UTC, midnight in Korea. A daily candle covers a Korean day |
| `Interval::Week1` | Sunday 15:00 UTC, Monday midnight in Korea. Upbit's and Binance's open Monday 00:00 UTC |
| `Min1` through `Hour1` | the same UTC grid as the other venues. Only `Hour4` and the daily-and-longer intervals shift |
| `open_orders` | Bithumb's resting-order state |
| No credentials | `Error::Auth`, before any request is built, not `Error::Unsupported` |
| A credential Bithumb refused | `Error::Exchange` carrying Bithumb's own name: HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_jwt`, `NotAllowIP`, `out_of_scope`. Upbit spells the last two `no_authorization_ip` and puts `out_of_scope` on 403, so one exchange's rule is wrong on the other. **Documented, not measured** |

Bithumb cuts `Month1` on the Korean calendar. Against `/v1/candles/months` for
`KRW-BTC`:

| Korean month | `open_time` | `closed` at |
| --- | --- | --- |
| March 2026 | `2026-02-28T15:00Z` | `2026-03-31T15:00Z` |
| April 2026 | `2026-03-31T15:00Z` | `2026-04-30T15:00Z` |

Five months in twelve differ from the UTC month. Every other interval has a
fixed length and is unaffected. Upbit's monthly candles open at midnight UTC on
the 1st.

## Warnings and alerts

Two designations, two endpoints, two meanings.

| Designation | Accessor | `MarketStatus` | Source |
| --- | --- | --- | --- |
| Warning, 유의 종목 | `market_warnings()`, or `MarketInfo::status` | `Unknown`, where Upbit's warning lands too | the `market_warning` field of `/v1/market/all?isDetails=true`. Designated by hand and announced; the market keeps trading |
| Caution, 주의 종목 | `market_alerts()` only | `Active` | Bithumb's 경보제, raised and cleared automatically against published criteria, one row per criterion with a severity step and an expiry |

The string `CAUTION` appears on both sides and means something different in each.

| `CAUTION` | Read from | Means |
| --- | --- | --- |
| the `market_warning` field | `market_warnings()` | 유의, the warning. `NONE` is the only other value |
| `BithumbMarketAlert::step` | `market_alerts()` | 주의, the mildest alert step |

On 2026-07-30, of 486 listed markets 15 carried the warning, 18 carried at least
one alert, and 2 carried both. The alert set turns over through the day; the
warning set does not.

`BithumbAlertStep` compares by severity, so `step >= BithumbAlertStep::Warning`
is the filter for "past the mildest".

| Step | Bithumb's word | Rank |
| --- | --- | --- |
| `BithumbAlertStep::Caution` | `CAUTION`, 주의 | raised first |
| `BithumbAlertStep::Warning` | `WARNING`, 경고 | the middle step, and the rarest |
| `BithumbAlertStep::Danger` | `DANGER`, 위험 | the gravest Bithumb documents, and the commonest in practice |
| `BithumbAlertStep::Unknown` | anything else | above `Danger`, so a threshold surfaces a step Bithumb adds later |

`BithumbMarketAlert::kind` is Bithumb's criterion, verbatim and untyped:

- `PRICE_SUDDEN_FLUCTUATION`, 가격 급등락
- `PRICE_DIFFERENCE_HIGH`, 글로벌 시세 차이
- `SPECIFIC_ACCOUNT_HIGH_TRANSACTION`, 소수계정 거래 집중
- `TRADING_VOLUME_SUDDEN_FLUCTUATION`, 거래량 급등
- `DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION`, 입금량 급등

## Bithumb-only calls

Through `Client::adapter()`.

| Method | Returns |
| --- | --- |
| `market_warnings()` | every listed market paired with its warning label, verbatim as Bithumb spells it, `"NONE"` where there is none. Doubles as a market list |
| `market_alerts()` | one `BithumbMarketAlert` per raised alert: the market, the criterion, the step, the expiry. An unalerted market is absent, and a market alerted on several criteria appears once per criterion |

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
`Feature::Balances`, `Feature::OpenOrders`, `Feature::Trading` and
`Feature::AccountStream`.

```rust
use maxt::{Client, adapters::BithumbAdapter};

fn client() -> Client<BithumbAdapter> {
    let access_key = std::env::var("BITHUMB_ACCESS_KEY").expect("BITHUMB_ACCESS_KEY");
    let secret = std::env::var("BITHUMB_SECRET_KEY").expect("BITHUMB_SECRET_KEY");
    Client::new(BithumbAdapter::new().with_credentials(access_key, secret))
}
```

| Subject | Rule |
| --- | --- |
| Signing | one JWT per private call, HS256 over the secret key, naming the access key, a fresh nonce and a millisecond timestamp |
| Parameters | also named as a SHA-512 hash, so a tampered query invalidates the signature |
| The secret | signs locally, never leaves the process |
| Private WebSocket | authenticates in the opening handshake, not in a frame, with a token minted afresh on every handshake |
| Clock drift | breaks otherwise-correct credentials, because the token claims a timestamp. Check the machine's clock first when working keys start failing |

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
