//! Bithumb's JSON, read into `maxt` types.
//!
//! Bithumb's v1 API is shaped like Upbit's: market codes read `QUOTE-BASE`,
//! and money arrives as bare JSON numbers, and sometimes as strings on the
//! private endpoints. Every money field is read out of the raw JSON text,
//! never through `f64`, so the digits Bithumb sent are the digits a caller
//! sees.
//!
//! Timestamps arrive as Unix milliseconds everywhere except the public
//! `orderbook` frame, which Bithumb
//! [documents](https://apidocs.bithumb.com/reference/호가-orderbook.md) and
//! sends as microseconds. Each field is read at its own documented scale by
//! [`millis`] or [`micros`]; see [`epoch`] for why the scale is not inferred
//! from the value.

use rust_decimal::Decimal;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::types::{
    AccountEvent, Balance, Candle, Exchange, Interval, Level, Market, MarketEvent, MarketInfo,
    MarketKind, MarketStatus, Order, OrderBook, OrderStatus, Side, Ticker, Timestamp, Trade,
};

use super::{BithumbAlertStep, BithumbMarketAlert};

/// The identifier carried in every error this adapter raises.
pub(crate) const EXCHANGE: &str = Exchange::Bithumb.id();

/// The exchange's own code for a market, for example `KRW-BTC`.
///
/// Bithumb names the quote asset first, the opposite of most exchanges.
pub(crate) fn native_symbol(market: &Market) -> Result<String> {
    if market.exchange != Exchange::Bithumb {
        return Err(Error::invalid_request(
            "market.exchange",
            format!("expected a Bithumb market, got {}", market.exchange),
        ));
    }
    if market.kind != MarketKind::Spot {
        return Err(Error::invalid_request(
            "market.kind",
            "Bithumb lists spot markets only",
        ));
    }
    asset("market.quote", &market.quote)?;
    asset("market.base", &market.base)?;

    Ok(format!("{}-{}", market.quote, market.base))
}

/// The market a Bithumb response names.
///
/// A code this adapter cannot read in a *response* is a response `maxt` does
/// not understand, so this reports [`Error::Decode`] where [`native_symbol`]
/// reports [`Error::InvalidRequest`].
fn market_field(value: &Value, name: &'static str) -> Result<Market> {
    let symbol = text(value, name)?;
    split_symbol(symbol)
        .ok_or_else(|| Error::decode(format!("`{name}` is not a market code: `{symbol}`")))
}

fn split_symbol(symbol: &str) -> Option<Market> {
    let (quote, base) = symbol.split_once('-')?;
    let market = Market::spot(Exchange::Bithumb, base, quote);
    // Round-trip rather than trust the split: it is what rejects lowercase,
    // empty, and punctuation-bearing halves in one place instead of three.
    native_symbol(&market).ok().filter(|code| code == symbol)?;
    Some(market)
}

fn asset(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(Error::invalid_request(
            field,
            format!("`{value}` is not a Bithumb asset code"),
        ));
    }
    Ok(())
}

/// Turns a non-2xx REST body into the exchange's own verdict.
///
/// Bithumb reports failures as `{"error":{"name":..,"message":..}}`. A body
/// that does not fit that shape is still surfaced verbatim, which is when a
/// caller most wants to see what arrived.
pub(crate) fn exchange_error(status: u16, body: &str) -> Error {
    match serde_json::from_str::<Value>(body)
        .ok()
        .as_ref()
        .and_then(|value| value.get("error").cloned())
    {
        Some(error) => Error::exchange_http(
            EXCHANGE,
            status,
            error
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("bithumb_error"),
            error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or(body)
                .trim()
                .to_string(),
        ),
        None => Error::exchange_http(EXCHANGE, status, "bithumb_error", body.trim().to_string()),
    }
}

/// Reads a response body, refusing anything that is not JSON.
pub(crate) fn body(raw: &str) -> Result<Value> {
    serde_json::from_str(raw).map_err(|err| Error::decode(format!("response is not JSON: {err}")))
}

fn entries(value: &Value) -> Result<&Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| Error::decode("expected a JSON array"))
}

fn field<'a>(value: &'a Value, name: &'static str) -> Result<&'a Value> {
    value
        .get(name)
        .filter(|found| !found.is_null())
        .ok_or_else(|| Error::decode(format!("missing `{name}`")))
}

fn text<'a>(value: &'a Value, name: &'static str) -> Result<&'a str> {
    field(value, name)?
        .as_str()
        .ok_or_else(|| Error::decode(format!("`{name}` is not a string")))
}

/// Reads a money field, from a JSON number or a JSON string.
///
/// `serde_json` is configured to keep numbers as their original digits, so the
/// text below is exactly what Bithumb sent.
pub(crate) fn dec(value: &Value, name: &'static str) -> Result<Decimal> {
    match field(value, name)? {
        Value::Number(number) => decimal(&number.to_string(), name),
        Value::String(raw) => decimal(raw, name),
        _ => Err(Error::decode(format!("`{name}` is not a number"))),
    }
}

fn dec_opt(value: &Value, name: &'static str) -> Result<Option<Decimal>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => dec(value, name).map(Some),
    }
}

/// Parses an exact decimal, rejecting anything that would need rounding.
///
/// A price Bithumb sent that `Decimal` cannot hold exactly is a decode
/// failure. Rounding it would silently drop the last digit of a price.
pub(crate) fn decimal(raw: &str, name: &'static str) -> Result<Decimal> {
    // Bithumb prints small sizes with an exponent;
    // [`crate::adapters::decimal::exact`] reads that spelling without relaxing
    // the no-rounding rule above, and is what every adapter uses.
    crate::adapters::decimal::exact(raw)
        .map_err(|err| Error::decode(format!("`{name}` is not an exact decimal `{raw}`: {err}")))
}

/// Reads one of Bithumb's millisecond timestamps, which is all of them bar one.
pub(crate) fn millis(value: &Value, name: &'static str) -> Result<Timestamp> {
    epoch(value, name, 1_000_000, "millisecond")
}

/// Reads the one microsecond timestamp Bithumb publishes.
///
/// Only the public `orderbook` frame uses this scale. Bithumb's field table
/// calls it `타임스탬프 (microseconds)` where the `trade` and `ticker` frames
/// beside it say `milliseconds`, and a capture on 2026-07-30 agrees: a book
/// frame stamped `1785397747576054` and a ticker frame stamped
/// `1785397735276` name the same second.
fn micros(value: &Value, name: &'static str) -> Result<Timestamp> {
    epoch(value, name, 1_000, "microsecond")
}

/// Reads a Unix timestamp at the scale its field is documented to use.
///
/// The scale is declared per field rather than inferred from the magnitude.
/// Bithumb states a unit on every timestamp it publishes and those units
/// differ between two frames of the same socket, so a value that stops
/// matching its documented unit means the response shape changed. Sizing the
/// value at read time would absorb exactly that change in silence, which is
/// how a book feed can decode into the year 58545 with a green test suite.
///
/// [`EARLIEST`] is what makes a wrong scale loud in both directions. Reading
/// microseconds as milliseconds overflows the multiply; reading milliseconds
/// as microseconds does not, and would land in 1970 without the floor.
fn epoch(
    value: &Value,
    name: &'static str,
    nanos_per_unit: i64,
    unit: &'static str,
) -> Result<Timestamp> {
    /// 2001-09-09T01:46:40Z. Older than every exchange `maxt` speaks to, and
    /// still a thousandfold away from a present-day instant read one scale
    /// too small.
    const EARLIEST: i64 = 1_000_000_000_000_000_000;

    let raw = field(value, name)?
        .as_i64()
        .ok_or_else(|| Error::decode(format!("`{name}` is not a whole-number timestamp")))?;

    raw.checked_mul(nanos_per_unit)
        .filter(|nanos| *nanos >= EARLIEST)
        .map(Timestamp::from_nanos)
        .ok_or_else(|| Error::decode(format!("`{name}` is not a {unit} timestamp: {raw}")))
}

fn millis_opt(value: &Value, name: &'static str) -> Result<Option<Timestamp>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => millis(value, name).map(Some),
    }
}

/// `BID` means the taker lifted an ask, which is a buy in `maxt` terms.
///
/// Bithumb spells the same distinction three ways depending on the endpoint:
/// `BID`/`ASK` on public data, `bid`/`ask` on private REST, `buy`/`sell` on the
/// private stream.
fn side(value: &Value, name: &'static str) -> Result<Side> {
    match text(value, name)? {
        "BID" | "bid" | "buy" => Ok(Side::Buy),
        "ASK" | "ask" | "sell" => Ok(Side::Sell),
        other => Err(Error::decode(format!(
            "`{name}` is neither side: `{other}`"
        ))),
    }
}

/// Bithumb's own identifier for a trade, kept verbatim.
///
/// It arrives as a large integer that outruns an `f64` mantissa, so it is
/// carried as the digits Bithumb sent, never through a numeric type.
fn trade_id(value: &Value) -> Option<String> {
    match value.get("sequential_id")? {
        Value::Number(number) => Some(number.to_string()),
        Value::String(raw) => Some(raw.clone()),
        _ => None,
    }
}

/// Reads `/v1/market/all`.
///
/// `market_warning` carries Bithumb's investment-warning designation, 유의
/// 종목, which the common [`MarketStatus`] has no room for; the label itself is
/// reachable through
/// [`BithumbAdapter::market_warnings`](super::BithumbAdapter::market_warnings).
/// Bithumb's other designation, 주의 종목, is absent from this payload
/// altogether and is read by [`market_alerts`].
pub(crate) fn markets(value: &Value) -> Result<Vec<MarketInfo>> {
    entries(value)?
        .iter()
        .map(|entry| {
            Ok(MarketInfo {
                market: market_field(entry, "market")?,
                native_symbol: text(entry, "market")?.to_string(),
                // A warned market still trades, so it is not `Paused`; it is
                // not plainly healthy either, so it is not `Active`. Upbit puts
                // its own warning here too, which is what makes the two Korean
                // adapters agree.
                status: match entry.get("market_warning").and_then(Value::as_str) {
                    None | Some("NONE") => MarketStatus::Active,
                    Some(_) => MarketStatus::Unknown,
                },
                korean_name: entry
                    .get("korean_name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                english_name: entry
                    .get("english_name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect()
}

/// Reads `/v1/market/all` for the warning label the common API drops.
///
/// The label is `CAUTION` or `NONE`, spellings that describe 유의 종목 despite
/// the first one reading like the other designation's name.
pub(crate) fn market_warnings(value: &Value) -> Result<Vec<(Market, String)>> {
    entries(value)?
        .iter()
        .map(|entry| {
            Ok((
                market_field(entry, "market")?,
                entry
                    .get("market_warning")
                    .and_then(Value::as_str)
                    .unwrap_or("NONE")
                    .to_string(),
            ))
        })
        .collect()
}

/// Reads `/v1/market/virtual_asset_warning`, Bithumb's 경보제.
///
/// One row is one raised alert, so a market flagged on several criteria is
/// listed once per criterion and a market flagged on none is absent. The order
/// Bithumb sends is kept: it groups a market's rows together but ranks nothing.
pub(crate) fn market_alerts(value: &Value) -> Result<Vec<(Market, BithumbMarketAlert)>> {
    entries(value)?
        .iter()
        .map(|entry| {
            Ok((
                market_field(entry, "market")?,
                BithumbMarketAlert {
                    kind: text(entry, "warning_type")?.to_string(),
                    step: match text(entry, "warning_step")? {
                        "CAUTION" => BithumbAlertStep::Caution,
                        "WARNING" => BithumbAlertStep::Warning,
                        "DANGER" => BithumbAlertStep::Danger,
                        _ => BithumbAlertStep::Unknown,
                    },
                    ends_at: alert_end(text(entry, "end_date")?)?,
                },
            ))
        })
        .collect()
}

/// Bithumb dates an alert's end on a Korean wall clock carrying no zone marker.
///
/// Korea has kept a fixed nine-hour offset with no daylight saving since 1988,
/// so the shift is a constant rather than a zone lookup, the same reasoning
/// [`next_open`] runs on. Reading the string as UTC would put every expiry nine
/// hours late, which for an alert lapsing this evening is the difference
/// between live and finished.
fn alert_end(raw: &str) -> Result<Timestamp> {
    const KST_OFFSET_SECS: i64 = 9 * 3_600;

    chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .map(|naive| {
            Timestamp::from_secs(naive.and_utc().timestamp().saturating_sub(KST_OFFSET_SECS))
        })
        .map_err(|err| Error::decode(format!("`end_date` is not a Korean wall clock: {err}")))
}

/// Reads `/v1/trades/ticks`, newest first.
///
/// Bithumb already answers newest-first, walking backwards from `to` the same
/// way Upbit's twin of this endpoint does, so the sort changes nothing today.
/// It keeps [`Client::trades`](crate::Client::trades) holding its promise as a
/// property of this adapter. The sort is stable, so trades sharing one
/// millisecond keep the order Bithumb ranked them in.
pub(crate) fn trades(value: &Value) -> Result<Vec<Trade>> {
    let mut trades = entries(value)?
        .iter()
        .map(|entry| {
            Ok(Trade {
                market: market_field(entry, "market")?,
                timestamp: millis(entry, "timestamp")?,
                price: dec(entry, "trade_price")?,
                quantity: dec(entry, "trade_volume")?,
                taker_side: side(entry, "ask_bid")?,
                id: trade_id(entry),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    trades.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));

    Ok(trades)
}

/// Reads one entry of `/v1/orderbook`, or one public `orderbook` frame.
///
/// Bithumb pairs each bid with an ask inside one `orderbook_units` entry and
/// does not promise an order across them, so both sides are sorted here rather
/// than assumed.
pub(crate) fn order_book(
    entry: &Value,
    market: Market,
    timestamp: Timestamp,
    depth: Option<u32>,
) -> Result<OrderBook> {
    let units = entries(field(entry, "orderbook_units")?)?;
    let mut bids = Vec::with_capacity(units.len());
    let mut asks = Vec::with_capacity(units.len());

    for unit in units {
        bids.push(Level {
            price: dec(unit, "bid_price")?,
            quantity: dec(unit, "bid_size")?,
        });
        asks.push(Level {
            price: dec(unit, "ask_price")?,
            quantity: dec(unit, "ask_size")?,
        });
    }

    bids.sort_by(|left, right| right.price.cmp(&left.price));
    asks.sort_by(|left, right| left.price.cmp(&right.price));

    // Truncating after the sort is what makes `depth` mean "the best levels"
    // rather than "the levels Bithumb happened to list first".
    if let Some(depth) = depth {
        let depth = usize::try_from(depth).unwrap_or(usize::MAX);
        bids.truncate(depth);
        asks.truncate(depth);
    }

    Ok(OrderBook {
        market,
        timestamp,
        bids,
        asks,
    })
}

/// Reads one entry of `/v1/ticker`, or one public `ticker` frame.
///
/// Bithumb publishes two clocks and they are not interchangeable: `timestamp`
/// is when the summary was built, `trade_timestamp` is when the fill behind
/// `trade_price` happened. On a quiet market they drift apart by however long
/// it has been since anyone traded, and `/v1/ticker` currently sends one
/// number for both.
///
/// Both are pulled back onto the UTC epoch by [`ticker_clock_shift`], which is
/// a no-op on the stream frame.
pub(crate) fn ticker(entry: &Value, market: Market) -> Result<Ticker> {
    let shift = ticker_clock_shift(entry)?;
    let onto_utc = |stamp: Timestamp| Timestamp::from_nanos(stamp.as_nanos().saturating_add(shift));

    Ok(Ticker {
        market,
        timestamp: onto_utc(millis(entry, "timestamp")?),
        last_trade_time: millis_opt(entry, "trade_timestamp")?.map(onto_utc),
        last_price: dec(entry, "trade_price")?,
        // `change_price` and `change_rate` are unsigned; only the signed pair
        // says which way the market moved.
        change: dec_opt(entry, "signed_change_price")?,
        change_rate: dec_opt(entry, "signed_change_rate")?,
        high: dec_opt(entry, "high_price")?,
        low: dec_opt(entry, "low_price")?,
        // The plain `acc_trade_*` fields cover the current session, not a
        // rolling window; only the `_24h` pair is what `Ticker` promises.
        volume: dec_opt(entry, "acc_trade_volume_24h")?,
        quote_volume: dec_opt(entry, "acc_trade_price_24h")?,
    })
}

/// How far a ticker's epoch fields sit from the UTC instant beside them, in
/// nanoseconds.
///
/// `/v1/ticker` documents `timestamp` and `trade_timestamp` as
/// `Unix timestamp, Unit: ms` but stamps both with the Korean wall clock, nine
/// hours ahead of the UTC instant the same payload spells out in `trade_date`
/// and `trade_time`. Observed on 2026-07-30 across `KRW-BTC`, `KRW-ETH` and
/// `BTC-ETH`, on every poll, at exactly nine hours.
///
/// The gap is measured against those two fields rather than assumed, so a
/// payload whose clocks already agree is left alone and the correction lapses
/// on its own the day Bithumb repairs the epoch fields. Any third gap is a
/// response `maxt` does not understand.
///
/// The public `ticker` frame gets no correction and needs none: its epoch
/// fields are already UTC. Its `trade_time` is Korean, so it carries no UTC
/// wall clock to measure against, and `trade_date_kst` is what tells the two
/// shapes apart, since only `/v1/ticker` sends it.
fn ticker_clock_shift(entry: &Value) -> Result<i64> {
    const KST_OFFSET_SECS: i64 = 9 * 3_600;

    if entry.get("trade_date_kst").is_none() {
        return Ok(0);
    }

    let stated = chrono::NaiveDateTime::parse_from_str(
        &format!(
            "{}{}",
            text(entry, "trade_date")?,
            text(entry, "trade_time")?
        ),
        "%Y%m%d%H%M%S",
    )
    .map_err(|err| {
        Error::decode(format!(
            "`trade_date`/`trade_time` is not a UTC time: {err}"
        ))
    })?
    .and_utc()
    .timestamp();

    match millis(entry, "trade_timestamp")?.as_secs() - stated {
        0 => Ok(0),
        KST_OFFSET_SECS => Ok(-KST_OFFSET_SECS * 1_000_000_000),
        other => Err(Error::decode(format!(
            "`trade_timestamp` is {other}s from the `trade_time` beside it"
        ))),
    }
}

/// Reads any of the `/v1/candles/...` responses.
///
/// Bithumb dates candles by a naive UTC wall clock, and stamps `timestamp`
/// with when the candle was last *touched*. `candle_date_time_utc` is the only
/// field that gives the interval's start.
///
/// `now` decides [`Candle::closed`]: Bithumb serves the running interval
/// alongside the finished ones and does not mark which is which, so the only
/// thing separating them is whether the window has ended. [`next_open`] says
/// when that is, at every interval including [`Interval::Month1`]: the month in
/// progress is the running interval here as much as the minute in progress is.
pub(crate) fn candles(value: &Value, interval: Interval, now: Timestamp) -> Result<Vec<Candle>> {
    entries(value)?
        .iter()
        .map(|entry| {
            let open_time = candle_open_time(text(entry, "candle_date_time_utc")?)?;
            let closed =
                next_open(interval, open_time).is_some_and(|end_of_window| end_of_window <= now);

            Ok(Candle {
                market: market_field(entry, "market")?,
                interval,
                open_time,
                open: dec(entry, "opening_price")?,
                high: dec(entry, "high_price")?,
                low: dec(entry, "low_price")?,
                close: dec(entry, "trade_price")?,
                volume: dec(entry, "candle_acc_trade_volume")?,
                quote_volume: dec_opt(entry, "candle_acc_trade_price")?,
                closed,
            })
        })
        .collect()
}

/// When the candle opening at `open_time` stops moving, which is when the next
/// one opens.
///
/// Bithumb cuts every interval on a Korean-time boundary, and for a month that
/// is not a UTC month. `/v1/candles/months?market=KRW-BTC` dates the March 2026
/// candle `2026-02-28T15:00:00`, which is `2026-03-01T00:00` in KST, and the
/// next one `2026-03-31T15:00:00`. [`Interval::advance`] steps whole UTC
/// calendar months, and one of those from 28 February is 28 March, three days
/// before the bar is finished. Five months in twelve end early that way, so a
/// consumer reading [`Candle::closed`] would commit a monthly bar that is still
/// running.
///
/// Shifting into KST, stepping, and shifting back is the arithmetic Bithumb's
/// own boundaries follow. Korea has kept a fixed nine-hour offset with no
/// daylight saving since 1988, so the shift is a constant rather than a zone
/// lookup. At every interval that does have a fixed length the two shifts
/// cancel and this is exactly [`Interval::advance`].
fn next_open(interval: Interval, open_time: Timestamp) -> Option<Timestamp> {
    const KST_OFFSET_NANOS: i64 = 9 * 3_600 * 1_000_000_000;

    let in_kst = Timestamp::from_nanos(open_time.as_nanos().checked_add(KST_OFFSET_NANOS)?);

    interval
        .advance(in_kst, 1)?
        .as_nanos()
        .checked_sub(KST_OFFSET_NANOS)
        .map(Timestamp::from_nanos)
}

fn candle_open_time(raw: &str) -> Result<Timestamp> {
    chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
        .map(|naive| Timestamp::from_secs(naive.and_utc().timestamp()))
        .map_err(|err| Error::decode(format!("`candle_date_time_utc` is not a UTC time: {err}")))
}

/// Reads `/v1/accounts`, and the `assets` of a private `myAsset` frame.
pub(crate) fn balances(value: &Value) -> Result<Vec<Balance>> {
    entries(value)?
        .iter()
        .map(|entry| {
            Ok(Balance {
                asset: text(entry, "currency")?.to_ascii_uppercase(),
                available: dec(entry, "balance")?,
                locked: dec(entry, "locked")?,
            })
        })
        .collect()
}

/// Reads `/v1/orders`.
pub(crate) fn orders(value: &Value) -> Result<Vec<Order>> {
    entries(value)?.iter().map(order).collect()
}

/// Reads one order from a private REST response.
pub(crate) fn order(entry: &Value) -> Result<Order> {
    let filled_quantity = dec(entry, "executed_volume")?;
    let remaining_quantity = dec(entry, "remaining_volume")?;

    Ok(Order {
        id: text(entry, "uuid")?.to_string(),
        market: market_field(entry, "market")?,
        side: side(entry, "side")?,
        status: rest_order_status(text(entry, "state")?, filled_quantity),
        filled_quantity,
        remaining_quantity,
        price: dec_opt(entry, "price")?,
        created_at: entry
            .get("created_at")
            .and_then(Value::as_str)
            .and_then(offset_time),
    })
}

/// `wait` and `watch` are both live; `watch` is a stop order waiting to trigger.
fn rest_order_status(state: &str, filled: Decimal) -> OrderStatus {
    match state {
        "wait" | "watch" if filled.is_zero() => OrderStatus::Open,
        "wait" | "watch" => OrderStatus::PartiallyFilled,
        "done" => OrderStatus::Filled,
        "cancel" => OrderStatus::Cancelled,
        _ => OrderStatus::Unknown,
    }
}

/// Bithumb stamps order acknowledgements in Korean time with an explicit offset.
fn offset_time(raw: &str) -> Option<Timestamp> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .and_then(|parsed| parsed.timestamp_nanos_opt())
        .map(Timestamp::from_nanos)
}

/// Reads the acknowledgement `/v2/orders` and `/v2/order` answer with.
///
/// The acknowledgement carries no fill state, so the caller supplies what it
/// asked for: an accepted order is unfilled, and a cancelled one is finished.
pub(crate) fn order_ack(
    entry: &Value,
    market: Market,
    side: Side,
    status: OrderStatus,
    remaining_quantity: Decimal,
    price: Option<Decimal>,
) -> Result<Order> {
    Ok(Order {
        id: text(entry, "order_id")?.to_string(),
        market,
        side,
        status,
        filled_quantity: Decimal::ZERO,
        remaining_quantity,
        price,
        created_at: entry
            .get("created_at")
            .and_then(Value::as_str)
            .and_then(offset_time),
    })
}

/// Reads one public WebSocket frame.
///
/// `Ok(None)` is a frame that carries no market data. Bithumb answers a
/// subscription and a client keepalive on the same socket the data arrives
/// on.
pub(crate) fn market_event(frame: &Value) -> Result<Option<MarketEvent>> {
    if let Some(error) = frame.get("error") {
        return Err(frame_error(error));
    }
    let Some(kind) = frame.get("type").and_then(Value::as_str) else {
        return Ok(None);
    };

    let event = match kind {
        "trade" => MarketEvent::Trade(Trade {
            market: market_field(frame, "code")?,
            // Two clocks again: `timestamp` is when Bithumb published the
            // frame, `trade_timestamp` is when the trade matched. The match
            // time is the one comparable against another exchange's trades.
            timestamp: millis(frame, "trade_timestamp")?,
            price: dec(frame, "trade_price")?,
            quantity: dec(frame, "trade_volume")?,
            taker_side: side(frame, "ask_bid")?,
            id: trade_id(frame),
        }),
        "orderbook" => {
            let market = market_field(frame, "code")?;
            // Microseconds here and milliseconds on every other frame; see
            // [`micros`].
            let timestamp = micros(frame, "timestamp")?;
            MarketEvent::OrderBook(order_book(frame, market, timestamp, None)?)
        }
        "ticker" => {
            let market = market_field(frame, "code")?;
            MarketEvent::Ticker(ticker(frame, market)?)
        }
        _ => return Ok(None),
    };

    Ok(Some(event))
}

/// Reads one private WebSocket frame.
///
/// One `myAsset` frame carries every changed balance, so this yields a list.
pub(crate) fn account_events(frame: &Value) -> Result<Vec<AccountEvent>> {
    if let Some(error) = frame.get("error") {
        return Err(frame_error(error));
    }

    match frame.get("type").and_then(Value::as_str) {
        Some("myAsset") => Ok(balances(field(frame, "assets")?)?
            .into_iter()
            .map(AccountEvent::Balance)
            .collect()),
        Some("myOrder") => Ok(vec![AccountEvent::Order(my_order(frame)?)]),
        _ => Ok(Vec::new()),
    }
}

fn my_order(frame: &Value) -> Result<Order> {
    let state = text(frame, "state")?;
    // Bithumb omits the cumulative quantities on a resting order, where they
    // are knowable, and sends them on every other state. Inferring them
    // anywhere else would invent a fill that may not have happened.
    let filled_quantity = match dec_opt(frame, "executed_quantity")? {
        Some(quantity) => quantity,
        None if state == "wait" => Decimal::ZERO,
        None => {
            return Err(Error::decode(
                "`executed_quantity` is missing on a filled order",
            ));
        }
    };
    let remaining_quantity = match dec_opt(frame, "remaining_quantity")? {
        Some(quantity) => quantity,
        None if state == "wait" => dec(frame, "order_quantity")?,
        None => {
            return Err(Error::decode(
                "`remaining_quantity` is missing on a filled order",
            ));
        }
    };

    Ok(Order {
        id: text(frame, "order_id")?.to_string(),
        market: market_field(frame, "code")?,
        side: side(frame, "side")?,
        status: stream_order_status(state, remaining_quantity),
        filled_quantity,
        remaining_quantity,
        price: dec_opt(frame, "order_price")?,
        created_at: Some(millis(frame, "order_timestamp")?),
    })
}

/// `done` means the order left the book. That is a fill only when nothing is
/// left on it. Otherwise the remainder was cancelled.
fn stream_order_status(state: &str, remaining: Decimal) -> OrderStatus {
    match state {
        "wait" => OrderStatus::Open,
        "trade" if remaining.is_zero() => OrderStatus::Filled,
        "trade" => OrderStatus::PartiallyFilled,
        "done" if remaining.is_zero() => OrderStatus::Filled,
        "done" | "cancel" => OrderStatus::Cancelled,
        _ => OrderStatus::Unknown,
    }
}

/// Reads Bithumb's WebSocket error frame, which uses `name`/`message` like the
/// REST envelope but arrives without an HTTP status.
fn frame_error(error: &Value) -> Error {
    Error::exchange(
        EXCHANGE,
        error
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("bithumb_websocket_error"),
        error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("the WebSocket returned an error frame")
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://apidocs.bithumb.com/reference/거래-대상-목록-조회.md
    //
    // Three of the 486 entries captured on 2026-07-30 from
    // `/v1/market/all?isDetails=true`. `KRW-ZIL` was one of the 15 markets
    // carrying Bithumb's 유의 종목 flag that day, spelled `CAUTION`. `KRW-ACS`
    // carried none of it while sitting under two 경보제 alerts, both at the
    // gravest step, which is why it is here.
    const MARKET_LIST: &str = r#"[
      {
        "market": "KRW-BTC",
        "korean_name": "비트코인",
        "english_name": "Bitcoin",
        "market_warning": "NONE"
      },
      {
        "market": "KRW-ZIL",
        "korean_name": "질리카",
        "english_name": "Zilliqa",
        "market_warning": "CAUTION"
      },
      {
        "market": "KRW-ACS",
        "korean_name": "액세스프로토콜",
        "english_name": "Access Protocol",
        "market_warning": "NONE"
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/경보제-조회.md
    //
    // Five of the 22 rows captured on 2026-07-30 from
    // `/v1/market/virtual_asset_warning`. All three documented steps appear,
    // `KRW-OBSR` twice because two criteria were raised on it at once.
    const MARKET_ALERTS: &str = r#"[
      {
        "market": "KRW-ZIL",
        "warning_type": "TRADING_VOLUME_SUDDEN_FLUCTUATION",
        "warning_step": "DANGER",
        "end_date": "2026-07-31 06:59:59"
      },
      {
        "market": "KRW-OBSR",
        "warning_type": "TRADING_VOLUME_SUDDEN_FLUCTUATION",
        "warning_step": "DANGER",
        "end_date": "2026-07-31 06:59:59"
      },
      {
        "market": "KRW-OBSR",
        "warning_type": "DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION",
        "warning_step": "WARNING",
        "end_date": "2026-07-31 07:04:59"
      },
      {
        "market": "KRW-KAIA",
        "warning_type": "SPECIFIC_ACCOUNT_HIGH_TRANSACTION",
        "warning_step": "CAUTION",
        "end_date": "2026-07-31 07:09:59"
      },
      {
        "market": "KRW-ACS",
        "warning_type": "DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION",
        "warning_step": "DANGER",
        "end_date": "2026-07-31 07:04:59"
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/현재가-조회.md
    //
    // Captured verbatim on 2026-07-30 from `/v1/ticker?markets=KRW-BTC`.
    // `trade_time` is `074830` UTC while both epoch fields decode to
    // `16:48:30`, the Korean wall clock: this is the nine-hour gap
    // [`ticker_clock_shift`] measures and undoes. Bithumb also sends one
    // number for `timestamp` and `trade_timestamp` here, though it documents
    // them as two clocks.
    const TICKER: &str = r#"[
      {
        "market": "KRW-BTC",
        "trade_date": "20260730",
        "trade_time": "074830",
        "trade_date_kst": "20260730",
        "trade_time_kst": "164830",
        "trade_timestamp": 1785430110156,
        "opening_price": 92112000,
        "high_price": 92770000,
        "low_price": 90904000,
        "trade_price": 91171000,
        "prev_closing_price": 92144000,
        "change": "FALL",
        "change_price": 973000,
        "change_rate": 0.0106,
        "signed_change_price": -973000,
        "signed_change_rate": -0.0106,
        "trade_volume": 0.02582544,
        "acc_trade_price": 19833795210.02728,
        "acc_trade_price_24h": 27001571740.8207,
        "acc_trade_volume": 216.55225961,
        "acc_trade_volume_24h": 294.0810771,
        "highest_52_week_price": 179734000,
        "highest_52_week_date": "2025-10-10",
        "lowest_52_week_price": 81110000,
        "lowest_52_week_date": "2026-02-07",
        "timestamp": 1785430110156
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/체결-내역-조회.md
    //
    // Captured verbatim on 2026-07-30 from
    // `/v1/trades/ticks?market=KRW-ETC&count=3`. A market quiet enough that
    // the three fills land in three different seconds; on a busy one Bithumb
    // returns several sharing a millisecond, and a `sequential_id` that is
    // that millisecond times ten thousand rather than a per-fill number.
    const TRADES: &str = r#"[
      {
        "market": "KRW-ETC",
        "trade_date_utc": "2026-07-30",
        "trade_time_utc": "07:47:41",
        "timestamp": 1785397661964,
        "trade_price": 9620,
        "trade_volume": 10.07,
        "prev_closing_price": 9590,
        "change_price": 30,
        "ask_bid": "BID",
        "sequential_id": 17853976619640000
      },
      {
        "market": "KRW-ETC",
        "trade_date_utc": "2026-07-30",
        "trade_time_utc": "07:45:54",
        "timestamp": 1785397554763,
        "trade_price": 9620,
        "trade_volume": 0.51975052,
        "prev_closing_price": 9590,
        "change_price": 30,
        "ask_bid": "BID",
        "sequential_id": 17853975547630000
      },
      {
        "market": "KRW-ETC",
        "trade_date_utc": "2026-07-30",
        "trade_time_utc": "07:40:04",
        "timestamp": 1785397204627,
        "trade_price": 9625,
        "trade_volume": 4.8,
        "prev_closing_price": 9590,
        "change_price": 35,
        "ask_bid": "BID",
        "sequential_id": 17853972046270000
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/호가-조회.md
    //
    // Captured verbatim on 2026-07-30 from `/v1/orderbook?markets=KRW-BTC`.
    // Thirty units, twice what the stream frame carries, and no `level` key:
    // that one belongs to the stream frame only. `maxt` sorts both sides
    // anyway, because Bithumb promises nothing about `orderbook_units`
    // ordering and `maxt` does.
    const ORDER_BOOK: &str = r#"[
      {
        "market": "KRW-BTC",
        "timestamp": 1785397720965,
        "total_ask_size": 3.7055,
        "total_bid_size": 5.622,
        "orderbook_units": [
          { "ask_price": 91226000, "bid_price": 91196000, "ask_size": 0.0469, "bid_size": 0.0002 },
          { "ask_price": 91233000, "bid_price": 91172000, "ask_size": 0.0396, "bid_size": 0.0031 },
          { "ask_price": 91234000, "bid_price": 91171000, "ask_size": 0, "bid_size": 0.0496 },
          { "ask_price": 91235000, "bid_price": 91170000, "ask_size": 0.2324, "bid_size": 0.0004 },
          { "ask_price": 91238000, "bid_price": 91166000, "ask_size": 0.0491, "bid_size": 0.0065 },
          { "ask_price": 91239000, "bid_price": 91164000, "ask_size": 0.0197, "bid_size": 0.0001 },
          { "ask_price": 91244000, "bid_price": 91163000, "ask_size": 0.0224, "bid_size": 4.7357 },
          { "ask_price": 91245000, "bid_price": 91162000, "ask_size": 0.0258, "bid_size": 0.1104 },
          { "ask_price": 91249000, "bid_price": 91161000, "ask_size": 0.1065, "bid_size": 0.0088 },
          { "ask_price": 91255000, "bid_price": 91160000, "ask_size": 0.0142, "bid_size": 0.1 },
          { "ask_price": 91270000, "bid_price": 91159000, "ask_size": 0.0192, "bid_size": 0.0775 },
          { "ask_price": 91274000, "bid_price": 91157000, "ask_size": 0.0002, "bid_size": 0.137 },
          { "ask_price": 91276000, "bid_price": 91156000, "ask_size": 0.0088, "bid_size": 0.0003 },
          { "ask_price": 91277000, "bid_price": 91154000, "ask_size": 0.0665, "bid_size": 0.011 },
          { "ask_price": 91280000, "bid_price": 91150000, "ask_size": 0.0294, "bid_size": 0.0104 },
          { "ask_price": 91282000, "bid_price": 91149000, "ask_size": 0.0019, "bid_size": 0.0127 },
          { "ask_price": 91283000, "bid_price": 91148000, "ask_size": 0.0037, "bid_size": 0.0179 },
          { "ask_price": 91285000, "bid_price": 91147000, "ask_size": 0.137, "bid_size": 0.1376 },
          { "ask_price": 91286000, "bid_price": 91146000, "ask_size": 2.198, "bid_size": 0.0327 },
          { "ask_price": 91288000, "bid_price": 91145000, "ask_size": 0.0066, "bid_size": 0.0001 },
          { "ask_price": 91291000, "bid_price": 91144000, "ask_size": 0.137, "bid_size": 0.0001 },
          { "ask_price": 91292000, "bid_price": 91143000, "ask_size": 0.0667, "bid_size": 0.0109 },
          { "ask_price": 91293000, "bid_price": 91142000, "ask_size": 0.0002, "bid_size": 0 },
          { "ask_price": 91297000, "bid_price": 91141000, "ask_size": 0.0002, "bid_size": 0.0004 },
          { "ask_price": 91300000, "bid_price": 91140000, "ask_size": 0.0103, "bid_size": 0.0002 },
          { "ask_price": 91303000, "bid_price": 91139000, "ask_size": 0.0043, "bid_size": 0.05 },
          { "ask_price": 91307000, "bid_price": 91138000, "ask_size": 0.0636, "bid_size": 0.0307 },
          { "ask_price": 91311000, "bid_price": 91137000, "ask_size": 0.3549, "bid_size": 0.0001 },
          { "ask_price": 91312000, "bid_price": 91136000, "ask_size": 0.0009, "bid_size": 0.0071 },
          { "ask_price": 91314000, "bid_price": 91134000, "ask_size": 0.0395, "bid_size": 0.0705 }
        ]
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/분minute-캔들-조회.md
    //
    // Captured verbatim on 2026-07-30 from
    // `/v1/candles/minutes/1?market=KRW-BTC&count=1`, mid-minute: the bar
    // opens 07:48:00 UTC and `timestamp` is the 07:48:30 fill that last
    // touched it.
    const MINUTE_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-07-30T07:48:00",
        "candle_date_time_kst": "2026-07-30T16:48:00",
        "opening_price": 91226000,
        "high_price": 91233000,
        "low_price": 91171000,
        "trade_price": 91171000,
        "timestamp": 1785397710000,
        "candle_acc_trade_price": 7044404.52985,
        "candle_acc_trade_volume": 0.07725993,
        "unit": 1
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/주week-캔들-조회.md
    //
    // Captured verbatim on 2026-07-30 from
    // `/v1/candles/weeks?market=KRW-BTC&count=1&to=2026-06-29T00:00:00`. A
    // Bithumb week opens on a Korean Monday, so this one is dated
    // `2026-06-21T15:00:00` UTC.
    const WEEK_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-06-21T15:00:00",
        "candle_date_time_kst": "2026-06-22T00:00:00",
        "opening_price": 96700000,
        "high_price": 98632000,
        "low_price": 88888000,
        "trade_price": 91022000,
        "timestamp": 1782658793854,
        "candle_acc_trade_price": 313245537093.97363,
        "candle_acc_trade_volume": 3378.18477138,
        "first_day_of_period": "2026-06-22"
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/월month-캔들-조회.md
    //
    // Captured verbatim on 2026-07-30 from
    // `/v1/candles/months?market=KRW-BTC&count=3&to=2026-04-01T00:00:00`.
    // Note where a Bithumb month opens: `2026-02-28T15:00:00` UTC, which is
    // `2026-03-01T00:00` in KST. The boundaries are Korean months, not UTC
    // ones, and the entries below are one month apart on that calendar even
    // though their UTC day numbers are 28, 31 and 31.
    const MONTH_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-02-28T15:00:00",
        "candle_date_time_kst": "2026-03-01T00:00:00",
        "opening_price": 94301000,
        "high_price": 112300000,
        "low_price": 94050000,
        "trade_price": 101614000,
        "timestamp": 1774969194120,
        "candle_acc_trade_price": 2555582225932.368,
        "candle_acc_trade_volume": 24722.59544281,
        "first_day_of_period": "2026-03-01"
      },
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-01-31T15:00:00",
        "candle_date_time_kst": "2026-02-01T00:00:00",
        "opening_price": 121216000,
        "high_price": 121960000,
        "low_price": 81110000,
        "trade_price": 94300000,
        "timestamp": 1772290797407,
        "candle_acc_trade_price": 4649384858247.525,
        "candle_acc_trade_volume": 46020.33836068,
        "first_day_of_period": "2026-02-01"
      },
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2025-12-31T15:00:00",
        "candle_date_time_kst": "2026-01-01T00:00:00",
        "opening_price": 128474000,
        "high_price": 143100000,
        "low_price": 119124000,
        "trade_price": 121228000,
        "timestamp": 1769871599231,
        "candle_acc_trade_price": 2293055672758.656,
        "candle_acc_trade_volume": 17306.1272608,
        "first_day_of_period": "2026-01-01"
      }
    ]"#;

    // https://apidocs.bithumb.com/reference/체결-trade.md
    //
    // Captured verbatim on 2026-07-30 from `wss://ws-api.bithumb.com/websocket/v1`,
    // subscribed to `trade` on `KRW-BTC`. Both epoch fields are milliseconds
    // and UTC; `trade_time` beside them is Korean, which is why nothing reads
    // it.
    const WS_TRADE: &str = r#"{
      "type": "trade",
      "code": "KRW-BTC",
      "trade_price": 91196000,
      "trade_volume": 0.00021931,
      "ask_bid": "ASK",
      "prev_closing_price": 92144000,
      "change": "FALL",
      "change_price": 948000,
      "trade_date": "2026-07-30",
      "trade_time": "16:48:55",
      "trade_timestamp": 1785397735004,
      "timestamp": 1785397735274,
      "sequential_id": 921046844841158138,
      "stream_type": "SNAPSHOT"
    }"#;

    // https://apidocs.bithumb.com/reference/호가-orderbook.md
    //
    // Captured verbatim on 2026-07-30 from `wss://ws-api.bithumb.com/websocket/v1`,
    // subscribed to `orderbook` on `KRW-BTC`. Fifteen units, `level` 1, and a
    // sixteen-digit `timestamp`: this frame's clock is microseconds while
    // every other frame on the same socket is milliseconds.
    const WS_ORDER_BOOK: &str = r#"{
      "type": "orderbook",
      "code": "KRW-BTC",
      "total_ask_size": 3.4039,
      "total_bid_size": 0.0852,
      "orderbook_units": [
        { "ask_price": 91196000, "bid_price": 91175000, "ask_size": 0.1995, "bid_size": 0.0000 },
        { "ask_price": 91209000, "bid_price": 91171000, "ask_size": 0.0224, "bid_size": 0.0289 },
        { "ask_price": 91210000, "bid_price": 91170000, "ask_size": 0.0224, "bid_size": 0.0005 },
        { "ask_price": 91211000, "bid_price": 91166000, "ask_size": 0.0688, "bid_size": 0.0065 },
        { "ask_price": 91215000, "bid_price": 91164000, "ask_size": 0.0301, "bid_size": 0.0001 },
        { "ask_price": 91226000, "bid_price": 91163000, "ask_size": 0.1535, "bid_size": 0.0010 },
        { "ask_price": 91231000, "bid_price": 91161000, "ask_size": 0.0192, "bid_size": 0.0021 },
        { "ask_price": 91234000, "bid_price": 91160000, "ask_size": 0.0000, "bid_size": 0.0000 },
        { "ask_price": 91235000, "bid_price": 91156000, "ask_size": 0.2324, "bid_size": 0.0003 },
        { "ask_price": 91251000, "bid_price": 91154000, "ask_size": 0.0626, "bid_size": 0.0110 },
        { "ask_price": 91255000, "bid_price": 91150000, "ask_size": 0.1513, "bid_size": 0.0104 },
        { "ask_price": 91256000, "bid_price": 91149000, "ask_size": 0.0313, "bid_size": 0.0127 },
        { "ask_price": 91258000, "bid_price": 91148000, "ask_size": 2.1980, "bid_size": 0.0111 },
        { "ask_price": 91259000, "bid_price": 91147000, "ask_size": 0.1370, "bid_size": 0.0006 },
        { "ask_price": 91262000, "bid_price": 91146000, "ask_size": 0.0754, "bid_size": 0.0000 }
      ],
      "level": 1,
      "timestamp": 1785397747576054,
      "stream_type": "SNAPSHOT"
    }"#;

    // https://apidocs.bithumb.com/reference/현재가-ticker.md
    //
    // Captured verbatim on 2026-07-30 from `wss://ws-api.bithumb.com/websocket/v1`,
    // subscribed to `ticker` on `KRW-BTC`. Both epoch fields are milliseconds
    // and UTC, unlike the `/v1/ticker` twin of this payload, and the frame
    // sends no `trade_date_kst`, which is what [`ticker_clock_shift`] keys off.
    const WS_TICKER: &str = r#"{
      "type": "ticker",
      "code": "KRW-BTC",
      "opening_price": 92112000,
      "high_price": 92770000,
      "low_price": 90904000,
      "trade_price": 91196000,
      "prev_closing_price": 92144000,
      "change": "FALL",
      "change_price": 948000,
      "signed_change_price": -948000,
      "change_rate": 0.01028824,
      "signed_change_rate": -0.01028824,
      "trade_volume": 0.00021931,
      "acc_trade_volume": 216.56322055,
      "acc_trade_volume_24h": 294.09163804,
      "acc_trade_price": 19834795209.4663,
      "acc_trade_price_24h": 27002534842.65972,
      "trade_date": "20260730",
      "trade_time": "164855",
      "trade_timestamp": 1785397735004,
      "ask_bid": "ASK",
      "acc_ask_volume": 115.83894934,
      "acc_bid_volume": 100.72427121,
      "highest_52_week_price": 179734000,
      "highest_52_week_date": "2025-10-09",
      "lowest_52_week_price": 81110000,
      "lowest_52_week_date": "2026-02-06",
      "market_state": "ACTIVE",
      "is_trading_suspended": false,
      "market_warning": "NONE",
      "timestamp": 1785397735276,
      "stream_type": "SNAPSHOT"
    }"#;

    // https://apidocs.bithumb.com/reference/내-자산-myasset.md
    //
    // Bithumb's own documented example, not a capture: the private socket
    // needs credentials, and this round was run against public endpoints
    // only. Bithumb documents `asset_timestamp` and `timestamp` here as
    // milliseconds, which is the scale [`account_events`] reads them at.
    const WS_MY_ASSET: &str = r#"{
      "type": "myAsset",
      "assets": [
        {
          "currency": "KRW",
          "balance": "2061832.35",
          "locked": "3824127.3"
        }
      ],
      "asset_timestamp": 1727052537592,
      "timestamp": 1727052537687
    }"#;

    // https://apidocs.bithumb.com/reference/내-주문-및-체결-myorder.md
    //
    // Bithumb's own documented example, not a capture, for the reason given
    // above [`WS_MY_ASSET`]. Bithumb documents `order_timestamp`,
    // `trade_timestamp` and `timestamp` here as milliseconds.
    const WS_MY_ORDER: &str = r#"{
      "type": "myOrder",
      "code": "KRW-BTC",
      "order_id": "C0101000000001818113",
      "client_order_id": "my-client-order-id-1",
      "side": "buy",
      "order_type": "limit",
      "state": "trade",
      "time_in_force": "post_only",
      "order_price": 1927000,
      "order_quantity": 0.55,
      "order_amount": 1059850,
      "order_timestamp": 1727052318074,
      "timestamp": 1727052318369,
      "trade_id": "C0101000000001744207",
      "trade_price": 1927000,
      "trade_quantity": 0.4697,
      "trade_amount": 905111.9,
      "trade_timestamp": 1727052318148,
      "executed_quantity": 0.4697,
      "remaining_quantity": 0.0803,
      "executed_amount": 905111.9,
      "paid_fee": 0,
      "remaining_fee": 0,
      "reserved_fee": 0
    }"#;

    fn btc_krw() -> Market {
        Market::spot(Exchange::Bithumb, "BTC", "KRW")
    }

    fn etc_krw() -> Market {
        Market::spot(Exchange::Bithumb, "ETC", "KRW")
    }

    fn acs_krw() -> Market {
        Market::spot(Exchange::Bithumb, "ACS", "KRW")
    }

    fn zil_krw() -> Market {
        Market::spot(Exchange::Bithumb, "ZIL", "KRW")
    }

    fn obsr_krw() -> Market {
        Market::spot(Exchange::Bithumb, "OBSR", "KRW")
    }

    fn parsed(raw: &str) -> Value {
        body(raw).expect("fixture is JSON")
    }

    fn exact(raw: &str) -> Decimal {
        Decimal::from_str_exact(raw).expect("test literal is a decimal")
    }

    #[test]
    fn a_market_survives_a_round_trip_through_bithumbs_own_code() {
        let code = native_symbol(&btc_krw()).expect("BTC/KRW is a Bithumb market");

        assert_eq!(code, "KRW-BTC");
        assert_eq!(split_symbol(&code).expect("round trip"), btc_krw());
    }

    #[test]
    fn the_two_directions_agree_on_every_shape_bithumb_lists() {
        for symbol in ["KRW-BTC", "BTC-ETH", "USDT-XRP", "KRW-1INCH"] {
            let market = split_symbol(symbol).expect("a listed code");
            assert_eq!(native_symbol(&market).expect("and back"), symbol);
        }
    }

    #[test]
    fn the_quote_asset_comes_first_in_a_bithumb_code() {
        // The one mistake this mapping invites: reading `KRW-BTC` as KRW priced
        // in BTC would invert every price on the market.
        let market = split_symbol("KRW-BTC").expect("a listed code");

        assert_eq!(market.base, "BTC");
        assert_eq!(market.quote, "KRW");
        assert_eq!(
            native_symbol(&Market::spot(Exchange::Bithumb, "ETH", "BTC")).expect("listed pair"),
            "BTC-ETH"
        );
    }

    #[test]
    fn a_market_that_is_not_bithumbs_never_becomes_a_symbol() {
        let upbit = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let perpetual = Market::perpetual(Exchange::Bithumb, "BTC", "KRW");

        assert!(matches!(
            native_symbol(&upbit),
            Err(Error::InvalidRequest {
                field: "market.exchange",
                ..
            })
        ));
        assert!(matches!(
            native_symbol(&perpetual),
            Err(Error::InvalidRequest {
                field: "market.kind",
                ..
            })
        ));
    }

    #[test]
    fn a_malformed_symbol_is_rejected_rather_than_guessed_at() {
        for symbol in ["KRWBTC", "krw-btc", "KRW-", "-BTC", "KRW-BTC-PERP", ""] {
            assert!(split_symbol(symbol).is_none(), "{symbol}");
        }
    }

    #[test]
    fn a_market_code_that_could_smuggle_a_query_parameter_is_rejected() {
        let injected = Market::spot(Exchange::Bithumb, "BTC&count=500", "KRW");

        assert!(matches!(
            native_symbol(&injected),
            Err(Error::InvalidRequest {
                field: "market.base",
                ..
            })
        ));
        assert!(split_symbol("KRW-BTC&count=500").is_none());
    }

    #[test]
    fn money_keeps_every_digit_bithumb_sent() {
        let candles = candles(
            &parsed(WEEK_CANDLES),
            Interval::Week1,
            Timestamp::from_millis(1_782_658_793_854),
        )
        .expect("week candles parse");

        // 313245537093.97363 is not representable in f64: routing it through
        // one would land on 313245537093.9736328125.
        assert_eq!(
            candles[0].quote_volume.expect("quote volume"),
            exact("313245537093.97363")
        );
        assert_eq!(candles[0].volume, exact("3378.18477138"));
        assert_eq!(candles[0].volume.scale(), 8);
    }

    #[test]
    fn a_decimal_string_and_a_decimal_number_read_the_same() {
        let from_number = dec(&parsed(r#"{"v": 0.01010101}"#), "v").expect("number");
        let from_string = dec(&parsed(r#"{"v": "0.01010101"}"#), "v").expect("string");

        assert_eq!(from_number, from_string);
        assert_eq!(from_number.scale(), 8);
    }

    #[test]
    fn scientific_notation_is_read_rather_than_refused() {
        let tiny = dec(&parsed(r#"{"v": 8.428e-05}"#), "v").expect("exponent form");

        assert_eq!(tiny, exact("0.00008428"));
    }

    #[test]
    fn a_number_too_precise_to_hold_is_a_decode_error_not_a_rounded_price() {
        // Twenty-nine decimal places is one past what `Decimal` carries.
        assert!(matches!(
            decimal("0.000000000000000000000000000001", "price"),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn a_market_list_keeps_both_names_and_flags_a_warning_as_not_plainly_active() {
        let markets = markets(&parsed(MARKET_LIST)).expect("market list parses");

        assert_eq!(markets.len(), 3);
        assert_eq!(markets[0].market, btc_krw());
        assert_eq!(markets[0].native_symbol, "KRW-BTC");
        assert_eq!(markets[0].korean_name.as_deref(), Some("비트코인"));
        assert_eq!(markets[0].status, MarketStatus::Active);
        // The designation itself has no home in `MarketStatus`, so it is only
        // reachable through the Bithumb-specific call.
        assert_eq!(markets[1].status, MarketStatus::Unknown);
        assert_eq!(
            market_warnings(&parsed(MARKET_LIST)).expect("warnings parse")[1].1,
            "CAUTION"
        );
    }

    #[test]
    fn an_alerted_market_stays_active_because_the_two_designations_are_separate() {
        let markets = markets(&parsed(MARKET_LIST)).expect("market list parses");
        let alerts = market_alerts(&parsed(MARKET_ALERTS)).expect("alerts parse");

        // `KRW-ACS` was under two alerts at the gravest step on the capture
        // day and still carried no 유의 종목 flag. Reporting it as `Unknown`
        // would put a fifth of the exchange there and hide the 15 that are
        // actually warned.
        assert_eq!(markets[2].market, acs_krw());
        assert_eq!(markets[2].status, MarketStatus::Active);
        assert!(
            alerts
                .iter()
                .any(|(market, alert)| *market == acs_krw()
                    && alert.step == BithumbAlertStep::Danger)
        );
        // And the reverse: a warned market need not be alerted at the top step.
        assert_eq!(markets[1].status, MarketStatus::Unknown);
    }

    #[test]
    fn an_alert_carries_its_criterion_its_step_and_a_korean_expiry_read_as_utc() {
        let alerts = market_alerts(&parsed(MARKET_ALERTS)).expect("alerts parse");

        assert_eq!(alerts.len(), 5);
        assert_eq!(alerts[0].0, zil_krw());
        assert_eq!(alerts[0].1.kind, "TRADING_VOLUME_SUDDEN_FLUCTUATION");
        assert_eq!(alerts[0].1.step, BithumbAlertStep::Danger);
        // `2026-07-31 06:59:59` is a Korean wall clock, so the instant is nine
        // hours earlier in UTC. Reading it as UTC would keep a lapsed alert
        // looking live for those nine hours.
        assert_eq!(alerts[0].1.ends_at, Timestamp::from_secs(1_785_448_799));
        assert_eq!(alerts[3].1.step, BithumbAlertStep::Caution);
        assert_eq!(alerts[3].1.ends_at, Timestamp::from_secs(1_785_449_399));
    }

    #[test]
    fn a_market_under_two_criteria_keeps_both_rows_at_their_own_steps() {
        let alerts = market_alerts(&parsed(MARKET_ALERTS)).expect("alerts parse");
        let obsr: Vec<_> = alerts
            .iter()
            .filter(|(market, _)| *market == obsr_krw())
            .map(|(_, alert)| alert)
            .collect();

        // One row per criterion, not one row per market: collapsing them would
        // lose either the second criterion or the step that differs.
        assert_eq!(obsr.len(), 2);
        assert_eq!(obsr[0].step, BithumbAlertStep::Danger);
        assert_eq!(obsr[1].step, BithumbAlertStep::Warning);
        assert_ne!(obsr[0].kind, obsr[1].kind);
    }

    #[test]
    fn steps_compare_by_severity_and_an_unfamiliar_one_outranks_the_gravest() {
        assert!(BithumbAlertStep::Caution < BithumbAlertStep::Warning);
        assert!(BithumbAlertStep::Warning < BithumbAlertStep::Danger);
        // A step Bithumb adds later must not slip under a severity threshold.
        assert!(BithumbAlertStep::Unknown > BithumbAlertStep::Danger);

        let odd = market_alerts(&parsed(
            r#"[{"market":"KRW-BTC","warning_type":"NEW_CRITERION","warning_step":"SEVERE","end_date":"2026-07-31 06:59:59"}]"#,
        ))
        .expect("an unfamiliar step is not a decode failure");

        assert_eq!(odd[0].1.step, BithumbAlertStep::Unknown);
        assert_eq!(odd[0].1.kind, "NEW_CRITERION");
    }

    #[test]
    fn an_expiry_bithumb_cannot_have_sent_is_a_decode_error_not_a_guess() {
        assert!(matches!(
            market_alerts(&parsed(
                r#"[{"market":"KRW-BTC","warning_type":"PRICE_SUDDEN_FLUCTUATION","warning_step":"CAUTION","end_date":"2026-07-31T06:59:59Z"}]"#,
            )),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn a_rest_trade_reports_the_taker_side_and_bithumbs_own_identifier() {
        let trades = trades(&parsed(TRADES)).expect("trades parse");

        assert_eq!(trades[0].market, etc_krw());
        assert_eq!(trades[0].taker_side, Side::Buy);
        // 07:47:41 UTC, the `trade_time_utc` beside it. Unlike `/v1/ticker`,
        // this endpoint's clock needs no correction.
        assert_eq!(
            trades[0].timestamp,
            Timestamp::from_millis(1_785_397_661_964)
        );
        assert_eq!(trades[0].price, Decimal::from(9_620));
        // Seventeen digits: past what an f64 mantissa holds exactly.
        assert_eq!(trades[0].id.as_deref(), Some("17853976619640000"));
    }

    #[test]
    fn recent_trades_come_back_newest_first() {
        // Bithumb answers `/v1/trades/ticks` in reverse chronological order,
        // which is what the fixture holds; the common API promises the same,
        // so 07:47:41 must stay in front of 07:40:04.
        let trades = trades(&parsed(TRADES)).expect("trades parse");

        assert_eq!(
            trades
                .iter()
                .map(|trade| trade.timestamp.as_millis())
                .collect::<Vec<_>>(),
            vec![1_785_397_661_964, 1_785_397_554_763, 1_785_397_204_627]
        );
    }

    #[test]
    fn trades_are_reordered_rather_than_trusted_to_arrive_sorted() {
        // The same three trades shuffled: the guarantee has to survive an order
        // bithumb does not currently send.
        let mut shuffled = parsed(TRADES);
        shuffled
            .as_array_mut()
            .expect("the fixture is an array")
            .rotate_left(1);

        let trades = trades(&shuffled).expect("trades parse");

        assert!(
            trades
                .windows(2)
                .all(|pair| pair[0].timestamp > pair[1].timestamp),
            "trades came back {:?}",
            trades
                .iter()
                .map(|trade| trade.timestamp.as_millis())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_stream_trade_is_stamped_with_the_match_time_not_the_publish_time() {
        let event = market_event(&parsed(WS_TRADE)).expect("frame parses");

        let Some(MarketEvent::Trade(trade)) = event else {
            panic!("expected a trade event");
        };
        assert_eq!(trade.taker_side, Side::Sell);
        // `trade_timestamp`, not the 270-millisecond-later `timestamp`.
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_785_397_735_004));
        assert_eq!(trade.id.as_deref(), Some("921046844841158138"));
    }

    #[test]
    fn both_sides_of_a_book_come_back_best_first() {
        let entry = &parsed(ORDER_BOOK)[0];
        let book = order_book(
            entry,
            btc_krw(),
            Timestamp::from_millis(1_785_397_720_965),
            None,
        )
        .expect("book parses");

        // The whole `/v1/orderbook` reply, which is thirty levels a side and
        // not the fifteen the socket sends.
        assert_eq!(book.bids.len(), 30);
        assert_eq!(book.asks.len(), 30);
        assert_eq!(book.bids[0].price, Decimal::from(91_196_000));
        assert_eq!(book.bids[1].price, Decimal::from(91_172_000));
        assert_eq!(book.asks[0].price, Decimal::from(91_226_000));
        assert_eq!(book.asks[1].price, Decimal::from(91_233_000));
        assert_eq!(book.spread().expect("two sides"), Decimal::from(30_000));
    }

    #[test]
    fn a_book_is_reordered_rather_than_trusted_to_arrive_sorted() {
        // Bithumb happens to answer sorted, so the guarantee has to survive an
        // order it does not currently send: the same real book, reversed.
        let mut entry = parsed(ORDER_BOOK)[0].clone();
        entry["orderbook_units"]
            .as_array_mut()
            .expect("the fixture has units")
            .reverse();

        let book = order_book(
            &entry,
            btc_krw(),
            Timestamp::from_millis(1_785_397_720_965),
            None,
        )
        .expect("book parses");

        assert!(
            book.bids
                .windows(2)
                .all(|pair| pair[0].price > pair[1].price)
        );
        assert!(
            book.asks
                .windows(2)
                .all(|pair| pair[0].price < pair[1].price)
        );
        assert_eq!(book.bids[0].price, Decimal::from(91_196_000));
        assert_eq!(book.asks[0].price, Decimal::from(91_226_000));
    }

    #[test]
    fn depth_takes_the_best_levels_not_the_first_ones_bithumb_listed() {
        let mut entry = parsed(ORDER_BOOK)[0].clone();
        entry["orderbook_units"]
            .as_array_mut()
            .expect("the fixture has units")
            .reverse();

        let book = order_book(
            &entry,
            btc_krw(),
            Timestamp::from_millis(1_785_397_720_965),
            Some(1),
        )
        .expect("book parses");

        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.bids[0].price, Decimal::from(91_196_000));
        assert_eq!(book.asks[0].price, Decimal::from(91_226_000));
    }

    #[test]
    fn a_stream_book_is_sorted_and_timestamped_in_microseconds() {
        let event = market_event(&parsed(WS_ORDER_BOOK)).expect("frame parses");

        let Some(MarketEvent::OrderBook(book)) = event else {
            panic!("expected an order book event");
        };
        // 2026-07-30T07:49:07Z. Read as milliseconds this lands in the year
        // 58545, which is the shape of the bug that made this feed produce
        // nothing but decode errors.
        assert_eq!(
            book.timestamp,
            Timestamp::from_micros(1_785_397_747_576_054)
        );
        assert_eq!(book.timestamp.as_secs(), 1_785_397_747);
        // Fifteen levels a side, which is what the socket sends and half what
        // `/v1/orderbook` does.
        assert_eq!(book.bids.len(), 15);
        assert_eq!(book.asks.len(), 15);
        assert_eq!(
            book.best_bid().expect("a bid").price,
            Decimal::from(91_175_000)
        );
    }

    #[test]
    fn a_millisecond_clock_offered_as_microseconds_is_refused() {
        // The mirror of the book bug: a stream frame that starts sending
        // milliseconds must fail loudly rather than decode to 1970. Nothing
        // about the value's magnitude is guessed at, so this is the only thing
        // standing between a unit change and a silent wrong answer.
        let mut frame = parsed(WS_ORDER_BOOK);
        frame["timestamp"] = serde_json::json!(1_785_397_747_576i64);

        let err = market_event(&frame).expect_err("a millisecond book clock is refused");

        assert!(
            err.to_string().contains("not a microsecond timestamp"),
            "got {err}"
        );
    }

    #[test]
    fn a_rest_ticker_is_pulled_back_off_the_korean_wall_clock() {
        let rest = ticker(&parsed(TICKER)[0], btc_krw()).expect("ticker parses");

        // The raw field reads 1785430110156, which is 16:48:30 UTC: the Korean
        // wall clock served as if it were the UTC epoch. `trade_time` in the
        // same payload says 074830, and that is the instant a caller gets.
        assert_eq!(
            rest.timestamp,
            Timestamp::from_millis(1_785_397_710_156),
            "the ticker clock was not pulled back nine hours"
        );
        assert_eq!(rest.timestamp.to_string(), "2026-07-30T07:48:30.156Z");
        // `/v1/ticker` sends one number for both clocks, so both land together.
        assert_eq!(rest.last_trade_time, Some(rest.timestamp));
        assert_eq!(rest.last_price, Decimal::from(91_171_000));
        assert_eq!(rest.change, Some(Decimal::from(-973_000)));
        assert_eq!(rest.change_rate, Some(exact("-0.0106")));
        // The rolling 24-hour volume, not the 216.55 of the current session.
        assert_eq!(rest.volume, Some(exact("294.0810771")));
    }

    #[test]
    fn a_ticker_whose_clocks_already_agree_is_left_alone() {
        // What the correction does the day Bithumb repairs the epoch fields:
        // nothing. The gap is measured, never assumed.
        let mut entry = parsed(TICKER)[0].clone();
        entry["trade_timestamp"] = serde_json::json!(1_785_397_710_156i64);
        entry["timestamp"] = serde_json::json!(1_785_397_710_156i64);

        let rest = ticker(&entry, btc_krw()).expect("ticker parses");

        assert_eq!(rest.timestamp, Timestamp::from_millis(1_785_397_710_156));
    }

    #[test]
    fn a_ticker_clock_that_is_neither_utc_nor_korean_is_refused() {
        let mut entry = parsed(TICKER)[0].clone();
        entry["trade_timestamp"] = serde_json::json!(1_785_401_310_156i64);

        let err = ticker(&entry, btc_krw()).expect_err("an unknown clock is refused");

        assert!(err.to_string().contains("3600s from the"), "got {err}");
    }

    #[test]
    fn a_stream_ticker_needs_no_clock_correction() {
        let stream = market_event(&parsed(WS_TICKER)).expect("frame parses");

        let Some(MarketEvent::Ticker(stream)) = stream else {
            panic!("expected a ticker event");
        };
        // The socket's epoch fields are already UTC, and the frame carries no
        // `trade_date_kst`, so nothing is subtracted.
        assert_eq!(stream.timestamp, Timestamp::from_millis(1_785_397_735_276));
        // The summary was built 272 milliseconds after the fill it reports.
        assert_eq!(
            stream.last_trade_time,
            Some(Timestamp::from_millis(1_785_397_735_004))
        );
        assert!(stream.last_trade_time < Some(stream.timestamp));
        assert_eq!(stream.last_price, Decimal::from(91_196_000));
    }

    #[test]
    fn a_candle_opens_at_its_utc_wall_clock_not_at_its_timestamp_field() {
        let after_the_minute = Timestamp::from_secs(1_785_397_800);
        let candles = candles(&parsed(MINUTE_CANDLES), Interval::Min1, after_the_minute)
            .expect("candles parse");

        // 2026-07-30T07:48:00Z, not the 07:48:30 the `timestamp` field carries.
        assert_eq!(candles[0].open_time, Timestamp::from_secs(1_785_397_680));
        assert_eq!(candles[0].open, Decimal::from(91_226_000));
        assert_eq!(candles[0].close, Decimal::from(91_171_000));
        assert!(candles[0].closed);
    }

    #[test]
    fn the_running_candle_is_not_reported_as_closed() {
        let inside_the_minute = Timestamp::from_secs(1_785_397_710);
        let candles = candles(&parsed(MINUTE_CANDLES), Interval::Min1, inside_the_minute)
            .expect("candles parse");

        assert!(!candles[0].closed);
    }

    #[test]
    fn the_month_in_progress_is_running_like_any_other_interval() {
        // The newest entry is the Korean month of March 2026, opening
        // 2026-02-28T15:00Z. It is not over in the middle of it. Answering
        // `true` because a month has no fixed length hands a consumer weeks of
        // a month as a settled bar, and `Candle::closed` is what a consumer
        // commits on.
        let mid_march = Timestamp::from_secs(1_773_500_000); // 2026-03-14T20:13:20Z
        let april = Timestamp::from_secs(1_774_969_200); // 2026-03-31T15:00:00Z

        let running =
            candles(&parsed(MONTH_CANDLES), Interval::Month1, mid_march).expect("candles parse");
        let settled =
            candles(&parsed(MONTH_CANDLES), Interval::Month1, april).expect("candles parse");

        assert!(!running[0].closed);
        assert!(settled[0].closed);
    }

    #[test]
    fn a_monthly_candle_closes_on_the_korean_month_boundary_it_was_cut_on() {
        // Bithumb's months are Korean months, so the candle in `MONTH_CANDLES`
        // opening 2026-02-28T15:00Z runs until the next one opens at
        // 2026-03-31T15:00Z, three days later than a UTC calendar step from 28
        // February reaches. Those three days are the window in which a bar that
        // is still moving would be handed over as settled.
        let utc_month_step = Timestamp::from_secs(1_774_710_000); // 2026-03-28T15:00:00Z
        let korean_month_step = Timestamp::from_secs(1_774_969_200); // 2026-03-31T15:00:00Z

        let mid_window = candles(&parsed(MONTH_CANDLES), Interval::Month1, utc_month_step)
            .expect("candles parse");
        let at_the_boundary = candles(&parsed(MONTH_CANDLES), Interval::Month1, korean_month_step)
            .expect("candles parse");

        assert!(
            !mid_window[0].closed,
            "the March candle still has three days to run on 28 March"
        );
        assert!(at_the_boundary[0].closed);
        // The one behind it is settled throughout, on either reading, so the
        // assertions above are about the boundary and not about the payload.
        assert!(mid_window[1].closed);
    }

    #[test]
    fn an_error_body_keeps_bithumbs_own_code_and_message() {
        // https://apidocs.bithumb.com/reference/api-주요-에러-코드-목록
        let error = exchange_error(
            401,
            r#"{"error":{"name":"invalid_access_key","message":"잘못된 액세스 키"}}"#,
        );

        let Error::Exchange {
            exchange,
            code,
            message,
            status,
            ..
        } = &error
        else {
            panic!("expected an exchange error");
        };
        assert_eq!(*exchange, "bithumb");
        assert_eq!(code, "invalid_access_key");
        assert_eq!(message, "잘못된 액세스 키");
        assert_eq!(*status, Some(401));
        assert!(!error.is_retryable());
    }

    #[test]
    fn an_unreadable_error_body_is_still_reported_verbatim() {
        let error = exchange_error(502, "  <html>bad gateway</html>  ");

        let Error::Exchange { code, message, .. } = &error else {
            panic!("expected an exchange error");
        };
        assert_eq!(code, "bithumb_error");
        assert_eq!(message, "<html>bad gateway</html>");
        assert!(error.is_retryable());
    }

    #[test]
    fn a_websocket_error_frame_becomes_an_exchange_error() {
        let frame = parsed(r#"{"error":{"name":"WRONG_FORMAT","message":"Format is wrong"}}"#);

        let error = market_event(&frame).expect_err("an error frame is not data");
        assert!(matches!(error, Error::Exchange { ref code, .. } if code == "WRONG_FORMAT"));
    }

    #[test]
    fn frames_that_are_not_market_data_are_skipped_rather_than_failed() {
        for frame in [r#"{"status":"UP"}"#, r#"{"type":"unknown"}"#] {
            assert!(
                market_event(&parsed(frame))
                    .expect("a control frame is not an error")
                    .is_none(),
                "{frame}"
            );
        }
    }

    #[test]
    fn a_balance_frame_yields_one_event_per_asset_with_exact_amounts() {
        let events = account_events(&parsed(WS_MY_ASSET)).expect("frame parses");

        assert_eq!(events.len(), 1);
        let AccountEvent::Balance(balance) = &events[0] else {
            panic!("expected a balance event");
        };
        assert_eq!(balance.asset, "KRW");
        assert_eq!(balance.available, exact("2061832.35"));
        assert_eq!(balance.total(), exact("5885959.65"));
    }

    #[test]
    fn a_partly_filled_order_frame_is_not_reported_as_finished() {
        let events = account_events(&parsed(WS_MY_ORDER)).expect("frame parses");

        let [AccountEvent::Order(order)] = events.as_slice() else {
            panic!("expected one order event");
        };
        assert_eq!(order.id, "C0101000000001818113");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(order.status.is_live());
        assert_eq!(order.filled_quantity, exact("0.4697"));
        assert_eq!(order.remaining_quantity, exact("0.0803"));
    }

    #[test]
    fn an_order_that_left_the_book_unfilled_is_cancelled_not_filled() {
        let mut frame = parsed(WS_MY_ORDER);
        frame["state"] = Value::String("done".to_string());

        let events = account_events(&frame).expect("frame parses");
        let [AccountEvent::Order(order)] = events.as_slice() else {
            panic!("expected one order event");
        };
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn a_finished_order_without_its_quantities_is_refused_rather_than_guessed() {
        let mut frame = parsed(WS_MY_ORDER);
        frame["state"] = Value::String("done".to_string());
        frame["executed_quantity"] = Value::Null;
        frame["remaining_quantity"] = Value::Null;

        assert!(matches!(account_events(&frame), Err(Error::Decode { .. })));
    }

    #[test]
    fn a_resting_order_frame_may_omit_its_quantities() {
        let mut frame = parsed(WS_MY_ORDER);
        frame["state"] = Value::String("wait".to_string());
        frame["executed_quantity"] = Value::Null;
        frame["remaining_quantity"] = Value::Null;

        let events = account_events(&frame).expect("frame parses");
        let [AccountEvent::Order(order)] = events.as_slice() else {
            panic!("expected one order event");
        };
        assert_eq!(order.status, OrderStatus::Open);
        assert!(order.filled_quantity.is_zero());
        assert_eq!(order.remaining_quantity, exact("0.55"));
    }

    #[test]
    fn private_rest_orders_read_bid_and_ask_as_buy_and_sell() {
        // https://apidocs.bithumb.com/reference/대기-주문-조회
        let raw = r#"[{
          "market": "KRW-BTC",
          "uuid": "C0661000000000760010",
          "side": "ask",
          "ord_type": "limit",
          "state": "wait",
          "price": "1055",
          "volume": "16",
          "remaining_volume": "11",
          "executed_volume": "5",
          "created_at": "2024-07-14T13:35:41+09:00"
        }]"#;

        let orders = orders(&parsed(raw)).expect("orders parse");

        assert_eq!(orders[0].side, Side::Sell);
        assert_eq!(orders[0].status, OrderStatus::PartiallyFilled);
        assert_eq!(orders[0].filled_quantity, Decimal::from(5));
        assert_eq!(orders[0].price, Some(Decimal::from(1055)));
        // 13:35:41+09:00 is 04:35:41Z.
        assert_eq!(
            orders[0].created_at,
            Some(Timestamp::from_secs(1_720_931_741))
        );
    }

    #[test]
    fn a_balance_response_uppercases_the_asset_bithumb_lowercased() {
        let balances = balances(&parsed(
            r#"[{"currency":"btc","balance":"1.25","locked":"0.5"}]"#,
        ))
        .expect("balances parse");

        assert_eq!(balances[0].asset, "BTC");
        assert_eq!(balances[0].total(), exact("1.75"));
    }
}
