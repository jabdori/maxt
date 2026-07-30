//! Upbit's JSON, and what it means in `maxt` terms.
//!
//! Everything that reads an Upbit payload lives here, so that the REST, stream,
//! and private modules above only have to decide *which* payload they are
//! looking at. Numbers never pass through `f64`: `serde_json` is configured with
//! `arbitrary_precision`, so a [`serde_json::Number`] still holds the digits
//! Upbit sent and [`Decimal`] is built from that text.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Number;

use super::UpbitMarketEvent;
use crate::error::{Error, Result};
use crate::types::{
    Balance, Candle, Exchange, Interval, Level, Market, MarketInfo, MarketKind, MarketStatus,
    Order, OrderBook, OrderStatus, Side, Ticker, Timestamp, Trade,
};

pub(crate) const EXCHANGE: &str = Exchange::Upbit.id();

// ---------------------------------------------------------------------------
// Raw payloads
// ---------------------------------------------------------------------------

/// One entry of `GET /v1/market/all?is_details=true`.
///
/// The four deployments do not send the same shape. Upbit Korea sends
/// `market_event` and no `market_warning`; Singapore, Indonesia and Thailand
/// send `market_warning` and no `market_event`, and no `korean_name` either.
/// Both designation fields are therefore optional and both are read.
#[derive(Debug, Deserialize)]
pub(crate) struct RawMarket {
    pub(crate) market: String,
    pub(crate) korean_name: Option<String>,
    pub(crate) english_name: Option<String>,
    #[serde(default)]
    pub(crate) market_event: Option<RawMarketEvent>,
    /// The older spelling of `market_event.warning`, still what the non-Korean
    /// deployments send. Upbit calls it deprecated. Its values are `NONE` and
    /// `CAUTION`.
    #[serde(default)]
    pub(crate) market_warning: Option<String>,
}

/// Upbit Korea's two designations for a listing, which are not the same thing.
///
/// `warning` is 유의 종목: Upbit designates it by hand, announces it, and asks
/// the project to fix whatever caused it. A market can stay designated for
/// weeks, and if the cause is not resolved Upbit may end trading support for
/// it. Eleven of the eight hundred markets carried it on 2026-07-30.
///
/// `caution` is 주의 종목: raised and cleared automatically against published
/// criteria, one boolean per criterion, describing how the market is trading
/// right now rather than anything about the listing. A hundred and ninety
/// markets carried at least one on the same day, a hundred and seventy five of
/// them `GLOBAL_PRICE_DIFFERENCES` alone.
#[derive(Debug, Deserialize)]
pub(crate) struct RawMarketEvent {
    #[serde(default)]
    pub(crate) warning: bool,
    /// Keyed by Upbit's own name for each criterion, read as a map rather than
    /// as the five fields Upbit sends today so that a criterion added later is
    /// carried through instead of dropped. `BTreeMap` so the order a caller
    /// sees is Upbit's names sorted, not whatever the payload happened to use.
    #[serde(default)]
    pub(crate) caution: BTreeMap<String, bool>,
}

/// One entry of `GET /v1/trades/ticks`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawTrade {
    pub(crate) market: String,
    pub(crate) timestamp: i64,
    pub(crate) trade_price: Number,
    pub(crate) trade_volume: Number,
    pub(crate) ask_bid: String,
    /// Upbit's own trade number. See [`trade_id`].
    #[serde(default)]
    pub(crate) sequential_id: Option<Number>,
}

/// One entry of `GET /v1/orderbook`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawOrderBook {
    pub(crate) market: String,
    pub(crate) timestamp: i64,
    pub(crate) orderbook_units: Vec<RawOrderBookUnit>,
}

/// One level pair. Upbit publishes a bid and an ask together at each index,
/// not as two independent ladders.
#[derive(Debug, Deserialize)]
pub(crate) struct RawOrderBookUnit {
    pub(crate) bid_price: Number,
    pub(crate) bid_size: Number,
    pub(crate) ask_price: Number,
    pub(crate) ask_size: Number,
}

/// One entry of `GET /v1/ticker`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawTicker {
    pub(crate) market: String,
    /// When Upbit built this summary.
    pub(crate) timestamp: i64,
    /// When the trade behind `trade_price` executed. Upbit publishes both, and
    /// on a quiet market they differ by however long it has been since a fill.
    pub(crate) trade_timestamp: i64,
    pub(crate) trade_price: Number,
    pub(crate) signed_change_price: Number,
    pub(crate) signed_change_rate: Number,
    pub(crate) high_price: Number,
    pub(crate) low_price: Number,
    pub(crate) acc_trade_volume_24h: Number,
    pub(crate) acc_trade_price_24h: Number,
}

/// One entry of any of `GET /v1/candles/{minutes/{unit},days,weeks}`.
///
/// The three endpoints differ only in fields `maxt` does not carry, so one
/// shape reads all of them. `unit` is present on minute candles only.
#[derive(Debug, Deserialize)]
pub(crate) struct RawCandle {
    pub(crate) market: String,
    pub(crate) candle_date_time_utc: String,
    pub(crate) opening_price: Number,
    pub(crate) high_price: Number,
    pub(crate) low_price: Number,
    pub(crate) trade_price: Number,
    pub(crate) candle_acc_trade_price: Number,
    pub(crate) candle_acc_trade_volume: Number,
    #[serde(default)]
    pub(crate) unit: Option<u32>,
}

/// One entry of `GET /v1/accounts`. Upbit sends account figures as strings.
#[derive(Debug, Deserialize)]
pub(crate) struct RawBalance {
    pub(crate) currency: String,
    pub(crate) balance: String,
    pub(crate) locked: String,
}

/// An order as `GET /v1/orders/open`, `POST /v1/orders`, and
/// `DELETE /v1/order` all report it.
#[derive(Debug, Deserialize)]
pub(crate) struct RawOrder {
    pub(crate) market: String,
    pub(crate) uuid: String,
    pub(crate) side: String,
    pub(crate) state: String,
    #[serde(default)]
    pub(crate) price: Option<String>,
    pub(crate) remaining_volume: String,
    pub(crate) executed_volume: String,
    #[serde(default)]
    pub(crate) created_at: Option<String>,
}

/// Upbit's error body. Every REST failure uses this shape.
#[derive(Debug, Deserialize)]
struct RawErrorEnvelope {
    error: RawError,
}

#[derive(Debug, Deserialize)]
struct RawError {
    name: String,
    message: String,
}

/// A `trade` frame from the public WebSocket.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamTrade {
    pub(crate) code: String,
    pub(crate) trade_timestamp: i64,
    pub(crate) trade_price: Number,
    pub(crate) trade_volume: Number,
    pub(crate) ask_bid: String,
    /// Upbit's own trade number, the same one the REST tick carries. See
    /// [`trade_id`].
    #[serde(default)]
    pub(crate) sequential_id: Option<Number>,
}

/// An `orderbook` frame from the public WebSocket.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamOrderBook {
    pub(crate) code: String,
    pub(crate) timestamp: i64,
    pub(crate) orderbook_units: Vec<RawOrderBookUnit>,
}

/// A `ticker` frame from the public WebSocket.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamTicker {
    pub(crate) code: String,
    /// When Upbit built this summary.
    pub(crate) timestamp: i64,
    /// When the trade behind `trade_price` executed.
    pub(crate) trade_timestamp: i64,
    pub(crate) trade_price: Number,
    pub(crate) signed_change_price: Number,
    pub(crate) signed_change_rate: Number,
    pub(crate) high_price: Number,
    pub(crate) low_price: Number,
    pub(crate) acc_trade_volume_24h: Number,
    pub(crate) acc_trade_price_24h: Number,
}

/// A `candle.*` frame from the public WebSocket.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamCandle {
    pub(crate) code: String,
    pub(crate) candle_date_time_utc: String,
    pub(crate) opening_price: Number,
    pub(crate) high_price: Number,
    pub(crate) low_price: Number,
    pub(crate) trade_price: Number,
    pub(crate) candle_acc_trade_price: Number,
    pub(crate) candle_acc_trade_volume: Number,
    /// `SNAPSHOT` on the first frame a subscription gets for this market,
    /// `REALTIME` on every later one.
    ///
    /// A snapshot follows a connect or a reconnect, so it is the one frame
    /// after which [`super::stream::Decoder`] cannot vouch for what it is
    /// holding.
    #[serde(default)]
    pub(crate) stream_type: Option<String>,
}

/// A `myOrder` frame from the private WebSocket.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamOrder {
    pub(crate) code: String,
    pub(crate) uuid: String,
    pub(crate) ask_bid: String,
    pub(crate) state: String,
    pub(crate) price: Option<Number>,
    pub(crate) remaining_volume: Number,
    pub(crate) executed_volume: Number,
    pub(crate) order_timestamp: i64,
}

/// A `myAsset` frame from the private WebSocket.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamAssets {
    pub(crate) assets: Vec<RawStreamAsset>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamAsset {
    pub(crate) currency: String,
    pub(crate) balance: Number,
    pub(crate) locked: Number,
}

// ---------------------------------------------------------------------------
// Scalars
// ---------------------------------------------------------------------------

/// Reads a JSON number as a [`Decimal`], from its digits rather than its value.
///
/// A number Upbit sends that `Decimal` cannot hold exactly is a decode failure.
/// Rounding it would silently lose the last digit of a price.
pub(crate) fn decimal(value: &Number, field: &str) -> Result<Decimal> {
    decimal_text(&value.to_string(), field)
}

/// Reads a decimal that Upbit sent as a JSON string, which its account and
/// order endpoints do.
pub(crate) fn decimal_text(text: &str, field: &str) -> Result<Decimal> {
    // Upbit prints small sizes in scientific notation (`8.428e-05` in the trade
    // stream); [`crate::adapters::decimal::exact`] reads that spelling without
    // relaxing the no-rounding rule above, and is what every adapter uses.
    crate::adapters::decimal::exact(text)
        .map_err(|err| Error::decode(format!("`{field}` is not a decimal: {text} ({err})")))
}

/// Converts one of Upbit's millisecond timestamps.
pub(crate) fn millis(millis: i64, field: &str) -> Result<Timestamp> {
    millis
        .checked_mul(1_000_000)
        .map(Timestamp::from_nanos)
        .ok_or_else(|| Error::decode(format!("`{field}` is out of range: {millis}ms")))
}

/// Parses `candle_date_time_utc`, which Upbit sends as a naive UTC datetime
/// with neither a zone suffix nor sub-second digits.
pub(crate) fn candle_open_time(raw: &str) -> Result<Timestamp> {
    NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
        .map(|naive| Timestamp::from_secs(naive.and_utc().timestamp()))
        .map_err(|err| {
            Error::decode(format!(
                "`candle_date_time_utc` is not a UTC datetime: {raw} ({err})"
            ))
        })
}

/// Formats a timestamp for Upbit's `to` candle cursor, which is second
/// resolution and rejects anything else.
///
/// Upbit reads `to` as exclusive, and a sub-second `to` is rounded up rather
/// than truncated so that no candle is lost to the rounding. Both halves of
/// that were read off the live endpoint on 2026-07-30, on `KRW-BTC` second
/// candles:
///
/// | `to` sent | newest candle Upbit answered with |
/// | --- | --- |
/// | `2026-07-30T06:52:00Z` | `06:51:59` |
/// | `2026-07-30T06:52:01Z` | `06:52:00` |
///
/// So a `to` of `06:52:00.500` truncated to `06:52:00Z` would drop the
/// `06:52:00` candle, which [`CandleRequest::to`](crate::CandleRequest) keeps:
/// that window opens before the requested instant. Rounding up asks for one
/// second more than needed, and the caller's own `to` then excludes whatever
/// that extra second brought back.
pub(crate) fn to_cursor(at: Timestamp) -> Result<String> {
    // Ceiling division, so a timestamp already on a whole second is unchanged.
    let nanos = at.as_nanos();
    let secs = nanos.div_euclid(1_000_000_000) + i64::from(nanos.rem_euclid(1_000_000_000) != 0);

    DateTime::<Utc>::from_timestamp(secs, 0)
        .map(|utc| utc.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .ok_or_else(|| {
            Error::invalid_request("to", format!("{at} is not a representable datetime"))
        })
}

/// Reads Upbit's side spelling in either of the two casings it uses: `BID`/`ASK`
/// on the streams, `bid`/`ask` on REST.
pub(crate) fn side(raw: &str) -> Result<Side> {
    if raw.eq_ignore_ascii_case("bid") {
        Ok(Side::Buy)
    } else if raw.eq_ignore_ascii_case("ask") {
        Ok(Side::Sell)
    } else {
        Err(Error::decode(format!("unknown Upbit order side `{raw}`")))
    }
}

/// Maps Upbit's order `state`.
///
/// `wait` and `watch` describe a resting order, so a partial fill shows up in
/// the volumes rather than the state. `trade` appears on the private stream
/// only, where the remaining volume separates a partial fill from a complete
/// one.
pub(crate) fn order_status(state: &str, filled: Decimal, remaining: Decimal) -> OrderStatus {
    match state {
        "wait" | "watch" if filled.is_zero() => OrderStatus::Open,
        "wait" | "watch" => OrderStatus::PartiallyFilled,
        "trade" if remaining.is_zero() => OrderStatus::Filled,
        "trade" => OrderStatus::PartiallyFilled,
        "done" => OrderStatus::Filled,
        "cancel" => OrderStatus::Cancelled,
        // Upbit reports an order stopped by self-trade prevention as `prevented`.
        "prevented" => OrderStatus::Rejected,
        _ => OrderStatus::Unknown,
    }
}

/// Turns a non-2xx REST response into an [`Error::Exchange`] carrying Upbit's
/// own `name` and `message`.
///
/// Upbit's gateway answers some failures with plain text instead of the JSON
/// envelope, rate limiting in particular. The body is kept verbatim in that
/// case.
pub(crate) fn exchange_error(status: u16, body: &str) -> Error {
    match serde_json::from_str::<RawErrorEnvelope>(body) {
        Ok(envelope) => Error::exchange_http(
            EXCHANGE,
            status,
            envelope.error.name,
            envelope.error.message,
        ),
        Err(_) => Error::exchange_http(EXCHANGE, status, "unknown", body.trim()),
    }
}

/// Reads a successful REST body, reporting a shape change as
/// [`Error::Decode`].
pub(crate) fn json<T: for<'de> Deserialize<'de>>(body: &str) -> Result<T> {
    serde_json::from_str(body)
        .map_err(|err| Error::decode(format!("unreadable Upbit response: {err}")))
}

// ---------------------------------------------------------------------------
// Market codes
// ---------------------------------------------------------------------------

/// Builds Upbit's native code for a market.
///
/// Upbit writes the quote asset first, so `KRW-BTC` is BTC priced in KRW. That
/// is the reverse of how most exchanges spell the same pair.
pub(crate) fn native_symbol(market: &Market) -> Result<String> {
    if market.exchange != Exchange::Upbit {
        return Err(Error::invalid_request(
            "market",
            format!("{market} is not an Upbit market"),
        ));
    }
    if market.kind != MarketKind::Spot {
        return Err(Error::unsupported(
            crate::feature::Feature::Markets,
            EXCHANGE,
            "upbit lists spot markets only",
        ));
    }
    check_code_part("base", &market.base)?;
    check_code_part("quote", &market.quote)?;

    Ok(format!("{}-{}", market.quote, market.base))
}

/// Reads one of Upbit's native codes back into a [`Market`].
pub(crate) fn market_from_native_symbol(symbol: &str) -> Result<Market> {
    let Some((quote, base)) = symbol.split_once('-') else {
        return Err(Error::invalid_request(
            "symbol",
            format!("`{symbol}` is not an Upbit market code: expected QUOTE-BASE"),
        ));
    };
    check_code_part("quote", quote)?;
    check_code_part("base", base)?;

    Ok(Market::spot(Exchange::Upbit, base, quote))
}

/// Rejects anything that would change meaning once it reaches a query string.
///
/// Upbit's own codes are uppercase ASCII letters and digits; letting anything
/// else through would let a `&` in a market name append a parameter to the
/// request, and would break the signature that private calls hash the query
/// string into.
fn check_code_part(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::invalid_request(field, "must not be empty"));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(Error::invalid_request(
            field,
            format!(
                "`{value}` is not an Upbit asset code: expected uppercase ASCII letters and digits"
            ),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Raw to domain
// ---------------------------------------------------------------------------

/// Whether Upbit has put its investment warning, 유의 종목, on this listing.
///
/// Two spellings of one designation. Upbit Korea sends
/// `market_event.warning: true`; the other three deployments send
/// `market_warning: "CAUTION"`. That they are the same designation is visible
/// in the payloads: on 2026-07-30 `BTC-ZIL` read `market_event.warning: true`
/// on Upbit Korea and `market_warning: "CAUTION"` on Upbit Indonesia, and every
/// one of Indonesia's six `CAUTION` markets was in Korea's set of eleven
/// warned ones. `BTC-SIGN`, which carried a `caution` criterion in Korea and no
/// warning, read `NONE` on Indonesia, so the older field never reported a
/// caution and [`cautions`] has no fallback to make.
fn warned(raw: &RawMarket) -> bool {
    match &raw.market_event {
        Some(event) => event.warning,
        None => !matches!(raw.market_warning.as_deref(), None | Some("NONE")),
    }
}

/// The 주의 종목 criteria Upbit currently has raised on this listing, under
/// Upbit's own names.
///
/// Empty on the deployments that send no `market_event`, which is the honest
/// answer there: those payloads do not carry the criteria at all.
fn cautions(raw: &RawMarket) -> Vec<String> {
    raw.market_event
        .iter()
        .flat_map(|event| &event.caution)
        .filter(|(_, raised)| **raised)
        .map(|(criterion, _)| criterion.clone())
        .collect()
}

/// Maps a market listing, dropping quote assets the caller did not ask about.
///
/// Only the warning reaches [`MarketStatus`]. A warned market is still fully
/// tradable, so it is not `Paused`, and it is not plainly healthy either, so it
/// is not `Active`. A caution is deliberately not treated the same: it is an
/// automatic reading of how a market is trading this hour, it was carried by
/// 190 of 800 markets on 2026-07-30 against the warning's 11, and the field
/// Upbit is replacing never reported it. Folding it in would make `Unknown` the
/// answer for a quarter of Upbit and bury the eleven designations that matter
/// inside it. The criteria are reachable in full through
/// [`UpbitAdapter::market_events`](super::UpbitAdapter::market_events).
pub(crate) fn market_info(raw: &RawMarket) -> Result<MarketInfo> {
    Ok(MarketInfo {
        market: market_from_native_symbol(&raw.market)?,
        native_symbol: raw.market.clone(),
        status: if warned(raw) {
            MarketStatus::Unknown
        } else {
            MarketStatus::Active
        },
        korean_name: raw.korean_name.clone(),
        english_name: raw.english_name.clone(),
    })
}

/// Reads `/v1/market/all` for the designations [`MarketStatus`] has no room
/// for.
pub(crate) fn market_events(raw: &[RawMarket]) -> Result<Vec<(Market, UpbitMarketEvent)>> {
    raw.iter()
        .map(|entry| {
            Ok((
                market_from_native_symbol(&entry.market)?,
                UpbitMarketEvent {
                    warning: warned(entry),
                    cautions: cautions(entry),
                },
            ))
        })
        .collect()
}

/// Upbit's own identifier for a trade, kept as the digits Upbit sent.
///
/// It is a seventeen-digit integer that outruns an `f64` mantissa, so it is
/// never carried through a numeric type. Upbit
/// [documents](https://global-docs.upbit.com/reference/today-trades-history)
/// it as a basis for deciding whether two records are the same trade, and says
/// it does not guarantee the order of trades, so it identifies and does not
/// sort.
fn trade_id(sequential_id: Option<&Number>) -> Option<String> {
    sequential_id.map(Number::to_string)
}

pub(crate) fn trade(raw: &RawTrade) -> Result<Trade> {
    Ok(Trade {
        market: market_from_native_symbol(&raw.market)?,
        timestamp: millis(raw.timestamp, "timestamp")?,
        price: decimal(&raw.trade_price, "trade_price")?,
        quantity: decimal(&raw.trade_volume, "trade_volume")?,
        taker_side: side(&raw.ask_bid)?,
        id: trade_id(raw.sequential_id.as_ref()),
    })
}

pub(crate) fn stream_trade(raw: &RawStreamTrade) -> Result<Trade> {
    Ok(Trade {
        market: market_from_native_symbol(&raw.code)?,
        // The stream carries two clocks: `timestamp` is when Upbit published the
        // frame, `trade_timestamp` is when the trade matched. The match time is
        // the one comparable against another exchange's trades.
        timestamp: millis(raw.trade_timestamp, "trade_timestamp")?,
        price: decimal(&raw.trade_price, "trade_price")?,
        quantity: decimal(&raw.trade_volume, "trade_volume")?,
        taker_side: side(&raw.ask_bid)?,
        id: trade_id(raw.sequential_id.as_ref()),
    })
}

pub(crate) fn order_book(raw: &RawOrderBook) -> Result<OrderBook> {
    book(&raw.market, raw.timestamp, &raw.orderbook_units)
}

pub(crate) fn stream_order_book(raw: &RawStreamOrderBook) -> Result<OrderBook> {
    book(&raw.code, raw.timestamp, &raw.orderbook_units)
}

fn book(symbol: &str, timestamp: i64, units: &[RawOrderBookUnit]) -> Result<OrderBook> {
    let mut bids = Vec::with_capacity(units.len());
    let mut asks = Vec::with_capacity(units.len());

    for unit in units {
        bids.push(Level {
            price: decimal(&unit.bid_price, "bid_price")?,
            quantity: decimal(&unit.bid_size, "bid_size")?,
        });
        asks.push(Level {
            price: decimal(&unit.ask_price, "ask_price")?,
            quantity: decimal(&unit.ask_size, "ask_size")?,
        });
    }

    // Upbit ships its units best-first already, but `OrderBook`'s ordering is a
    // guarantee to the caller and cheap to enforce, so it is enforced.
    bids.sort_by(|left, right| right.price.cmp(&left.price));
    asks.sort_by(|left, right| left.price.cmp(&right.price));

    Ok(OrderBook {
        market: market_from_native_symbol(symbol)?,
        timestamp: millis(timestamp, "timestamp")?,
        bids,
        asks,
    })
}

pub(crate) fn ticker(raw: &RawTicker) -> Result<Ticker> {
    Ok(Ticker {
        market: market_from_native_symbol(&raw.market)?,
        timestamp: millis(raw.timestamp, "timestamp")?,
        last_trade_time: Some(millis(raw.trade_timestamp, "trade_timestamp")?),
        last_price: decimal(&raw.trade_price, "trade_price")?,
        // `change_price` and `change_rate` are unsigned on Upbit; the signed
        // pair is the only one that says which way the market moved.
        change: Some(decimal(&raw.signed_change_price, "signed_change_price")?),
        change_rate: Some(decimal(&raw.signed_change_rate, "signed_change_rate")?),
        high: Some(decimal(&raw.high_price, "high_price")?),
        low: Some(decimal(&raw.low_price, "low_price")?),
        // The plain `acc_trade_*` fields cover the current UTC day, not a
        // rolling window; only the `_24h` pair matches what `Ticker` promises.
        volume: Some(decimal(&raw.acc_trade_volume_24h, "acc_trade_volume_24h")?),
        quote_volume: Some(decimal(&raw.acc_trade_price_24h, "acc_trade_price_24h")?),
    })
}

pub(crate) fn stream_ticker(raw: &RawStreamTicker) -> Result<Ticker> {
    Ok(Ticker {
        market: market_from_native_symbol(&raw.code)?,
        timestamp: millis(raw.timestamp, "timestamp")?,
        last_trade_time: Some(millis(raw.trade_timestamp, "trade_timestamp")?),
        last_price: decimal(&raw.trade_price, "trade_price")?,
        change: Some(decimal(&raw.signed_change_price, "signed_change_price")?),
        change_rate: Some(decimal(&raw.signed_change_rate, "signed_change_rate")?),
        high: Some(decimal(&raw.high_price, "high_price")?),
        low: Some(decimal(&raw.low_price, "low_price")?),
        volume: Some(decimal(&raw.acc_trade_volume_24h, "acc_trade_volume_24h")?),
        quote_volume: Some(decimal(&raw.acc_trade_price_24h, "acc_trade_price_24h")?),
    })
}

/// Maps a REST candle at the interval the request asked for.
///
/// `now` decides [`Candle::closed`]: Upbit serves the interval in progress
/// alongside the finished ones and marks neither, so the only thing that
/// separates them is whether the window has ended yet.
pub(crate) fn candle(raw: &RawCandle, interval: Interval, now: Timestamp) -> Result<Candle> {
    // A minute candle names its own unit. If it disagrees with the endpoint we
    // called, something changed on Upbit's side and the interval label would be
    // a lie.
    if let (Some(unit), Some(expected)) = (raw.unit, minute_unit(interval))
        && unit != expected
    {
        return Err(Error::decode(format!(
            "asked upbit for {expected}-minute candles and got {unit}-minute candles"
        )));
    }

    let open_time = candle_open_time(&raw.candle_date_time_utc)?;
    let closed = has_ended(open_time, interval, now);

    build_candle(
        &raw.market,
        interval,
        open_time,
        closed,
        CandlePrices {
            open: &raw.opening_price,
            high: &raw.high_price,
            low: &raw.low_price,
            close: &raw.trade_price,
            volume: &raw.candle_acc_trade_volume,
            quote_volume: &raw.candle_acc_trade_price,
        },
    )
}

/// Whether the window a REST candle covers has already ended.
///
/// Upbit marks no candle finished, so on a REST response the only thing
/// separating a settled candle from the one still forming is the clock, and the
/// reading machine's clock is the one available. [`Interval::advance`] says
/// where the window ends, at every interval
/// including [`Interval::Month1`]: the monthly candle for February is settled on
/// 1 March and open before then, which is the same rule the other intervals get
/// rather than a special case for the one with no fixed length.
fn has_ended(open_time: Timestamp, interval: Interval, now: Timestamp) -> bool {
    interval
        .advance(open_time, 1)
        .is_some_and(|end_of_window| end_of_window <= now)
}

/// Maps a streamed candle, always as a forming one.
///
/// No single Upbit candle frame says its window has ended, and no clock reading
/// of one frame can say it either. Upbit stops publishing a window the instant
/// the next one opens, so a frame's own `timestamp` never reaches
/// `open_time + interval`: on `candle.1m` for `KRW-BTC` the last frame of a
/// minute lands around `:55` to `:59`, and the next frame already carries the
/// next `candle_date_time_utc`. Reading the local clock instead answers a
/// different question, because decoding happens when the consumer polls and
/// under [`Overflow::Backpressure`](crate::Overflow::Backpressure) that can be
/// minutes later, which would mark a bar settled that was mid-window when Upbit
/// sent it.
///
/// [`Candle::closed`] is therefore decided one level up, by
/// [`super::stream::Decoder`], from the one thing Upbit does publish: the frame
/// that opens the next window. See [`candle`] for the REST path, where the
/// clock is the right instrument because a REST response is a set of finished
/// windows plus at most one running one.
pub(crate) fn stream_candle(raw: &RawStreamCandle, interval: Interval) -> Result<Candle> {
    let open_time = candle_open_time(&raw.candle_date_time_utc)?;

    build_candle(
        &raw.code,
        interval,
        open_time,
        false,
        CandlePrices {
            open: &raw.opening_price,
            high: &raw.high_price,
            low: &raw.low_price,
            close: &raw.trade_price,
            volume: &raw.candle_acc_trade_volume,
            quote_volume: &raw.candle_acc_trade_price,
        },
    )
}

struct CandlePrices<'a> {
    open: &'a Number,
    high: &'a Number,
    low: &'a Number,
    close: &'a Number,
    volume: &'a Number,
    quote_volume: &'a Number,
}

fn build_candle(
    symbol: &str,
    interval: Interval,
    open_time: Timestamp,
    closed: bool,
    prices: CandlePrices<'_>,
) -> Result<Candle> {
    Ok(Candle {
        market: market_from_native_symbol(symbol)?,
        interval,
        open_time,
        open: decimal(prices.open, "opening_price")?,
        high: decimal(prices.high, "high_price")?,
        low: decimal(prices.low, "low_price")?,
        close: decimal(prices.close, "trade_price")?,
        volume: decimal(prices.volume, "candle_acc_trade_volume")?,
        quote_volume: Some(decimal(prices.quote_volume, "candle_acc_trade_price")?),
        closed,
    })
}

/// The `unit` a minute-candle payload should carry for an interval, or `None`
/// for the second, day, week and month endpoints, which have no unit.
pub(crate) fn minute_unit(interval: Interval) -> Option<u32> {
    match interval {
        Interval::Min1 => Some(1),
        Interval::Min3 => Some(3),
        Interval::Min5 => Some(5),
        Interval::Min15 => Some(15),
        Interval::Min30 => Some(30),
        Interval::Hour1 => Some(60),
        Interval::Hour4 => Some(240),
        _ => None,
    }
}

pub(crate) fn balance(raw: &RawBalance) -> Result<Balance> {
    Ok(Balance {
        asset: raw.currency.to_ascii_uppercase(),
        available: decimal_text(&raw.balance, "balance")?,
        locked: decimal_text(&raw.locked, "locked")?,
    })
}

pub(crate) fn stream_balance(raw: &RawStreamAsset) -> Result<Balance> {
    Ok(Balance {
        asset: raw.currency.to_ascii_uppercase(),
        available: decimal(&raw.balance, "balance")?,
        locked: decimal(&raw.locked, "locked")?,
    })
}

pub(crate) fn order(raw: &RawOrder) -> Result<Order> {
    let filled = decimal_text(&raw.executed_volume, "executed_volume")?;
    let remaining = decimal_text(&raw.remaining_volume, "remaining_volume")?;

    Ok(Order {
        id: raw.uuid.clone(),
        market: market_from_native_symbol(&raw.market)?,
        side: side(&raw.side)?,
        status: order_status(&raw.state, filled, remaining),
        filled_quantity: filled,
        remaining_quantity: remaining,
        price: raw
            .price
            .as_deref()
            .map(|price| decimal_text(price, "price"))
            .transpose()?,
        created_at: raw.created_at.as_deref().map(created_at).transpose()?,
    })
}

pub(crate) fn stream_order(raw: &RawStreamOrder) -> Result<Order> {
    let filled = decimal(&raw.executed_volume, "executed_volume")?;
    let remaining = decimal(&raw.remaining_volume, "remaining_volume")?;

    Ok(Order {
        id: raw.uuid.clone(),
        market: market_from_native_symbol(&raw.code)?,
        side: side(&raw.ask_bid)?,
        status: order_status(&raw.state, filled, remaining),
        filled_quantity: filled,
        remaining_quantity: remaining,
        price: raw
            .price
            .as_ref()
            .map(|price| decimal(price, "price"))
            .transpose()?,
        created_at: Some(millis(raw.order_timestamp, "order_timestamp")?),
    })
}

/// Reads `created_at`, which REST orders carry as an offset datetime in the
/// region's local zone.
fn created_at(raw: &str) -> Result<Timestamp> {
    DateTime::parse_from_rfc3339(raw)
        .map(|at| Timestamp::from_secs(at.timestamp()))
        .map_err(|err| {
            Error::decode(format!(
                "`created_at` is not an RFC 3339 datetime: {raw} ({err})"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Four entries of `GET https://api.upbit.com/v1/market/all?is_details=true`,
    /// captured on 2026-07-30 and pasted unedited.
    ///
    /// That call answered for 800 markets. None of them carried
    /// `market_warning`; all 800 carried `market_event`. Eleven had
    /// `warning: true`, and 190 had at least one `caution` criterion raised, 175
    /// of those `GLOBAL_PRICE_DIFFERENCES` alone. Only four markets had both.
    /// The four below are one of each combination, in that order: neither,
    /// warning only, both, caution only.
    // https://global-docs.upbit.com/reference/list-trading-pairs.md
    const MARKET_LIST: &str = r#"[
      {"market":"KRW-BTC","korean_name":"비트코인","english_name":"Bitcoin",
       "market_event":{"warning":false,"caution":{"PRICE_FLUCTUATIONS":false,
        "TRADING_VOLUME_SOARING":false,"DEPOSIT_AMOUNT_SOARING":false,
        "GLOBAL_PRICE_DIFFERENCES":false,"CONCENTRATION_OF_SMALL_ACCOUNTS":false}}},
      {"market":"KRW-AERGO","korean_name":"아르고","english_name":"Aergo",
       "market_event":{"warning":true,"caution":{"PRICE_FLUCTUATIONS":false,
        "TRADING_VOLUME_SOARING":false,"DEPOSIT_AMOUNT_SOARING":false,
        "GLOBAL_PRICE_DIFFERENCES":false,"CONCENTRATION_OF_SMALL_ACCOUNTS":false}}},
      {"market":"KRW-ZIL","korean_name":"질리카","english_name":"Zilliqa",
       "market_event":{"warning":true,"caution":{"PRICE_FLUCTUATIONS":false,
        "TRADING_VOLUME_SOARING":false,"DEPOSIT_AMOUNT_SOARING":false,
        "GLOBAL_PRICE_DIFFERENCES":true,"CONCENTRATION_OF_SMALL_ACCOUNTS":false}}},
      {"market":"BTC-SIGN","korean_name":"사인","english_name":"Sign",
       "market_event":{"warning":false,"caution":{"PRICE_FLUCTUATIONS":false,
        "TRADING_VOLUME_SOARING":false,"DEPOSIT_AMOUNT_SOARING":false,
        "GLOBAL_PRICE_DIFFERENCES":true,"CONCENTRATION_OF_SMALL_ACCOUNTS":false}}}
    ]"#;

    /// The same two markets from
    /// `GET https://id-api.upbit.com/v1/market/all?is_details=true`, captured
    /// the same day and pasted unedited.
    ///
    /// Upbit Indonesia answered for 447 markets. Not one carried
    /// `market_event`; all 447 carried `market_warning`, six of them `CAUTION`.
    /// None carried `korean_name`. The two below are `BTC-ZIL`, which Upbit
    /// Korea reported `warning: true` in [`MARKET_LIST`], and `BTC-SIGN`, which
    /// Korea reported as a caution with no warning. That the first is `CAUTION`
    /// here and the second `NONE` is what identifies the older field with
    /// `market_event.warning` rather than with `market_event.caution`.
    const MARKET_LIST_LEGACY_FIELD: &str = r#"[
      {"market":"BTC-ZIL","english_name":"Zilliqa","market_warning":"CAUTION"},
      {"market":"BTC-SIGN","english_name":"Sign","market_warning":"NONE"}
    ]"#;

    /// Two ticks of
    /// `GET https://api.upbit.com/v1/trades/ticks?market=KRW-BTC&count=2`,
    /// captured on 2026-07-30 and pasted unedited.
    // https://global-docs.upbit.com/reference/list-pair-trades.md
    const TRADES: &str = r#"[
      {"market":"KRW-BTC","trade_date_utc":"2026-07-30","trade_time_utc":"07:41:00",
       "timestamp":1785397260660,"trade_price":91200000.0,"trade_volume":0.00010971,
       "prev_closing_price":91424000.0,"change_price":-224000.0,"ask_bid":"BID",
       "sequential_id":17853972606600000},
      {"market":"KRW-BTC","trade_date_utc":"2026-07-30","trade_time_utc":"07:41:00",
       "timestamp":1785397260652,"trade_price":91200000.0,"trade_volume":5.485e-05,
       "prev_closing_price":91424000.0,"change_price":-224000.0,"ask_bid":"BID",
       "sequential_id":17853972606520000}
    ]"#;

    // https://global-docs.upbit.com/reference/list-orderbooks.md
    const ORDER_BOOK: &str = r#"[
      {
        "market": "KRW-BTC",
        "timestamp": 1781917323000,
        "total_ask_size": 1.5,
        "total_bid_size": 2.5,
        "orderbook_units": [
          {
            "ask_price": 100010000.0,
            "bid_price": 100000000.0,
            "ask_size": 0.5,
            "bid_size": 0.6
          },
          {
            "ask_price": 100020000.0,
            "bid_price": 99990000.0,
            "ask_size": 0.4,
            "bid_size": 0.7
          }
        ],
        "level": 0
      }
    ]"#;

    /// `GET https://api.upbit.com/v1/ticker?markets=KRW-BTC`, captured on
    /// 2026-07-30 and pasted unedited.
    ///
    /// A falling market, so `change` is `FALL` and `signed_change_price` is
    /// negative while `change_price` is not.
    // https://global-docs.upbit.com/reference/list-tickers.md
    const TICKER: &str = r#"[
      {
        "market": "KRW-BTC",
        "trade_date": "20260730",
        "trade_time": "074100",
        "trade_date_kst": "20260730",
        "trade_time_kst": "164100",
        "trade_timestamp": 1785397260652,
        "opening_price": 91374000.0,
        "high_price": 92119000.0,
        "low_price": 90888000.0,
        "trade_price": 91200000.0,
        "prev_closing_price": 91424000.0,
        "change": "FALL",
        "change_price": 224000.0,
        "change_rate": 0.0024501225,
        "signed_change_price": -224000.0,
        "signed_change_rate": -0.0024501225,
        "trade_volume": 0.00016456,
        "acc_trade_price": 27800806282.39862,
        "acc_trade_price_24h": 74674126733.71954,
        "acc_trade_volume": 304.11940843,
        "acc_trade_volume_24h": 814.34496604,
        "highest_52_week_price": 179869000.0,
        "highest_52_week_date": "2025-10-09",
        "lowest_52_week_price": 88770000.0,
        "lowest_52_week_date": "2026-07-01",
        "timestamp": 1785397260707
      }
    ]"#;

    // https://global-docs.upbit.com/reference/list-candles-minutes.md
    const MINUTE_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-06-20T01:02:00",
        "opening_price": 99000000.0,
        "high_price": 101000000.0,
        "low_price": 98000000.0,
        "trade_price": 100000000.0,
        "timestamp": 1781917323000,
        "candle_acc_trade_price": 1000000.0,
        "candle_acc_trade_volume": 0.01,
        "unit": 1
      }
    ]"#;

    // https://global-docs.upbit.com/reference/list-candles-days.md
    const DAY_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-06-20T00:00:00",
        "opening_price": 99000000.0,
        "high_price": 101000000.0,
        "low_price": 98000000.0,
        "trade_price": 100000000.0,
        "prev_closing_price": 98500000.0,
        "change_price": 1500000.0,
        "change_rate": 0.0152284263,
        "timestamp": 1781917323000,
        "candle_acc_trade_price": 1000000.0,
        "candle_acc_trade_volume": 0.01
      }
    ]"#;

    // https://global-docs.upbit.com/reference/list-candles-weeks.md
    const WEEK_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-06-15T00:00:00",
        "opening_price": 99000000.0,
        "high_price": 101000000.0,
        "low_price": 98000000.0,
        "trade_price": 100000000.0,
        "timestamp": 1781481600000,
        "candle_acc_trade_price": 39991.1838817,
        "candle_acc_trade_volume": 0.26276451,
        "first_day_of_period": "2026-06-15"
      }
    ]"#;

    // https://global-docs.upbit.com/reference/websocket-trade.md
    const STREAM_TRADE: &str = r#"{
      "type": "trade",
      "code": "KRW-BTC",
      "timestamp": 1696585056910,
      "trade_date": "2023-10-06",
      "trade_time": "09:37:36",
      "trade_timestamp": 1696585056846,
      "trade_price": 37625,
      "trade_volume": 8.428e-05,
      "ask_bid": "ASK",
      "prev_closing_price": 37296,
      "change": "RISE",
      "change_price": 329,
      "best_ask_price": 32293000,
      "best_ask_size": 0.04414411,
      "best_bid_price": 32291000,
      "best_bid_size": 0.01202163
    }"#;

    // https://global-docs.upbit.com/reference/websocket-myasset.md
    const STREAM_ASSETS: &str = r#"{
      "type": "myAsset",
      "asset_uuid": "00000000-0000-0000-0000-000000000003",
      "assets": [
        {
          "currency": "KRW",
          "balance": 1386929.37231066771348207123,
          "locked": 10329.670127489597585685
        }
      ],
      "asset_timestamp": 1781917323000,
      "timestamp": 1781917323001
    }"#;

    // https://global-docs.upbit.com/reference/open-order
    const OPEN_ORDERS: &str = r#"[
      {
        "uuid": "ac2dc2a3-fce9-40a2-a4f6-5987c25c438f",
        "side": "ask",
        "ord_type": "limit",
        "price": "125000000",
        "state": "wait",
        "market": "KRW-BTC",
        "created_at": "2024-06-13T10:28:36+09:00",
        "volume": "0.0001",
        "remaining_volume": "0.0001",
        "reserved_fee": "0",
        "remaining_fee": "0",
        "paid_fee": "0",
        "locked": "0.0001",
        "executed_volume": "0",
        "trades_count": 0
      }
    ]"#;

    // https://global-docs.upbit.com/reference/overall-account-inquiry
    const ACCOUNTS: &str = r#"[
      {
        "currency": "krw",
        "balance": "1000000.0",
        "locked": "0.0",
        "avg_buy_price": "0",
        "avg_buy_price_modified": false,
        "unit_currency": "KRW"
      }
    ]"#;

    // No URL: Upbit no longer publishes an error-code reference, and there is
    // no replacement page to cite. The shape below is the one every failing
    // Upbit response carries, and is quoted from a live 401 rather than from
    // documentation.
    const ERROR_BODY: &str =
        r#"{"error":{"name":"invalid_access_key","message":"Invalid access key"}}"#;

    fn decimal_of(text: &str) -> Decimal {
        decimal_text(text, "test").expect("test literal is a decimal")
    }

    #[test]
    fn a_market_round_trips_through_upbits_quote_first_code() {
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

        let symbol = native_symbol(&market).expect("spot markets have a code");
        assert_eq!(symbol, "KRW-BTC");
        assert_eq!(
            market_from_native_symbol(&symbol).expect("its own code reads back"),
            market
        );
    }

    #[test]
    fn the_two_directions_agree_on_every_shape_upbit_lists() {
        for symbol in ["KRW-BTC", "BTC-ETH", "USDT-XRP", "KRW-1INCH"] {
            let market = market_from_native_symbol(symbol).expect("a listed code");
            assert_eq!(native_symbol(&market).expect("and back"), symbol);
        }
    }

    #[test]
    fn quote_and_base_are_not_interchangeable() {
        // `BTC-ETH` is ETH priced in BTC, so reading it as BTC/ETH would invert
        // every price on the market.
        let market = market_from_native_symbol("BTC-ETH").expect("a listed code");

        assert_eq!(market.base, "ETH");
        assert_eq!(market.quote, "BTC");
    }

    #[test]
    fn a_market_code_that_could_smuggle_a_query_parameter_is_rejected() {
        let injected = Market::spot(Exchange::Upbit, "BTC&count=500", "KRW");

        assert!(matches!(
            native_symbol(&injected),
            Err(Error::InvalidRequest { field: "base", .. })
        ));
        assert!(matches!(
            market_from_native_symbol("KRW-BTC&count=500"),
            Err(Error::InvalidRequest { field: "base", .. })
        ));
    }

    #[test]
    fn a_code_without_a_separator_is_not_an_upbit_market() {
        assert!(matches!(
            market_from_native_symbol("BTCKRW"),
            Err(Error::InvalidRequest {
                field: "symbol",
                ..
            })
        ));
        assert!(matches!(
            market_from_native_symbol("KRW-"),
            Err(Error::InvalidRequest { field: "base", .. })
        ));
    }

    #[test]
    fn another_exchanges_market_never_gets_an_upbit_code() {
        let elsewhere = Market::spot(Exchange::Bithumb, "BTC", "KRW");
        let perpetual = Market::perpetual(Exchange::Upbit, "BTC", "KRW");

        assert!(matches!(
            native_symbol(&elsewhere),
            Err(Error::InvalidRequest {
                field: "market",
                ..
            })
        ));
        assert!(matches!(
            native_symbol(&perpetual),
            Err(Error::Unsupported { .. })
        ));
    }

    #[test]
    fn decimals_keep_the_digits_upbit_sent() {
        // 1386929.37231066771348207123 is not representable in f64: routing it
        // through one would land on 1386929.3723106677, losing ten digits.
        let assets: RawStreamAssets = json(STREAM_ASSETS).expect("official myAsset payload");
        let balance = stream_balance(&assets.assets[0]).expect("a balance");

        assert_eq!(
            balance.available.to_string(),
            "1386929.37231066771348207123"
        );
        assert_eq!(balance.locked.to_string(), "10329.670127489597585685");
        assert_eq!(balance.asset, "KRW");
    }

    #[test]
    fn scientific_notation_is_read_as_the_size_it_denotes() {
        // Upbit prints small stream volumes with an exponent.
        let raw: RawStreamTrade = json(STREAM_TRADE).expect("official trade frame");
        let trade = stream_trade(&raw).expect("a trade");

        assert_eq!(trade.quantity, decimal_of("0.00008428"));
        assert_eq!(trade.price, decimal_of("37625"));
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_696_585_056_846));
    }

    #[test]
    fn a_number_too_precise_to_hold_is_a_decode_error_not_a_rounded_price() {
        // Twenty-nine decimal places is one past what `Decimal` carries.
        let error = decimal_text("0.000000000000000000000000000001", "price").unwrap_err();

        assert!(matches!(error, Error::Decode { .. }));
    }

    #[test]
    fn a_trade_ticks_ask_bid_names_the_taker() {
        let raw: Vec<RawTrade> = json(TRADES).expect("a live trades payload");
        let trade = trade(&raw[0]).expect("a trade");

        // `BID` means the taker lifted an ask.
        assert_eq!(trade.taker_side, Side::Buy);
        assert_eq!(trade.market, Market::spot(Exchange::Upbit, "BTC", "KRW"));
        assert_eq!(trade.price, decimal_of("91200000.0"));
        assert_eq!(trade.quantity, decimal_of("0.00010971"));
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_785_397_260_660));
        // Seventeen digits, kept as the digits Upbit sent rather than carried
        // through a numeric type that cannot hold them.
        assert_eq!(trade.id.as_deref(), Some("17853972606600000"));
    }

    /// Two consecutive ticks of `GET /v1/trades/ticks?market=KRW-BTC&count=200`,
    /// captured on 2026-07-30 and pasted unedited.
    ///
    /// They agree on market, timestamp, price, quantity and side, and differ
    /// only in `sequential_id`. Fifteen of that page's two hundred ticks
    /// collided the same way, so `(timestamp, price, quantity, taker_side)` does
    /// not identify an Upbit trade and `sequential_id` is the only field that
    /// does.
    const LIVE_COLLIDING_TICKS: &str = r#"[
      {"market":"KRW-BTC","trade_date_utc":"2026-07-30","trade_time_utc":"07:04:01",
       "timestamp":1785395041124,"trade_price":91368000.0,"trade_volume":0.00010945,
       "prev_closing_price":91424000.0,"change_price":-56000.0,"ask_bid":"BID",
       "sequential_id":17853950411240001},
      {"market":"KRW-BTC","trade_date_utc":"2026-07-30","trade_time_utc":"07:04:01",
       "timestamp":1785395041124,"trade_price":91368000.0,"trade_volume":0.00010945,
       "prev_closing_price":91424000.0,"change_price":-56000.0,"ask_bid":"BID",
       "sequential_id":17853950411240000}
    ]"#;

    /// One trade as both paths reported it, captured on 2026-07-30 from
    /// `GET /v1/trades/ticks` and from `wss://api.upbit.com/websocket/v1` in the
    /// same window, and pasted unedited. Of the 170 stream frames captured, 169
    /// matched a tick on that page by `sequential_id`.
    const LIVE_TICK_OF_ONE_TRADE: &str = r#"[
      {"market":"KRW-BTC","trade_date_utc":"2026-07-30","trade_time_utc":"07:04:03",
       "timestamp":1785395043779,"trade_price":91361000.0,"trade_volume":0.01275549,
       "prev_closing_price":91424000.0,"change_price":-63000.0,"ask_bid":"ASK",
       "sequential_id":17853950437790001}
    ]"#;

    const LIVE_STREAM_FRAME_OF_ONE_TRADE: &str = r#"{
      "type":"trade","code":"KRW-BTC","timestamp":1785395043831,
      "trade_date":"2026-07-30","trade_time":"07:04:03","trade_timestamp":1785395043779,
      "trade_price":91361000.0,"trade_volume":0.01275549,"ask_bid":"ASK",
      "prev_closing_price":91424000.0,"change":"FALL","change_price":63000.0,
      "sequential_id":17853950437790001,"best_ask_price":91370000,"best_ask_size":0.0070459,
      "best_bid_price":91361000,"best_bid_size":0.00196362,"stream_type":"REALTIME"}"#;

    #[test]
    fn trades_alike_in_every_other_field_are_told_apart_by_the_id_upbit_sends() {
        let raw: Vec<RawTrade> = json(LIVE_COLLIDING_TICKS).expect("captured ticks");
        let first = trade(&raw[0]).expect("a trade");
        let second = trade(&raw[1]).expect("the tick after it");

        assert_eq!(first.timestamp, second.timestamp);
        assert_eq!(first.price, second.price);
        assert_eq!(first.quantity, second.quantity);
        assert_eq!(first.taker_side, second.taker_side);
        // So everything but the id says these are one trade, and they are two.
        assert_eq!(first.id.as_deref(), Some("17853950411240001"));
        assert_eq!(second.id.as_deref(), Some("17853950411240000"));
    }

    #[test]
    fn one_trade_carries_one_id_whichever_path_it_arrived_on() {
        let raw: Vec<RawTrade> = json(LIVE_TICK_OF_ONE_TRADE).expect("a captured tick");
        let over_rest = trade(&raw[0]).expect("a trade over REST");
        let raw: RawStreamTrade = json(LIVE_STREAM_FRAME_OF_ONE_TRADE).expect("a captured frame");
        let over_stream = stream_trade(&raw).expect("the same trade on the stream");

        assert_eq!(over_rest.id.as_deref(), Some("17853950437790001"));
        assert_eq!(over_rest.id, over_stream.id);
        // The two paths agree on the trade itself, not only on its name.
        assert_eq!(over_rest.timestamp, over_stream.timestamp);
        assert_eq!(over_rest.price, over_stream.price);
        assert_eq!(over_rest.quantity, over_stream.quantity);
        assert_eq!(over_rest.taker_side, over_stream.taker_side);
    }

    #[test]
    fn a_book_comes_back_best_first_on_both_sides() {
        let raw: Vec<RawOrderBook> = json(ORDER_BOOK).expect("official orderbook payload");
        let book = order_book(&raw[0]).expect("a book");

        assert_eq!(
            book.bids
                .iter()
                .map(|level| level.price)
                .collect::<Vec<_>>(),
            vec![decimal_of("100000000.0"), decimal_of("99990000.0")]
        );
        assert_eq!(
            book.asks
                .iter()
                .map(|level| level.price)
                .collect::<Vec<_>>(),
            vec![decimal_of("100010000.0"), decimal_of("100020000.0")]
        );
        assert_eq!(book.spread(), Some(decimal_of("10000.0")));
    }

    #[test]
    fn book_ordering_is_enforced_rather_than_assumed() {
        let mut raw: Vec<RawOrderBook> = json(ORDER_BOOK).expect("official orderbook payload");
        raw[0].orderbook_units.reverse();

        let book = order_book(&raw[0]).expect("a book");

        assert_eq!(
            book.best_bid().expect("a bid").price,
            decimal_of("100000000.0")
        );
        assert_eq!(
            book.best_ask().expect("an ask").price,
            decimal_of("100010000.0")
        );
    }

    #[test]
    fn a_ticker_reports_the_signed_change_and_the_rolling_window() {
        let raw: Vec<RawTicker> = json(TICKER).expect("a live ticker payload");
        let ticker = ticker(&raw[0]).expect("a ticker");

        assert_eq!(ticker.last_price, decimal_of("91200000.0"));
        // `signed_change_price`, not the unsigned `change_price` of 224000.0
        // that sits beside it on a falling market.
        assert_eq!(ticker.change, Some(decimal_of("-224000.0")));
        assert_eq!(ticker.change_rate, Some(decimal_of("-0.0024501225")));
        // The 24-hour figures, not the same-day `acc_trade_volume` of
        // 304.11940843 and `acc_trade_price` of 27800806282.39862.
        assert_eq!(ticker.volume, Some(decimal_of("814.34496604")));
        assert_eq!(ticker.quote_volume, Some(decimal_of("74674126733.71954")));
    }

    #[test]
    fn a_candle_opens_at_the_start_of_its_window() {
        let raw: Vec<RawCandle> = json(MINUTE_CANDLES).expect("official minute candle payload");
        let now = Timestamp::from_secs(1_781_917_400);
        let candle = candle(&raw[0], Interval::Min1, now).expect("a candle");

        assert_eq!(candle.open_time, Timestamp::from_secs(1_781_917_320));
        assert_eq!(candle.open, decimal_of("99000000.0"));
        assert_eq!(candle.close, decimal_of("100000000.0"));
        assert_eq!(candle.quote_volume, Some(decimal_of("1000000.0")));
        assert!(candle.closed);
    }

    #[test]
    fn the_candle_still_forming_is_the_only_open_one() {
        let raw: Vec<RawCandle> = json(MINUTE_CANDLES).expect("official minute candle payload");
        // One second into the window the candle covers.
        let now = Timestamp::from_secs(1_781_917_321);

        assert!(
            !candle(&raw[0], Interval::Min1, now)
                .expect("a candle")
                .closed
        );
    }

    /// The last two frames Upbit sent for the 06:51 minute of 2026-07-30 on
    /// `candle.1m` `KRW-BTC`, captured from `wss://api.upbit.com/websocket/v1`
    /// and pasted unedited.
    ///
    /// The second is the last frame that minute ever got. It is stamped
    /// 06:51:59.691, 309ms short of the 06:52:00.000 the minute ends at, and
    /// the frame after it already carries `2026-07-30T06:52:00`. Across four
    /// complete window transitions in that capture, no frame's own `timestamp`
    /// reached its window's end, so no clock reading of a single frame can
    /// report a settled bar here.
    const LIVE_STREAM_FRAMES: &str = r#"[
      {"code":"KRW-BTC","candle_date_time_utc":"2026-07-30T06:51:00",
       "opening_price":91180000.00000000,"high_price":91180000.00000000,
       "low_price":91180000.00000000,"trade_price":91180000.00000000,
       "candle_acc_trade_price":6818155.91840000,"candle_acc_trade_volume":0.07477688,
       "timestamp":1785394318967,"stream_type":"REALTIME"},
      {"code":"KRW-BTC","candle_date_time_utc":"2026-07-30T06:51:00",
       "opening_price":91180000.00000000,"high_price":91180000.00000000,
       "low_price":91180000.00000000,"trade_price":91180000.00000000,
       "candle_acc_trade_price":7467242.63160000,"candle_acc_trade_volume":0.08189562,
       "timestamp":1785394319691,"stream_type":"REALTIME"}
    ]"#;

    #[test]
    fn every_streamed_candle_frame_reads_as_a_forming_bar() {
        // Including the last frame a window ever gets, which is why nothing here
        // asks a clock: it would answer "still forming" for every frame Upbit
        // sends, so the answer would be true by accident rather than by
        // construction. `Candle::closed` is settled one level up, by
        // `super::stream::Decoder`, off the frame that opens the next window.
        let frames: Vec<RawStreamCandle> = json(LIVE_STREAM_FRAMES).expect("captured frames");

        for raw in &frames {
            let candle = stream_candle(raw, Interval::Min1).expect("a candle");

            assert!(!candle.closed);
            assert_eq!(candle.open_time, Timestamp::from_secs(1_785_394_260));
        }
        // And the figures are the ones Upbit sent, growing frame by frame while
        // the window is open.
        let last = stream_candle(&frames[1], Interval::Min1).expect("the last frame of 06:51");
        assert_eq!(last.volume, decimal_of("0.08189562"));
    }

    #[test]
    fn a_month_candle_is_settled_by_the_calendar_and_not_left_open_forever() {
        // A monthly candle for June 2025 is settled on 1 July and open before
        // then. A month has no fixed length, and treating that as "no answer"
        // leaves every monthly candle of a ten-year history reported unsettled,
        // so `last_settled_close`, the idiom `Candle` documents, answers `None`
        // for a market with ten years of history.
        let june = Timestamp::from_secs(1_748_736_000); // 2025-06-01T00:00:00Z
        let july = Timestamp::from_secs(1_751_328_000); // 2025-07-01T00:00:00Z

        let a_nanosecond_before_july = Timestamp::from_nanos(july.as_nanos() - 1);

        assert!(!has_ended(june, Interval::Month1, a_nanosecond_before_july));
        assert!(has_ended(june, Interval::Month1, july));
        // And the length is the month's own: 30 days for June, so a 31-day step
        // would call it open a day too long and a 28-day one a day too early.
        assert_eq!(july.as_secs() - june.as_secs(), 30 * 86_400);
    }

    #[test]
    fn day_and_week_candles_read_through_the_same_shape() {
        let now = Timestamp::from_secs(1_800_000_000);
        let day: Vec<RawCandle> = json(DAY_CANDLES).expect("official day candle payload");
        let week: Vec<RawCandle> = json(WEEK_CANDLES).expect("official week candle payload");

        let day = candle(&day[0], Interval::Day1, now).expect("a day candle");
        let week = candle(&week[0], Interval::Week1, now).expect("a week candle");

        assert_eq!(day.open_time, Timestamp::from_secs(1_781_913_600));
        assert_eq!(week.open_time, Timestamp::from_secs(1_781_481_600));
        assert_eq!(week.volume, decimal_of("0.26276451"));
        assert!(day.closed && week.closed);
    }

    #[test]
    fn a_minute_candle_whose_unit_contradicts_the_endpoint_is_rejected() {
        let raw: Vec<RawCandle> = json(MINUTE_CANDLES).expect("official minute candle payload");
        let error =
            candle(&raw[0], Interval::Min5, Timestamp::from_secs(1_800_000_000)).unwrap_err();

        assert!(matches!(error, Error::Decode { .. }));
    }

    #[test]
    fn a_listing_warning_stops_the_market_being_reported_as_plainly_active() {
        let raw: Vec<RawMarket> = json(MARKET_LIST).expect("a live market list payload");
        let status = |entry: &RawMarket| market_info(entry).expect("a listing").status;

        // KRW-BTC: neither designation.
        assert_eq!(status(&raw[0]), MarketStatus::Active);
        // KRW-AERGO: warned, and that is what stops it reading `Active`.
        assert_eq!(status(&raw[1]), MarketStatus::Unknown);
        // KRW-ZIL: warned and cautioned, which reads the same as warned.
        assert_eq!(status(&raw[2]), MarketStatus::Unknown);

        let warned = market_info(&raw[1]).expect("a listing");
        assert_eq!(warned.native_symbol, "KRW-AERGO");
        assert_eq!(warned.english_name.as_deref(), Some("Aergo"));
        assert_eq!(warned.korean_name.as_deref(), Some("아르고"));
    }

    #[test]
    fn a_caution_on_its_own_leaves_the_market_active_and_stays_readable() {
        // BTC-SIGN was cautioned for `GLOBAL_PRICE_DIFFERENCES` and not warned.
        // 190 of Upbit's 800 markets were cautioned that day against 11 warned,
        // so folding a caution into `Unknown` would answer "not plainly
        // healthy" for a quarter of the exchange and bury the 11. It stays
        // `Active`, and the criterion is readable in full instead of dropped.
        let raw: Vec<RawMarket> = json(MARKET_LIST).expect("a live market list payload");
        let events = market_events(&raw).expect("a live market list payload");

        assert_eq!(
            market_info(&raw[3]).expect("a listing").status,
            MarketStatus::Active
        );

        let (market, event) = &events[3];
        assert_eq!(*market, Market::spot(Exchange::Upbit, "SIGN", "BTC"));
        assert!(!event.warning);
        assert_eq!(event.cautions, ["GLOBAL_PRICE_DIFFERENCES"]);

        // And the two designations stay apart in both directions: KRW-AERGO is
        // warned with no criterion raised, KRW-ZIL carries both.
        assert_eq!(events[0].1, UpbitMarketEvent::default());
        assert!(events[1].1.warning && events[1].1.cautions.is_empty());
        assert!(events[2].1.warning);
        assert_eq!(events[2].1.cautions, ["GLOBAL_PRICE_DIFFERENCES"]);
    }

    #[test]
    fn the_deployments_that_still_send_the_older_field_report_the_same_warning() {
        // Upbit Korea sends `market_event` and no `market_warning`; Singapore,
        // Indonesia and Thailand send `market_warning` and no `market_event`.
        // Reading only one of the two would report every market on three of the
        // four deployments as plainly healthy, or every market on the fourth.
        let korea: Vec<RawMarket> = json(MARKET_LIST).expect("a live Upbit Korea payload");
        let indonesia: Vec<RawMarket> =
            json(MARKET_LIST_LEGACY_FIELD).expect("a live Upbit Indonesia payload");

        // BTC-ZIL: `market_event.warning` in Korea, `CAUTION` in Indonesia.
        assert_eq!(
            market_info(&indonesia[0]).expect("a listing").status,
            MarketStatus::Unknown
        );
        assert_eq!(
            market_info(&korea[2]).expect("a listing").status,
            MarketStatus::Unknown
        );

        // BTC-SIGN: cautioned in Korea, `NONE` in Indonesia. The older field
        // never reported a caution, so neither deployment reads it as a
        // warning.
        assert_eq!(
            market_info(&indonesia[1]).expect("a listing").status,
            MarketStatus::Active
        );

        // Those payloads carry no criteria at all, which is what is reported.
        let events = market_events(&indonesia).expect("a live Upbit Indonesia payload");
        assert!(events[0].1.warning && events[0].1.cautions.is_empty());
        assert_eq!(events[1].1, UpbitMarketEvent::default());
    }

    #[test]
    fn an_error_body_becomes_upbits_own_code_and_message() {
        let error = exchange_error(401, ERROR_BODY);

        assert!(matches!(
            &error,
            Error::Exchange { exchange: "upbit", code, message, status: Some(401), .. }
                if code == "invalid_access_key" && message == "Invalid access key"
        ));
        assert!(!error.is_retryable());
    }

    #[test]
    fn a_non_json_failure_keeps_its_body_instead_of_a_parse_complaint() {
        let error = exchange_error(429, "  Too many API requests.  ");

        assert!(matches!(
            &error,
            Error::Exchange { code, message, .. }
                if code == "unknown" && message == "Too many API requests."
        ));
        assert!(error.is_rate_limited());
    }

    #[test]
    fn a_resting_order_is_open_until_something_fills() {
        let zero = Decimal::ZERO;
        let some = Decimal::ONE;

        assert_eq!(order_status("wait", zero, some), OrderStatus::Open);
        assert_eq!(
            order_status("watch", some, some),
            OrderStatus::PartiallyFilled
        );
        assert_eq!(order_status("trade", some, zero), OrderStatus::Filled);
        assert_eq!(
            order_status("trade", some, some),
            OrderStatus::PartiallyFilled
        );
        assert_eq!(order_status("done", some, zero), OrderStatus::Filled);
        assert_eq!(order_status("cancel", zero, some), OrderStatus::Cancelled);
        assert_eq!(order_status("prevented", zero, some), OrderStatus::Rejected);
        assert_eq!(
            order_status("something-new", zero, zero),
            OrderStatus::Unknown
        );
    }

    #[test]
    fn an_order_carries_its_uuid_and_its_local_creation_time() {
        let raw: Vec<RawOrder> = json(OPEN_ORDERS).expect("official open order payload");
        let order = order(&raw[0]).expect("an order");

        assert_eq!(order.id, "ac2dc2a3-fce9-40a2-a4f6-5987c25c438f");
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.status, OrderStatus::Open);
        assert_eq!(order.price, Some(decimal_of("125000000")));
        assert_eq!(order.remaining_quantity, decimal_of("0.0001"));
        // 2024-06-13T10:28:36+09:00 is 01:28:36 UTC.
        assert_eq!(order.created_at, Some(Timestamp::from_secs(1_718_242_116)));
    }

    #[test]
    fn a_balance_arrives_as_text_and_stays_exact() {
        let raw: Vec<RawBalance> = json(ACCOUNTS).expect("official account payload");
        let balance = balance(&raw[0]).expect("a balance");

        assert_eq!(balance.asset, "KRW");
        assert_eq!(balance.available, decimal_of("1000000.0"));
        assert_eq!(balance.total(), decimal_of("1000000.0"));
    }

    #[test]
    fn a_to_cursor_is_written_at_second_resolution_in_utc() {
        let cursor = to_cursor(Timestamp::from_secs(1_499_040_000)).expect("representable");

        assert_eq!(cursor, "2017-07-03T00:00:00Z");
    }

    #[test]
    fn a_sub_second_to_is_rounded_up_so_its_own_second_survives_an_exclusive_cursor() {
        // Upbit's `to` is exclusive: asked for `2026-07-30T06:52:00Z` on
        // `KRW-BTC` second candles it answered with `06:51:59` as its newest,
        // and asked for `06:52:01Z` it answered with `06:52:00`. Truncating a
        // `to` of `06:52:00.500` down to `06:52:00Z` would therefore drop the
        // `06:52:00` candle, which `CandleRequest::to` keeps because that
        // window opens before the instant asked for. Visible at `Sec1`, where
        // one second is one candle.
        let half_past = Timestamp::from_millis(1_785_394_320_500);
        assert_eq!(
            to_cursor(half_past).expect("representable"),
            "2026-07-30T06:52:01Z"
        );

        // The candle that rounding down would have lost is inside the caller's
        // own window, so it is one `Client::candles` has to return.
        let dropped = Timestamp::from_secs(1_785_394_320);
        assert!(dropped < half_past);

        // A whole second is left alone: rounding it up would ask Upbit for a
        // candle the caller's `to` then excludes again, one wasted row a page.
        assert_eq!(
            to_cursor(dropped).expect("representable"),
            "2026-07-30T06:52:00Z"
        );
        // And a nanosecond past it still rounds up.
        assert_eq!(
            to_cursor(Timestamp::from_nanos(1_785_394_320_000_000_001)).expect("representable"),
            "2026-07-30T06:52:01Z"
        );
    }
}
