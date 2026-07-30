//! Hyperliquid's WebSocket, public and private.
//!
//! One socket carries everything, but unlike most exchanges the subscriptions
//! are not batched: each market and feed pair is its own `subscribe` frame, and
//! each arrives back on its own channel.
//!
//! This is also the only unbroken record of Hyperliquid's trades.
//! [`Client::trades`](crate::Client::trades) reads the `recentTrades` endpoint,
//! which serves a fixed window of the last ten and no time range, so a gap wider
//! than ten trades cannot be read back from it.

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::transport::{Heartbeat, HeartbeatFrame};
use crate::types::{
    AccountEvent, Candle, Feed, Interval, Market, MarketEvent, Subscription, Timestamp,
};

use super::parse::{self, EXCHANGE, Universe};

/// What holds a Hyperliquid socket open when nothing is trading.
///
/// Hyperliquid closes a connection it has received nothing on for 60 seconds,
/// which is short enough that an order book on a thin market reaches it between
/// updates. `{"method":"ping"}` is answered with a `pong` channel frame, the
/// one [`decode`] and [`decode_account`] drop, so a heartbeat both resets
/// Hyperliquid's own timer and produces the inbound traffic the idle timer
/// wants. A WebSocket ping would be answered by the protocol stack, which is
/// not what Hyperliquid's 60 seconds measures.
pub(crate) const HEARTBEAT: Heartbeat = Heartbeat {
    interval: Duration::from_secs(15),
    frame: HeartbeatFrame::Text(r#"{"method":"ping"}"#),
    // Four unanswered heartbeats before a socket is written off.
    min_idle_timeout: Duration::from_secs(60),
};

/// Builds the frames sent on connect, and again after every reconnect.
///
/// One frame per market and feed pair, plus one shared frame per feed that is
/// account-wide instead of market-scoped.
pub(crate) fn subscribe_frames(
    subscription: &Subscription,
    universe: &Universe,
) -> Result<Vec<String>> {
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

    let mut frames = Vec::new();
    for market in subscription.markets() {
        let native = universe.native_symbol(market)?;
        for feed in subscription.feeds() {
            frames.push(frame(&feed_subscription(*feed, native)?));
        }
    }

    Ok(frames)
}

/// The frames a private subscription opens, for one account.
pub(crate) fn account_subscribe_frames(user: &str) -> Vec<String> {
    vec![
        frame(&json!({ "type": "orderUpdates", "user": user })),
        // Hyperliquid streams spot balances and perpetual state on separate
        // channels; `AccountEvent::Balance` is a spot-shaped idea, so this is
        // the one that maps.
        frame(&json!({ "type": "spotState", "user": user })),
    ]
}

fn frame(subscription: &Value) -> String {
    json!({ "method": "subscribe", "subscription": subscription }).to_string()
}

fn feed_subscription(feed: Feed, native: &str) -> Result<Value> {
    Ok(match feed {
        Feed::Trades => json!({ "type": "trades", "coin": native }),
        Feed::OrderBook => json!({ "type": "l2Book", "coin": native }),
        // Not `allMids`: that streams a bare mid price for every market at once,
        // where this carries the same fields the REST summary is built from.
        Feed::Ticker => json!({ "type": "activeAssetCtx", "coin": native }),
        Feed::Candles(interval) => {
            let name = parse::interval_name(interval)
                .ok_or_else(|| parse::unsupported_interval(interval, Feature::CandleStream))?;
            json!({ "type": "candle", "coin": native, "interval": name })
        }
    })
}

/// Reads the frames of one public connection, in order.
///
/// Every frame but a candle is a self-contained event, so the decoder holds
/// nothing for it. A candle is the exception. Hyperliquid republishes the
/// forming window on every update and marks none of them finished, and it stops
/// publishing a window a couple of seconds *before* the window's own close time,
/// so a frame's `T` is still in the future when the last frame of that window
/// arrives. Read live on 2026-07-30, `candle` `1m` on BTC, 148 frames over 210
/// seconds and five windows: the last frame of a window was stamped 1.7, 2.1 and
/// 2.4 seconds before its `T` on three of them, and 0.3 seconds after it on one.
/// So neither the payload nor a clock reading of one frame can say a window has
/// settled, and a clock says so at most by luck.
///
/// What Hyperliquid does publish is the moment a window is over: the first frame
/// carrying a later `t`. So the decoder keeps the most recent frame of each
/// candle feed, and when a later window opens it emits the held one with
/// [`Candle::closed`] set, ahead of the new forming one. That is the contract
/// [`Candle::closed`] states, of one settled emission per window after a run of
/// forming ones, and it is reached without ever calling a window finished before
/// Hyperliquid itself moved on from it.
///
/// One decoder belongs to one connection. It holds one candle per market and
/// interval subscribed, and nothing else.
#[derive(Debug, Default)]
pub(crate) struct Decoder {
    /// The last frame seen of each candle feed, held until a later window opens
    /// or a reconnect drops it.
    latest: HashMap<(Market, Interval), Candle>,
}

impl Decoder {
    /// One frame off the public socket.
    ///
    /// Returns however many events the frame carried: Hyperliquid batches
    /// trades and candles into a list, answers a subscription with a control
    /// frame that is not market data at all, and a candle frame that opens a
    /// new window also settles the one before it.
    pub(crate) fn decode(
        &mut self,
        frame: &str,
        universe: &Universe,
        at: Timestamp,
    ) -> Result<Vec<MarketEvent>> {
        let (channel, data) = split(frame)?;

        Ok(match channel.as_str() {
            "trades" => read::<Vec<parse::RawTrade>>(&data)?
                .iter()
                .map(|raw| parse::trade(raw, universe).map(MarketEvent::Trade))
                .collect::<Result<Vec<_>>>()?,
            "l2Book" => vec![MarketEvent::OrderBook(parse::order_book(
                &read(&data)?,
                universe,
            )?)],
            "candle" => one_or_many::<parse::RawCandle>(&data)?
                .iter()
                .map(|raw| parse::candle(raw, universe, at))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flat_map(|candle| self.candle(candle))
                .collect(),
            // Perpetual and spot contexts arrive on channels of their own,
            // carrying the same shape.
            "activeAssetCtx" | "activeSpotAssetCtx" => {
                let raw: RawActiveAssetCtx = read(&data)?;
                let market = universe.market_from_native_symbol(&raw.coin)?;
                vec![MarketEvent::Ticker(parse::ticker(&raw.ctx, market, at)?)]
            }
            // `subscriptionResponse` acknowledges a subscribe and `pong` answers
            // a keepalive. Neither is something a caller should see.
            "subscriptionResponse" | "pong" => Vec::new(),
            "error" => return Err(channel_error(&data)),
            other => {
                return Err(Error::decode(format!(
                    "unexpected hyperliquid channel `{other}`"
                )));
            }
        })
    }

    /// Drops every held window, because the connection was replaced.
    ///
    /// A held frame is from before a gap the decoder cannot measure, and its
    /// window may have moved on unseen; emitting those figures as a settled bar
    /// would report a window's final state from a reading taken before the
    /// trades that finished it. After a reconnect the first window to settle is
    /// the one that opens next.
    pub(crate) fn reconnected(&mut self) {
        self.latest.clear();
    }

    /// Reads one candle, settling the window before it when this one opens a
    /// later window.
    fn candle(&mut self, forming: Candle) -> Vec<MarketEvent> {
        // Hyperliquid's payload cannot say a window is over, so nothing arriving
        // on this channel is settled by itself. `parse::candle` compares `T`
        // against the read clock, which is right over REST and a race here: a
        // straggler landing after `T` would be the second settled emission of a
        // window this decoder settles on its own.
        let forming = Candle {
            closed: false,
            ..forming
        };
        let key = (forming.market.clone(), forming.interval);

        // Ordered before anything is stored. A frame naming a window older than
        // the held one is a repeat of a window already settled, and storing it
        // would throw the newer bar away and then settle the stale one in its
        // place when the next window opens.
        let settled = match self.latest.get(&key) {
            Some(held) if forming.open_time < held.open_time => return Vec::new(),
            Some(held) if forming.open_time > held.open_time => Some(Candle {
                closed: true,
                ..held.clone()
            }),
            _ => None,
        };
        self.latest.insert(key, forming.clone());

        settled
            .into_iter()
            .chain([forming])
            .map(MarketEvent::Candle)
            .collect()
    }
}

/// One frame off the private socket.
pub(crate) fn decode_account(frame: &str, universe: &Universe) -> Result<Vec<AccountEvent>> {
    let (channel, data) = split(frame)?;

    Ok(match channel.as_str() {
        "orderUpdates" => read::<Vec<parse::RawStreamOrder>>(&data)?
            .iter()
            .map(|raw| parse::stream_order(raw, universe).map(AccountEvent::Order))
            .collect::<Result<Vec<_>>>()?,
        "spotState" => read::<parse::RawStreamSpotState>(&data)?
            .spot_state
            .balances
            .iter()
            .map(|raw| parse::balance(raw).map(AccountEvent::Balance))
            .collect::<Result<Vec<_>>>()?,
        "subscriptionResponse" | "pong" => Vec::new(),
        "error" => return Err(channel_error(&data)),
        other => {
            return Err(Error::decode(format!(
                "unexpected hyperliquid channel `{other}`"
            )));
        }
    })
}

/// The context body of an `activeAssetCtx` frame.
#[derive(Debug, Deserialize)]
struct RawActiveAssetCtx {
    coin: String,
    ctx: parse::RawAssetCtx,
}

/// Splits a frame into its channel and its payload.
fn split(frame: &str) -> Result<(String, Value)> {
    let value: Value = serde_json::from_str(frame)
        .map_err(|err| Error::decode(format!("unreadable hyperliquid frame: {err}")))?;

    let channel = value
        .get("channel")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::decode("hyperliquid frame carries no `channel`"))?
        .to_string();
    let data = value.get("data").cloned().unwrap_or(Value::Null);

    Ok((channel, data))
}

fn read<T: for<'de> Deserialize<'de>>(data: &Value) -> Result<T> {
    serde_json::from_value(data.clone())
        .map_err(|err| Error::decode(format!("unreadable hyperliquid frame data: {err}")))
}

/// Reads a payload Hyperliquid sends as a list sometimes and as a bare object
/// otherwise, which its candle channel does.
fn one_or_many<T: for<'de> Deserialize<'de>>(data: &Value) -> Result<Vec<T>> {
    match data {
        Value::Array(_) => read(data),
        _ => read(data).map(|single| vec![single]),
    }
}

/// Reads Hyperliquid's error frame, whose `data` is a bare sentence.
fn channel_error(data: &Value) -> Error {
    let message = data
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| data.to_string());

    Error::exchange(EXCHANGE, "subscription_error", message)
}

#[cfg(test)]
mod tests {
    use super::super::parse::tests::{btc_perp, universe};
    use super::*;
    use crate::types::{Exchange, Interval, Market, OrderStatus, Side};
    use rust_decimal::Decimal;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    const TRADES: &str = r#"{
      "channel": "trades",
      "data": [
        {
          "coin": "BTC",
          "side": "A",
          "px": "29295.0",
          "sz": "0.98639",
          "hash": "0xa166e3fa63c25663024b03f2e0da011a00307e4017465df020210d3d432e7cb8",
          "time": 1681923600000,
          "tid": 118906512037719,
          "users": [
            "0x0000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000002"
          ]
        }
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    const L2_BOOK: &str = r#"{
      "channel": "l2Book",
      "data": {
        "coin": "BTC",
        "time": 1754450974231,
        "levels": [
          [{"px": "113377.0", "sz": "7.6699", "n": 17}],
          [{"px": "113397.0", "sz": "0.11543", "n": 3}]
        ]
      }
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    // The candle channel sends its numbers unquoted, where REST quotes them.
    const CANDLE: &str = r#"{
      "channel": "candle",
      "data": {
        "T": 1681923659999,
        "c": 29258.0,
        "h": 29309.0,
        "i": "1m",
        "l": 29250.0,
        "n": 12,
        "o": 29295.0,
        "s": "BTC",
        "t": 1681923600000,
        "v": 0.98639
      }
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    const ACTIVE_ASSET_CTX: &str = r#"{
      "channel": "activeAssetCtx",
      "data": {
        "coin": "BTC",
        "ctx": {
          "dayNtlVlm": "1169046.29406",
          "prevDayPx": "15.322",
          "markPx": "14.3161",
          "midPx": "14.314",
          "funding": "0.0000125",
          "openInterest": "688.11",
          "oraclePx": "14.325",
          "dayBaseVlm": "81584.5"
        }
      }
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    const ORDER_UPDATES: &str = r#"{
      "channel": "orderUpdates",
      "data": [
        {
          "order": {
            "coin": "BTC",
            "side": "B",
            "limitPx": "29792.0",
            "sz": "0.4",
            "oid": 91490942,
            "timestamp": 1681247412573,
            "origSz": "1.0"
          },
          "status": "open",
          "statusTimestamp": 1681247412573
        }
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    const SPOT_STATE: &str = r#"{
      "channel": "spotState",
      "data": {
        "user": "0x14791697260e4c9a71f18484c9f997b308e59325",
        "spotState": {
          "balances": [
            {"coin": "PURR", "token": 1, "hold": "3.0", "total": "2000.0"}
          ]
        }
      }
    }"#;

    fn at() -> Timestamp {
        Timestamp::from_millis(1_700_000_000_000)
    }

    /// One frame through a decoder that has seen nothing else.
    ///
    /// Every frame but a candle is self-contained, so a fresh decoder is the
    /// whole story for them. The candle tests below drive a decoder of their
    /// own, because what they are about is what it holds between frames.
    fn decode(frame: &str, universe: &Universe, at: Timestamp) -> Result<Vec<MarketEvent>> {
        Decoder::default().decode(frame, universe, at)
    }

    fn subscription() -> Subscription {
        Subscription::new()
            .market(btc_perp())
            .market(Market::spot(Exchange::Hyperliquid, "HYPE", "USDC"))
            .feed(Feed::Trades)
            .feed(Feed::Candles(Interval::Min1))
    }

    #[test]
    fn each_market_and_feed_pair_is_its_own_subscribe_frame() {
        // Hyperliquid takes no market list per subscription, so two markets and
        // two feeds are four frames rather than one.
        let frames = subscribe_frames(&subscription(), &universe()).expect("a valid subscription");
        let parsed: Vec<Value> = frames
            .iter()
            .map(|frame| serde_json::from_str(frame).expect("valid JSON"))
            .collect();

        assert_eq!(parsed.len(), 4);
        assert!(parsed.iter().all(|frame| frame["method"] == "subscribe"));
        assert_eq!(parsed[0]["subscription"]["type"], "trades");
        assert_eq!(parsed[0]["subscription"]["coin"], "BTC");
        assert_eq!(parsed[1]["subscription"]["type"], "candle");
        assert_eq!(parsed[1]["subscription"]["interval"], "1m");
        // The spot market goes out under its index name, not `HYPE`.
        assert_eq!(parsed[2]["subscription"]["coin"], "@107");
    }

    #[test]
    fn subscribing_to_nothing_is_refused_before_the_socket_opens() {
        let universe = universe();
        let no_feed = Subscription::new().market(btc_perp());
        let no_market = Subscription::new().feed(Feed::Trades);

        assert!(matches!(
            subscribe_frames(&no_feed, &universe),
            Err(Error::InvalidRequest { field: "feeds", .. })
        ));
        assert!(matches!(
            subscribe_frames(&no_market, &universe),
            Err(Error::InvalidRequest {
                field: "markets",
                ..
            })
        ));
    }

    #[test]
    fn an_interval_hyperliquid_does_not_stream_is_refused() {
        assert!(matches!(
            feed_subscription(Feed::Candles(Interval::Sec1), "BTC"),
            Err(Error::Unsupported {
                feature: Feature::CandleStream,
                ..
            })
        ));
    }

    #[test]
    fn an_unlisted_market_is_refused_before_the_socket_opens() {
        let unlisted = Subscription::new()
            .market(Market::perpetual(Exchange::Hyperliquid, "NOPE", "USDC"))
            .feed(Feed::Trades);

        assert!(subscribe_frames(&unlisted, &universe()).is_err());
    }

    #[test]
    fn a_trade_frame_carries_a_list_and_becomes_one_event_per_trade() {
        let events = decode(TRADES, &universe(), at()).expect("a data frame");
        let [MarketEvent::Trade(trade)] = events.as_slice() else {
            panic!("expected one trade event");
        };

        assert_eq!(trade.market, btc_perp());
        // `A` means the taker hit a bid.
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.id.as_deref(), Some("118906512037719"));
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_681_923_600_000));
    }

    #[test]
    fn an_order_book_frame_becomes_a_sorted_book() {
        let events = decode(L2_BOOK, &universe(), at()).expect("a data frame");
        let [MarketEvent::OrderBook(book)] = events.as_slice() else {
            panic!("expected one order book event");
        };

        assert_eq!(
            book.best_bid().expect("a bid").price,
            Decimal::new(1_133_770, 1)
        );
        assert_eq!(
            book.best_ask().expect("an ask").price,
            Decimal::new(1_133_970, 1)
        );
        assert_eq!(book.timestamp, Timestamp::from_millis(1_754_450_974_231));
    }

    #[test]
    fn a_candle_frame_reads_unquoted_numbers_without_going_through_a_float() {
        let events = decode(CANDLE, &universe(), at()).expect("a data frame");
        let [MarketEvent::Candle(candle)] = events.as_slice() else {
            panic!("expected one candle event");
        };

        assert_eq!(candle.interval, Interval::Min1);
        assert_eq!(candle.volume, Decimal::new(98_639, 5));
        assert_eq!(candle.open_time, Timestamp::from_millis(1_681_923_600_000));
        // Forming, though this frame's `T` is years before `at`. Nothing on
        // this channel says a window has ended, so the first frame of one never
        // settles it; the frame that opens the next window does.
        assert!(!candle.closed);
    }

    /// A candle frame for one window, with the figures a settled emission has
    /// to carry through.
    fn candle_frame(open_ms: i64, close: i64) -> String {
        json!({
            "channel": "candle",
            "data": {
                "T": open_ms + 59_999,
                "c": close,
                "h": close,
                "i": "1m",
                "l": close,
                "n": 12,
                "o": close,
                "s": "BTC",
                "t": open_ms,
                "v": 1
            }
        })
        .to_string()
    }

    /// Two consecutive one-minute windows off Hyperliquid's `candle` channel.
    ///
    /// Read live on 2026-07-30 subscribed to `{"type":"candle","coin":"BTC",
    /// "interval":"1m"}`, 148 frames over 210 seconds. Every window's frames
    /// carry the same `t` and `T` and are republished as trades move them, and
    /// the last frame of a window arrives before that window's `T`:
    ///
    /// ```text
    /// recv=1785397557557  t=1785397500000  T=1785397559999  T_minus_now= 2442
    /// recv=1785397561416  t=1785397560000  T=1785397619999  T_minus_now=58583
    /// ```
    ///
    /// So the second line is the only announcement the first window ever gets
    /// that it is over.
    const WINDOW_ONE_MS: i64 = 1_785_397_500_000;
    const WINDOW_TWO_MS: i64 = 1_785_397_560_000;

    #[test]
    fn a_window_settles_when_a_later_one_opens_because_no_frame_of_it_ever_says_so() {
        let universe = universe();
        let mut decoder = Decoder::default();
        // A clock past both windows' close times, so nothing here rides on one:
        // if the read clock could settle a window, every one of these frames
        // would already be settled.
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);

        // The first window, republished as it forms. None of it is settled.
        for close in [100, 101, 102] {
            let events = decoder
                .decode(&candle_frame(WINDOW_ONE_MS, close), &universe, long_after)
                .expect("a candle frame");
            let [MarketEvent::Candle(forming)] = events.as_slice() else {
                panic!("a forming window is one event: {events:?}");
            };
            assert!(!forming.closed);
            assert_eq!(forming.open_time, Timestamp::from_millis(WINDOW_ONE_MS));
        }

        // The next window opening is what settles the one before it.
        let events = decoder
            .decode(&candle_frame(WINDOW_TWO_MS, 200), &universe, long_after)
            .expect("a candle frame");
        let [MarketEvent::Candle(settled), MarketEvent::Candle(forming)] = events.as_slice() else {
            panic!("a window ending is the settled one then the new one: {events:?}");
        };

        assert!(settled.closed, "the window before this frame is over");
        assert_eq!(settled.open_time, Timestamp::from_millis(WINDOW_ONE_MS));
        // The settled bar carries that window's own last figures, not the new
        // window's and not the first frame's.
        assert_eq!(settled.close, Decimal::from(102));
        assert!(!forming.closed);
        assert_eq!(forming.open_time, Timestamp::from_millis(WINDOW_TWO_MS));
    }

    #[test]
    fn exactly_one_settled_emission_per_window() {
        let universe = universe();
        let mut decoder = Decoder::default();
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);
        let mut settled = 0usize;
        let mut forming = 0usize;

        for frame in [
            candle_frame(WINDOW_ONE_MS, 100),
            candle_frame(WINDOW_ONE_MS, 101),
            candle_frame(WINDOW_TWO_MS, 200),
            candle_frame(WINDOW_TWO_MS, 201),
            candle_frame(WINDOW_TWO_MS + 60_000, 300),
        ] {
            for event in decoder.decode(&frame, &universe, long_after).expect("ok") {
                let MarketEvent::Candle(candle) = event else {
                    panic!("the candle channel carries candles");
                };
                if candle.closed {
                    settled += 1;
                } else {
                    forming += 1;
                }
            }
        }

        // Two windows ended over those five frames, and the third is still open.
        assert_eq!(settled, 2);
        assert_eq!(forming, 5);
    }

    #[test]
    fn a_frame_for_an_older_window_does_not_displace_the_one_being_held() {
        // A repeat or a reordering of a window already settled. Storing it would
        // throw away the newer bar and then settle this stale one in its place
        // when the next window opens, reporting figures from two windows back as
        // the newer window's own close.
        let universe = universe();
        let mut decoder = Decoder::default();
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);

        decoder
            .decode(&candle_frame(WINDOW_ONE_MS, 100), &universe, long_after)
            .expect("ok");
        decoder
            .decode(&candle_frame(WINDOW_TWO_MS, 200), &universe, long_after)
            .expect("ok");

        // The late frame itself is not reported: its window already settled, and
        // a forming event after that would contradict the one settled emission
        // per window this decoder promises.
        let late = decoder
            .decode(&candle_frame(WINDOW_ONE_MS, 999), &universe, long_after)
            .expect("ok");
        assert!(late.is_empty(), "{late:?}");

        // And the window it settles next is still the newer one, with the newer
        // one's figures.
        let events = decoder
            .decode(
                &candle_frame(WINDOW_TWO_MS + 60_000, 300),
                &universe,
                long_after,
            )
            .expect("ok");
        let [MarketEvent::Candle(settled), _] = events.as_slice() else {
            panic!("expected a settled window and a forming one: {events:?}");
        };
        assert_eq!(settled.open_time, Timestamp::from_millis(WINDOW_TWO_MS));
        assert_eq!(settled.close, Decimal::from(200));
    }

    #[test]
    fn each_market_and_interval_settles_on_its_own_window() {
        // One connection carries a subscription per market and feed pair, so the
        // held windows have to be kept apart or one market's frame settles
        // another's bar.
        let universe = universe();
        let mut decoder = Decoder::default();
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);
        let hype = |open_ms: i64| candle_frame(open_ms, 7).replace("\"BTC\"", "\"@107\"");

        decoder
            .decode(&candle_frame(WINDOW_ONE_MS, 100), &universe, long_after)
            .expect("ok");

        // A different market opening a later window settles nothing of BTC's.
        let events = decoder
            .decode(&hype(WINDOW_TWO_MS), &universe, long_after)
            .expect("ok");
        assert_eq!(events.len(), 1, "{events:?}");

        // BTC's own next window still settles BTC's held bar.
        let events = decoder
            .decode(&candle_frame(WINDOW_TWO_MS, 200), &universe, long_after)
            .expect("ok");
        let [MarketEvent::Candle(settled), _] = events.as_slice() else {
            panic!("expected BTC's window to settle: {events:?}");
        };
        assert_eq!(settled.market, btc_perp());
        assert_eq!(settled.open_time, Timestamp::from_millis(WINDOW_ONE_MS));
    }

    #[test]
    fn a_ticker_frame_reads_the_same_context_the_rest_summary_does() {
        let events = decode(ACTIVE_ASSET_CTX, &universe(), at()).expect("a data frame");
        let [MarketEvent::Ticker(ticker)] = events.as_slice() else {
            panic!("expected one ticker event");
        };

        assert_eq!(ticker.market, btc_perp());
        assert_eq!(ticker.last_price, Decimal::new(14_314, 3));
        assert_eq!(ticker.change, Some(Decimal::new(-1_008, 3)));
        assert_eq!(ticker.timestamp, at());
        assert_eq!(ticker.last_trade_time, None);
    }

    #[test]
    fn an_acknowledgement_or_a_keepalive_is_not_reported_as_market_data() {
        let universe = universe();

        assert!(
            decode(
                r#"{"channel":"subscriptionResponse","data":{"method":"subscribe"}}"#,
                &universe,
                at()
            )
            .expect("a control frame")
            .is_empty()
        );
        assert!(
            decode(r#"{"channel":"pong"}"#, &universe, at())
                .expect("a control frame")
                .is_empty()
        );
    }

    #[test]
    fn the_heartbeat_is_the_command_hyperliquid_answers_with_a_pong() {
        let HeartbeatFrame::Text(ping) = HEARTBEAT.frame else {
            panic!("hyperliquid's keepalive is a command, not a protocol ping");
        };
        // Hyperliquid reads this socket's text frames as commands, so the
        // heartbeat has to be the `ping` method rather than a bare word.
        let sent: Value = serde_json::from_str(ping).expect("the heartbeat is JSON");
        assert_eq!(sent["method"], "ping");

        // Sending it is what makes the `pong` arm below reachable at all: until
        // there was a heartbeat, nothing ever asked for one. Both sockets carry
        // it, and neither may read it as data or as a fault.
        assert!(
            decode(r#"{"channel":"pong"}"#, &universe(), at())
                .expect("the answer to this heartbeat")
                .is_empty()
        );
        assert!(
            decode_account(r#"{"channel":"pong"}"#, &universe())
                .expect("the answer to this heartbeat")
                .is_empty()
        );

        // Hyperliquid hangs up after 60 seconds with nothing received, which is
        // short enough that a thin market reaches it between updates.
        assert!(HEARTBEAT.interval * 4 <= Duration::from_secs(60));
        assert!(HEARTBEAT.min_idle_timeout >= HEARTBEAT.interval * 3);
    }

    #[test]
    fn an_error_frame_carries_hyperliquids_own_sentence() {
        let error = decode(
            r#"{"channel":"error","data":"Invalid subscription {\"type\":\"nope\"}"}"#,
            &universe(),
            at(),
        )
        .expect_err("an error frame");

        assert!(matches!(
            &error,
            Error::Exchange { exchange: "hyperliquid", code, message, .. }
                if code == "subscription_error" && message.starts_with("Invalid subscription")
        ));
    }

    #[test]
    fn a_frame_maxt_cannot_place_is_a_decode_error_not_a_silent_drop() {
        let universe = universe();

        assert!(matches!(
            decode(r#"{"channel":"webData2","data":{}}"#, &universe, at()),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode(r#"{"data":{}}"#, &universe, at()),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode("not json", &universe, at()),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn the_private_socket_opens_one_frame_per_kind_of_account_change() {
        let frames = account_subscribe_frames("0xabc");
        let parsed: Vec<Value> = frames
            .iter()
            .map(|frame| serde_json::from_str(frame).expect("valid JSON"))
            .collect();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["subscription"]["type"], "orderUpdates");
        assert_eq!(parsed[0]["subscription"]["user"], "0xabc");
        assert_eq!(parsed[1]["subscription"]["type"], "spotState");
    }

    #[test]
    fn an_order_update_reports_how_far_along_the_order_is() {
        let events = decode_account(ORDER_UPDATES, &universe()).expect("a data frame");
        let [AccountEvent::Order(order)] = events.as_slice() else {
            panic!("expected one order event");
        };

        assert_eq!(order.id, "91490942");
        assert_eq!(order.side, Side::Buy);
        // Still open, but six tenths of it has gone.
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_quantity, Decimal::new(6, 1));
        assert_eq!(order.remaining_quantity, Decimal::new(4, 1));
    }

    #[test]
    fn a_spot_state_frame_becomes_one_balance_event_per_asset() {
        let events = decode_account(SPOT_STATE, &universe()).expect("a data frame");
        let [AccountEvent::Balance(balance)] = events.as_slice() else {
            panic!("expected one balance event");
        };

        assert_eq!(balance.asset, "PURR");
        assert_eq!(balance.available, Decimal::new(1_997, 0));
        assert_eq!(balance.locked, Decimal::new(3, 0));
    }
}
