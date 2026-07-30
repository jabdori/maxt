# The common API

[English](common-api.md) | [한국어](common-api.ko.md)

`Client` wraps an adapter and exposes everything the supported exchanges have in
common. The calls mean the same thing whichever exchange is underneath. Anything
one exchange offers that the others do not stays on its own adapter.

## Market data

None of these needs credentials.

| Call | Answers |
| --- | --- |
| `markets(kind)` | every market of one kind the exchange lists |
| `ticker(&market)` | the rolling 24-hour summary |
| `order_book(&market, depth)` | an order book snapshot |
| `trades(&market, limit)` | recent trades, newest first |
| `candles(&CandleRequest)` | historical candles, oldest first |

| Case | What happens |
| --- | --- |
| `markets` on a kind the exchange does not list | an empty list, not an error |
| `depth` outside the set the exchange serves | `Error::InvalidRequest`, never rounded to a nearby depth. Each provider page lists the set |
| `limit` on `trades` outside its range | `Error::InvalidRequest`, by the same rule. The ceiling is one call's worth and differs sharply: 1 000 on Binance, 500 on Upbit and Bithumb, 10 on Hyperliquid, whose endpoint takes no count at all. Leave `limit` unset to take what the exchange sends |
| `trades` ordering | Binance sorts descending by trade id, exact when several trades share a millisecond; Upbit, Bithumb and Hyperliquid sort by timestamp |
| reading `MarketInfo::native_symbol` | what the exchange itself calls the market, for reconciling against its own screen or documentation |

### What a listing entry carries

`markets` returns `MarketInfo`, not a bare `Market`.

| Field | Holds |
| --- | --- |
| `market` | the identity every other call takes |
| `native_symbol` | the exchange's own symbol, verbatim: `KRW-BTC`, `BTCUSDT`, `BTC` |
| `status` | whether the exchange is accepting orders on it |
| `korean_name`, `english_name` | the asset's names, when the exchange publishes them. Binance and Hyperliquid publish neither, so both are `None` there |

| `MarketStatus` | Means |
| --- | --- |
| `Active` | listed and trading |
| `Paused` | halted, but the listing still exists and the pair comes back |
| `Delisted` | gone |
| `Unknown` | the exchange's answer maps onto none of the other three |

`MarketStatus` is `#[non_exhaustive]`, so match on it with a `_` arm.

`Unknown` is not the same as untradable, and on the Korean exchanges it usually
is not. Upbit and Bithumb both designate a market for investment warning while
leaving it tradable, and `MarketStatus` has no value meaning "trading, but
flagged", so those read as `Unknown` here. Each exchange's own label stays
verbatim on `BithumbAdapter::market_warnings` and `UpbitAdapter::market_events`.
Both exchanges publish a second, milder designation that does not reach
`MarketStatus` at all: `UpbitAdapter::market_events` and
`BithumbAdapter::market_alerts` are the only places they are readable. Treat
`Unknown` as "ask the exchange", not as a refusal.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::{Client, MarketKind, MarketStatus};

async fn tradable_krw_markets(client: &Client<UpbitAdapter>) -> maxt::Result<usize> {
    let listed = client.markets(MarketKind::Spot).await?;

    let tradable = listed
        .iter()
        // A listing is not a promise that it trades right now.
        .filter(|info| info.market.quote == "KRW" && info.status == MarketStatus::Active)
        .count();

    for info in listed.iter().take(3) {
        // `english_name` is `None` wherever the exchange publishes no name.
        let name = info.english_name.as_deref().unwrap_or(&info.native_symbol);
        println!("{} ({name}) is {} upstream", info.market, info.native_symbol);
    }

    Ok(tradable)
}
```

## Candles

Two of the four exchanges have no start-time parameter, and all four cap a
single response. `maxt` reconciles that internally, so one contract holds
everywhere, and the answer is always oldest first whichever order the exchange
replied in.

| The request | What comes back |
| --- | --- |
| `from` | honoured on every exchange; no adapter reports it as `Error::Unsupported` |
| `limit` | honoured past the per-response cap, by paging. The cap is a property of one HTTP response, not of the call |
| `from` and `limit` together | the oldest `limit` candles at or after `from`, which is what a backfill asks for |
| `limit` alone | the newest that many |
| `limit` of `0` | `Error::InvalidRequest` on `limit`; leave it unset for one default page |
| `from` at or after `to` | `Error::InvalidRequest` on `from` |

### `Candle::closed`

`true` once the candle's window has ended, on all four exchanges. The last
candle of a response is the one that can be `false`, because it is the one
still forming.

| Where the window's end comes from | Exchanges |
| --- | --- |
| the exchange's own close time, read off the payload | Binance, Hyperliquid |
| the interval, stepped one window on from `open_time` in the zone the exchange cuts on: UTC at Upbit, KST at Bithumb | Upbit and Bithumb over REST |
| the frame that opens the next window, because no frame of a window is ever stamped at or past that window's end | the Upbit and Hyperliquid candle streams |
| the exchange's own flag on the frame | Binance's candle stream |

They answer the same question, so a consumer commits on `closed` alone and does
not branch per exchange.

Two streams cannot answer from a single frame, so `maxt` answers them from the
transition. Upbit stops publishing a window the instant the next one opens;
Hyperliquid stops about two seconds before the window's own close time. Neither
ever stamps a frame at or past its window's end, so neither the payload nor a
clock reading of one frame can call a bar settled. On both, each subscription
holds a window's last frame and emits it with `closed` set when the exchange
opens the next one, so a window is never called finished before the exchange
stopped publishing it, and the final window of a subscription you drop is never
settled at all. Binance states it outright with its own flag, and Bithumb has no
candle stream. That settled emission repeats the window's `open_time`, so a
consumer keyed on `open_time` overwrites rather than appends. A reconnect drops
whatever was held rather than settling it across a gap of unknown length, so the
window a `MarketEvent::Reconnected` interrupts gets no settled emission. That one
emission per window is also why `Overflow::DropNewest` is wrong for
`Feed::Candles`; see [Stream configuration](#stream-configuration).

**An interval names a length, not a grid.** Where a candle of that length opens
is the exchange's own decision, and the four do not agree at `Hour4`, `Day1`,
`Day3`, `Week1` or `Month1`. `Interval`'s own documentation carries the whole
table, read off all four live. In short: Bithumb cuts every window in Korean
time, so it matches the others at every interval that divides nine hours and
misses at every one that does not, which is `Hour4` and everything daily or
longer. Hyperliquid measures `Day3`, `Week1` and `Month1` from the Unix epoch
rather than from the calendar, so its weeks open on a Thursday and its months
are 30-day buckets. `closed` still answers the same question on all four. Only
`open_time` differs, and joining two exchanges on `open_time` is safe only where
that table says the same thing about both.

```rust
use maxt::{Candle, Decimal};

// The forming candle is not a settled one. Take the last that is.
fn last_settled_close(candles: &[Candle]) -> Option<Decimal> {
    candles.iter().rev().find(|candle| candle.closed).map(|candle| candle.close)
}
```

### Paging is bounded at a hundred calls

Every one of these exchanges pages backwards one response at a time, so a wide
window costs one sequential round trip per page and nothing makes it faster.
`maxt` walks at most a hundred pages, so the widest one call can assemble is a
hundred times the exchange's per-response cap: about twenty thousand candles at
Upbit's two hundred a call. Ask for more and the request is
`Error::InvalidRequest` **before the first call**, naming the field that made it
too wide and saying what the ceiling is:

| Request | Refused on |
| --- | --- |
| a `from` far enough back that the window needs more than a hundred pages, with no `limit` | `from` |
| a `limit` above a hundred times the per-response cap | `limit` |

Refusing up front is the point: a walk that a rate limit abandons halfway has
already spent the calls. To read more than the ceiling, set `limit` and step the
window yourself. `src/adapters/candles.rs` holds the walk and the ceiling.

## Intervals

| Interval set | What holds |
| --- | --- |
| the baseline | `client.supports(Feature::Candles) == true` guarantees ten on every exchange: `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1`. Write against those and the same code reads candles from all four. **It does not read the same grid from all four**: at `Hour4`, `Day1`, `Week1` and `Month1` the open times differ by venue, and `Interval` documents where |
| beyond it | per-exchange. `Hour2`, `Hour8`, `Hour12` and `Day3` are Binance and Hyperliquid only; `Sec1` is Upbit and Binance spot, and Binance USD-M does not serve it. Check the provider page |
| an interval an exchange does not serve | `Error::Unsupported` naming `Feature::Candles`. No `Feature` separates the sets, so the flag cannot tell you which |

**`Unsupported` on an interval means `maxt` maps no endpoint there.** It is not a
claim that the exchange never aggregated it. Upbit and Bithumb both serve
ten-minute candles and Upbit serves yearly ones, and `Interval` names neither, so
`maxt` cannot ask. Where an exchange publishes an interval `maxt` reaches, it
reaches it: the baseline is read off the four exchanges' own documentation, not
off the adapters, which is what `BASELINE_INTERVALS` in
[`tests/unsupported_is_honest.rs`](../tests/unsupported_is_honest.rs) asserts.

## Account

Everything here needs credentials.

| Call | Answers |
| --- | --- |
| `balances()` | every asset the account holds, available and locked. An exchange may report every asset it lists, most of them empty, so filter on what you actually care about |
| `open_orders()` | open orders across every market |
| `open_orders_on(&market)` | open orders on one market |
| `subscribe_account()` | a live stream of balance and order updates |
| `Balance::total()` | available plus locked |

Size an order off `Balance::available`. `locked` is promised to resting orders,
and spending it is what the exchange rejects.

## Orders

Both calls need credentials.

| Call or type | What it does |
| --- | --- |
| `place_order(&OrderRequest)` | returns an `Order` carrying the exchange's own identifier |
| `cancel_order(&market, order_id)` | takes that identifier back |
| `OrderRequest::market(market, side, size)` | builds a market order, which has no price |
| `OrderRequest::limit(market, side, size, price)` | builds a limit order, which always has one |
| `Size::Base`, `Size::Quote` | names the asset the size is denominated in, because a market buy is usually sized in the quote asset and a market sell in the base asset |
| `.time_in_force(..)` | `TimeInForce::PostOnly` to place only if the order rests on the book |
| `.reduce_only()` | restricts the order to closing an existing position. Derivatives only |
| `OrderStatus::is_live()` | true while the order can still fill: accepted, open, or partially filled |

Both constructors take the size as a `Size`, never a bare number, so a market
buy sized in won cannot be mistaken for one sized in bitcoin.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Decimal, Exchange, Market, OrderRequest, OrderStatus, Side, Size, TimeInForce};

async fn buy_a_little_bitcoin(client: &Client<UpbitAdapter>) -> maxt::Result<()> {
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    // The price comes off the book, not out of this page. A figure written here
    // is safe only while the market stays above it, it turns into an immediate
    // taker fill the day the market does not, and nothing announces the change.
    // The deepest bid the exchange returned is below every other bid by
    // construction, so a buy at it cannot cross the ask whatever the market is
    // doing, and it is already a price this market accepts, which an invented
    // number need not be.
    let book = client.order_book(&market, None).await?;
    let Some(deepest_bid) = book.bids.last() else {
        return Ok(());
    };

    let order = client
        .place_order(
            &OrderRequest::limit(
                market.clone(),
                Side::Buy,
                // 0.001 BTC. `Size::Quote` here would have meant 0.001 KRW.
                Size::Base(Decimal::new(1, 3)),
                deepest_bid.price,
            )
            // Rejected outright rather than filled as a taker if the book moved
            // between that read and this call.
            .time_in_force(TimeInForce::PostOnly),
        )
        .await?;

    // Accepted is not filled. `is_live` is the test for "can still fill".
    if order.status.is_live() {
        // The id off the returned order is what cancels it. A cancel races the
        // book, so trust the order that comes back, not the one that went in.
        let cancelled = client.cancel_order(&market, &order.id).await?;
        assert_eq!(cancelled.id, order.id);
        if cancelled.status == OrderStatus::Filled {
            println!("filled before the cancel landed");
        }
    }

    Ok(())
}
```

Not every exchange accepts every combination. Hyperliquid has no market order
type, and quote-denominated sizing is not universal. The provider pages state
which shapes each one takes.

**No limit price written into a document stays safe.** Whether a figure rests on
the book or fills as a taker is a fact about the market on the day it is run, and
a page cannot know that day. So:

| To place an order that will not fill on arrival | How |
| --- | --- |
| price it | off a book read moments earlier, at or below the best bid for a buy and at or above the best ask for a sell |
| guard it | `TimeInForce::PostOnly`, which has the exchange reject it rather than fill it if the book moved in between |
| never | copy a price out of a document, including this one. Every figure here is either read from the exchange at run time or is a size, never a price |

## Order precision and minimum size

Tick size, lot step, and minimum order value decide whether an order is accepted
at all. The common API does not carry them: no two exchanges express them alike,
and one flattened type would have to state something untrue about most of them.
Two adapters expose their exchange's own answer, and both are reached through
`Client::adapter`.

| Venue configuration | Where the rules are |
| --- | --- |
| Binance spot | `BinanceAdapter::spot_symbol_filters(&market)`, returning `BinanceSymbolFilters`: `tick_size`, `min_price`, `max_price`, `step_size`, `min_quantity`, `max_quantity`, `min_notional`. **Reported, not enforced:** `place_order` never reads them |
| Hyperliquid | `HyperliquidAdapter::asset_context(&market)`, whose `size_decimals` and `price_decimals` say how many decimals each may carry. Decimals only, not a minimum order value. `maxt` checks both before signing, so an order finer than the asset allows is refused locally |
| Binance USD-M | not exposed. `spot_symbol_filters` reports `Error::Unsupported` on a USD-M adapter, whose listing carries a different set of filters |
| Upbit, Bithumb | not exposed at all |

**Exposing the rules is not checking against them.** Hyperliquid is the one
venue configuration of the five that validates locally
(`src/adapters/hyperliquid/rest.rs`, both rules, before signing). On the other
four, Binance spot included, an order whose price or size is too fine is found
out by the exchange refusing it, which arrives as `Error::Exchange` carrying
that exchange's own code and message. Reading `spot_symbol_filters` and
rounding to what it says is the caller's job. Nothing in `maxt` rounds an order
to fit, on any of the five.

## Derivatives

Meaningful only on perpetual markets. On a spot-only adapter every one of these
is `Error::Unsupported`, which
[`tests/unsupported_is_honest.rs`](../tests/unsupported_is_honest.rs) checks on
every configuration. One Hyperliquid adapter carries both kinds, so there the
same calls read as supported and refuse per market: hand one a spot market and
it is `Error::Unsupported` too.

| Call | Answers | Credentials |
| --- | --- | --- |
| `positions()`, `positions_on(&market)` | open positions. A position with no size is not one, and no adapter reports one, whatever its venue publishes | yes |
| `margin_summary()` | account-wide margin state | yes |
| `funding_rates(&HistoryRequest)` | a market's funding rate history, a property of the market rather than of any account | no |
| `funding_payments(&HistoryRequest)` | what one account was actually charged | yes |
| `set_margin(&MarginRequest)` | sets leverage, margin mode, or both. At least one is required, and Hyperliquid requires both: one alone there is `Error::InvalidRequest` | yes |

### Sizing off the right figure

`MarginSummary` carries three optional figures, and they are not
interchangeable.

| Field | Is |
| --- | --- |
| `equity` | balance plus unrealized profit and loss |
| `margin_balance` | what is **already posted** against open positions and orders |
| `available_balance` | what is **free to open with** |

Size a new order off `available_balance`. `margin_balance` is money already
committed, and an order sized off it is sized off funds the account has spent.
Each is `Option`, because not every exchange publishes all three. A sizing rule
that reads a missing one as zero opens nothing; one that treats it as unlimited
opens far too much. Read `None` as "the exchange did not say" and stop.

### Positions

| Field | Read it as |
| --- | --- |
| `quantity` | unsigned. The direction is in `side`, which is `None` when the position is flat |
| `is_flat()` | the size test. An exchange may report flat positions on markets you no longer hold, so skip those; each one is not an open risk |
| any unpublished field | `None`, and `None` is not zero |
| `leverage`, `margin_mode` on Binance | always `None`: the position endpoint `maxt` reads stopped publishing either (`src/adapters/binance/private.rs`). `None` there says nothing about how the position is margined. Defaulting `leverage` to `1` would report a position opened at 20x as unleveraged and understate the risk twentyfold. `maxt` does not subscribe to `ACCOUNT_CONFIG_UPDATE` on the account stream either, for the same reason: it carries a leverage change `maxt` has nowhere to report |

### A worked derivatives read

```rust
use maxt::adapters::BinanceAdapter;
use maxt::{
    Client, Decimal, Exchange, HistoryRequest, MarginMode, MarginRequest, Market,
};

async fn size_a_perpetual_order() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::usd_m_futures().with_credentials("key", "secret"));
    let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
    let leverage = Decimal::from(5);

    // One request, two Binance calls: `POST /fapi/v1/leverage`, then
    // `POST /fapi/v1/marginType`. Binance offers no atomic pair, so between
    // them the account sits at the new leverage under the old mode. If the mode
    // call fails, and Binance routinely refuses a `marginType` change against
    // an open position, the account stays there. Read the state back before
    // sizing anything off the leverage you asked for.
    client
        .set_margin(
            &MarginRequest::new(market.clone())
                .leverage(leverage)
                .margin_mode(MarginMode::Isolated),
        )
        .await?;

    // `available_balance` is what can back a new position. `margin_balance` is
    // what already backs the old ones.
    let margin = client.margin_summary().await?;
    match margin.available_balance {
        Some(free) => println!("{} {} free, {} of notional at 5x", free, margin.asset, free * leverage),
        None => println!("{} publishes no free-margin figure", client.exchange()),
    }

    for position in client.positions_on(&market).await? {
        // A flat position is still a position the exchange may report.
        if position.is_flat() {
            continue;
        }
        // `None` is "not published", so summing it as zero understates exposure.
        println!(
            "{:?} {} at {:?}, notional {:?}",
            position.side, position.quantity, position.entry_price, position.notional
        );
    }

    // Public: the market's rate, not the account's bill.
    let rates = client
        .funding_rates(&HistoryRequest::new(market.clone()).limit(100))
        .await?;
    let mean: Decimal = rates.items.iter().map(|rate| rate.rate).sum::<Decimal>()
        / Decimal::from(rates.items.len().max(1));
    println!("{mean} mean rate over {} observations", rates.items.len());

    // Private: signed, and negative is what the account paid out.
    let paid = client
        .funding_payments(&HistoryRequest::new(market).limit(100))
        .await?;
    let net: Decimal = paid.items.iter().map(|payment| payment.amount).sum();
    println!("{net} net funding, more pages: {}", paid.has_more());

    Ok(())
}
```

## Subscriptions

| Call | Answers | Credentials |
| --- | --- | --- |
| `subscribe(&Subscription)` | a market data stream | no |
| `subscribe_with(&Subscription, &StreamConfig)` | the same, with the connection tuned | no |
| `subscribe_account()` | a balance and order stream | yes |
| `subscribe_account_with(&StreamConfig)` | the same, with the connection tuned | yes |

Build a `Subscription` with `Subscription::new()`, then `.market(..)` and
`.feed(..)` for each one you want, or `.markets_iter(..)` to add several markets
at once from anything iterable. Adding the same market or feed twice costs
nothing and changes nothing. Insertion order is kept, and it is the order the
exchange is asked in.

| Case | What happens |
| --- | --- |
| the markets and feeds it names | one connection, however many of each. It is a cross product: three markets and three feeds is nine streams over one socket |
| `Feed::Candles` at two intervals | two feeds, not a replacement |
| no market, or no feed | `Error::InvalidRequest` |
| a feed the exchange does not publish | `Error::Unsupported` for the whole subscription, before a socket is opened. It is not dropped from the list |
| dropping the stream | closes the connection |

```rust
use maxt::{Exchange, Feed, Interval, Market, Subscription};

fn majors() -> Subscription {
    let subscription = Subscription::new()
        .markets_iter(["BTC", "ETH", "XRP"].map(|base| Market::spot(Exchange::Upbit, base, "KRW")))
        .feed(Feed::Trades)
        .feed(Feed::OrderBook)
        .feed(Feed::Candles(Interval::Min1))
        // A different interval is a different feed.
        .feed(Feed::Candles(Interval::Hour1))
        // A repeat is free and changes nothing.
        .feed(Feed::Trades);

    assert_eq!(subscription.markets().len(), 3);
    assert_eq!(subscription.feeds().len(), 4);
    // Insertion order is what the exchange is asked in.
    assert_eq!(subscription.feeds()[0], Feed::Trades);
    subscription
}
```

## Stream configuration

`StreamConfig` decides how a live connection behaves when it degrades. It is an
ordinary struct with public fields, so set the ones that differ and inherit the
rest with `..StreamConfig::default()`.

| Field | Default | Notes |
| --- | --- | --- |
| `buffer_size` | 4096 events | how many events a consumer that has fallen behind may bank |
| `overflow` | `Overflow::Backpressure` | lose nothing |
| `max_reconnect_attempts` | `None` | retries forever; `Some(n)` gives up after `n` reconnects, whatever came of them |
| `initial_reconnect_delay_ms` | 1 000 | the first backoff, doubling on each failure; floored at 1 ms |
| `max_reconnect_delay_ms` | 30 000 | the ceiling that doubling stops at; floored at 1 ms |
| `idle_timeout_ms` | 30 000 | a floor each adapter may raise, see [Heartbeats](#heartbeats) |

The backoff doubles once per consecutive failure from
`initial_reconnect_delay_ms` and stops at `max_reconnect_delay_ms`: one second,
two, four, eight, on to thirty. A zero in either field is read as one
millisecond, because doubling from zero never leaves zero and a ceiling of zero
flattens every delay to nothing.

What resets the backoff to the first delay is a connection the exchange spoke
on, not a handshake that succeeded. **Nothing resets `max_reconnect_attempts`**:
it counts reconnects, not failures.

| A reconnect that | Counts against `max_reconnect_attempts` | Resets the backoff |
| --- | --- | --- |
| failed to open at all | yes | no |
| opened a socket the exchange then sent nothing on | yes | no |
| opened a socket that carried at least one frame | yes | yes |

The connection reads raw frames and parses none of them, so an exchange's
rejection of the subscription is a frame like its data. A bad symbol, a retired
stream name, a revoked credential on a private stream and an HTML error from a
gateway all arrive the same way, and the adapter decodes them one layer up. A
budget that any frame reset would put no bound at all on a venue that answers
every connection with one.

What the rule costs:

| Situation | What happens |
| --- | --- |
| a venue recycles working sockets on its own schedule | those reconnects spend the budget too, so a finite `Some(n)` ends a healthy stream eventually. It is for letting a process supervisor see a failure, not for surviving a venue's housekeeping |
| a venue rejects every connection, under the default `None` | it is reconnected to at `initial_reconnect_delay_ms` for as long as the process lives, because the rejection is a frame and resets the backoff. Reading it is the consumer's job; setting `max_reconnect_attempts` is what bounds it |
| an endpoint accepts connections and stays mute | it backs off to the ceiling, reports once three reconnects in a row have left no working connection, and is bounded like an unreachable endpoint |

**No delay field decides what counts.** Raising `max_reconnect_delay_ms` makes
the retries gentler and changes nothing else.

| `Overflow` | Behaviour | Right for |
| --- | --- | --- |
| `Backpressure` (default) | stop reading the socket until the consumer catches up | anything that must not lose an event; the exchange may disconnect a consumer that stalls too long |
| `DropNewest` | discard arriving events while the buffer is full | tickers and book snapshots, where the next event on that feed restates the whole of what the dropped one said |
| no drop-the-oldest | the sending side cannot evict from the front of a full queue. For newest-wins, drain the stream in a tight loop and keep only the last event you saw | nothing; the policy does not exist |

The question to ask of a feed is not whether one event matters on its own, but
whether a later event on that same feed restates it.

| Feed | What `DropNewest` costs |
| --- | --- |
| `Feed::Ticker`, `Feed::OrderBook` | staleness, and nothing else. Every event is a complete current value, and the next one restates it |
| `Feed::Trades` | a total that is silently short. Each trade is a distinct fact no later event repeats |
| `Feed::Candles` | the bar itself. A window gets [exactly one emission](#candleclosed) with `Candle::closed` set, carrying that window's own final figures, and everything after it belongs to the next window. Nothing restates it, so the series is short one bar wherever it is stored |

**A dropped event leaves no trace, and errors are dropped the same way.**
`DropNewest` discards silently, with no counter, no stream event and no log line,
and it applies to everything the connection delivers but one. A reconnect failure
worth reporting, and the transport error raised when `max_reconnect_attempts`
runs out, both go through the overflow policy, so a `DropNewest` stream with a
full buffer can end having reported nothing at all, leaving `None` as the only
signal that it is over. A consumer that asked never to be waited on is not waited
on for the failure that ends its stream either (`src/transport/ws.rs`). Size
`buffer_size` so that stays hypothetical.

**The exception is the news of a reconnect.** `MarketEvent::Reconnected` and
`AccountEvent::Reconnected` are held rather than discarded when the buffer is
full, and delivered ahead of the first event that finds room. Nothing waits: the
connection goes on reading, and goes on dropping data, while it waits for that
room. A consumer that never heard of a gap would carry on trusting a book and a
balance the gap invalidated, and no later event corrects that, which is what
separates this notice from the data around it.

## Reconnects

`MarketEvent::Reconnected` means the connection dropped, was re-established, and
the subscription was restored. Everything the exchange published in between was
missed, and `maxt` will not guess what was in the gap.

| Feed | What the gap costs |
| --- | --- |
| `Feed::OrderBook` | nothing. Every book event from every adapter is a self-contained snapshot, several levels deep on each side, never a diff. No sequence number to track, no local book to rebuild, no resynchronization step. Overwrite your copy on each event and a reconnect costs one missed message. How many levels each feed carries is on its provider page |
| `Feed::Trades` | a real hole. Trades are the sequence, and the ones published during the gap are not repeated. Backfill over REST with `trades` and deduplicate the overlap on `Trade::id` where the exchange fills it in. On Binance that is exact: `@trade` and `/trades` name the same fill with the same id on both venues. Hyperliquid serves only the last ten, so a wider gap stays a hole there |

Restoring the subscription includes re-authenticating it where the exchange
requires that, so a private stream survives a reconnect without the caller
resubscribing. How each exchange authenticates a socket, and why the token is
signed fresh every handshake, is on
[Upbit](providers/upbit.md#credentials) and
[Bithumb](providers/bithumb.md#credentials).

`AccountEvent::Reconnected` is heavier than either market gap. Fills may have
happened while the socket was down, so a local view of balances and open orders
is now a guess. Re-read both over REST. Neither notice can be lost to a full
buffer, whatever `overflow` is set to, so the instruction to re-read reaches even
a consumer that is dropping everything else.

## Stream termination

Only `None` ends a stream. An `Err` item is a report the stream polls past: a
frame that could not be read, and a reconnect that has stopped looking transient,
are both delivered as errors while the subscription goes on running, so a
consumer that breaks out of its loop on the first `Err` abandons a stream that
was about to recover. `None` arrives after
`StreamConfig::max_reconnect_attempts` runs out, or once the stream is dropped.
Both `MarketStream` and `AccountStream` behave this way.

```rust
use futures_util::StreamExt;
use maxt::adapters::UpbitAdapter;
use maxt::{Client, MarketEvent, Subscription};

async fn watch(client: &Client<UpbitAdapter>, subscription: &Subscription) -> maxt::Result<()> {
    let mut stream = client.subscribe(subscription).await?;

    // `while let Some(..)`. Using `?` on the item would end the loop on a
    // report the stream was going to carry on past.
    while let Some(item) = stream.next().await {
        match item {
            Ok(MarketEvent::Trade(trade)) => println!("{} {}", trade.price, trade.quantity),
            Ok(other) => println!("{other:?}"),
            Err(error) => eprintln!("reported, still subscribed: {error}"),
        }
    }

    Ok(()) // the stream said `None`
}
```

## Heartbeats

`idle_timeout_ms` closes and reconnects a socket that has said nothing for that
long. It measures **inbound silence only**: the timer is re-armed when a frame
arrives from the exchange and again once that frame has reached the consumer, so
a consumer slow enough to hold up `Overflow::Backpressure` does not cause a
disconnect, and `maxt`'s own outbound keepalives never push the deadline out. A
socket that stopped answering cannot be kept alive by writes alone.

Each adapter sends its exchange's own keepalive on an interval, producing the
inbound traffic the idle timer watches for, and raises `idle_timeout_ms` to a
floor its exchange's pace can meet. Ask for longer than a floor and you get what
you asked for; ask for less and you get the floor.

| Exchange | Keepalive interval | Frame sent | Idle floor | Why that frame |
| --- | --- | --- | --- | --- |
| Upbit | 15s | text `PING` | 60s | reads every text frame as a command and answers this one, so the keepalive resets Upbit's own timer and comes back as inbound traffic |
| Bithumb | 15s | text `PING` | 60s | the same |
| Hyperliquid | 15s | `{"method":"ping"}` | 60s | the same |
| Binance | 60s | a protocol ping | 240s | answers an unknown text frame with an error, so the keepalive lives below the API. Its server ping arrives every three minutes and it hangs up only after ten with no pong, so three minutes of silence is a healthy socket there. That is why there is a floor at all: the 30-second default would reconnect a working connection |

## `Feature` and `Client::supports`

`client.supports(feature)` answers for the adapter as configured, credentials
included, locally and with no request. Ask when the answer should change what
your program does: it costs no round trip and no rate-limit token. Catch
`Error::Unsupported` when the answer would end the program anyway.

| `Feature` | Gates | Credentials | Derivatives only |
| --- | --- | --- | --- |
| `Markets` | `markets` | no | no |
| `Trades` | `trades` | no | no |
| `OrderBook` | `order_book` | no | no |
| `Ticker` | `ticker` | no | no |
| `Candles` | `candles` | no | no |
| `TradeStream` | `Feed::Trades` in a `subscribe` | no | no |
| `OrderBookStream` | `Feed::OrderBook` | no | no |
| `TickerStream` | `Feed::Ticker` | no | no |
| `CandleStream` | `Feed::Candles(_)` | no | no |
| `Balances` | `balances` | yes | no |
| `OpenOrders` | `open_orders`, `open_orders_on` | yes | no |
| `AccountStream` | `subscribe_account`, `subscribe_account_with` | yes | no |
| `Trading` | `place_order`, `cancel_order` | yes | no |
| `Positions` | `positions`, `positions_on` | yes | yes |
| `Margin` | `margin_summary` | yes | yes |
| `FundingRates` | `funding_rates` | no | yes |
| `FundingPayments` | `funding_payments` | yes | yes |
| `MarginConfig` | `set_margin` | yes | yes |
| `ReduceOnlyOrders` | `OrderRequest::reduce_only` on a `place_order` | yes | yes |

`ReduceOnlyOrders` gates a field, not a call. A `place_order` whose request was
built with `.reduce_only()` needs it; the same call without that field needs only
`Trading`.

`Feature::needs_credentials()` and `Feature::is_derivatives_only()` are const and
answer without an adapter at all. `Feature` is `#[non_exhaustive]`, so match on
it with a `_` arm.

### A `true` still has to be checked at the call

**`supports` answers for the feature, not for the argument you pass it.**

| Answer | How far the test behind it goes |
| --- | --- |
| `false` | worth trusting. [`tests/unsupported_is_honest.rs`](../tests/unsupported_is_honest.rs) asserts over the whole feature by adapter-configuration cross product that a declined feature refuses as `Unsupported` naming that same feature, never as a transport error, an auth error, or a success. That is a feature declined wholesale, which is what a `false` is |
| `true` | narrower than it looks. The same test checks that a claimed feature never answers `Unsupported`, but it can only make the call at one representative argument, so a feature an adapter carries for some arguments and not others still reads `true` |

Three shipped cases:

| Reads `true` | Still refuses `Unsupported` |
| --- | --- |
| `Feature::Candles` | at an interval outside the exchange's REST set: `Interval::Sec1` on Binance USD-M, for one |
| `Feature::CandleStream` | at an interval the exchange serves over REST but does not stream. On Upbit, `Feed::Candles` at `Day1`, `Week1` or `Month1`, which its own test `a_candle_interval_upbit_does_not_stream_is_refused` asserts |
| `Feature::FundingRates`, `Feature::ReduceOnlyOrders` and the rest of the derivatives half on Hyperliquid | when the market handed in is a spot one. One Hyperliquid adapter lists both kinds, so the feature is carried by the adapter and not by every market on it |

So a router that reads `supports(Feature::CandleStream)` and sends Upbit daily
candles to a subscription builds one that dies at `subscribe`. Branch on the
feature to pick an exchange, then handle `Error::Unsupported` at the call anyway.
The REST and stream interval sets are per-exchange, stated on each provider page,
and no `Feature` distinguishes them.

## `Error`

| Variant | Means | Retry the same request? |
| --- | --- | --- |
| `InvalidRequest { field, detail }` | you asked wrong; rejected before it left the process | never |
| `Unsupported { feature, exchange, detail }` | `maxt` maps no endpoint there; a key will not change it | never |
| `Auth { detail }` | `maxt` could not build a credentialed request, so none was sent: missing, malformed, or unusable for signing | never |
| `Exchange { exchange, code, message, status, kind }` | the exchange answered and refused, keeping its own code and message verbatim. A credential *it* rejected is here, not in `Auth` | depends on `kind` |
| `Transport { detail }` | the connection failed: DNS, TLS, socket, timeout | yes |
| `Decode { detail }` | the exchange answered with a payload `maxt` could not read | no; report it as a bug |

| Predicate | True for |
| --- | --- |
| `is_retryable()` | the last column above, folded into one call |
| `is_rate_limited()` | the exchange's own "too many requests" verdict, which asks for a longer pause than a transport blip does |

```rust
use maxt::{Client, Error, Market, adapters::HyperliquidAdapter};

async fn print_last_price(client: &Client<HyperliquidAdapter>, market: &Market) {
    match client.ticker(market).await {
        Ok(ticker) => println!("{}", ticker.last_price),
        Err(error) if error.is_rate_limited() => println!("slow down: {error}"),
        Err(error) if error.is_retryable() => println!("try again behind a backoff: {error}"),
        Err(Error::Unsupported { feature, exchange, .. }) => {
            println!("{exchange} does not offer {feature}; nothing to retry")
        }
        Err(error) => println!("give up: {error}"),
    }
}
```

Missing credentials are `Error::Auth` on every adapter, not `Error::Unsupported`.
`Auth` means the endpoint exists and the key does not, so supplying a key
resolves it; `Unsupported` means `maxt` maps no endpoint there, which a key
will not change. An adapter
built without credentials reports `false` from
`client.supports(Feature::Balances)` and fails the call as `Auth` before anything
reaches the network.

### A credential the exchange refused

`Auth` is drawn at the process boundary, not at the credential. A key `maxt`
sent and the exchange read and refused comes back as `Error::Exchange`, under
that exchange's own code:

| What you did | What comes back |
| --- | --- |
| built an adapter with no credentials | `Error::Auth`, before anything is sent |
| supplied a key `maxt` cannot sign with | `Error::Auth`, before anything is sent |
| supplied a wrong or revoked key | `Error::Exchange`, carrying the exchange's code |

`maxt` does not fold the third row into the second, because the four exchanges
disagree about how a refused credential is even spelled:

| Exchange | A refused credential | Standing |
| --- | --- | --- |
| Binance | HTTP 400 `-1022` for a bad signature, HTTP 401 `-2015` for a bad key, HTTP 401 `-2014` for no key | measured 2026-07-31 |
| Upbit | HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_access_key`, `nonce_used`, `no_authorization_ip`, `no_authorization_token`; HTTP 403 `out_of_scope` | from Upbit's published table |
| Bithumb | HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_jwt`, `NotAllowIP`, `out_of_scope` | from Bithumb's published table |
| Hyperliquid | HTTP 200, `status: "err"`, an English sentence naming an address that changes per request, no code at all | from Hyperliquid's published signing guide |

A rule written on HTTP status alone would call Binance's bad signature a
rejection and its bad key an auth failure, and would never fire for Hyperliquid
at all. A rule written on codes would be four lists, three of them unverifiable
from here, going quietly stale the first time an exchange renamed one. So the
code reaches you intact and the branch is yours. Each provider page lists its
own under **Surprises**.

### `is_retryable()` and a clock outside the receive window

`is_retryable()` asks whether sending the *identical* request again could
succeed. A signed request carries the timestamp it was signed with, so a
timestamp the exchange refused as outside its receive window is `false` here,
even though clock drift is transient.

| Cause | What resolves it |
| --- | --- |
| the machine's clock is wrong | fix the clock. Every rebuild fails until you do, and a retry loop spends its whole budget learning that |
| the request was delayed in flight | build the request again and send it once. `maxt` reads the clock when it builds, so the new one carries a fresh timestamp |

Neither is a retry loop, which is what `is_retryable()` exists to authorise.

## `Decimal`

| Aspect | What it means |
| --- | --- |
| Every price, quantity, and amount | a `rust_decimal::Decimal`, re-exported as `maxt::Decimal` so a caller cannot end up on a different version of it by accident |
| A number `Decimal` cannot hold exactly | `Error::Decode`, never a rounded value. That covers ordinary over-precision and the exponent form exchanges switch into for small values alike: `1e-30` is `Error::Decode`, not a silent zero, and an over-precise `1.234…e5` is `Error::Decode` too. `Decimal::from_scientific` would have rounded both |
| Where the rule lives | one shared reader every adapter goes through, so it holds identically on all four (`src/adapters/decimal.rs`) |

`f64` cannot represent most decimal prices exactly. Summing fills, comparing a
limit price against a book level, or checking a balance against an order size in
binary floating point accumulates error, and that error shows up as a rejected
order or a position that does not reconcile. Convert at the edge if a calculation
genuinely needs floating point, not on the way in.

## `Timestamp`

| Aspect | What it means |
| --- | --- |
| Unit | nanoseconds since the Unix epoch, UTC |
| Why one unit at all | exchanges publish seconds, milliseconds, microseconds, and nanoseconds, and converting them to one resolution is what lets events from two exchanges be ordered against each other |
| `Display` | RFC 3339 in UTC to millisecond precision, so `Timestamp::from_millis(1_700_000_000_000)` prints as `2023-11-14T22:13:20.000Z` |
| Round-tripping | the sub-millisecond part is not printed, so use `as_nanos` whenever the exact value matters |
| Not a `chrono` or `time` type | `maxt` does not force a date-time library on you. Convert at the edge with `as_nanos` or `into_system_time` |
| A field that would have to be invented to be filled in | `None` instead. `Ticker::last_trade_time` is `None` on Binance and Hyperliquid because neither says when the last price traded |

Some payloads carry the read time instead of an exchange clock, because the
exchange publishes none. A Binance spot order book and a Hyperliquid ticker are
both like this, and each field says so in its own documentation. Treat such a
timestamp as an upper bound on the data's age; measuring staleness against it
will under-report.

## `Market` and `MarketKind`

A `Market` is an exchange, a kind, a base asset, and a quote asset, with the
assets uppercased for you.

| Item | What it does |
| --- | --- |
| `Market::spot(exchange, base, quote)` | a spot market |
| `Market::perpetual(exchange, base, quote)` | a perpetual futures market |
| `Market::new(exchange, kind, base, quote)` | a market of an explicit kind, for when the kind is itself a runtime value |
| `Display` | `binance:BTC/USDT`, and `binance:BTC/USDT:perp` for the perpetual |
| `Hash`, `Ord` | so it works as a map key without a wrapper |
| `MarketKind::is_derivative()` | what the derivatives half of `Client` is meaningful for |

Prefer the first two. `Market::new` is the one to use when the kind arrives from
configuration or a command line rather than from the source, which is the same
situation that makes a `Client<Box<dyn Adapter>>` worth the loss of the typed
adapter methods.

```rust
use maxt::{Exchange, Market, MarketKind};

fn market_from_config(kind: &str) -> Option<Market> {
    let kind = match kind {
        "spot" => MarketKind::Spot,
        "perp" => MarketKind::Perpetual,
        _ => return None,
    };
    // Lowercase in, uppercase out: the constructor normalizes the assets.
    Some(Market::new(Exchange::Binance, kind, "btc", "usdt"))
}

fn check() {
    let perp = market_from_config("perp").expect("a perpetual");
    assert_eq!(perp.to_string(), "binance:BTC/USDT:perp");
    // `new` with a spot kind builds exactly what `spot` builds.
    assert_eq!(
        market_from_config("spot"),
        Some(Market::spot(Exchange::Binance, "BTC", "USDT"))
    );
    assert_eq!(market_from_config("futures"), None);
}
```

Spot and perpetual on the same pair are different markets, not one market with a
flag. They have different prices, different books, and on Binance different hosts
and different balances. Making the kind part of the identity stops a perpetual
quantity being compared against a spot price.

## `Page` and `Cursor`

Funding rate and funding payment history arrive one page at a time.

| Item | What it does |
| --- | --- |
| `Page::next` | `Some(cursor)` while there is more, `None` at the end of the history |
| `Page::has_more()` | the same question as a `bool` |
| `Cursor` | opaque. The exchange produces it and only that exchange can read it, so pass it back unchanged and do not parse it |
| `Cursor::as_str()` | the contents, for persisting a position between runs |
| `Cursor::new(string)` | the way back: wraps a saved string into a cursor again |
| `HistoryRequest::cursor(cursor)` | resumes the walk from one |
| `HistoryRequest::limit` | a page size, not a total. Each adapter caps it and may default it, and the provider pages state each |

| Page length | What to expect |
| --- | --- |
| a short or empty page | not a signal. A page can be short for other reasons, so only an absent cursor ends the walk. Stopping on a short page truncates the history |
| an unset `limit` | Binance defaults it to 100. Hyperliquid takes no size at all, so it reads 500 at a time and trims to your `limit` |
| **longer than `limit`** | possible on Hyperliquid. The next cursor resumes one millisecond past the last entry kept, so a cut landing inside a run of entries sharing one millisecond would strand the rest of that run. The trim backs up to the start of the run, and when the run reaches the front of the page there is nothing to back up to, so the whole run is kept (`src/adapters/hyperliquid/rest.rs`). Extra entries a caller can drop beat entries the cursor has already moved past |

Size buffers off what a page returns, not off `limit`.

```rust
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Cursor, Exchange, HistoryRequest, Market, Timestamp};

/// Every funding rate from `start` to the present.
///
/// `from` is what makes this the whole history. The cursor walks *forward*, so
/// with `from` unset Binance answers with the newest page, the first cursor
/// already points past the end of the history, and the walk finishes two round
/// trips later having read only that page.
async fn funding_rates_since(start: Timestamp) -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::usd_m_futures());
    let mut request = HistoryRequest::new(Market::perpetual(Exchange::Binance, "BTC", "USDT"))
        .from(start)
        .limit(100);

    loop {
        let page = client.funding_rates(&request).await?;
        for rate in &page.items {
            println!("{} {}", rate.timestamp, rate.rate);
        }

        let Some(cursor) = page.next else { break };
        request = request.cursor(cursor);
    }

    Ok(())
}

/// The same walk across two runs of the process.
///
/// `as_str` gets the position out, `Cursor::new` gets it back in. Store the
/// string and nothing about its shape: one exchange's cursor is a timestamp,
/// another's is an order id, and neither is a promise.
async fn resume(client: &Client<BinanceAdapter>, saved: Option<String>) -> maxt::Result<Option<String>> {
    let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
    let mut request = HistoryRequest::new(market).limit(100);
    if let Some(saved) = saved {
        request = request.cursor(Cursor::new(saved));
    }

    let page = client.funding_payments(&request).await?;
    println!("{} entries, more to come: {}", page.items.len(), page.has_more());

    Ok(page.next.map(|cursor| cursor.as_str().to_string()))
}
```

## Rate limits

`maxt` does not throttle: no client-side limiter, no request queue, no automatic
backoff. A call goes out when you make it, and pacing is yours.

| Exchange | Published limit | Where it is stated |
| --- | --- | --- |
| Upbit | 10 requests/second per IP for public quotation; 30/second per account for private reads, 8/second for orders | [Rate limits](https://global-docs.upbit.com/reference/rate-limits) |
| Bithumb | 150 requests/second public, 140/second private, with orders additionally held to 10/second | [API 요청 수 제한 안내](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내) |
| Binance | a request-weight budget per IP per interval, not a request count, with each endpoint carrying its own weight. The documentation states no fixed ceiling: the current one is in the `rateLimits` array of `/api/v3/exchangeInfo` on spot and `/fapi/v1/exchangeInfo` on USD-M | [Spot REST limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits), [USD-M REST limits](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info) |
| Hyperliquid | 1,200 weight per minute per IP, plus a per-address budget of one request per USDC traded since the address was created, starting at 10,000. **The address budget counts actions only**, so of `maxt`'s calls it charges `place_order`, `cancel_order` and `set_margin`, and nothing else | [Rate limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits) |

| Pacing it yourself | What to know |
| --- | --- |
| Weight is not a request count | Binance's and Hyperliquid's figures are weight, so what a program can spend depends on which endpoints it calls. Both publish the per-endpoint weight |
| Reading the running total | Binance returns it on every response in an `X-MBX-USED-WEIGHT-(intervalNum)(intervalLetter)` header, `X-MBX-USED-WEIGHT-1M` for the one-minute limiter. `maxt` does not read it, so a program pacing against the budget reads it itself |
| Bithumb's scope | unstated. Treat the limits as per IP, the stricter reading |
| A backoff for all four | on `is_rate_limited()`, sleep and retry with the delay doubling each time, capped. Binance bans an IP that keeps ignoring a `429` |
| Batching where it exists | one Upbit request for thirty tickers costs one of ten per second instead of thirty, through [`Client::adapter`](#clientadapter). `maxt` caps the length of that market list nowhere and Upbit publishes no cap either, so a list long enough to make the URL unwieldy is refused by Upbit or by something in front of it and arrives as `Error::Exchange`. Find your own working ceiling; see [Upbit](providers/upbit.md#upbit-only-calls) |

## `Client::adapter`

`Client::adapter()` hands back the adapter, with whatever typed methods that
exchange has beyond the common API. This is the escape hatch, and
`Client::into_adapter()` unwraps back to the adapter itself.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Market};

// One request instead of thirty. Upbit answers for many markets at once, and
// its quota is counted per request.
async fn tickers(client: &Client<UpbitAdapter>, markets: &[Market]) -> maxt::Result<()> {
    println!("{} tickers", client.adapter().tickers(markets).await?.len());
    Ok(())
}
```

Both need the concrete adapter type. A `Client<Box<dyn Adapter>>` gives back
`&Box<dyn Adapter>`, which has the common API and nothing more. That is the cost
of choosing the exchange at runtime.

What each exchange keeps on its own adapter, and why, is on its provider page:
[Upbit](providers/upbit.md), [Bithumb](providers/bithumb.md),
[Binance](providers/binance.md), [Hyperliquid](providers/hyperliquid.md).
