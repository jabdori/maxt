# Common API reference

[English](common-api.md) | [한국어](common-api.ko.md)

`Client<A>` exposes one common contract over an `Adapter`. Provider-specific
methods remain on `A` and are available through `Client::adapter()`.
`Client<Box<dyn Adapter>>` supports runtime provider selection.

## API surface

Configure credentials on the adapter before `Client::new(adapter)`.

| Surface | Methods |
| --- | --- |
| Client | `exchange`, `supports`, `adapter`, `into_adapter` |
| Public REST | `markets`, `trades`, `order_book`, `ticker`, `candles`, `funding_rates` |
| Public stream | `subscribe`, `subscribe_with` |
| Private reads | `balances`, `open_orders`, `open_orders_on`, `positions`, `positions_on`, `margin_summary`, `funding_payments` |
| Private stream | `subscribe_account`, `subscribe_account_with` |
| Private writes | `place_order`, `cancel_order`, `set_margin` |

Public REST and market streams require no credentials. Provider and
`MarketKind` support is listed in the [provider matrix](providers.md).

## Data contracts

| Type or field | Contract |
| --- | --- |
| `Market` | `{ exchange, kind, base, quote }`; `base` and `quote` are uppercase |
| `Trade` | REST results are newest-first; `taker_side` is the aggressor side |
| `Candle` | `open_time ASC`; `open_time` is the interval start |
| `OrderBook::bids` | `price DESC` |
| `OrderBook::asks` | `price ASC` |
| `Ticker` | Provider summary; field source and aggregation window are provider-specific |
| `Decimal` | 96-bit coefficient and scale `0..=28`; prices, quantities, rates, and amounts never use `f64` |
| `Option<T>` | Provider omission maps to `None`; no inference or zero fill |
| `Timestamp` | UTC nanoseconds since Unix epoch; `Display` is millisecond RFC 3339; exact value via `as_nanos()` |

A `timestamp` documented as local read time records when `maxt` read the
response, not provider event time.

`parse_decimal_exact()` accepts plain or scientific notation only when the
value fits `Decimal` exactly. It returns an error instead of rounding or
truncating.

## Public REST

| Method | Contract |
| --- | --- |
| `markets(kind)` | Listed `MarketInfo`; a valid `kind` with no listings returns `[]` |
| `trades(market, limit)` | Recent trades, newest-first |
| `order_book(market, depth)` | One snapshot; if `depth` is set, `bids.len() <= depth` and `asks.len() <= depth` |
| `ticker(market)` | One provider summary |
| `candles(request)` | Historical candles, `open_time ASC` |
| `funding_rates(request)` | Public perpetual funding-rate `Page<FundingRate>` |

Limits, book depths, and timestamp sources are provider-specific.

## Candles

### Intervals

`supports(Feature::Candles) == true` guarantees:

`Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`,
`Month1`

Other intervals are provider-specific. An interval without a provider mapping
returns `Error::Unsupported`; it is never rounded to another interval.

### `CandleRequest`

Results are always sorted by `open_time ASC`.

| Fields set | Selection |
| --- | --- |
| `from`, `to`, `limit` | Earliest `limit` rows where `from <= open_time < to` |
| `from`, `to` | All rows where `from <= open_time < to` |
| `from`, `limit` | Earliest `limit` rows where `from <= open_time` |
| `to`, `limit` | Latest `limit` rows where `open_time < to` |
| `limit` | Latest `limit` rows |
| `from` | All rows where `from <= open_time` |
| `to` | One provider page where `open_time < to` |
| None | Latest provider page |

| Validation | Result |
| --- | --- |
| `limit == 0` | `Error::InvalidRequest` |
| `from >= to` | `Error::InvalidRequest` |
| Provider calls | At most 100 per request |
| Estimated candles `> 100 * provider_page_cap` | Reject before network I/O |

| `Candle::closed` source | Contract |
| --- | --- |
| REST | `interval_end <= local_read_time` |
| Stream | Provider-specific; see the provider reference |

## Streams

### `Subscription`

| Input | Contract |
| --- | --- |
| Subscription set | `markets × feeds` |
| Duplicate market or feed | Remove; preserve first insertion order |
| `markets.is_empty() || feeds.is_empty()` | `Error::InvalidRequest` |
| Logical stream | One or more WebSocket connections |

### `StreamConfig`

| Field | Default | Contract |
| --- | ---: | --- |
| `max_reconnect_attempts` | `None` | No finite reconnect limit |
| `initial_reconnect_delay_ms` | `1_000` | First reconnect delay |
| `max_reconnect_delay_ms` | `30_000` | Backoff ceiling |
| `idle_timeout_ms` | `30_000` | `max(config, provider_minimum)` |
| `buffer_size` | `4_096` | `0 -> 1` |
| `overflow` | `Backpressure` | Block the producer while full |

| Overflow policy | Full buffer |
| --- | --- |
| `Backpressure` | Pause socket reads; no intentional event loss |
| `DropNewest` | Drop new data and errors; preserve reconnect notices |

`DropNewest` is valid only for replaceable snapshots. Trades and closed-candle
events are not replaceable.

### State

| State | Contract |
| --- | --- |
| `Some(Ok(event))` | Event |
| `Some(Err(error))` | Non-terminal error |
| `None` | Stream terminated |
| `MarketEvent::Reconnected` | Market events were lost during the disconnect |
| `AccountEvent::Reconnected` | Account events were lost during the disconnect |
| Built-in stream `Drop` | Drop the source and signal all built-in connection tasks to stop |
| Custom stream `Drop` | Drop the source; the producer owns cleanup |
| `close().await` | Await adapter-provided async cleanup, then drop the source |

Reconnect limits apply per underlying connection and do not reset after
healthy traffic. After `AccountEvent::Reconnected`, reload state with
`balances()` and `open_orders()`.

## Capability checks

`Client::supports(feature)` performs no network I/O.

| State | Contract |
| --- | --- |
| Operation mapped; required credentials present | `supports(feature) == true`; arguments and provider permissions still apply |
| Private operation mapped; credentials absent | `supports(feature) == false`; call returns `Error::Auth` |
| Structurally unsupported | `supports(feature) == false`; call returns `Error::Unsupported` |

## Errors and retries

| `Error` | Source | `is_retryable()` |
| --- | --- | --- |
| `InvalidRequest` | Local validation before network I/O | `false` |
| `Unsupported` | No mapped operation | `false` |
| `Adapter` | Adapter or foreign-dispatcher contract failure | `false` |
| `Auth` | Credentialed request cannot be built locally | `false` |
| `Exchange` | Provider error response | `kind.is_retryable()` |
| `Transport` | DNS, TLS, socket, timeout | `true` |
| `Decode` | Response schema mismatch | `false` |

`maxt` does not retry or throttle REST calls. Apply provider limits and
backoff in the application. `place_order` or `cancel_order` followed by
`Error::Transport` has an unknown outcome; query account or order state before
retrying.

## Private accounts and orders

| Type or method | Contract |
| --- | --- |
| `open_orders*` | Point-in-time snapshot; full provider pagination is not guaranteed |
| `OrderRequest::size` | `Size::Base` or `Size::Quote` |
| Order precision | `MarketInfo` has no common tick size, lot size, or minimum notional |
| `cancel_order` | May race a fill; returned `Order` is a provider acknowledgement and may omit final fill state |
| `positions*` | Remove rows where `position.quantity == 0` |
| `MarginSummary` | Provider omission maps to `None` |
| `FundingPayment::amount < 0` | Account paid funding |

Construct order values with `Decimal`. Supported order shapes and validation
rules are provider-specific.

### `HistoryRequest`

| Field or state | Contract |
| --- | --- |
| `from` | `from <= item.timestamp` |
| `to` | `item.timestamp < to` |
| `cursor` | Opaque resume point from `Page::next`; overrides `from` and must be passed back unchanged to the same adapter |
| `limit` | Page-size target, not a hard maximum |
| Same-timestamp group crosses `limit` | The page may stop below `limit` and defer the group, or exceed `limit` when the first group alone is larger |
| Next request | Set `request.cursor = page.next` |
| Continue | `page.next.is_some()`, even when `items.is_empty()` |
| Stop | `page.next == None` |

### `MarginRequest`

| State | Contract |
| --- | --- |
| Local validation | `leverage.is_some() || margin_mode.is_some()` |
| Provider validation | May require one field or both |
| `set_margin()` | No atomicity or rollback guarantee across both changes |

## Provider-specific APIs

`Client::adapter(&self) -> &A` exposes non-portable batching, native context,
alert, and ledger methods. See each provider reference.

## External adapters

External types may implement `Adapter` with `exchange()` and `supports()`.
Override supported methods; all other methods return `Error::Unsupported`.
Preserve the common contracts above. A new exchange still requires a new
`Exchange` variant in `maxt`.

[Adapter checklist](../CONTRIBUTING.md#adapter-checklist)
