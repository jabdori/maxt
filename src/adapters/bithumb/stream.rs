//! Bithumb's WebSockets, public and private.
//!
//! One socket carries every market and every feed: the subscribe payload is a
//! list whose first element is a client-chosen ticket, followed by one element
//! per feed naming the market codes it applies to.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::stream::{AccountStream, MarketStream};
use crate::transport::{Heartbeat, HeartbeatFrame, WsCommand, WsConnect, connect};
use crate::types::{AccountEvent, Feed, MarketEvent, StreamConfig, Subscription};

use super::parse::{self, EXCHANGE};
use super::{BithumbCredentials, PRIVATE_WEBSOCKET_URL, WEBSOCKET_URL, private};

/// The frame layout Bithumb calls `DEFAULT`: full field names, one JSON object
/// per event. The alternative, `SIMPLE`, abbreviates every key.
const FRAME_FORMAT: &str = "DEFAULT";

/// What holds a Bithumb socket open when the market is asleep.
///
/// Bithumb reads the bare text `PING` as a keepalive and answers it with
/// `{"status":"UP"}`, the status frame [`decode`] drops. One heartbeat both
/// tells Bithumb the client is alive and produces the inbound traffic the idle
/// timer is watching for. Bithumb closes a connection it has exchanged nothing
/// with for about 120 seconds, so eight heartbeats fit inside one such window.
///
/// Both sockets use it. The private one carries nothing until the account
/// moves, which on a quiet account is never.
pub(crate) const HEARTBEAT: Heartbeat = Heartbeat {
    interval: Duration::from_secs(15),
    frame: HeartbeatFrame::Text("PING"),
    // Four unanswered heartbeats before a socket is written off.
    min_idle_timeout: Duration::from_secs(60),
};

/// Builds the frame sent on connect and again after every reconnect.
///
/// `ticket` labels the connection on Bithumb's side; it appears in their
/// support tooling and is not otherwise interpreted.
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
        .map_err(|err| Error::decode(format!("could not build Bithumb subscribe frame: {err}")))
}

/// The `type` Bithumb expects for a feed.
///
/// Bithumb's public socket carries trades, order books, and tickers and
/// nothing else. Candles exist over REST only, so a candle feed is refused
/// here. Dropping it from the subscription or rebuilding it from trades would
/// hand back a stream that quietly is not what was asked for.
fn feed_type(feed: Feed) -> Result<&'static str> {
    match feed {
        Feed::Trades => Ok("trade"),
        Feed::OrderBook => Ok("orderbook"),
        Feed::Ticker => Ok("ticker"),
        Feed::Candles(interval) => Err(Error::unsupported(
            Feature::CandleStream,
            EXCHANGE,
            format!(
                "bithumb publishes no candle stream, so {interval:?} candles cannot be \
                 subscribed to; read them from `Client::candles` or build them from `Feed::Trades`"
            ),
        )),
    }
}

/// Builds the private subscribe frame.
///
/// `myAsset` is account-wide and `myOrder` covers every market when no codes
/// are named, which is what the common API asks for: it subscribes an account,
/// not a market list.
fn account_subscribe_frame(ticket: &str) -> Result<String> {
    let payload = serde_json::json!([
        { "ticket": ticket },
        { "type": "myOrder" },
        { "type": "myAsset" },
        { "format": FRAME_FORMAT },
    ]);

    serde_json::to_string(&payload).map_err(|err| {
        Error::decode(format!(
            "could not build Bithumb account subscribe frame: {err}"
        ))
    })
}

/// Opens a public market data subscription.
pub(crate) async fn subscribe(
    subscription: &Subscription,
    config: &StreamConfig,
) -> Result<MarketStream> {
    let session = connect(
        WsConnect {
            url: WEBSOCKET_URL.to_string(),
            headers: None,
            subscribe: WsConnect::fixed(vec![subscribe_frame(subscription, &ticket())?]),
            heartbeat: Some(HEARTBEAT),
        },
        config,
    )
    .await?;

    Ok(MarketStream::new(
        session.filter_map(|item| std::future::ready(market_item(item))),
    ))
}

/// How to open one private connection, and how to authenticate every one of
/// them.
///
/// Bithumb authenticates the private socket in the opening handshake rather
/// than in a frame, and its token carries the millisecond clock it was signed
/// at. One token therefore opens one handshake: a socket that reconnects an
/// hour later and presented the token it opened with would be refused, and the
/// reconnect loop would retry that same dead token forever. The header is
/// signed here, per handshake, so what a reconnect presents is as fresh as what
/// the first connection did.
fn account_connect(credentials: &BithumbCredentials, ticket: &str) -> Result<WsConnect> {
    let credentials = credentials.clone();

    Ok(WsConnect {
        url: PRIVATE_WEBSOCKET_URL.to_string(),
        headers: Some(Box::new(move || {
            Ok(vec![(
                "authorization".to_string(),
                private::websocket_authorization(&credentials)?,
            )])
        })),
        // Fixed: the frame names a ticket and two channels and signs nothing,
        // so it is as good on the tenth reconnect as on the first. The
        // credential this socket presents is in the header above.
        subscribe: WsConnect::fixed(vec![account_subscribe_frame(ticket)?]),
        heartbeat: Some(HEARTBEAT),
    })
}

/// Opens a private account subscription.
pub(crate) async fn subscribe_account(
    credentials: &BithumbCredentials,
    config: &StreamConfig,
) -> Result<AccountStream> {
    let session = connect(account_connect(credentials, &ticket())?, config).await?;

    Ok(AccountStream::new(session.flat_map(|item| {
        futures_util::stream::iter(account_items(item))
    })))
}

fn ticket() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn market_item(item: Result<WsCommand>) -> Option<Result<MarketEvent>> {
    match item {
        Err(err) => Some(Err(err)),
        Ok(WsCommand::Reconnected) => Some(Ok(MarketEvent::Reconnected)),
        Ok(WsCommand::Text(text)) => decode(&text).transpose(),
        Ok(WsCommand::Binary(bytes)) => match utf8(&bytes) {
            Ok(text) => decode(&text).transpose(),
            Err(err) => Some(Err(err)),
        },
    }
}

fn account_items(item: Result<WsCommand>) -> Vec<Result<AccountEvent>> {
    match item {
        Err(err) => vec![Err(err)],
        Ok(WsCommand::Reconnected) => vec![Ok(AccountEvent::Reconnected)],
        Ok(WsCommand::Text(text)) => decode_account(&text),
        Ok(WsCommand::Binary(bytes)) => match utf8(&bytes) {
            Ok(text) => decode_account(&text),
            Err(err) => vec![Err(err)],
        },
    }
}

/// Reads one public frame.
///
/// `None` means the frame carried no market data. Bithumb answers a keepalive
/// with a status frame, which a caller has no use for.
pub(crate) fn decode(frame: &str) -> Result<Option<MarketEvent>> {
    let object = frame_object(frame)?;
    if object.get("status").and_then(Value::as_str).is_some() {
        return Ok(None);
    }

    parse::market_event(&Value::Object(object))
}

/// Reads one private frame, which may describe several balances at once.
pub(crate) fn decode_account(frame: &str) -> Vec<Result<AccountEvent>> {
    let events = frame_object(frame).and_then(|object| {
        if object.get("status").and_then(Value::as_str).is_some() {
            return Ok(Vec::new());
        }
        parse::account_events(&Value::Object(object))
    });

    match events {
        Ok(events) => events.into_iter().map(Ok).collect(),
        Err(err) => vec![Err(err)],
    }
}

/// Unwraps a frame to its object.
///
/// Bithumb sends bare objects but wraps them in a single-element list on some
/// endpoints, so both are accepted.
fn frame_object(frame: &str) -> Result<Map<String, Value>> {
    let value: Value = serde_json::from_str(frame)
        .map_err(|err| Error::decode(format!("unreadable Bithumb frame: {err}")))?;

    match value {
        Value::Object(object) => Ok(object),
        Value::Array(mut items) if items.len() == 1 => match items.remove(0) {
            Value::Object(object) => Ok(object),
            _ => Err(Error::decode("Bithumb frame list holds a non-object")),
        },
        _ => Err(Error::decode(
            "Bithumb frame is neither an object nor a one-object list",
        )),
    }
}

/// Bithumb answers some subscriptions with the same JSON in a binary frame.
fn utf8(bytes: &[u8]) -> Result<String> {
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|err| Error::decode(format!("Bithumb binary frame is not UTF-8: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Interval, Market, Side, Timestamp};

    // https://apidocs.bithumb.com/reference/체결-trade.md
    const TRADE: &str = r#"{
      "type": "trade",
      "code": "KRW-BTC",
      "trade_price": 100000000.0,
      "trade_volume": 0.01,
      "ask_bid": "ASK",
      "prev_closing_price": 99000000.0,
      "change": "RISE",
      "change_price": 1000000.0,
      "trade_date": "2026-06-20",
      "trade_time": "10:02:03",
      "trade_timestamp": 1781917323000,
      "timestamp": 1781917323001,
      "sequential_id": 17819173230000000
    }"#;

    // https://apidocs.bithumb.com/reference/내-자산-myasset.md
    const MY_ASSET: &str = r#"{
      "type": "myAsset",
      "assets": [
        { "currency": "KRW", "balance": "2061832.35", "locked": "3824127.3" },
        { "currency": "BTC", "balance": "0.5", "locked": "0" }
      ],
      "asset_timestamp": 1727052537592,
      "timestamp": 1727052537687
    }"#;

    fn credentials() -> BithumbCredentials {
        BithumbCredentials {
            access_key: "test-access".to_string(),
            secret_key: "test-secret".to_string(),
        }
    }

    fn subscription() -> Subscription {
        Subscription::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .market(Market::spot(Exchange::Bithumb, "ETH", "KRW"))
            .feed(Feed::Trades)
            .feed(Feed::OrderBook)
    }

    #[test]
    fn one_frame_carries_every_market_and_every_feed() {
        let frame = subscribe_frame(&subscription(), "ticket-1").expect("a valid subscription");
        let value: Value = serde_json::from_str(&frame).expect("valid JSON");

        assert_eq!(value[0]["ticket"], "ticket-1");
        assert_eq!(value[1]["type"], "trade");
        assert_eq!(value[1]["codes"][0], "KRW-BTC");
        assert_eq!(value[1]["codes"][1], "KRW-ETH");
        assert_eq!(value[2]["type"], "orderbook");
        assert_eq!(value[3]["format"], "DEFAULT");
        assert_eq!(value.as_array().expect("a list").len(), 4);
    }

    #[test]
    fn a_candle_feed_is_refused_because_bithumb_publishes_none() {
        // Every interval, including the ones the REST endpoints do serve: the
        // gap is the stream, not the interval.
        for interval in [
            Interval::Sec1,
            Interval::Min1,
            Interval::Min5,
            Interval::Hour1,
            Interval::Hour4,
            Interval::Day1,
            Interval::Week1,
        ] {
            assert!(
                matches!(
                    feed_type(Feed::Candles(interval)),
                    Err(Error::Unsupported {
                        feature: Feature::CandleStream,
                        exchange: "bithumb",
                        ..
                    })
                ),
                "{interval:?}"
            );
        }
    }

    #[test]
    fn a_candle_feed_takes_the_whole_subscription_down_rather_than_being_dropped() {
        let mixed = subscription().feed(Feed::Candles(Interval::Min1));

        let error = subscribe_frame(&mixed, "ticket-1")
            .expect_err("a candle feed cannot be subscribed to on Bithumb");

        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::CandleStream,
                ..
            }
        ));
    }

    #[test]
    fn the_feeds_bithumb_does_carry_are_named_the_way_it_names_them() {
        assert_eq!(feed_type(Feed::Trades).expect("carried"), "trade");
        assert_eq!(feed_type(Feed::OrderBook).expect("carried"), "orderbook");
        assert_eq!(feed_type(Feed::Ticker).expect("carried"), "ticker");
    }

    #[test]
    fn subscribing_to_nothing_is_refused_before_the_socket_opens() {
        let no_feed = Subscription::new().market(Market::spot(Exchange::Bithumb, "BTC", "KRW"));
        let no_market = Subscription::new().feed(Feed::Trades);

        assert!(matches!(
            subscribe_frame(&no_feed, "t"),
            Err(Error::InvalidRequest { field: "feeds", .. })
        ));
        assert!(matches!(
            subscribe_frame(&no_market, "t"),
            Err(Error::InvalidRequest {
                field: "markets",
                ..
            })
        ));
    }

    #[test]
    fn a_private_subscription_asks_for_both_order_and_balance_updates() {
        let frame = account_subscribe_frame("ticket-2").expect("a valid frame");
        let value: Value = serde_json::from_str(&frame).expect("valid JSON");

        assert_eq!(value[0]["ticket"], "ticket-2");
        assert_eq!(value[1]["type"], "myOrder");
        // No `codes`: the subscription follows the account, not a market list.
        assert!(value[1].get("codes").is_none());
        assert_eq!(value[2]["type"], "myAsset");
    }

    #[test]
    fn the_private_socket_signs_a_token_for_every_handshake_rather_than_replaying_one() {
        let connection = account_connect(&credentials(), "ticket-3").expect("a private connection");
        let headers = connection.headers.as_ref().expect("a signed handshake");

        let first = headers().expect("signed");
        let second = headers().expect("signed");

        assert_eq!(first.len(), 1);
        assert_eq!(first[0].0, "authorization");
        assert!(first[0].1.starts_with("Bearer "));
        // The token names the millisecond it was signed at, so the one that
        // opened the first handshake is not one a reconnect may present: an
        // hour-old token is refused, and a connection that replayed it would
        // retry the same dead credential forever.
        assert_ne!(first, second);
    }

    #[test]
    fn a_trade_frame_becomes_a_trade_event() {
        let Some(MarketEvent::Trade(trade)) = decode(TRADE).expect("a data frame") else {
            panic!("expected a trade event");
        };

        assert_eq!(trade.market, Market::spot(Exchange::Bithumb, "BTC", "KRW"));
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_781_917_323_000));
    }

    #[test]
    fn a_frame_wrapped_in_a_single_element_list_reads_the_same() {
        let wrapped = format!("[{TRADE}]");

        assert!(matches!(
            decode(&wrapped).expect("a data frame"),
            Some(MarketEvent::Trade(_))
        ));
    }

    #[test]
    fn a_keepalive_answer_is_not_reported_as_market_data() {
        assert!(
            decode(r#"{"status":"UP"}"#)
                .expect("a control frame")
                .is_none()
        );
        assert!(decode_account(r#"{"status":"UP"}"#).is_empty());
    }

    #[test]
    fn the_heartbeat_is_the_frame_bithumb_answers_rather_than_one_it_errors_on() {
        // Bithumb reads every text frame on these sockets as a command. `PING`
        // is the one it answers with a status frame; anything else comes back
        // as an error that would take the subscription down with it.
        assert_eq!(HEARTBEAT.frame, HeartbeatFrame::Text("PING"));
        // That answer is the inbound traffic the idle timer is waiting for, and
        // it reaches both sockets, so neither may read it as data or as a fault.
        assert!(
            decode(r#"{"status":"UP"}"#)
                .expect("the answer to this heartbeat")
                .is_none()
        );
        assert!(decode_account(r#"{"status":"UP"}"#).is_empty());
        // Bithumb hangs up after about 120 seconds with nothing sent or
        // received, so several heartbeats have to fit inside one such window.
        assert!(HEARTBEAT.interval * 4 <= Duration::from_secs(120));
        assert!(HEARTBEAT.min_idle_timeout >= HEARTBEAT.interval * 3);
    }

    #[test]
    fn a_frame_maxt_cannot_read_is_an_error_not_a_silent_drop() {
        assert!(matches!(decode("not json"), Err(Error::Decode { .. })));
        assert!(matches!(decode("[1,2]"), Err(Error::Decode { .. })));
        assert!(matches!(decode("[[]]"), Err(Error::Decode { .. })));
    }

    #[test]
    fn one_balance_frame_becomes_one_event_per_asset() {
        let events = decode_account(MY_ASSET);

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(Result::is_ok));
    }

    #[test]
    fn a_reconnect_reaches_the_caller_on_both_streams() {
        assert!(matches!(
            market_item(Ok(WsCommand::Reconnected)),
            Some(Ok(MarketEvent::Reconnected))
        ));
        assert!(matches!(
            account_items(Ok(WsCommand::Reconnected)).as_slice(),
            [Ok(AccountEvent::Reconnected)]
        ));
    }

    #[test]
    fn a_binary_frame_carries_the_same_json_a_text_frame_would() {
        let binary = WsCommand::Binary(TRADE.as_bytes().to_vec());

        assert!(matches!(
            market_item(Ok(binary)),
            Some(Ok(MarketEvent::Trade(_)))
        ));
        assert!(matches!(
            market_item(Ok(WsCommand::Binary(vec![0xff, 0xfe]))),
            Some(Err(Error::Decode { .. }))
        ));
    }

    #[test]
    fn a_transport_failure_reaches_the_caller_rather_than_ending_the_stream() {
        let failure = Err(Error::transport("gave up reconnecting"));

        assert!(matches!(
            market_item(failure.clone()),
            Some(Err(Error::Transport { .. }))
        ));
        assert!(matches!(
            account_items(failure).as_slice(),
            [Err(Error::Transport { .. })]
        ));
    }
}
