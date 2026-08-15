//! Upbit's public market data WebSocket.
//!
//! One connection can carry multiple markets and feed types.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use rust_decimal::Decimal;
use serde_json::{Map, Value};
use tokio::sync::oneshot;

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::transport::{Heartbeat, HeartbeatFrame, WsSendHandle};
use crate::types::{Candle, Feed, Interval, Market, MarketEvent, Subscription, Timestamp};

use super::parse::{self, EXCHANGE, RawStreamCandle};
use super::{
    UpbitCandleStreamEvent, UpbitMarketStreamEvent, UpbitOrderBookStreamEvent,
    UpbitTickerStreamEvent, UpbitTradeStreamEvent,
};

/// Full-name JSON frame format.
const FRAME_FORMAT: &str = "DEFAULT";

/// One subscription returned by Upbit's `LIST_SUBSCRIPTIONS` operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedSubscription {
    /// Upbit's WebSocket data type, such as `ticker` or `orderbook`.
    pub feed_type: String,
    /// Markets carried by this data type; account-wide items have no markets.
    pub markets: Vec<Market>,
    /// Order-book aggregation level when one was requested.
    pub level: Option<Decimal>,
}

/// Upbit's response to one `LIST_SUBSCRIPTIONS` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionList {
    /// Ticket that correlated the request and response.
    pub ticket: String,
    /// Data types confirmed by Upbit on the queried active connection.
    pub subscriptions: Vec<ListedSubscription>,
}

/// Coordinates one `LIST_SUBSCRIPTIONS` request on an existing connection.
///
/// The socket lifecycle itself remains shared transport code. Upbit alone owns
/// the operation's response shape and correlation ticket.
pub(crate) struct SubscriptionControl {
    send: WsSendHandle,
    pending: Mutex<Option<PendingSubscriptionList>>,
}

struct PendingSubscriptionList {
    ticket: String,
    response: oneshot::Sender<Result<SubscriptionList>>,
}

impl SubscriptionControl {
    pub(crate) fn new(send: WsSendHandle) -> Self {
        Self {
            send,
            pending: Mutex::new(None),
        }
    }

    /// Sends the documented operation through this exact active connection.
    pub(crate) async fn list_subscriptions(&self) -> Result<SubscriptionList> {
        let ticket = uuid::Uuid::new_v4().to_string();
        let mut connection_epoch = self.send.connection_epoch();
        let (response, received) = oneshot::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if pending.is_some() {
                return Err(Error::adapter(
                    "an Upbit LIST_SUBSCRIPTIONS request is already waiting on this connection",
                ));
            }
            *pending = Some(PendingSubscriptionList {
                ticket: ticket.clone(),
                response,
            });
        }

        let guard = PendingGuard {
            control: self,
            ticket: ticket.clone(),
            active: true,
        };
        self.send
            .send_text(list_subscriptions_frame(&ticket)?)
            .await?;
        let result = tokio::select! {
            biased;
            changed = connection_epoch.changed() => {
                match changed {
                    Ok(()) => Err(Error::transport(
                        "Upbit WebSocket reconnected before answering LIST_SUBSCRIPTIONS",
                    )),
                    Err(_) => Err(Error::transport(
                        "Upbit WebSocket closed before answering LIST_SUBSCRIPTIONS",
                    )),
                }
            }
            result = received => result.map_err(|_| {
                Error::transport("Upbit WebSocket closed before answering LIST_SUBSCRIPTIONS")
            })?,
        };
        guard.disarm();
        result
    }

    /// Consumes an operation reply so it is never decoded as market data.
    pub(crate) fn handle_frame(&self, frame: &str) -> bool {
        let Ok(object) = frame_object(frame) else {
            return false;
        };
        let listed = object.get("method").and_then(Value::as_str) == Some("LIST_SUBSCRIPTIONS");
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !listed && !(pending.is_some() && object.contains_key("error")) {
            return false;
        }
        let Some(waiting) = pending.as_ref() else {
            return listed;
        };
        if listed
            && object
                .get("ticket")
                .and_then(Value::as_str)
                .is_some_and(|ticket| ticket != waiting.ticket)
        {
            // A cancelled earlier call can still receive its reply after a
            // later query has started. It must not fail that newer query.
            return true;
        }
        let Some(waiting) = pending.take() else {
            return listed;
        };
        let expected_ticket = waiting.ticket.clone();
        let result = reserialize(&object)
            .and_then(|frame| subscription_list(&frame))
            .and_then(|result| {
                if result.ticket == expected_ticket {
                    Ok(result)
                } else {
                    Err(Error::decode(format!(
                        "Upbit subscription-list ticket `{}` does not match request `{}`",
                        result.ticket, expected_ticket
                    )))
                }
            });
        let _ = waiting.response.send(result);
        true
    }

    /// A reconnect cannot return a reply to an operation sent on the old socket.
    pub(crate) fn fail_pending(&self) {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(pending) = pending {
            let _ = pending.response.send(Err(Error::transport(
                "Upbit WebSocket reconnected before answering LIST_SUBSCRIPTIONS",
            )));
        }
    }

    fn clear(&self, ticket: &str) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if pending
            .as_ref()
            .is_some_and(|pending| pending.ticket == ticket)
        {
            pending.take();
        }
    }
}

struct PendingGuard<'a> {
    control: &'a SubscriptionControl,
    ticket: String,
    active: bool,
}

impl PendingGuard<'_> {
    fn disarm(mut self) {
        self.active = false;
    }
}

impl Drop for PendingGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            self.control.clear(&self.ticket);
        }
    }
}

/// Sends text `PING` every 15 seconds and waits at least 60 seconds for traffic.
pub(crate) const HEARTBEAT: Heartbeat = Heartbeat {
    interval: Duration::from_secs(15),
    frame: HeartbeatFrame::Text("PING"),
    min_idle_timeout: Duration::from_secs(60),
};

/// Builds the public subscription frame used on connect and reconnect.
pub(crate) fn subscribe_frame(subscription: &Subscription, ticket: &str) -> Result<String> {
    if subscription.markets().is_empty() {
        return Err(Error::invalid_request(
            "markets",
            "subscribe to at least one market",
        ));
    }
    if subscription.feeds().is_empty() {
        return Err(Error::invalid_request(
            "feeds",
            "subscribe to at least one feed",
        ));
    }

    let codes = subscription
        .markets()
        .iter()
        .map(parse::native_symbol)
        .collect::<Result<Vec<_>>>()?;

    let mut payload = Vec::with_capacity(subscription.feeds().len() + 2);
    payload.push(serde_json::json!({ "ticket": ticket }));
    for feed in subscription.feeds() {
        payload.push(serde_json::json!({ "type": feed_type(*feed)?, "codes": codes }));
    }
    payload.push(serde_json::json!({ "format": FRAME_FORMAT }));

    serde_json::to_string(&payload)
        .map_err(|err| Error::decode(format!("could not build Upbit subscribe frame: {err}")))
}

/// Builds the operation frame that asks an active connection what it carries.
pub(crate) fn list_subscriptions_frame(ticket: &str) -> Result<String> {
    if ticket.trim().is_empty() {
        return Err(Error::invalid_request(
            "ticket",
            "an Upbit subscription-list request needs a ticket",
        ));
    }

    serde_json::to_string(&serde_json::json!([
        { "ticket": ticket },
        { "method": "LIST_SUBSCRIPTIONS" },
        { "format": FRAME_FORMAT }
    ]))
    .map_err(|err| Error::decode(format!("could not build Upbit operation frame: {err}")))
}

/// Decodes Upbit's full-name `LIST_SUBSCRIPTIONS` response.
pub(crate) fn subscription_list(frame: &str) -> Result<SubscriptionList> {
    let object = frame_object(frame)?;
    if let Some(error) = object.get("error") {
        return Err(frame_error(error));
    }
    if object.get("method").and_then(Value::as_str) != Some("LIST_SUBSCRIPTIONS") {
        return Err(Error::decode(
            "Upbit subscription-list response has no `LIST_SUBSCRIPTIONS` method".to_string(),
        ));
    }

    let ticket = required_text(&object, "ticket")?;
    let subscriptions = object
        .get("result")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("Upbit subscription-list response has no `result` array"))?
        .iter()
        .map(|entry| {
            let entry = entry
                .as_object()
                .ok_or_else(|| Error::decode("Upbit subscription-list result is not an object"))?;
            let feed_type = required_text(entry, "type")?;
            let markets = entry
                .get("codes")
                .map(|codes| {
                    codes
                        .as_array()
                        .ok_or_else(|| {
                            Error::decode("Upbit subscription-list `codes` is not an array")
                        })?
                        .iter()
                        .map(|code| {
                            code.as_str()
                                .ok_or_else(|| {
                                    Error::decode("Upbit subscription-list market code is not text")
                                })
                                .and_then(parse::market_from_native_symbol)
                        })
                        .collect()
                })
                .transpose()?
                .unwrap_or_default();
            let level = entry
                .get("level")
                .map(|level| parse::decimal_text(&level.to_string(), "level"))
                .transpose()?;

            Ok(ListedSubscription {
                feed_type,
                markets,
                level,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(SubscriptionList {
        ticket,
        subscriptions,
    })
}

fn required_text(object: &Map<String, Value>, field: &'static str) -> Result<String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| Error::decode(format!("Upbit subscription-list `{field}` is missing")))
}

/// The `type` Upbit expects for a feed.
fn feed_type(feed: Feed) -> Result<String> {
    Ok(match feed {
        Feed::Trades => "trade".to_string(),
        Feed::OrderBook => "orderbook".to_string(),
        Feed::Ticker => "ticker".to_string(),
        Feed::Candles(interval) => candle_type(interval)
            .ok_or_else(|| {
                Error::unsupported(
                    Feature::CandleStream,
                    EXCHANGE,
                    format!(
                        "upbit streams candles at 1s, 1m, 3m, 5m, 10m, 15m, 30m, 1h and 4h, and \
                         none at {interval:?}; every one of those it streams and `Interval` can \
                         name is mapped, so this is absent from upbit's stream rather than \
                         missing from `maxt`. `Client::candles` serves 1d, 1w and 1M over REST"
                    ),
                )
            })?
            .to_string(),
    })
}

/// Returns the WebSocket candle type for an interval exposed by `maxt`.
///
/// Daily and longer candles are REST-only.
fn candle_type(interval: Interval) -> Option<&'static str> {
    Some(match interval {
        Interval::Sec1 => "candle.1s",
        Interval::Min1 => "candle.1m",
        Interval::Min3 => "candle.3m",
        Interval::Min5 => "candle.5m",
        Interval::Min10 => "candle.10m",
        Interval::Min15 => "candle.15m",
        Interval::Min30 => "candle.30m",
        Interval::Hour1 => "candle.60m",
        Interval::Hour4 => "candle.240m",
        _ => return None,
    })
}

fn candle_interval(frame_type: &str) -> Option<Interval> {
    Some(match frame_type {
        "candle.1s" => Interval::Sec1,
        "candle.1m" => Interval::Min1,
        "candle.3m" => Interval::Min3,
        "candle.5m" => Interval::Min5,
        "candle.10m" => Interval::Min10,
        "candle.15m" => Interval::Min15,
        "candle.30m" => Interval::Min30,
        "candle.60m" => Interval::Hour1,
        "candle.240m" => Interval::Hour4,
        _ => return None,
    })
}

/// Decodes frames for one public connection in arrival order.
///
/// Non-candle frames are stateless. For each candle market and interval, the
/// decoder holds the latest forming window until a later window settles it.
#[derive(Debug, Default)]
pub(crate) struct Decoder {
    /// Latest forming candle per market and interval.
    latest: HashMap<(Market, Interval), Candle>,
    /// Latest candle already emitted as settled per market and interval.
    settled: HashMap<(Market, Interval), Timestamp>,
}

/// Initial frame type after a connection or reconnection.
const SNAPSHOT: &str = "SNAPSHOT";

impl Decoder {
    /// Decodes one frame into zero or more events.
    ///
    /// Status frames produce no events. A candle transition produces the
    /// settled candle first and the new forming candle second.
    pub(crate) fn decode(&mut self, frame: &str) -> Result<Vec<MarketEvent>> {
        self.decode_at(frame, Timestamp::now())
    }

    fn decode_at(&mut self, frame: &str, now: Timestamp) -> Result<Vec<MarketEvent>> {
        let object = frame_object(frame)?;

        if let Some(error) = object.get("error") {
            return Err(frame_error(error));
        }
        if object.get("status").and_then(Value::as_str) == Some("UP") {
            return Ok(Vec::new());
        }

        let Some(frame_type) = object.get("type").and_then(Value::as_str) else {
            return Err(Error::decode("Upbit frame carries no `type`".to_string()));
        };
        let frame_type = frame_type.to_string();
        let body = reserialize(&object)?;

        let event = match frame_type.as_str() {
            "trade" => MarketEvent::Trade(parse::stream_trade(&parse::json(&body)?)?),
            "orderbook" => MarketEvent::OrderBook(parse::stream_order_book(&parse::json(&body)?)?),
            "ticker" => MarketEvent::Ticker(parse::stream_ticker(&parse::json(&body)?)?),
            candle if candle.starts_with("candle.") => {
                let interval = candle_interval(candle).ok_or_else(|| {
                    Error::decode(format!("upbit sent an unmapped candle feed `{candle}`"))
                })?;
                return self.candle(&parse::json(&body)?, interval, now);
            }
            other => {
                return Err(Error::decode(format!(
                    "unexpected Upbit frame type `{other}`"
                )));
            }
        };

        Ok(vec![event])
    }

    /// Applies one candle frame to the per-feed completion state.
    ///
    /// A same-window frame replaces and emits the forming candle. A later
    /// real-time window settles the held candle at most once, then emits the new
    /// forming candle; an older frame is ignored. An ended snapshot is emitted
    /// closed immediately, while a current snapshot replaces the held state
    /// without settling pre-reconnect data.
    fn candle(
        &mut self,
        raw: &RawStreamCandle,
        interval: Interval,
        now: Timestamp,
    ) -> Result<Vec<MarketEvent>> {
        let forming = parse::stream_candle(raw, interval)?;
        let snapshot = raw.stream_type.as_deref() == Some(SNAPSHOT);
        let key = (forming.market.clone(), interval);

        if self
            .settled
            .get(&key)
            .is_some_and(|settled| forming.open_time <= *settled)
        {
            return Ok(Vec::new());
        }

        if self
            .latest
            .get(&key)
            .is_some_and(|held| forming.open_time < held.open_time)
        {
            return Ok(Vec::new());
        }

        if snapshot && parse::has_ended(forming.open_time, interval, now) {
            self.latest.remove(&key);
            self.settled.insert(key, forming.open_time);
            return Ok(vec![MarketEvent::Candle(Candle {
                closed: true,
                ..forming
            })]);
        }

        let settled = self
            .latest
            .insert(key.clone(), forming.clone())
            .filter(|_| !snapshot)
            .filter(|held| held.open_time < forming.open_time)
            .map(|held| {
                self.settled.insert(key, held.open_time);
                MarketEvent::Candle(Candle {
                    closed: true,
                    ..held
                })
            });

        Ok(settled
            .into_iter()
            .chain([MarketEvent::Candle(forming)])
            .collect())
    }
}

/// Decodes one public connection while retaining Upbit-specific fields.
///
/// Candle completion state mirrors [`Decoder`] so a settled candle keeps the
/// provider frame that originally described that candle.
#[derive(Debug, Default)]
pub(crate) struct DetailedDecoder {
    latest: HashMap<(Market, Interval), UpbitCandleStreamEvent>,
    settled: HashMap<(Market, Interval), Timestamp>,
}

impl DetailedDecoder {
    pub(crate) fn decode(&mut self, frame: &str) -> Result<Vec<UpbitMarketStreamEvent>> {
        self.decode_at(frame, Timestamp::now())
    }

    fn decode_at(&mut self, frame: &str, now: Timestamp) -> Result<Vec<UpbitMarketStreamEvent>> {
        let object = frame_object(frame)?;
        if let Some(error) = object.get("error") {
            return Err(frame_error(error));
        }
        if object.get("status").and_then(Value::as_str) == Some("UP") {
            return Ok(Vec::new());
        }
        let Some(frame_type) = object.get("type").and_then(Value::as_str) else {
            return Err(Error::decode("Upbit frame carries no `type`"));
        };
        let frame_type = frame_type.to_owned();
        let raw_json = reserialize(&object)?;
        let value = Value::Object(object);

        let event = match frame_type.as_str() {
            "trade" => {
                let common = parse::stream_trade(&parse::json(&raw_json)?)?;
                UpbitMarketStreamEvent::Trade(UpbitTradeStreamEvent {
                    previous_closing_price: optional_decimal(&value, "prev_closing_price")?,
                    change: optional_text(&value, "change")?,
                    change_price: optional_decimal(&value, "change_price")?,
                    best_ask_price: optional_decimal(&value, "best_ask_price")?,
                    best_ask_size: optional_decimal(&value, "best_ask_size")?,
                    best_bid_price: optional_decimal(&value, "best_bid_price")?,
                    best_bid_size: optional_decimal(&value, "best_bid_size")?,
                    common,
                    raw_json,
                })
            }
            "orderbook" => {
                let common = parse::stream_order_book(&parse::json(&raw_json)?)?;
                UpbitMarketStreamEvent::OrderBook(UpbitOrderBookStreamEvent {
                    total_ask_size: optional_decimal(&value, "total_ask_size")?,
                    total_bid_size: optional_decimal(&value, "total_bid_size")?,
                    level: optional_decimal(&value, "level")?,
                    stream_type: optional_text(&value, "stream_type")?,
                    common,
                    raw_json,
                })
            }
            "ticker" => {
                let common = parse::stream_ticker(&parse::json(&raw_json)?)?;
                UpbitMarketStreamEvent::Ticker(UpbitTickerStreamEvent {
                    change_direction: optional_text(&value, "change")?,
                    market_state: optional_text(&value, "market_state")?,
                    trading_suspended: optional_bool(&value, "is_trading_suspended")?,
                    delisting_date: optional_text(&value, "delisting_date")?,
                    market_warning: optional_text(&value, "market_warning")?,
                    common,
                    raw_json,
                })
            }
            candle if candle.starts_with("candle.") => {
                let interval = candle_interval(candle).ok_or_else(|| {
                    Error::decode(format!("upbit sent an unmapped candle feed `{candle}`"))
                })?;
                let raw: RawStreamCandle = parse::json(&raw_json)?;
                let forming = UpbitCandleStreamEvent {
                    common: parse::stream_candle(&raw, interval)?,
                    stream_type: raw.stream_type,
                    published_at: optional_millis(&value, "timestamp")?,
                    raw_json,
                };
                return self.candle(forming, now);
            }
            other => {
                return Err(Error::decode(format!(
                    "unexpected Upbit frame type `{other}`"
                )));
            }
        };

        Ok(vec![event])
    }

    fn candle(
        &mut self,
        forming: UpbitCandleStreamEvent,
        now: Timestamp,
    ) -> Result<Vec<UpbitMarketStreamEvent>> {
        let snapshot = forming.stream_type.as_deref() == Some(SNAPSHOT);
        let key = (forming.common.market.clone(), forming.common.interval);

        if self
            .settled
            .get(&key)
            .is_some_and(|settled| forming.common.open_time <= *settled)
        {
            return Ok(Vec::new());
        }
        if self
            .latest
            .get(&key)
            .is_some_and(|held| forming.common.open_time < held.common.open_time)
        {
            return Ok(Vec::new());
        }

        if snapshot && parse::has_ended(forming.common.open_time, forming.common.interval, now) {
            self.latest.remove(&key);
            self.settled.insert(key, forming.common.open_time);
            return Ok(vec![UpbitMarketStreamEvent::Candle(
                UpbitCandleStreamEvent {
                    common: Candle {
                        closed: true,
                        ..forming.common
                    },
                    ..forming
                },
            )]);
        }

        let settled = self
            .latest
            .insert(key.clone(), forming.clone())
            .filter(|_| !snapshot)
            .filter(|held| held.common.open_time < forming.common.open_time)
            .map(|held| {
                self.settled.insert(key, held.common.open_time);
                UpbitMarketStreamEvent::Candle(UpbitCandleStreamEvent {
                    common: Candle {
                        closed: true,
                        ..held.common
                    },
                    ..held
                })
            });

        Ok(settled
            .into_iter()
            .chain([UpbitMarketStreamEvent::Candle(forming)])
            .collect())
    }
}

fn optional_decimal(value: &Value, name: &'static str) -> Result<Option<Decimal>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => parse::decimal(number, name).map(Some),
        Some(Value::String(text)) => parse::decimal_text(text, name).map(Some),
        Some(_) => Err(Error::decode(format!("`{name}` is not a number"))),
    }
}

fn optional_millis(value: &Value, name: &'static str) -> Result<Option<Timestamp>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_i64()
            .ok_or_else(|| Error::decode(format!("`{name}` is not an i64 millisecond timestamp")))
            .and_then(|millis| parse::millis(millis, name))
            .map(Some),
        Some(_) => Err(Error::decode(format!(
            "`{name}` is not a millisecond timestamp"
        ))),
    }
}

fn optional_text(value: &Value, name: &'static str) -> Result<Option<String>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(Error::decode(format!("`{name}` is not a string"))),
    }
}

fn optional_bool(value: &Value, name: &'static str) -> Result<Option<bool>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(Error::decode(format!("`{name}` is not a boolean"))),
    }
}

/// Unwraps a frame to its object.
///
/// Upbit sends bare objects, but wraps them in a single-element list on some
/// deployments, so both are accepted.
pub(super) fn frame_object(frame: &str) -> Result<Map<String, Value>> {
    let value: Value = serde_json::from_str(frame)
        .map_err(|err| Error::decode(format!("unreadable Upbit frame: {err}")))?;

    match value {
        Value::Object(object) => Ok(object),
        Value::Array(mut items) if items.len() == 1 => match items.remove(0) {
            Value::Object(object) => Ok(object),
            _ => Err(Error::decode(
                "Upbit frame list holds a non-object".to_string(),
            )),
        },
        _ => Err(Error::decode(
            "Upbit frame is neither an object nor a one-object list".to_string(),
        )),
    }
}

/// Re-emits the unwrapped object so it can be read into a typed shape.
///
/// `serde_json` is configured with `arbitrary_precision`, so numbers survive
/// this round trip as the digits Upbit sent.
pub(super) fn reserialize(object: &Map<String, Value>) -> Result<String> {
    serde_json::to_string(object)
        .map_err(|err| Error::decode(format!("could not re-read Upbit frame: {err}")))
}

/// Reads Upbit's WebSocket error frame, which uses `name`/`message` like the
/// REST envelope but arrives without an HTTP status.
pub(crate) fn frame_error(error: &Value) -> Error {
    let code = error
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown");
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("upbit closed the subscription without saying why");

    Error::exchange(EXCHANGE, code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Market, Side, Timestamp};
    #[cfg(not(target_arch = "wasm32"))]
    use futures_util::{SinkExt, StreamExt};
    use rust_decimal::Decimal;

    // Representative public trade frame.
    const TRADE: &str = r#"{
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

    // Representative public order-book frame.
    const ORDER_BOOK: &str = r#"{
      "type": "orderbook",
      "code": "KRW-BTC",
      "timestamp": 1746602359173,
      "total_ask_size": 0.68780013,
      "total_bid_size": 0.78754733,
      "orderbook_units": [
        {
          "ask_price": 125056.0,
          "bid_price": 124743.0,
          "ask_size": 0.17,
          "bid_size": 0.17
        }
      ]
    }"#;

    // Representative public ticker frame.
    const TICKER: &str = r#"{
      "type": "ticker",
      "code": "KRW-BTC",
      "opening_price": 37249,
      "high_price": 37645,
      "low_price": 36732,
      "trade_price": 36929,
      "prev_closing_price": 37235,
      "acc_trade_price": 5530.55648491,
      "change": "FALL",
      "change_price": 306,
      "signed_change_price": -306,
      "change_rate": 0.0082180744,
      "signed_change_rate": -0.0082180744,
      "ask_bid": "ASK",
      "trade_volume": 0.00006314,
      "acc_trade_volume": 0.149474,
      "trade_date": "20230830",
      "trade_time": "082105",
      "trade_timestamp": 1693383665811,
      "acc_ask_volume": 0.0197006,
      "acc_bid_volume": 0.1297734,
      "highest_52_week_price": 42710,
      "highest_52_week_date": "2023-07-06",
      "lowest_52_week_price": 21332,
      "lowest_52_week_date": "2022-11-21",
      "market_state": "ACTIVE",
      "is_trading_suspended": false,
      "delisting_date": null,
      "market_warning": "NONE",
      "timestamp": 1693383690031,
      "acc_trade_price_24h": 8320.40577449,
      "acc_trade_volume_24h": 0.22569412
    }"#;

    // Candle fixtures cover snapshots, transitions, markets, and intervals.
    /// Initial snapshot for one minute window.
    const CANDLE_SNAPSHOT: &str = r#"{"type":"candle.1m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:45:00","candle_date_time_kst":"2026-07-30T16:45:00",
      "opening_price":91154000.00000000,"high_price":91157000.00000000,
      "low_price":91142000.00000000,"trade_price":91157000.00000000,
      "candle_acc_trade_volume":0.15511395,"candle_acc_trade_price":14139448.26754000,
      "timestamp":1785397531885,"stream_type":"SNAPSHOT"}"#;

    /// Forming update for the preceding minute.
    const CANDLE_LAST_OF_ITS_MINUTE: &str = r#"{"type":"candle.1m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:46:00","candle_date_time_kst":"2026-07-30T16:46:00",
      "opening_price":91157000.00000000,"high_price":91157000.00000000,
      "low_price":91133000.00000000,"trade_price":91157000.00000000,
      "candle_acc_trade_volume":0.80044670,"candle_acc_trade_price":72957159.34596000,
      "timestamp":1785397618309,"stream_type":"REALTIME"}"#;

    /// First update for its successor window.
    const CANDLE_NEXT_MINUTE: &str = r#"{"type":"candle.1m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:47:00","candle_date_time_kst":"2026-07-30T16:47:00",
      "opening_price":91157000.00000000,"high_price":91157000.00000000,
      "low_price":91157000.00000000,"trade_price":91157000.00000000,
      "candle_acc_trade_volume":0.00098728,"candle_acc_trade_price":89997.48296000,
      "timestamp":1785397620706,"stream_type":"REALTIME"}"#;

    /// First update for the following successor window.
    const CANDLE_MINUTE_AFTER_NEXT: &str = r#"{"type":"candle.1m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:48:00","candle_date_time_kst":"2026-07-30T16:48:00",
      "opening_price":91134000.00000000,"high_price":91134000.00000000,
      "low_price":91134000.00000000,"trade_price":91134000.00000000,
      "candle_acc_trade_volume":0.00109721,"candle_acc_trade_price":99993.13614000,
      "timestamp":1785397680638,"stream_type":"REALTIME"}"#;

    /// Current-window snapshot after reconnecting.
    const CANDLE_SNAPSHOT_AFTER_RECONNECT: &str = r#"{"type":"candle.1m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:47:00","candle_date_time_kst":"2026-07-30T16:47:00",
      "opening_price":91157000.00000000,"high_price":91157000.00000000,
      "low_price":91133000.00000000,"trade_price":91133000.00000000,
      "candle_acc_trade_volume":0.01011827,"candle_acc_trade_price":922159.13839000,
      "timestamp":1785397627219,"stream_type":"SNAPSHOT"}"#;

    /// A second interval for the same market.
    const CANDLE_OTHER_INTERVAL: &str = r#"{"type":"candle.5m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:50:00","candle_date_time_kst":"2026-07-30T16:50:00",
      "opening_price":91134000.00000000,"high_price":91134000.00000000,
      "low_price":91132000.00000000,"trade_price":91132000.00000000,
      "candle_acc_trade_volume":0.03684166,"candle_acc_trade_price":3357485.11323000,
      "timestamp":1785397829625,"stream_type":"SNAPSHOT"}"#;

    /// A one-minute feed for the first market.
    const CANDLE_ONE_MINUTE_0750: &str = r#"{"type":"candle.1m","code":"KRW-BTC",
      "candle_date_time_utc":"2026-07-30T07:50:00","candle_date_time_kst":"2026-07-30T16:50:00",
      "opening_price":91134000.00000000,"high_price":91134000.00000000,
      "low_price":91132000.00000000,"trade_price":91132000.00000000,
      "candle_acc_trade_volume":0.03684166,"candle_acc_trade_price":3357485.11323000,
      "timestamp":1785397829625,"stream_type":"SNAPSHOT"}"#;

    /// The same interval for a second market.
    const CANDLE_OTHER_MARKET: &str = r#"{"type":"candle.1m","code":"KRW-ETH",
      "candle_date_time_utc":"2026-07-30T07:50:00","candle_date_time_kst":"2026-07-30T16:50:00",
      "opening_price":2715000.00000000,"high_price":2716000.00000000,
      "low_price":2714000.00000000,"trade_price":2715000.00000000,
      "candle_acc_trade_volume":37.95606020,"candle_acc_trade_price":103034054.24344000,
      "timestamp":1785397818983,"stream_type":"SNAPSHOT"}"#;

    /// Start of the first fixture window, in Unix seconds.
    const MINUTE_0745: i64 = 1_785_397_500;
    const MINUTE_0746: i64 = MINUTE_0745 + 60;
    const MINUTE_0747: i64 = MINUTE_0745 + 120;
    const MINUTE_0748: i64 = MINUTE_0745 + 180;
    const MINUTE_0750: i64 = MINUTE_0745 + 300;

    /// Decodes one frame with fresh state.
    fn decode(frame: &str) -> Result<Vec<MarketEvent>> {
        Decoder::default().decode_at(frame, Timestamp::from_secs(MINUTE_0745 + 30))
    }

    /// Extracts the single event from a stateless frame.
    fn one(frame: &str) -> MarketEvent {
        let mut events = decode(frame).expect("a data frame");
        assert_eq!(events.len(), 1, "expected one event from {frame}");
        events.remove(0)
    }

    fn subscription() -> Subscription {
        Subscription::new()
            .market(Market::spot(Exchange::Upbit, "BTC", "KRW"))
            .market(Market::spot(Exchange::Upbit, "ETH", "KRW"))
            .feed(Feed::Trades)
            .feed(Feed::Candles(Interval::Min1))
    }

    #[test]
    fn one_frame_carries_every_market_and_every_feed() {
        let frame = subscribe_frame(&subscription(), "ticket-1").expect("a valid subscription");
        let value: Value = serde_json::from_str(&frame).expect("valid JSON");

        assert_eq!(value[0]["ticket"], "ticket-1");
        assert_eq!(value[1]["type"], "trade");
        assert_eq!(value[1]["codes"][0], "KRW-BTC");
        assert_eq!(value[1]["codes"][1], "KRW-ETH");
        assert_eq!(value[2]["type"], "candle.1m");
        assert_eq!(value[3]["format"], "DEFAULT");
        assert_eq!(value.as_array().expect("a list").len(), 4);
    }

    #[test]
    fn list_subscriptions_uses_the_documented_operation_frame() {
        let frame = list_subscriptions_frame("ticket-1").expect("a valid operation frame");
        assert_eq!(
            serde_json::from_str::<Value>(&frame).expect("valid JSON"),
            serde_json::json!([
                {"ticket": "ticket-1"},
                {"method": "LIST_SUBSCRIPTIONS"},
                {"format": "DEFAULT"}
            ])
        );
        assert!(matches!(
            list_subscriptions_frame("  "),
            Err(Error::InvalidRequest { field, .. }) if field == "ticket"
        ));
    }

    #[test]
    fn list_subscriptions_preserves_codes_levels_and_account_wide_items() {
        let response = subscription_list(
            r#"{
                "method":"LIST_SUBSCRIPTIONS",
                "result":[
                    {"type":"ticker","codes":["KRW-BTC"]},
                    {"type":"orderbook","codes":["KRW-BTC","BTC-ETH"],"level":1000.5},
                    {"type":"myAsset"}
                ],
                "ticket":"ticket-1"
            }"#,
        )
        .expect("the official response shape");

        assert_eq!(response.ticket, "ticket-1");
        assert_eq!(response.subscriptions[0].feed_type, "ticker");
        assert_eq!(
            response.subscriptions[0].markets,
            [Market::spot(Exchange::Upbit, "BTC", "KRW")]
        );
        assert_eq!(response.subscriptions[1].feed_type, "orderbook");
        assert_eq!(
            response.subscriptions[1].level,
            Some(Decimal::new(10_005, 1))
        );
        assert_eq!(
            response.subscriptions[1].markets[1],
            Market::spot(Exchange::Upbit, "ETH", "BTC")
        );
        assert!(response.subscriptions[2].markets.is_empty());
    }

    #[test]
    fn list_subscriptions_rejects_a_different_operation_or_malformed_result() {
        for frame in [
            r#"{"method":"OTHER","result":[],"ticket":"t"}"#,
            r#"{"method":"LIST_SUBSCRIPTIONS","ticket":"t"}"#,
            r#"{"method":"LIST_SUBSCRIPTIONS","result":[{"type":"ticker","codes":[1]}],"ticket":"t"}"#,
        ] {
            assert!(matches!(
                subscription_list(frame),
                Err(Error::Decode { .. })
            ));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn list_subscriptions_uses_the_already_open_connection() {
        use tokio::sync::mpsc;
        use tokio_tungstenite::tungstenite::Message;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("the local address");
        let (frames, mut received) = mpsc::channel(2);
        let (complete, wait_for_complete) = oneshot::channel();
        tokio::spawn(async move {
            let mut wait_for_complete = Some(wait_for_complete);
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                return;
            };
            for index in 0..2 {
                let Some(Ok(Message::Text(frame))) = socket.next().await else {
                    return;
                };
                let frame = frame.to_string();
                if frames.send(frame.clone()).await.is_err() {
                    return;
                }
                if index == 1 {
                    let ticket = serde_json::from_str::<Value>(&frame)
                        .ok()
                        .and_then(|value| value[0]["ticket"].as_str().map(str::to_owned));
                    let Some(ticket) = ticket else {
                        return;
                    };
                    let _ = socket
                        .send(Message::Text(
                            serde_json::json!({
                                "method": "LIST_SUBSCRIPTIONS",
                                "result": [{"type": "ticker", "codes": ["KRW-BTC"]}],
                                "ticket": ticket,
                            })
                            .to_string()
                            .into(),
                        ))
                        .await;
                    let _ = wait_for_complete
                        .take()
                        .expect("only one operation response")
                        .await;
                }
            }
        });

        let mut session = crate::transport::ws::connect(
            crate::transport::WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: crate::transport::WsConnect::fixed(vec![
                    subscribe_frame(
                        &Subscription::new()
                            .market(Market::spot(Exchange::Upbit, "BTC", "KRW"))
                            .feed(Feed::Ticker),
                        "initial-ticket",
                    )
                    .expect("the initial subscription frame"),
                ]),
                heartbeat: None,
            },
            &crate::types::StreamConfig {
                max_reconnect_attempts: Some(0),
                ..crate::types::StreamConfig::default()
            },
        )
        .await
        .expect("the initial connection");
        let control = std::sync::Arc::new(SubscriptionControl::new(session.send_handle()));
        let query = {
            let control = std::sync::Arc::clone(&control);
            tokio::spawn(async move { control.list_subscriptions().await })
        };

        let initial = received.recv().await.expect("the initial frame");
        assert!(initial.contains("initial-ticket"));
        let operation = received.recv().await.expect("the operation frame");
        let operation = serde_json::from_str::<Value>(&operation).expect("the operation JSON");
        assert_eq!(operation[1]["method"], "LIST_SUBSCRIPTIONS");
        assert_eq!(operation[2]["format"], "DEFAULT");

        while let Some(command) = session.next().await {
            let crate::transport::WsCommand::Text(frame) = command.expect("the response frame")
            else {
                continue;
            };
            if control.handle_frame(&frame) {
                break;
            }
        }
        let result = query
            .await
            .expect("the query task")
            .expect("the operation response");
        assert_eq!(
            result.ticket,
            operation[0]["ticket"]
                .as_str()
                .expect("the operation ticket")
        );
        assert_eq!(result.subscriptions[0].feed_type, "ticker");
        let _ = complete.send(());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn list_subscriptions_fails_when_its_socket_reconnects() {
        use tokio::sync::mpsc;
        use tokio_tungstenite::tungstenite::Message;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("the local address");
        let (ready, mut initial_subscribed) = mpsc::channel(1);
        tokio::spawn(async move {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                return;
            };
            let Some(Ok(Message::Text(_))) = socket.next().await else {
                return;
            };
            if ready.send(()).await.is_err() {
                return;
            }
            // The operation is accepted on this socket, then the exchange
            // closes it before replying. A reply from the next socket cannot
            // answer this request.
            let Some(Ok(Message::Text(_))) = socket.next().await else {
                return;
            };
            let _ = socket.close(None).await;

            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                return;
            };
            let _ = socket.next().await;
            std::future::pending::<()>().await;
        });

        let session = crate::transport::ws::connect(
            crate::transport::WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: crate::transport::WsConnect::fixed(vec!["subscribe".to_owned()]),
                heartbeat: None,
            },
            &crate::types::StreamConfig {
                initial_reconnect_delay_ms: 1,
                max_reconnect_delay_ms: 1,
                idle_timeout_ms: 60_000,
                ..crate::types::StreamConfig::default()
            },
        )
        .await
        .expect("the initial connection");
        let close = session.close_handle();
        let control = SubscriptionControl::new(session.send_handle());
        initial_subscribed
            .recv()
            .await
            .expect("the server received the initial subscription");

        let error = tokio::time::timeout(Duration::from_secs(5), control.list_subscriptions())
            .await
            .expect("the reconnect boundary is reported")
            .expect_err("the original socket was replaced");
        assert!(matches!(error, Error::Transport { .. }), "{error}");
        close.close().await.expect("the session closes");
    }

    #[test]
    fn subscribing_to_nothing_is_refused_before_the_socket_opens() {
        let no_feed = Subscription::new().market(Market::spot(Exchange::Upbit, "BTC", "KRW"));
        let no_market = Subscription::new().feed(Feed::Trades);

        assert!(matches!(
            subscribe_frame(&no_feed, "t"),
            Err(Error::InvalidRequest { field, .. }) if field == "feeds"
        ));
        assert!(matches!(
            subscribe_frame(&no_market, "t"),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
        ));
    }

    #[test]
    fn every_candle_feed_upbit_streams_round_trips_through_its_name() {
        for interval in [
            Interval::Sec1,
            Interval::Min1,
            Interval::Min3,
            Interval::Min5,
            Interval::Min15,
            Interval::Min30,
            Interval::Hour1,
            Interval::Hour4,
        ] {
            let name = candle_type(interval).expect("a streamed interval");
            assert_eq!(candle_interval(name), Some(interval), "{name}");
        }
    }

    #[test]
    fn a_candle_interval_upbit_does_not_stream_is_refused() {
        for interval in [Interval::Day1, Interval::Week1, Interval::Month1] {
            assert!(
                matches!(
                    feed_type(Feed::Candles(interval)),
                    Err(Error::Unsupported {
                        feature: Feature::CandleStream,
                        ..
                    })
                ),
                "{interval:?}"
            );
        }
    }

    #[test]
    fn a_trade_frame_becomes_a_trade_event() {
        let MarketEvent::Trade(trade) = one(TRADE) else {
            panic!("expected a trade event");
        };

        assert_eq!(trade.market, Market::spot(Exchange::Upbit, "BTC", "KRW"));
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.quantity, Decimal::new(8_428, 8));
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_696_585_056_846));
    }

    #[test]
    fn an_order_book_frame_becomes_a_sorted_book() {
        let MarketEvent::OrderBook(book) = one(ORDER_BOOK) else {
            panic!("expected an order book event");
        };

        assert_eq!(
            book.best_bid().expect("a bid").price,
            Decimal::new(1_247_430, 1)
        );
        assert_eq!(
            book.best_ask().expect("an ask").price,
            Decimal::new(1_250_560, 1)
        );
        assert_eq!(book.timestamp, Timestamp::from_millis(1_746_602_359_173));
    }

    #[test]
    fn a_ticker_frame_keeps_the_sign_of_a_falling_market() {
        let MarketEvent::Ticker(ticker) = one(TICKER) else {
            panic!("expected a ticker event");
        };

        assert_eq!(ticker.change, Some(Decimal::from(-306)));
        assert_eq!(ticker.change_rate, Some(Decimal::new(-82_180_744, 10)));
        assert_eq!(ticker.volume, Some(Decimal::new(22_569_412, 8)));
    }

    #[test]
    fn a_candle_frame_takes_its_interval_from_the_frame_type() {
        let MarketEvent::Candle(candle) = one(CANDLE_SNAPSHOT) else {
            panic!("expected a candle event");
        };

        assert_eq!(candle.interval, Interval::Min1);
        assert_eq!(candle.open_time, Timestamp::from_secs(MINUTE_0745));
        assert_eq!(candle.volume, Decimal::new(15_511_395, 8));
    }

    #[test]
    fn a_snapshot_for_an_ended_window_is_closed_immediately() {
        let mut decoder = Decoder::default();
        let events = decoder
            .decode_at(CANDLE_SNAPSHOT, Timestamp::from_secs(MINUTE_0746))
            .expect("an ended snapshot");
        let [MarketEvent::Candle(candle)] = events.as_slice() else {
            panic!("expected a candle event");
        };

        assert!(candle.closed);
        assert!(
            decoder
                .decode_at(CANDLE_SNAPSHOT, Timestamp::from_secs(MINUTE_0746))
                .expect("the same snapshot after reconnect")
                .is_empty(),
            "a settled snapshot must not be emitted twice"
        );
    }

    /// Extracts a settled candle followed by its forming successor.
    fn rollover(events: Vec<MarketEvent>) -> (Candle, Candle) {
        let candles = events
            .into_iter()
            .map(|event| match event {
                MarketEvent::Candle(candle) => candle,
                other => panic!("expected candle events, got {other:?}"),
            })
            .collect::<Vec<_>>();
        let [settled, forming] = <[Candle; 2]>::try_from(candles)
            .expect("expected a settled candle followed by a forming one");

        assert!(settled.closed, "the first of the pair is the settled one");
        assert!(!forming.closed, "the second of the pair is still forming");
        (settled, forming)
    }

    #[test]
    fn the_frame_that_opens_a_window_settles_the_one_before_it() {
        let mut decoder = Decoder::default();

        let forming = decoder
            .decode(CANDLE_LAST_OF_ITS_MINUTE)
            .expect("the last frame of 07:46");
        let [MarketEvent::Candle(forming)] = forming.as_slice() else {
            panic!("expected one candle event");
        };
        assert!(!forming.closed);
        assert_eq!(forming.open_time, Timestamp::from_secs(MINUTE_0746));

        let (settled, next) = rollover(
            decoder
                .decode(CANDLE_NEXT_MINUTE)
                .expect("the first frame of 07:47"),
        );

        assert_eq!(settled.open_time, forming.open_time);
        assert_eq!(settled.volume, Decimal::new(80_044_670, 8));
        assert_eq!(next.open_time, Timestamp::from_secs(MINUTE_0747));
    }

    #[test]
    fn a_frame_for_a_window_already_settled_neither_reopens_it_nor_displaces_the_held_one() {
        let mut decoder = Decoder::default();

        decoder
            .decode(CANDLE_LAST_OF_ITS_MINUTE)
            .expect("the last frame of 07:46");
        let (settled, _) = rollover(
            decoder
                .decode(CANDLE_NEXT_MINUTE)
                .expect("the first frame of 07:47"),
        );
        assert_eq!(settled.open_time, Timestamp::from_secs(MINUTE_0746));

        let late = decoder
            .decode(CANDLE_LAST_OF_ITS_MINUTE)
            .expect("a frame for a window that already settled");
        assert!(
            late.is_empty(),
            "a frame behind the held window carried events: {late:?}"
        );

        let (settled, forming) = rollover(
            decoder
                .decode(CANDLE_MINUTE_AFTER_NEXT)
                .expect("the first frame of 07:48"),
        );
        assert_eq!(settled.open_time, Timestamp::from_secs(MINUTE_0747));
        assert_eq!(settled.volume, Decimal::new(98_728, 8));
        assert_eq!(forming.open_time, Timestamp::from_secs(MINUTE_0748));
    }

    #[test]
    fn a_snapshot_for_a_later_window_replaces_the_held_one_rather_than_settling_it() {
        let mut decoder = Decoder::default();

        decoder
            .decode(CANDLE_LAST_OF_ITS_MINUTE)
            .expect("the last frame of 07:46");
        let after_reconnect = decoder
            .decode_at(
                CANDLE_SNAPSHOT_AFTER_RECONNECT,
                Timestamp::from_secs(MINUTE_0747 + 30),
            )
            .expect("a snapshot for the minute after the held one");

        assert!(
            matches!(
                after_reconnect.as_slice(),
                [MarketEvent::Candle(candle)]
                    if !candle.closed && candle.open_time == Timestamp::from_secs(MINUTE_0747)
            ),
            "07:46 was settled across a reconnect that may have hidden its later \
             frames: {after_reconnect:?}"
        );

        let (settled, _) = rollover(
            decoder
                .decode(CANDLE_MINUTE_AFTER_NEXT)
                .expect("the first frame of 07:48"),
        );
        assert_eq!(settled.open_time, Timestamp::from_secs(MINUTE_0747));
    }

    #[test]
    fn a_snapshot_for_the_window_already_held_replaces_it_without_settling_anything() {
        let mut decoder = Decoder::default();

        decoder
            .decode(CANDLE_NEXT_MINUTE)
            .expect("the first frame of 07:47");
        let after_reconnect = decoder
            .decode_at(
                CANDLE_SNAPSHOT_AFTER_RECONNECT,
                Timestamp::from_secs(MINUTE_0747 + 30),
            )
            .expect("a snapshot for the minute already held");
        assert_eq!(after_reconnect.len(), 1);

        let (settled, _) = rollover(
            decoder
                .decode(CANDLE_MINUTE_AFTER_NEXT)
                .expect("the first frame of 07:48"),
        );
        assert_eq!(settled.open_time, Timestamp::from_secs(MINUTE_0747));
        assert_eq!(settled.volume, Decimal::new(1_011_827, 8));
    }

    #[test]
    fn the_held_bars_are_one_per_market_and_interval_however_long_the_subscription_runs() {
        let mut decoder = Decoder::default();

        for _ in 0..1_000 {
            for frame in [
                CANDLE_LAST_OF_ITS_MINUTE,
                CANDLE_NEXT_MINUTE,
                CANDLE_MINUTE_AFTER_NEXT,
                CANDLE_SNAPSHOT,
                CANDLE_ONE_MINUTE_0750,
                CANDLE_OTHER_INTERVAL,
                CANDLE_OTHER_MARKET,
            ] {
                decoder
                    .decode_at(frame, Timestamp::from_secs(MINUTE_0750 + 30))
                    .expect("a candle frame");
            }
        }

        assert_eq!(decoder.latest.len(), 3);
    }

    #[test]
    fn two_intervals_on_one_market_settle_independently() {
        let mut decoder = Decoder::default();

        decoder
            .decode(CANDLE_LAST_OF_ITS_MINUTE)
            .expect("the last 1m frame of 07:46");
        let opened = decoder
            .decode_at(
                CANDLE_OTHER_INTERVAL,
                Timestamp::from_secs(MINUTE_0750 + 30),
            )
            .expect("the first 5m frame");
        assert!(matches!(
            opened.as_slice(),
            [MarketEvent::Candle(candle)]
                if candle.interval == Interval::Min5 && !candle.closed
        ));

        let (settled, _) = rollover(
            decoder
                .decode(CANDLE_NEXT_MINUTE)
                .expect("the first 1m frame of 07:47"),
        );
        assert_eq!(settled.interval, Interval::Min1);
        assert_eq!(settled.open_time, Timestamp::from_secs(MINUTE_0746));
    }

    #[test]
    fn a_frame_wrapped_in_a_single_element_list_reads_the_same() {
        let wrapped = format!("[{TRADE}]");

        assert!(matches!(one(&wrapped), MarketEvent::Trade(_)));
    }

    #[test]
    fn detailed_events_keep_native_fields_and_each_candle_owning_frame() {
        let mut decoder = DetailedDecoder::default();

        let trades = decoder.decode(TRADE).expect("a detailed trade frame");
        let [UpbitMarketStreamEvent::Trade(trade)] = trades.as_slice() else {
            panic!("expected one detailed trade");
        };
        assert_eq!(trade.change.as_deref(), Some("RISE"));
        assert_eq!(
            trade.best_ask_price.expect("best ask").to_string(),
            "32293000"
        );
        assert!(trade.raw_json.contains("best_bid_size"));

        let books = decoder.decode(ORDER_BOOK).expect("a detailed order book");
        let [UpbitMarketStreamEvent::OrderBook(book)] = books.as_slice() else {
            panic!("expected one detailed order book");
        };
        assert_eq!(
            book.total_ask_size.expect("ask total").to_string(),
            "0.68780013"
        );

        let tickers = decoder.decode(TICKER).expect("a detailed ticker");
        let [UpbitMarketStreamEvent::Ticker(ticker)] = tickers.as_slice() else {
            panic!("expected one detailed ticker");
        };
        assert_eq!(ticker.market_state.as_deref(), Some("ACTIVE"));
        assert_eq!(ticker.trading_suspended, Some(false));
        assert!(ticker.raw_json.contains("highest_52_week_price"));

        let _ = decoder
            .decode_at(
                CANDLE_LAST_OF_ITS_MINUTE,
                Timestamp::from_secs(MINUTE_0746 + 30),
            )
            .expect("a first candle");
        let candles = decoder
            .decode_at(CANDLE_NEXT_MINUTE, Timestamp::from_secs(MINUTE_0747 + 30))
            .expect("a later candle");
        let [
            UpbitMarketStreamEvent::Candle(settled),
            UpbitMarketStreamEvent::Candle(forming),
        ] = candles.as_slice()
        else {
            panic!("expected a settled and a forming candle: {candles:?}");
        };
        assert!(settled.common.closed);
        assert!(settled.raw_json.contains("07:46:00"));
        assert!(!forming.common.closed);
        assert!(forming.raw_json.contains("07:47:00"));
    }

    #[test]
    fn a_keepalive_answer_is_not_reported_as_market_data() {
        assert!(
            decode(r#"{"status":"UP"}"#)
                .expect("a control frame")
                .is_empty()
        );
    }

    #[test]
    fn the_heartbeat_is_the_frame_upbit_answers_rather_than_one_it_errors_on() {
        assert_eq!(HEARTBEAT.frame, HeartbeatFrame::Text("PING"));
        assert!(
            decode(r#"{"status":"UP"}"#)
                .expect("the answer to this heartbeat")
                .is_empty()
        );
        assert!(HEARTBEAT.interval * 4 <= Duration::from_secs(120));
        assert!(HEARTBEAT.min_idle_timeout >= HEARTBEAT.interval * 3);
    }

    #[test]
    fn an_error_frame_carries_upbits_own_name_and_message() {
        let error = decode(r#"{"error":{"name":"WRONG_FORMAT","message":"Wrong Format"}}"#)
            .expect_err("an error frame");

        assert!(matches!(
            &error,
            Error::Exchange { exchange: "upbit", code, message, status: None, .. }
                if code == "WRONG_FORMAT" && message == "Wrong Format"
        ));
    }

    #[test]
    fn a_frame_maxt_cannot_place_is_a_decode_error_not_a_silent_drop() {
        assert!(matches!(
            decode(r#"{"code":"KRW-BTC"}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode(r#"{"type":"candle.3m","code":"KRW-BTC"}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(decode("not json"), Err(Error::Decode { .. })));
        assert!(matches!(decode("[1,2]"), Err(Error::Decode { .. })));
    }
}
