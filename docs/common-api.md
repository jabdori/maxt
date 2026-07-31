# Common API reference

[English](common-api.md) | [한국어](common-api.ko.md)

`Client<A>` presents one contract over an `Adapter`. Use the common methods for
portable behavior and `Client::adapter()` for operations that only one provider
offers. A `Client<Box<dyn Adapter>>` is available when the provider is selected
at runtime; the cost is one dynamic dispatch per adapter call.

## API surface

Credentials are configured on the adapter before it is wrapped in `Client`.

| Access | Common methods |
| --- | --- |
| Public REST | `markets`, `trades`, `order_book`, `ticker`, `candles`, `funding_rates` |
| Public stream | `subscribe`, `subscribe_with` |
| Private reads | `balances`, `open_orders`, `open_orders_on`, `positions`, `positions_on`, `margin_summary`, `funding_payments` |
| Private stream | `subscribe_account`, `subscribe_account_with` |
| Private writes | `place_order`, `cancel_order`, `set_margin` |

Public means that the method needs no account credential, not that every
provider or market kind supports it. Check `Client::supports` and the relevant
[provider page](providers.md).

## Common contracts

| Subject | Contract |
| --- | --- |
| Market identity | `Market` includes exchange, spot or perpetual kind, base asset, and quote asset. Assets are normalized to uppercase. |
| Trades | REST results are newest-first. `Trade::taker_side` is the aggressor's side. |
| Candles | Results are oldest-first. `open_time` is the start of the window. |
| Order books | Bids are highest-first and asks lowest-first, so both sides are best-first. |
| Numbers | Prices, quantities, rates, and amounts use exact `maxt::Decimal`, never `f64`. |
| Missing values | A field the provider does not publish is `None`; it is not inferred and is not zero. |
| Time | `Timestamp` is UTC nanoseconds since the Unix epoch. `Display` emits RFC 3339 at millisecond precision; use `as_nanos()` when exact precision matters. |
| Fallback clocks | Where an exchange publishes no payload time, the relevant field documents that `maxt` uses local read time. This is not exchange event time. |

Ticker fields do not necessarily describe the same instant or reporting window.
In particular, Hyperliquid does not expose a last-trade price in its asset
context: its REST and stream ticker map `midPx`, falling back to `markPx`, into
`Ticker::last_price`, and `last_trade_time` is `None`. Do not interpret that
field as a Hyperliquid fill. See the [Hyperliquid page](providers/hyperliquid.md).

## Public market data

| Method | Result and main boundary |
| --- | --- |
| `markets(kind)` | Listed markets of that kind; an unsupported market kind may be an empty list. |
| `trades(market, limit)` | Recent trades, newest-first; per-call limits vary. |
| `order_book(market, depth)` | One snapshot; `depth` means levels per side and accepted values vary. |
| `ticker(market)` | A provider summary with provider-dependent periods and optional fields. |
| `candles(request)` | Historical candles, oldest-first, with bounded internal paging. |
| `funding_rates(request)` | Public perpetual history, one `Page` at a time. |

Exact trade counts, book depths, timestamp sources, and candle grids are listed
on the [Upbit](providers/upbit.md), [Bithumb](providers/bithumb.md),
[Binance](providers/binance.md), and
[Hyperliquid](providers/hyperliquid.md) pages.

### Intervals

When `Feature::Candles` is supported, the portable baseline is `Min1`, `Min3`,
`Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, and `Month1`.
Additional intervals and candle opening grids are provider-specific. An
unsupported interval returns `Error::Unsupported`; it is never rounded to a
nearby interval.

### Candle ranges and completion

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{CandleRequest, Client, Exchange, Interval, Market, Timestamp};

# async fn read() -> maxt::Result<()> {
let client = Client::new(UpbitAdapter::new());
let request = CandleRequest::new(
    Market::spot(Exchange::Upbit, "BTC", "KRW"),
    Interval::Min1,
)
.from(Timestamp::from_millis(1_700_000_000_000))
.to(Timestamp::from_millis(1_700_007_200_000))
.limit(120);

let candles = client.candles(&request).await?;
assert!(candles.windows(2).all(|pair| pair[0].open_time <= pair[1].open_time));
# Ok(())
# }
```

- `from` is inclusive by candle open time.
- `to` is exclusive by candle open time.
- With `from`, `limit` selects the oldest matching candles from that boundary.
- Without `from`, `limit` selects the newest candles.
- A set `limit` must be at least one. `from >= to` is invalid.
- `maxt` pages internally for at most 100 provider calls. Request longer
  histories in bounded batches.

`Candle::closed` means that the interval has ended. A REST response can include
the newest still-forming candle. Binance streams carry the provider's close
flag. Upbit and Hyperliquid emit a settled candle when a later window first
arrives; a final window or one interrupted by reconnect may therefore never
produce a settled event. Bithumb has no candle stream. The provider pages above
document interval and stream constraints without duplicating them here.

## Streams

A `Subscription` is a logical stream, not a promise of one WebSocket. It applies
every requested feed to every requested market, rejects an empty market or feed
set, removes duplicates, and preserves insertion order. Binance USD-M may split
one subscription over multiple sockets and merge their events into the returned
`MarketStream`.

`StreamConfig::default()` is:

| Field | Default | Meaning |
| --- | ---: | --- |
| `max_reconnect_attempts` | `None` | Reconnect without a finite attempt budget. |
| `initial_reconnect_delay_ms` | `1_000` | First reconnect delay. |
| `max_reconnect_delay_ms` | `30_000` | Backoff ceiling. |
| `idle_timeout_ms` | `30_000` | Requested minimum inactivity timeout; an adapter may raise it. |
| `buffer_size` | `4_096` | Consumer event queue. A requested zero is implemented as capacity one. |
| `overflow` | `Backpressure` | Wait for the consumer instead of dropping events. |

`Overflow::Backpressure` stops socket reads until the consumer catches up. It
does not discard an event, but a sufficiently long stall can still make the
venue close the connection. `Overflow::DropNewest` silently discards arriving
data and error items while the queue is full. It is suitable only when a later
event fully replaces the missed value, such as ticker and full-book snapshots;
it loses unique trades and the only settled emission of a candle.

Reconnect notices are not discarded by `DropNewest`: they are held until the
queue has room and delivered before later data. A notice means events during the
gap were missed. Rebuild order books from the next snapshot; after an account
reconnect, re-read balances and open orders over REST.

Reconnect budgets count every reconnect and apply to each underlying connection
separately. They are not reset by a healthy interval, so a finite budget can
eventually end a stream even when a venue routinely recycles sockets. An `Err`
stream item is a report and does not itself end the stream. Only `None` means
termination. If one socket in a split Binance USD-M subscription ends, the
logical stream ends and drops the remaining socket instead of silently losing
some feeds. Dropping the stream closes all underlying connections.

## Capability checks

`Feature::needs_credentials()` identifies features that touch an account.
`Client::supports(feature)` is local and performs no network call.

### A `true` still has to be checked at the call

`true` means the configured adapter has a mapped operation for that feature. It
does not guarantee that a particular market, interval, order shape, credential,
or exchange-side permission will be accepted.

A `false` has two materially different causes:

1. The provider structurally lacks the feature. Calling it returns
   `Error::Unsupported`; adding credentials cannot change that.
2. A shipped adapter maps a private endpoint but has no credentials. Calling it
   returns `Error::Auth`; configuring credentials can make `supports` true.

Match the call's `Result` even after checking the capability.

## Errors and retry safety

| Error | Meaning | Retry unchanged? |
| --- | --- | --- |
| `InvalidRequest` | Local validation rejected a field before sending. | No. |
| `Unsupported` | No mapped operation exists for this provider or market kind. | No. |
| `Auth` | A credentialed request could not be built locally. | No. |
| `Exchange` | The exchange rejected or failed the request; its code and message are retained. | Check `is_retryable()`. |
| `Transport` | DNS, TLS, socket, or timeout prevented a known result. | Plausibly, but see below. |
| `Decode` | The response shape could not be interpreted. | No; report it. |

`maxt` does not automatically retry REST calls. If an application builds an
automatic policy around `Error::is_retryable`, apply it only to reads and other
idempotent requests, with backoff and provider rate limits. A transport failure
after placing or cancelling an order does not prove that the exchange rejected
the write. Query order or account state before sending another write, or a retry
can duplicate the intended action.

## Private accounts and trading

Private paths are covered by offline fixtures and mock-server tests, but were
not part of the 2026-07-31 live conformance run. Start with read-only or testnet
credentials and follow the provider page for credential and order constraints.

`open_orders` and `open_orders_on` return a snapshot, not a subscription and not
a guarantee that every provider history page was traversed. In particular, the
current Bithumb adapter issues one `/v1/orders` request and returns at most that
endpoint's 100-order page. Do not build reconciliation on a stronger guarantee.

`OrderRequest` distinguishes base-asset and quote-asset sizing with `Size`.
Provider support for market orders, quote sizing, reduce-only behavior, and
`TimeInForce` variants differs. Cancellation races fills, so inspect the
returned order and then reconcile account state when execution matters.

### Order precision and minimum size

The common `MarketInfo` does not expose tick size, lot size, or minimum notional.
Some adapters validate provider precision before sending, while minimums can
still be exchange-side rules. Read the provider-specific order section and do
not round through `f64`; construct values with `maxt::Decimal`.

### Derivatives and history pages

`positions` and `positions_on` omit zero-size rows. `MarginSummary` fields remain
`None` when the provider does not publish them. Funding-rate history is public;
funding-payment history is private and signed, with negative amounts meaning the
account paid.

For `Page<T>`, pass `Page::next` back as the next request's cursor and stop only
when it is `None`. A short or empty `items` list is not an end marker while a
cursor is present.

`MarginRequest` can express leverage, margin mode, or both, but providers impose
different requirements. When both fields are accepted, `set_margin` is not
guaranteed to be atomic: one provider operation may succeed before another
fails. Re-read the resulting configuration instead of assuming rollback.

## Provider-specific methods

Provider-only batching, alerts, native contexts, and ledger calls stay on the
concrete adapter:

```rust
use maxt::adapters::UpbitAdapter;
use maxt::Client;

let client = Client::new(UpbitAdapter::new());
let upbit: &UpbitAdapter = client.adapter();
let _ = upbit.region();
```

This keeps the common contract portable without hiding useful venue features.

## Rate limits

`maxt` does not throttle REST calls or allocate a provider's request budget for
the application. Review the quota section on each provider page, centralize
application-side limiting, and back off on `Error::is_rate_limited()`.

## External adapters

The public `Adapter` trait is implementable outside the crate. Implement
`exchange` and `supports`, override supported operations, and preserve every
common ordering, validation, missing-value, decimal, and timestamp contract.
Optional methods default to `Error::Unsupported`; a new real exchange still
requires an `Exchange` variant in `maxt`. See the
[adapter checklist](../CONTRIBUTING.md#adapter-checklist).
