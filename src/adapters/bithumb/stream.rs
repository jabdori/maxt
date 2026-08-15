//! Bithumb public and private WebSocket transport.
//!
//! One public connection can subscribe to multiple markets and feed types.

use std::time::Duration;

use futures_util::StreamExt;
use rust_decimal::Decimal;
use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::stream::{AccountStream, MarketStream};
use crate::transport::{Heartbeat, HeartbeatFrame, WsCommand, WsConnect, connect};
use crate::types::{AccountEvent, Feed, MarketEvent, StreamConfig, Subscription, Timestamp};

use super::parse::{self, EXCHANGE};
use super::{
    BithumbAccountEvent, BithumbAccountStream, BithumbAssetEvent, BithumbCredentials,
    BithumbMarketEvent, BithumbMarketStream, BithumbOrderBookEvent, BithumbOrderEvent,
    BithumbTickerEvent, BithumbTradeEvent, PRIVATE_WEBSOCKET_URL, WEBSOCKET_URL, private,
};

/// Bithumb frame format with full field names.
const FRAME_FORMAT: &str = "DEFAULT";

/// Heartbeat shared by public and private sockets.
///
/// Bithumb answers text `PING` with `{"status":"UP"}` and may close an idle
/// connection after about 120 seconds.
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
        .map_err(|err| Error::decode(format!("could not build Bithumb subscribe frame: {err}")))
}

/// Maps supported public feeds to Bithumb type names.
///
/// Candle feeds are rejected because Bithumb publishes them only over REST.
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

/// Builds an account-wide private subscription for orders and balances.
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
    let close = session.close_handle();

    Ok(MarketStream::new_with_close(
        session.filter_map(|item| std::future::ready(market_item(item))),
        move || async move { close.close().await },
    ))
}

/// Opens a public market-data subscription that retains Bithumb-specific fields.
pub(crate) async fn subscribe_detailed(
    subscription: &Subscription,
    config: &StreamConfig,
) -> Result<BithumbMarketStream> {
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
    let close = session.close_handle();

    Ok(BithumbMarketStream::new_with_close(
        session.filter_map(|item| std::future::ready(detailed_market_item(item))),
        move || async move { close.close().await },
    ))
}

/// Builds a private connection that signs a fresh JWT for each handshake.
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
        // The subscription frame is reusable; the handshake header is regenerated.
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
    let close = session.close_handle();

    Ok(AccountStream::new_with_close(
        session.flat_map(|item| futures_util::stream::iter(account_items(item))),
        move || async move { close.close().await },
    ))
}

/// Opens a private account subscription that retains Bithumb-specific fields.
pub(crate) async fn subscribe_detailed_account(
    credentials: &BithumbCredentials,
    config: &StreamConfig,
) -> Result<BithumbAccountStream> {
    let session = connect(account_connect(credentials, &ticket())?, config).await?;
    let close = session.close_handle();

    Ok(BithumbAccountStream::new_with_close(
        session.flat_map(|item| futures_util::stream::iter(detailed_account_items(item))),
        move || async move { close.close().await },
    ))
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

fn detailed_market_item(item: Result<WsCommand>) -> Option<Result<BithumbMarketEvent>> {
    match item {
        Err(err) => Some(Err(err)),
        Ok(WsCommand::Reconnected) => Some(Ok(BithumbMarketEvent::Reconnected)),
        Ok(WsCommand::Text(text)) => decode_detailed(&text).transpose(),
        Ok(WsCommand::Binary(bytes)) => match utf8(&bytes) {
            Ok(text) => decode_detailed(&text).transpose(),
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

fn detailed_account_items(item: Result<WsCommand>) -> Vec<Result<BithumbAccountEvent>> {
    match item {
        Err(err) => vec![Err(err)],
        Ok(WsCommand::Reconnected) => vec![Ok(BithumbAccountEvent::Reconnected)],
        Ok(WsCommand::Text(text)) => decode_detailed_account(&text)
            .map(|events| events.into_iter().map(Ok).collect())
            .unwrap_or_else(|err| vec![Err(err)]),
        Ok(WsCommand::Binary(bytes)) => match utf8(&bytes) {
            Ok(text) => decode_detailed_account(&text)
                .map(|events| events.into_iter().map(Ok).collect())
                .unwrap_or_else(|err| vec![Err(err)]),
            Err(err) => vec![Err(err)],
        },
    }
}

/// Decodes one public frame; status frames return `Ok(None)`.
pub(crate) fn decode(frame: &str) -> Result<Option<MarketEvent>> {
    let object = frame_object(frame)?;
    if object.get("status").and_then(Value::as_str).is_some() {
        return Ok(None);
    }

    parse::market_event(&Value::Object(object))
}

/// Decodes one public frame without discarding Bithumb's provider fields.
pub(crate) fn decode_detailed(frame: &str) -> Result<Option<BithumbMarketEvent>> {
    let object = frame_object(frame)?;
    if object.get("status").and_then(Value::as_str).is_some() {
        return Ok(None);
    }
    let value = Value::Object(object);
    let Some(common) = parse::market_event(&value)? else {
        return Ok(None);
    };
    let raw_json = json(&value)?;

    Ok(Some(match common {
        MarketEvent::Trade(common) => BithumbMarketEvent::Trade(BithumbTradeEvent {
            previous_closing_price: optional_decimal(&value, "prev_closing_price")?,
            change: optional_text(&value, "change")?,
            change_price: optional_decimal(&value, "change_price")?,
            published_at: optional_millis(&value, "timestamp")?,
            stream_type: optional_text(&value, "stream_type")?,
            common,
            raw_json,
        }),
        MarketEvent::OrderBook(common) => BithumbMarketEvent::OrderBook(BithumbOrderBookEvent {
            total_ask_size: optional_decimal(&value, "total_ask_size")?,
            total_bid_size: optional_decimal(&value, "total_bid_size")?,
            level: optional_decimal(&value, "level")?,
            stream_type: optional_text(&value, "stream_type")?,
            common,
            raw_json,
        }),
        MarketEvent::Ticker(common) => BithumbMarketEvent::Ticker(BithumbTickerEvent {
            change_direction: optional_text(&value, "change")?,
            market_state: optional_text(&value, "market_state")?,
            trading_suspended: optional_bool(&value, "is_trading_suspended")?,
            market_warning: optional_text(&value, "market_warning")?,
            stream_type: optional_text(&value, "stream_type")?,
            common,
            raw_json,
        }),
        MarketEvent::Candle(_) | MarketEvent::Reconnected => {
            return Err(Error::decode(
                "Bithumb sent an unsupported market event for a detailed stream",
            ));
        }
    }))
}

/// Decodes one private frame into zero or more account events.
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

/// Decodes one private frame without discarding Bithumb's provider fields.
pub(crate) fn decode_detailed_account(frame: &str) -> Result<Vec<BithumbAccountEvent>> {
    let object = frame_object(frame)?;
    if object.get("status").and_then(Value::as_str).is_some() {
        return Ok(Vec::new());
    }
    let value = Value::Object(object);
    let raw_json = json(&value)?;

    match value.get("type").and_then(Value::as_str) {
        Some("myAsset") => {
            let balances = parse::account_events(&value)?
                .into_iter()
                .filter_map(|event| match event {
                    AccountEvent::Balance(balance) => Some(balance),
                    AccountEvent::Order(_) | AccountEvent::Reconnected => None,
                })
                .collect();
            Ok(vec![BithumbAccountEvent::Asset(BithumbAssetEvent {
                asset_timestamp: optional_millis(&value, "asset_timestamp")?,
                published_at: optional_millis(&value, "timestamp")?,
                balances,
                raw_json,
            })])
        }
        Some("myOrder") => {
            let mut events = parse::account_events(&value)?;
            let Some(AccountEvent::Order(common)) = events.pop() else {
                return Err(Error::decode("Bithumb myOrder frame carried no order"));
            };
            Ok(vec![BithumbAccountEvent::Order(BithumbOrderEvent {
                client_order_id: optional_text(&value, "client_order_id")?,
                order_type: optional_text(&value, "order_type")?,
                state: optional_text(&value, "state")?,
                time_in_force: optional_text(&value, "time_in_force")?,
                order_amount: optional_decimal(&value, "order_amount")?,
                trade_id: optional_text(&value, "trade_id")?,
                trade_price: optional_decimal(&value, "trade_price")?,
                trade_quantity: optional_decimal(&value, "trade_quantity")?,
                trade_amount: optional_decimal(&value, "trade_amount")?,
                trade_timestamp: optional_millis(&value, "trade_timestamp")?,
                executed_amount: optional_decimal(&value, "executed_amount")?,
                paid_fee: optional_decimal(&value, "paid_fee")?,
                remaining_fee: optional_decimal(&value, "remaining_fee")?,
                common,
                raw_json,
            })])
        }
        Some(_) | None => Ok(Vec::new()),
    }
}

fn json(value: &Value) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|err| Error::decode(format!("could not preserve Bithumb frame: {err}")))
}

fn optional_decimal(value: &Value, name: &'static str) -> Result<Option<Decimal>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => parse::dec(value, name).map(Some),
    }
}

fn optional_millis(value: &Value, name: &'static str) -> Result<Option<Timestamp>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => parse::millis(value, name).map(Some),
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

/// Accepts a bare object or a single-object list.
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

/// Decodes a UTF-8 binary frame.
fn utf8(bytes: &[u8]) -> Result<String> {
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|err| Error::decode(format!("Bithumb binary frame is not UTF-8: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Interval, Market, Side, Timestamp};

    // Shape reference: https://apidocs.bithumb.com/reference/체결-trade.md
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

    // Documentation example: https://apidocs.bithumb.com/reference/내-자산-myasset.md
    const MY_ASSET: &str = r#"{
      "type": "myAsset",
      "assets": [
        { "currency": "KRW", "balance": "2061832.35", "locked": "3824127.3" },
        { "currency": "BTC", "balance": "0.5", "locked": "0" }
      ],
      "asset_timestamp": 1727052537592,
      "timestamp": 1727052537687
    }"#;

    const ORDER_BOOK: &str = r#"{
      "type": "orderbook",
      "code": "KRW-BTC",
      "total_ask_size": "3.4",
      "total_bid_size": "5.6",
      "level": "0",
      "orderbook_units": [
        { "ask_price": "101", "ask_size": "2", "bid_price": "99", "bid_size": "3" }
      ],
      "timestamp": 1785397747576054,
      "stream_type": "SNAPSHOT"
    }"#;

    const TICKER: &str = r#"{
      "type": "ticker",
      "code": "KRW-BTC",
      "trade_price": "100",
      "timestamp": 1785397735276,
      "change": "RISE",
      "market_state": "ACTIVE",
      "is_trading_suspended": false,
      "market_warning": "NONE",
      "stream_type": "REALTIME"
    }"#;

    const MY_ORDER: &str = r#"{
      "type": "myOrder",
      "code": "KRW-BTC",
      "order_id": "order-1",
      "client_order_id": "client-1",
      "side": "buy",
      "order_type": "limit",
      "state": "trade",
      "time_in_force": "post_only",
      "order_price": "100",
      "order_quantity": "2",
      "order_amount": "200",
      "order_timestamp": 1727052318074,
      "timestamp": 1727052318369,
      "trade_id": "trade-1",
      "trade_price": "101",
      "trade_quantity": "1",
      "trade_amount": "101",
      "trade_timestamp": 1727052318148,
      "executed_quantity": "1",
      "remaining_quantity": "1",
      "executed_amount": "101",
      "paid_fee": "0.1",
      "remaining_fee": "0.1",
      "future_field": "kept"
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
        // Candle streaming is unsupported for every interval.
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
            Err(Error::InvalidRequest { field, .. }) if field == "feeds"
        ));
        assert!(matches!(
            subscribe_frame(&no_market, "t"),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
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
        // Each handshake receives a newly signed token.
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
        // `PING` is the documented text heartbeat.
        assert_eq!(HEARTBEAT.frame, HeartbeatFrame::Text("PING"));
        // The heartbeat response is a control frame on both sockets.
        assert!(
            decode(r#"{"status":"UP"}"#)
                .expect("the answer to this heartbeat")
                .is_none()
        );
        assert!(decode_account(r#"{"status":"UP"}"#).is_empty());
        // Several heartbeats fit within the documented idle window.
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
    fn detailed_market_events_keep_native_fields_and_the_original_frame() {
        let Some(BithumbMarketEvent::Trade(trade)) =
            decode_detailed(TRADE).expect("a detailed trade frame")
        else {
            panic!("expected a detailed trade");
        };
        assert_eq!(trade.common.id.as_deref(), Some("17819173230000000"));
        assert_eq!(trade.change.as_deref(), Some("RISE"));
        assert!(trade.raw_json.contains("prev_closing_price"));

        let Some(BithumbMarketEvent::OrderBook(book)) =
            decode_detailed(ORDER_BOOK).expect("a detailed order book")
        else {
            panic!("expected a detailed order book");
        };
        assert_eq!(book.total_ask_size.expect("ask size").to_string(), "3.4");
        assert_eq!(book.stream_type.as_deref(), Some("SNAPSHOT"));

        let Some(BithumbMarketEvent::Ticker(ticker)) =
            decode_detailed(TICKER).expect("a detailed ticker")
        else {
            panic!("expected a detailed ticker");
        };
        assert_eq!(ticker.market_state.as_deref(), Some("ACTIVE"));
        assert_eq!(ticker.trading_suspended, Some(false));
        assert!(ticker.raw_json.contains("market_warning"));
    }

    #[test]
    fn detailed_account_events_keep_provider_metadata_and_the_original_frame() {
        let assets = decode_detailed_account(MY_ASSET).expect("a detailed asset frame");
        let [BithumbAccountEvent::Asset(asset)] = assets.as_slice() else {
            panic!("expected one detailed asset event: {assets:?}");
        };
        assert_eq!(asset.balances.len(), 2);
        assert_eq!(
            asset.asset_timestamp,
            Some(Timestamp::from_millis(1_727_052_537_592))
        );

        let orders = decode_detailed_account(MY_ORDER).expect("a detailed order frame");
        let [BithumbAccountEvent::Order(order)] = orders.as_slice() else {
            panic!("expected one detailed order event: {orders:?}");
        };
        assert_eq!(order.common.id, "order-1");
        assert_eq!(order.client_order_id.as_deref(), Some("client-1"));
        assert_eq!(order.trade_id.as_deref(), Some("trade-1"));
        assert_eq!(
            order.executed_amount.expect("executed amount").to_string(),
            "101"
        );
        assert!(order.raw_json.contains("future_field"));
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
