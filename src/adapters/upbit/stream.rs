//! Upbit's public market data WebSocket.
//!
//! One connection can carry multiple markets and feed types.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::transport::{Heartbeat, HeartbeatFrame};
use crate::types::{Candle, Feed, Interval, Market, MarketEvent, Subscription, Timestamp};

use super::parse::{self, EXCHANGE, RawStreamCandle};

/// Full-name JSON frame format.
const FRAME_FORMAT: &str = "DEFAULT";

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
