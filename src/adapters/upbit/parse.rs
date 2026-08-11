//! Converts Upbit REST and WebSocket payloads into `maxt` types.
//!
//! Numeric payloads are parsed from their original digits without an `f64`
//! conversion.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Number;

use super::{UpbitMarketEvent, UpbitOrderBookInstrument, UpbitYearCandle};
use crate::error::{Error, Result};
use crate::types::{
    Balance, Candle, Exchange, Interval, Level, Market, MarketInfo, MarketKind, MarketStatus,
    Order, OrderBook, OrderStatus, Side, Ticker, Timestamp, Trade,
};

pub(crate) const EXCHANGE: &str = Exchange::Upbit.id();

// ---------------------------------------------------------------------------
// Raw payloads
// ---------------------------------------------------------------------------

/// One detailed market-list entry.
///
/// Korea uses `market_event`; the other regions use the legacy
/// `market_warning` field.
#[derive(Debug, Deserialize)]
pub(crate) struct RawMarket {
    pub(crate) market: String,
    pub(crate) korean_name: Option<String>,
    pub(crate) english_name: Option<String>,
    #[serde(default)]
    pub(crate) market_event: Option<RawMarketEvent>,
    /// Legacy warning field used outside Korea: `NONE` or `CAUTION`.
    #[serde(default)]
    pub(crate) market_warning: Option<String>,
}

/// Warning and caution fields returned by the Korean market-list payload.
#[derive(Debug, Deserialize)]
pub(crate) struct RawMarketEvent {
    #[serde(default)]
    pub(crate) warning: bool,
    /// Criterion names are retained and sorted, including unknown future keys.
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
    /// Summary publication time, in milliseconds since the Unix epoch.
    pub(crate) timestamp: i64,
    /// Last-trade time, in milliseconds since the Unix epoch.
    pub(crate) trade_timestamp: i64,
    pub(crate) trade_price: Number,
    pub(crate) signed_change_price: Number,
    pub(crate) signed_change_rate: Number,
    pub(crate) high_price: Number,
    pub(crate) low_price: Number,
    pub(crate) acc_trade_volume_24h: Number,
    pub(crate) acc_trade_price_24h: Number,
}

/// One public candle response entry.
///
/// `unit` is present only on minute-candle responses.
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

/// One entry of `GET /v1/candles/years`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawYearCandle {
    pub(crate) market: String,
    pub(crate) candle_date_time_utc: String,
    /// Korea sends this field; regional deployments can omit it.
    #[serde(default)]
    pub(crate) candle_date_time_kst: Option<String>,
    pub(crate) opening_price: Number,
    pub(crate) high_price: Number,
    pub(crate) low_price: Number,
    pub(crate) trade_price: Number,
    pub(crate) timestamp: i64,
    pub(crate) candle_acc_trade_price: Number,
    pub(crate) candle_acc_trade_volume: Number,
    pub(crate) first_day_of_period: String,
}

/// One entry of `GET /v1/orderbook/instruments`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawOrderBookInstrument {
    pub(crate) market: String,
    pub(crate) quote_currency: String,
    pub(crate) tick_size: String,
    /// Global Upbit regions currently omit this Korea-only aggregation data.
    #[serde(default)]
    pub(crate) supported_levels: Vec<String>,
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
    name: RawErrorName,
    message: String,
}

/// Upbit deployments send `error.name` as either text or a numeric HTTP-like
/// code. Keep both forms as the provider's error code.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawErrorName {
    Text(String),
    Number(Number),
}

impl RawErrorName {
    fn into_string(self) -> String {
        match self {
            Self::Text(name) => name,
            Self::Number(name) => name.to_string(),
        }
    }
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
    /// Summary publication time, in milliseconds since the Unix epoch.
    pub(crate) timestamp: i64,
    /// Last-trade time, in milliseconds since the Unix epoch.
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
    /// `SNAPSHOT` for the initial frame after a connection; otherwise
    /// `REALTIME`.
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

/// Parses a JSON number into [`Decimal`] from its original digits.
///
/// Values that cannot be represented exactly return [`Error::Decode`].
pub(crate) fn decimal(value: &Number, field: &str) -> Result<Decimal> {
    decimal_text(&value.to_string(), field)
}

/// Reads a decimal that Upbit sent as a JSON string, which its account and
/// order endpoints do.
pub(crate) fn decimal_text(text: &str, field: &str) -> Result<Decimal> {
    // Scientific notation is accepted without rounding.
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

/// Parses Upbit's Korea Standard Time candle opening field.
fn candle_korea_open_time(raw: &str) -> Result<Timestamp> {
    const KST_OFFSET_SECS: i64 = 9 * 3_600;

    NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
        .map(|naive| Timestamp::from_secs(naive.and_utc().timestamp() - KST_OFFSET_SECS))
        .map_err(|err| {
            Error::decode(format!(
                "`candle_date_time_kst` is not a Korea Standard Time datetime: {raw} ({err})"
            ))
        })
}

/// Formats an exclusive candle `to` cursor at second resolution.
///
/// A sub-second value is rounded up so a candle opening before the caller's
/// exact boundary remains eligible. The shared candle reader applies the exact
/// boundary after paging.
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

/// Parses Upbit's `BID`/`ASK` or `bid`/`ask` side values.
pub(crate) fn side(raw: &str) -> Result<Side> {
    if raw.eq_ignore_ascii_case("bid") {
        Ok(Side::Buy)
    } else if raw.eq_ignore_ascii_case("ask") {
        Ok(Side::Sell)
    } else {
        Err(Error::decode(format!("unknown Upbit order side `{raw}`")))
    }
}

/// Maps an Upbit order state and fill quantities to [`OrderStatus`].
///
/// `wait` and `watch` are open states. A `trade` state is filled only when the
/// remaining quantity is zero.
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
            envelope.error.name.into_string(),
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

/// Builds an Upbit market code with quote asset first, such as `KRW-BTC`.
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

/// Accepts only uppercase ASCII letters and digits in a market-code component.
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

/// Returns the listing's investment-warning flag (`유의 종목`).
///
/// Korea sends `market_event.warning`; other regions send the legacy
/// `market_warning` value.
fn warned(raw: &RawMarket) -> bool {
    match &raw.market_event {
        Some(event) => event.warning,
        None => !matches!(raw.market_warning.as_deref(), None | Some("NONE")),
    }
}

/// Returns active investment-caution (`주의 종목`) criterion names.
///
/// Regions without `market_event` return an empty list.
fn cautions(raw: &RawMarket) -> Vec<String> {
    raw.market_event
        .iter()
        .flat_map(|event| &event.caution)
        .filter(|(_, raised)| **raised)
        .map(|(criterion, _)| criterion.clone())
        .collect()
}

/// Maps a market listing to common market information.
///
/// Warnings map to [`MarketStatus::Unknown`]; otherwise the status is
/// [`MarketStatus::Active`]. Caution criteria remain available through
/// [`UpbitAdapter::market_events`](super::UpbitAdapter::market_events).
/// `Active` does not guarantee that a new order is accepted.
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

/// Maps detailed listing flags without collapsing caution criteria.
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

/// Returns Upbit's trade identifier without numeric conversion.
///
/// The identifier supports deduplication but is not an ordering key.
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
        // Use execution time rather than frame publication time.
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

    // Enforce the common order-book sort contract.
    bids.sort_by_key(|level| Reverse(level.price));
    asks.sort_by_key(|level| level.price);

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
        // The unsigned change fields do not carry direction.
        change: Some(decimal(&raw.signed_change_price, "signed_change_price")?),
        change_rate: Some(decimal(&raw.signed_change_rate, "signed_change_rate")?),
        high: Some(decimal(&raw.high_price, "high_price")?),
        low: Some(decimal(&raw.low_price, "low_price")?),
        // Use rolling 24-hour totals, not current-UTC-day totals.
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

/// Maps a REST candle for the requested interval.
///
/// `now` determines [`Candle::closed`] because REST payloads carry no completion
/// flag.
pub(crate) fn candle(raw: &RawCandle, interval: Interval, now: Timestamp) -> Result<Candle> {
    // Reject a minute payload whose unit disagrees with the requested endpoint.
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

/// Maps an Upbit yearly candle without pretending its interval is common.
pub(crate) fn year_candle(raw: &RawYearCandle) -> Result<UpbitYearCandle> {
    Ok(UpbitYearCandle {
        market: market_from_native_symbol(&raw.market)?,
        open_time: candle_open_time(&raw.candle_date_time_utc)?,
        korea_open_time: raw
            .candle_date_time_kst
            .as_deref()
            .map(candle_korea_open_time)
            .transpose()?,
        timestamp: millis(raw.timestamp, "timestamp")?,
        open: decimal(&raw.opening_price, "opening_price")?,
        high: decimal(&raw.high_price, "high_price")?,
        low: decimal(&raw.low_price, "low_price")?,
        close: decimal(&raw.trade_price, "trade_price")?,
        volume: decimal(&raw.candle_acc_trade_volume, "candle_acc_trade_volume")?,
        quote_volume: decimal(&raw.candle_acc_trade_price, "candle_acc_trade_price")?,
        first_day_of_period: raw.first_day_of_period.clone(),
    })
}

/// Maps a tick-size and aggregation-policy response.
pub(crate) fn orderbook_instrument(
    raw: &RawOrderBookInstrument,
) -> Result<UpbitOrderBookInstrument> {
    let market = market_from_native_symbol(&raw.market)?;
    if raw.quote_currency != market.quote {
        return Err(Error::decode(format!(
            "`quote_currency` {} does not match market {}",
            raw.quote_currency, raw.market
        )));
    }

    Ok(UpbitOrderBookInstrument {
        market,
        quote_currency: raw.quote_currency.clone(),
        tick_size: decimal_text(&raw.tick_size, "tick_size")?,
        supported_levels: raw
            .supported_levels
            .iter()
            .map(|level| decimal_text(level, "supported_levels"))
            .collect::<Result<Vec<_>>>()?,
    })
}

/// Returns whether a candle window has ended at `now`.
///
/// [`Interval::advance`] handles fixed and calendar-month boundaries.
pub(super) fn has_ended(open_time: Timestamp, interval: Interval, now: Timestamp) -> bool {
    interval
        .advance(open_time, 1)
        .is_some_and(|end_of_window| end_of_window <= now)
}

/// Maps a streamed candle with `closed = false`.
///
/// [`super::stream::Decoder`] sets completion when a later window arrives or an
/// initial snapshot already belongs to an ended window.
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
        Interval::Min10 => Some(10),
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

/// Reads `created_at`, which REST orders normally carry with an explicit
/// offset. Upbit's Global Test Order example omits its documented `+00:00`,
/// so that exact offset-free form is interpreted as UTC.
fn created_at(raw: &str) -> Result<Timestamp> {
    let nanos = match DateTime::parse_from_rfc3339(raw) {
        Ok(parsed) => parsed.timestamp_nanos_opt(),
        Err(rfc3339_error) => NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
            .map(|naive| naive.and_utc().timestamp_nanos_opt())
            .map_err(|naive_error| {
                Error::decode(format!(
                    "`created_at` is neither an RFC 3339 datetime nor Upbit's offset-free UTC form: {raw} ({rfc3339_error}; {naive_error})"
                ))
            })?,
    };
    nanos
        .map(Timestamp::from_nanos)
        .ok_or_else(|| Error::decode(format!("`created_at` is outside timestamp range: {raw}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Korean listing fixtures: neither flag, warning, both, and caution.
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

    // Legacy regional payloads expose only the warning designation.
    const MARKET_LIST_LEGACY_FIELD: &str = r#"[
      {"market":"BTC-ZIL","english_name":"Zilliqa","market_warning":"CAUTION"},
      {"market":"BTC-SIGN","english_name":"Sign","market_warning":"NONE"}
    ]"#;

    // Representative public trade payload.
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

    // Representative public order-book payload.
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

    // Falling-market ticker fixture with signed and unsigned change fields.
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

    // Representative minute-candle payload.
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

    // Representative day-candle payload.
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

    // Korea includes the KST opening field for yearly candles.
    const YEAR_CANDLES: &str = r#"[
      {
        "market": "KRW-BTC",
        "candle_date_time_utc": "2026-01-01T00:00:00",
        "candle_date_time_kst": "2026-01-01T09:00:00",
        "opening_price": 128000000.00000000,
        "high_price": 143050000.00000000,
        "low_price": 88770000.00000000,
        "trade_price": 89587000.00000000,
        "timestamp": 1786467753786,
        "candle_acc_trade_price": 37189906239683.17623000,
        "candle_acc_trade_volume": 348666.78732189,
        "first_day_of_period": "2026-01-01"
      }
    ]"#;

    // Korea includes supported aggregation levels; global deployments can omit
    // the field entirely.
    const ORDERBOOK_INSTRUMENTS: &str = r#"[
      {
        "market": "KRW-BTC",
        "quote_currency": "KRW",
        "tick_size": "1000",
        "supported_levels": ["0", "10000", "100000"]
      },
      {
        "market": "SGD-BTC",
        "quote_currency": "SGD",
        "tick_size": "1"
      }
    ]"#;

    // Representative week-candle payload.
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

    // Representative public trade frame.
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

    // Representative private asset frame used only by offline parsing tests.
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

    // Representative private open-order payload used only by offline parsing tests.
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

    // Global Test Order's current successful example omits the documented UTC offset.
    const TEST_ORDER_RESPONSE: &str = r#"{
      "uuid": "d098ceaf-6811-4df8-97f2-b7e01aefc03f",
      "side": "bid",
      "ord_type": "limit",
      "price": "153559.00",
      "state": "wait",
      "market": "SGD-BTC",
      "created_at": "2025-07-04T15:00:00",
      "volume": "1.0",
      "remaining_volume": "1.0",
      "executed_volume": "0.0"
    }"#;

    // Representative private account payload used only by offline parsing tests.
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

    // Representative exchange error envelope.
    const ERROR_BODY: &str =
        r#"{"error":{"name":"invalid_access_key","message":"Invalid access key"}}"#;
    const NUMERIC_ERROR_BODY: &str = r#"{"error":{"name":404,"message":"Code not found"}}"#;

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
        // `BTC-ETH` means ETH priced in BTC.
        let market = market_from_native_symbol("BTC-ETH").expect("a listed code");

        assert_eq!(market.base, "ETH");
        assert_eq!(market.quote, "BTC");
    }

    #[test]
    fn a_market_code_that_could_smuggle_a_query_parameter_is_rejected() {
        let injected = Market::spot(Exchange::Upbit, "BTC&count=500", "KRW");

        assert!(matches!(
            native_symbol(&injected),
            Err(Error::InvalidRequest { field, .. }) if field == "base"
        ));
        assert!(matches!(
            market_from_native_symbol("KRW-BTC&count=500"),
            Err(Error::InvalidRequest { field, .. }) if field == "base"
        ));
    }

    #[test]
    fn a_code_without_a_separator_is_not_an_upbit_market() {
        assert!(matches!(
            market_from_native_symbol("BTCKRW"),
            Err(Error::InvalidRequest { field, .. }) if field == "symbol"
        ));
        assert!(matches!(
            market_from_native_symbol("KRW-"),
            Err(Error::InvalidRequest { field, .. }) if field == "base"
        ));
    }

    #[test]
    fn another_exchanges_market_never_gets_an_upbit_code() {
        let elsewhere = Market::spot(Exchange::Bithumb, "BTC", "KRW");
        let perpetual = Market::perpetual(Exchange::Upbit, "BTC", "KRW");

        assert!(matches!(
            native_symbol(&elsewhere),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
        assert!(matches!(
            native_symbol(&perpetual),
            Err(Error::Unsupported { .. })
        ));
    }

    #[test]
    fn decimals_keep_the_digits_upbit_sent() {
        // The fixture exceeds `f64` precision.
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
        let raw: RawStreamTrade = json(STREAM_TRADE).expect("official trade frame");
        let trade = stream_trade(&raw).expect("a trade");

        assert_eq!(trade.quantity, decimal_of("0.00008428"));
        assert_eq!(trade.price, decimal_of("37625"));
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_696_585_056_846));
    }

    #[test]
    fn a_number_too_precise_to_hold_is_a_decode_error_not_a_rounded_price() {
        let error = decimal_text("0.000000000000000000000000000001", "price").unwrap_err();

        assert!(matches!(error, Error::Decode { .. }));
    }

    #[test]
    fn a_trade_ticks_ask_bid_names_the_taker() {
        let raw: Vec<RawTrade> = json(TRADES).expect("a live trades payload");
        let trade = trade(&raw[0]).expect("a trade");

        assert_eq!(trade.taker_side, Side::Buy);
        assert_eq!(trade.market, Market::spot(Exchange::Upbit, "BTC", "KRW"));
        assert_eq!(trade.price, decimal_of("91200000.0"));
        assert_eq!(trade.quantity, decimal_of("0.00010971"));
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_785_397_260_660));
        assert_eq!(trade.id.as_deref(), Some("17853972606600000"));
    }

    // Two trades that differ only by `sequential_id`.
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

    // Matching REST and WebSocket representations of one trade.
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
        assert_eq!(ticker.change, Some(decimal_of("-224000.0")));
        assert_eq!(ticker.change_rate, Some(decimal_of("-0.0024501225")));
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
        let now = Timestamp::from_secs(1_781_917_321);

        assert!(
            !candle(&raw[0], Interval::Min1, now)
                .expect("a candle")
                .closed
        );
    }

    // Two updates for the same forming candle window.
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
        // Completion is assigned by `stream::Decoder`, not by this mapper.
        let frames: Vec<RawStreamCandle> = json(LIVE_STREAM_FRAMES).expect("captured frames");

        for raw in &frames {
            let candle = stream_candle(raw, Interval::Min1).expect("a candle");

            assert!(!candle.closed);
            assert_eq!(candle.open_time, Timestamp::from_secs(1_785_394_260));
        }
        let last = stream_candle(&frames[1], Interval::Min1).expect("the last frame of 06:51");
        assert_eq!(last.volume, decimal_of("0.08189562"));
    }

    #[test]
    fn a_month_candle_is_settled_by_the_calendar_and_not_left_open_forever() {
        let june = Timestamp::from_secs(1_748_736_000); // 2025-06-01T00:00:00Z
        let july = Timestamp::from_secs(1_751_328_000); // 2025-07-01T00:00:00Z

        let a_nanosecond_before_july = Timestamp::from_nanos(july.as_nanos() - 1);

        assert!(!has_ended(june, Interval::Month1, a_nanosecond_before_july));
        assert!(has_ended(june, Interval::Month1, july));
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
    fn yearly_candles_keep_the_upbit_specific_interval_and_korea_time() {
        let raw: Vec<RawYearCandle> = json(YEAR_CANDLES).expect("a live yearly candle payload");
        let candle = year_candle(&raw[0]).expect("a yearly candle");

        assert_eq!(candle.market, Market::spot(Exchange::Upbit, "BTC", "KRW"));
        assert_eq!(candle.open_time, Timestamp::from_secs(1_767_225_600));
        assert_eq!(candle.korea_open_time, Some(candle.open_time));
        assert_eq!(candle.close, decimal_of("89587000.00000000"));
        assert_eq!(candle.volume, decimal_of("348666.78732189"));
        assert_eq!(candle.first_day_of_period, "2026-01-01");
    }

    #[test]
    fn orderbook_instruments_keep_region_specific_aggregation_metadata() {
        let raw: Vec<RawOrderBookInstrument> =
            json(ORDERBOOK_INSTRUMENTS).expect("official policy payloads");
        let korea = orderbook_instrument(&raw[0]).expect("a Korea policy");
        let global = orderbook_instrument(&raw[1]).expect("a global policy");

        assert_eq!(korea.tick_size, decimal_of("1000"));
        assert_eq!(
            korea.supported_levels,
            [decimal_of("0"), decimal_of("10000"), decimal_of("100000")]
        );
        assert_eq!(global.market, Market::spot(Exchange::Upbit, "BTC", "SGD"));
        assert_eq!(global.supported_levels, Vec::<Decimal>::new());
    }

    #[test]
    fn an_instrument_whose_quote_disagrees_with_its_market_is_rejected() {
        let raw: RawOrderBookInstrument =
            json(r#"{"market":"KRW-BTC","quote_currency":"BTC","tick_size":"1000"}"#)
                .expect("a response shape");

        assert!(matches!(
            orderbook_instrument(&raw),
            Err(Error::Decode { .. })
        ));
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

        assert_eq!(status(&raw[0]), MarketStatus::Active);
        assert_eq!(status(&raw[1]), MarketStatus::Unknown);
        assert_eq!(status(&raw[2]), MarketStatus::Unknown);

        let warned = market_info(&raw[1]).expect("a listing");
        assert_eq!(warned.native_symbol, "KRW-AERGO");
        assert_eq!(warned.english_name.as_deref(), Some("Aergo"));
        assert_eq!(warned.korean_name.as_deref(), Some("아르고"));
    }

    #[test]
    fn a_caution_on_its_own_leaves_the_market_active_and_stays_readable() {
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

        assert_eq!(events[0].1, UpbitMarketEvent::default());
        assert!(events[1].1.warning && events[1].1.cautions.is_empty());
        assert!(events[2].1.warning);
        assert_eq!(events[2].1.cautions, ["GLOBAL_PRICE_DIFFERENCES"]);
    }

    #[test]
    fn the_deployments_that_still_send_the_older_field_report_the_same_warning() {
        let korea: Vec<RawMarket> = json(MARKET_LIST).expect("a live Upbit Korea payload");
        let indonesia: Vec<RawMarket> =
            json(MARKET_LIST_LEGACY_FIELD).expect("a live Upbit Indonesia payload");

        assert_eq!(
            market_info(&indonesia[0]).expect("a listing").status,
            MarketStatus::Unknown
        );
        assert_eq!(
            market_info(&korea[2]).expect("a listing").status,
            MarketStatus::Unknown
        );

        assert_eq!(
            market_info(&indonesia[1]).expect("a listing").status,
            MarketStatus::Active
        );

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
    fn a_numeric_error_name_is_kept_as_the_exchange_code() {
        let error = exchange_error(404, NUMERIC_ERROR_BODY);

        assert!(matches!(
            &error,
            Error::Exchange { exchange: "upbit", code, message, status: Some(404), .. }
                if code == "404" && message == "Code not found"
        ));
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
    fn rest_order_time_keeps_subsecond_precision() {
        assert_eq!(
            created_at("2024-06-13T10:28:36.123456789+09:00").expect("an offset time"),
            Timestamp::from_nanos(1_718_242_116_123_456_789)
        );
    }

    #[test]
    fn test_order_accepts_the_global_example_without_an_offset() {
        let raw: RawOrder = json(TEST_ORDER_RESPONSE).expect("official test-order payload");
        let order = order(&raw).expect("test order");

        assert_eq!(order.id, "d098ceaf-6811-4df8-97f2-b7e01aefc03f");
        assert_eq!(order.status, OrderStatus::Open);
        assert_eq!(order.created_at, Some(Timestamp::from_secs(1_751_641_200)));
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
        let half_past = Timestamp::from_millis(1_785_394_320_500);
        assert_eq!(
            to_cursor(half_past).expect("representable"),
            "2026-07-30T06:52:01Z"
        );

        let dropped = Timestamp::from_secs(1_785_394_320);
        assert!(dropped < half_past);

        assert_eq!(
            to_cursor(dropped).expect("representable"),
            "2026-07-30T06:52:00Z"
        );
        assert_eq!(
            to_cursor(Timestamp::from_nanos(1_785_394_320_000_000_001)).expect("representable"),
            "2026-07-30T06:52:01Z"
        );
    }
}
