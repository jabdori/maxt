//! Hyperliquid WebSocket subscription construction and frame decoding.
//!
//! Each market/feed pair uses a separate subscription message.

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

use super::{
    native::{self, HyperliquidAccountEvent, HyperliquidCandleEvent, HyperliquidMarketEvent},
    parse::{self, EXCHANGE, Universe},
};

/// Application-level heartbeat used by public and private sockets.
/// Hyperliquid answers `{"method":"ping"}` with a `pong` channel frame.
pub(crate) const HEARTBEAT: Heartbeat = Heartbeat {
    interval: Duration::from_secs(15),
    frame: HeartbeatFrame::Text(r#"{"method":"ping"}"#),
    min_idle_timeout: Duration::from_secs(60),
};

/// Builds one subscription frame per market/feed pair.
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

/// Builds the account-scoped subscription frames.
pub(crate) fn account_subscribe_frames(user: &str) -> Vec<String> {
    vec![
        frame(&json!({ "type": "orderUpdates", "user": user })),
        // Account balance events map from the spot-state channel.
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
        // `activeAssetCtx` supplies the fields used by the REST ticker mapping.
        Feed::Ticker => json!({ "type": "activeAssetCtx", "coin": native }),
        Feed::Candles(interval) => {
            let name = parse::interval_name(interval)
                .ok_or_else(|| parse::unsupported_interval(interval, Feature::CandleStream))?;
            json!({ "type": "candle", "coin": native, "interval": name })
        }
    })
}

/// Stateful decoder for one public WebSocket connection.
///
/// Candle frames do not carry a closed flag. The latest candle for each
/// market/interval is held until a later window opens, at which point the held
/// candle is emitted once with [`Candle::closed`] set. Reconnect clears this
/// state because settlement cannot be inferred across a gap.
#[derive(Debug, Default)]
pub(crate) struct Decoder {
    /// Latest forming candle per market and interval.
    latest: HashMap<(Market, Interval), Candle>,
}

impl Decoder {
    /// Decodes one public frame into zero or more market events.
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
            // Perpetual and spot context channels share one payload shape.
            "activeAssetCtx" | "activeSpotAssetCtx" => {
                let raw: RawActiveAssetCtx = read(&data)?;
                let market = universe.market_from_native_symbol(&raw.coin)?;
                vec![MarketEvent::Ticker(parse::ticker(&raw.ctx, market, at)?)]
            }
            // Control frames are consumed internally.
            "subscriptionResponse" | "pong" => Vec::new(),
            "error" => return Err(channel_error(&data)),
            other => {
                return Err(Error::decode(format!(
                    "unexpected hyperliquid channel `{other}`"
                )));
            }
        })
    }

    /// Clears candle state after a connection gap.
    pub(crate) fn reconnected(&mut self) {
        self.latest.clear();
    }

    /// Emits a forming candle and, when the window advances, its predecessor.
    fn candle(&mut self, forming: Candle) -> Vec<MarketEvent> {
        // Stream settlement is based on window advancement, not local clock time.
        let forming = Candle {
            closed: false,
            ..forming
        };
        let key = (forming.market.clone(), forming.interval);

        // Ignore stale windows before they can replace newer held state.
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

/// Stateful decoder for one full-fidelity public WebSocket connection.
///
/// It shares the common stream's candle settlement rules while retaining the
/// native fields in every emitted event.
#[derive(Debug, Default)]
pub(crate) struct DetailedDecoder {
    /// Latest forming native candle per market and interval.
    latest: HashMap<(Market, Interval), HyperliquidCandleEvent>,
}

impl DetailedDecoder {
    /// Decodes one public frame without narrowing provider-native fields.
    pub(crate) fn decode(
        &mut self,
        frame: &str,
        universe: &Universe,
        at: Timestamp,
    ) -> Result<Vec<HyperliquidMarketEvent>> {
        let (channel, data) = split(frame)?;

        Ok(match channel.as_str() {
            "trades" => one_or_many_values(&data)
                .iter()
                .map(|value| {
                    let raw = read::<parse::RawTrade>(value)?;
                    native::stream_trade(&raw, universe, value).map(HyperliquidMarketEvent::Trade)
                })
                .collect::<Result<Vec<_>>>()?,
            "l2Book" => {
                let raw = read::<parse::RawBook>(&data)?;
                vec![HyperliquidMarketEvent::OrderBook(
                    native::stream_order_book(&raw, universe, &data)?,
                )]
            }
            "candle" => one_or_many_values(&data)
                .iter()
                .map(|value| {
                    let raw = read::<parse::RawCandle>(value)?;
                    native::stream_candle(&raw, universe, value, at)
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flat_map(|candle| self.candle(candle))
                .collect(),
            "activeAssetCtx" | "activeSpotAssetCtx" => {
                let raw: RawActiveAssetCtx = read(&data)?;
                vec![HyperliquidMarketEvent::AssetContext(
                    native::stream_asset_context(&raw.coin, &raw.ctx, universe, &data, at)?,
                )]
            }
            "subscriptionResponse" | "pong" => Vec::new(),
            "error" => return Err(channel_error(&data)),
            other => {
                return Err(Error::decode(format!(
                    "unexpected hyperliquid channel `{other}`"
                )));
            }
        })
    }

    /// Clears native candle state after a connection gap.
    pub(crate) fn reconnected(&mut self) {
        self.latest.clear();
    }

    fn candle(&mut self, forming: HyperliquidCandleEvent) -> Vec<HyperliquidMarketEvent> {
        let HyperliquidCandleEvent { common, provider } = forming;
        let forming = HyperliquidCandleEvent {
            common: Candle {
                closed: false,
                ..common
            },
            provider,
        };
        let key = (forming.common.market.clone(), forming.common.interval);

        let settled = match self.latest.get(&key) {
            Some(held) if forming.common.open_time < held.common.open_time => return Vec::new(),
            Some(held) if forming.common.open_time > held.common.open_time => {
                let HyperliquidCandleEvent { common, provider } = held.clone();
                Some(HyperliquidCandleEvent {
                    common: Candle {
                        closed: true,
                        ..common
                    },
                    provider,
                })
            }
            _ => None,
        };
        self.latest.insert(key, forming.clone());

        settled
            .into_iter()
            .chain([forming])
            .map(HyperliquidMarketEvent::Candle)
            .collect()
    }
}

/// Decodes one private frame into zero or more account events.
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

/// Decodes one private frame without narrowing provider-native fields.
pub(crate) fn decode_detailed_account(
    frame: &str,
    universe: &Universe,
) -> Result<Vec<HyperliquidAccountEvent>> {
    let (channel, data) = split(frame)?;

    Ok(match channel.as_str() {
        "orderUpdates" => one_or_many_values(&data)
            .iter()
            .map(|value| {
                let raw = read::<parse::RawStreamOrder>(value)?;
                native::stream_order_update(&raw, universe, value)
                    .map(HyperliquidAccountEvent::OrderUpdate)
            })
            .collect::<Result<Vec<_>>>()?,
        "spotState" => {
            let raw = read::<parse::RawStreamSpotState>(&data)?;
            vec![HyperliquidAccountEvent::SpotState(
                native::stream_spot_state(&raw, &data)?,
            )]
        }
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

/// Normalizes a payload represented as either one object or a list.
fn one_or_many<T: for<'de> Deserialize<'de>>(data: &Value) -> Result<Vec<T>> {
    match data {
        Value::Array(_) => read(data),
        _ => read(data).map(|single| vec![single]),
    }
}

fn one_or_many_values(data: &Value) -> Vec<Value> {
    match data {
        Value::Array(values) => values.clone(),
        value => vec![value.clone()],
    }
}

/// Converts an error-channel payload to an exchange error.
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

    // Payload fixtures follow the official WebSocket subscription schema:
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

    const ACTIVE_ASSET_CTX: &str = r#"{
      "channel": "activeAssetCtx",
      "data": {
        "coin": "BTC",
        "ctx": {
          "dayNtlVlm": 1169046.29406,
          "prevDayPx": 15.322,
          "markPx": 14.3161,
          "midPx": 14.314,
          "funding": 0.0000125,
          "openInterest": 688.11,
          "oraclePx": 14.325,
          "dayBaseVlm": 81584.5
        }
      }
    }"#;

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
            "origSz": "1.0",
            "cloid": "0x00000000000000000000000000000001"
          },
          "status": "open",
          "statusTimestamp": 1681247412573
        }
      ]
    }"#;

    const SPOT_STATE: &str = r#"{
      "channel": "spotState",
      "data": {
        "user": "0x14791697260e4c9a71f18484c9f997b308e59325",
        "spotState": {
          "balances": [
            {"coin": "PURR", "token": 1, "hold": "3.0", "total": "2000.0", "entryNtl": "100.0"}
          ]
        }
      }
    }"#;

    fn at() -> Timestamp {
        Timestamp::from_millis(1_700_000_000_000)
    }

    /// Decodes a self-contained frame with fresh state.
    fn decode(frame: &str, universe: &Universe, at: Timestamp) -> Result<Vec<MarketEvent>> {
        Decoder::default().decode(frame, universe, at)
    }

    /// Decodes a self-contained provider stream frame with fresh state.
    fn decode_detailed(
        frame: &str,
        universe: &Universe,
        at: Timestamp,
    ) -> Result<Vec<HyperliquidMarketEvent>> {
        DetailedDecoder::default().decode(frame, universe, at)
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
        // Subscriptions are emitted for the market/feed Cartesian product.
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
        // Spot subscriptions use the native indexed symbol.
        assert_eq!(parsed[2]["subscription"]["coin"], "@107");
    }

    #[test]
    fn subscribing_to_nothing_is_refused_before_the_socket_opens() {
        let universe = universe();
        let no_feed = Subscription::new().market(btc_perp());
        let no_market = Subscription::new().feed(Feed::Trades);

        assert!(matches!(
            subscribe_frames(&no_feed, &universe),
            Err(Error::InvalidRequest { field, .. }) if field == "feeds"
        ));
        assert!(matches!(
            subscribe_frames(&no_market, &universe),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
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
        // `A` is the taker's sell side.
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
        // A first frame is forming regardless of the local clock.
        assert!(!candle.closed);
    }

    /// Builds a one-minute candle frame.
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

    /// Consecutive one-minute candle windows.
    const WINDOW_ONE_MS: i64 = 1_785_397_500_000;
    const WINDOW_TWO_MS: i64 = 1_785_397_560_000;

    #[test]
    fn a_window_settles_when_a_later_one_opens_because_no_frame_of_it_ever_says_so() {
        let universe = universe();
        let mut decoder = Decoder::default();
        // A later clock confirms stream settlement does not use local time.
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);

        // Repeated updates remain forming within the same window.
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

        // A later opening settles the held window.
        let events = decoder
            .decode(&candle_frame(WINDOW_TWO_MS, 200), &universe, long_after)
            .expect("a candle frame");
        let [MarketEvent::Candle(settled), MarketEvent::Candle(forming)] = events.as_slice() else {
            panic!("a window ending is the settled one then the new one: {events:?}");
        };

        assert!(settled.closed, "the window before this frame is over");
        assert_eq!(settled.open_time, Timestamp::from_millis(WINDOW_ONE_MS));
        // Settlement uses the held window's latest values.
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

        // Each advanced window emits exactly once as closed.
        assert_eq!(settled, 2);
        assert_eq!(forming, 5);
    }

    #[test]
    fn a_frame_for_an_older_window_does_not_displace_the_one_being_held() {
        // A stale frame must not replace the newer held window.
        let universe = universe();
        let mut decoder = Decoder::default();
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);

        decoder
            .decode(&candle_frame(WINDOW_ONE_MS, 100), &universe, long_after)
            .expect("ok");
        decoder
            .decode(&candle_frame(WINDOW_TWO_MS, 200), &universe, long_after)
            .expect("ok");

        // An already settled window does not reappear as forming.
        let late = decoder
            .decode(&candle_frame(WINDOW_ONE_MS, 999), &universe, long_after)
            .expect("ok");
        assert!(late.is_empty(), "{late:?}");

        // The next settlement still uses the newer held window.
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
        // Held candle state is keyed by market and interval.
        let universe = universe();
        let mut decoder = Decoder::default();
        let long_after = Timestamp::from_millis(WINDOW_TWO_MS + 3_600_000);
        let hype = |open_ms: i64| candle_frame(open_ms, 7).replace("\"BTC\"", "\"@107\"");

        decoder
            .decode(&candle_frame(WINDOW_ONE_MS, 100), &universe, long_after)
            .expect("ok");

        // Another market cannot settle BTC's held candle.
        let events = decoder
            .decode(&hype(WINDOW_TWO_MS), &universe, long_after)
            .expect("ok");
        assert_eq!(events.len(), 1, "{events:?}");

        // BTC advances only when its own next window arrives.
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
    fn detailed_market_events_preserve_native_fields_alongside_common_projections() {
        let universe = universe();

        let events = decode_detailed(TRADES, &universe, at()).expect("a trade frame");
        let [HyperliquidMarketEvent::Trade(trade)] = events.as_slice() else {
            panic!("expected one detailed trade event: {events:?}");
        };
        assert_eq!(trade.common.market, btc_perp());
        assert_eq!(
            trade.provider.hash.as_deref(),
            Some("0xa166e3fa63c25663024b03f2e0da011a00307e4017465df020210d3d432e7cb8")
        );
        assert_eq!(trade.provider.users.len(), 2);

        let events = decode_detailed(L2_BOOK, &universe, at()).expect("an L2 frame");
        let [HyperliquidMarketEvent::OrderBook(book)] = events.as_slice() else {
            panic!("expected one detailed order-book event: {events:?}");
        };
        assert_eq!(book.common.market, btc_perp());
        assert_eq!(book.provider.bids[0].order_count, Some(17));

        let events = decode_detailed(CANDLE, &universe, at()).expect("a candle frame");
        let [HyperliquidMarketEvent::Candle(candle)] = events.as_slice() else {
            panic!("expected one detailed candle event: {events:?}");
        };
        assert!(!candle.common.closed);
        assert_eq!(candle.provider.trade_count, Some(12));

        let events = decode_detailed(ACTIVE_ASSET_CTX, &universe, at()).expect("a context frame");
        let [HyperliquidMarketEvent::AssetContext(context)] = events.as_slice() else {
            panic!("expected one detailed asset-context event: {events:?}");
        };
        assert_eq!(context.common.market, btc_perp());
        assert_eq!(context.funding_rate, Some(Decimal::new(125, 7)));
        assert_eq!(
            context.day_notional_volume,
            Some(Decimal::new(116_904_629_406, 5))
        );
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
        // The heartbeat is an application command, not a protocol ping frame.
        let sent: Value = serde_json::from_str(ping).expect("the heartbeat is JSON");
        assert_eq!(sent["method"], "ping");

        // Public and private decoders consume the response as control traffic.
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

        // Heartbeats arrive within the configured idle timeout.
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
        // Remaining size is lower than original size while status is open.
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

    #[test]
    fn detailed_account_events_preserve_provider_status_and_entry_notional() {
        let events = decode_detailed_account(ORDER_UPDATES, &universe()).expect("an order frame");
        let [HyperliquidAccountEvent::OrderUpdate(order)] = events.as_slice() else {
            panic!("expected one detailed order update: {events:?}");
        };
        assert_eq!(order.common.id, "91490942");
        assert_eq!(order.status, "open");
        assert_eq!(
            order.client_order_id.as_deref(),
            Some("0x00000000000000000000000000000001")
        );
        assert_eq!(
            order.status_at,
            Some(Timestamp::from_millis(1_681_247_412_573))
        );

        let events = decode_detailed_account(SPOT_STATE, &universe()).expect("a spot-state frame");
        let [HyperliquidAccountEvent::SpotState(state)] = events.as_slice() else {
            panic!("expected one detailed spot-state event: {events:?}");
        };
        assert_eq!(state.user, "0x14791697260e4c9a71f18484c9f997b308e59325");
        assert_eq!(state.balances[0].common.asset, "PURR");
        assert_eq!(
            state.balances[0].provider.entry_notional,
            Some(Decimal::from(100))
        );
    }
}
