//! Regression fixtures for the approved Binance execution-checklist units.

use std::collections::HashMap;

use rust_decimal::Decimal;
use serde_json::Value;

use super::{BinanceAdapter, BinanceAggregateTradesRequest, BinanceMarket, private, rest, stream};
use crate::error::{Error, Result};
use crate::request::OrderRequest;
use crate::types::{
    Exchange, Feed, Interval, Market, MarketEvent, Side, Size, Subscription, Timestamp,
};

fn spot() -> BinanceAdapter {
    BinanceAdapter::spot().with_credentials("key", "secret")
}

fn usd_m() -> BinanceAdapter {
    BinanceAdapter::usd_m_futures().with_credentials("key", "secret")
}

fn spot_market() -> Market {
    Market::spot(Exchange::Binance, "BTC", "USDT")
}

fn usd_m_market() -> Market {
    Market::perpetual(Exchange::Binance, "BTC", "USDT")
}

fn signed_params(request: &crate::transport::HttpRequest) -> String {
    let target = request.target();
    let (params, _) = target
        .split_once("&signature=")
        .expect("a signed Binance request");
    let (params, _) = params
        .rsplit_once("&timestamp=")
        .expect("a timestamped Binance request");
    params.to_string()
}

fn decode(adapter: &BinanceAdapter, market: Market, frame: &str) -> Result<MarketEvent> {
    let symbol = adapter.symbol(&market)?.to_ascii_lowercase();
    stream::decode(&HashMap::from([(symbol, market)]), frame)?
        .ok_or_else(|| Error::decode("fixture produced no market event"))
}

#[test]
fn spot_aggregate_trades_uses_its_rest_contract_and_fixture() {
    let request = BinanceAggregateTradesRequest::new(spot_market())
        .with_from_id(26_129)
        .limit(50);
    assert_eq!(
        rest::aggregate_trades_request(&spot(), &request)
            .expect("a Spot aggregate-trade request")
            .target(),
        "/api/v3/aggTrades?symbol=BTCUSDT&fromId=26129&limit=50"
    );

    let raw: Vec<super::parse::RawAggregateTrade> = super::parse::json(
        r#"[{"a":26129,"p":"0.01633102","q":"4.70443515","f":27781,"l":27784,"T":1498793709153,"m":true,"M":true}]"#,
        "Spot aggregate trades",
    )
    .expect("the official Spot fixture");
    let trade = super::parse::aggregate_trade(&spot_market(), &raw[0]).expect("an aggregate");
    assert_eq!(
        (
            trade.aggregate_id,
            trade.first_trade_id,
            trade.last_trade_id
        ),
        (26_129, 27_781, 27_784)
    );
    assert_eq!(trade.normal_quantity, None);
    assert_eq!(trade.taker_side, Side::Sell);
}

#[test]
fn usd_m_aggregate_trades_uses_its_rest_contract_and_fixture() {
    let request = BinanceAggregateTradesRequest::new(usd_m_market())
        .start_time(Timestamp::from_millis(1_623_319_461_670))
        .end_time(Timestamp::from_millis(1_623_319_462_670));
    assert_eq!(
        rest::aggregate_trades_request(&usd_m(), &request)
            .expect("a USD-M aggregate-trade request")
            .target(),
        "/fapi/v1/aggTrades?symbol=BTCUSDT&startTime=1623319461670&endTime=1623319462670&limit=500"
    );

    let raw: Vec<super::parse::RawAggregateTrade> = super::parse::json(
        r#"[{"a":26130,"p":"0.01633103","q":"1.2","nq":"1.00000000","f":27785,"l":27785,"T":1498793709253,"m":false}]"#,
        "USD-M aggregate trades",
    )
    .expect("the official USD-M fixture");
    let trade = super::parse::aggregate_trade(&usd_m_market(), &raw[0]).expect("an aggregate");
    assert_eq!(
        trade.normal_quantity.expect("normal quantity").to_string(),
        "1.00000000"
    );
    assert_eq!(trade.taker_side, Side::Buy);
}

#[test]
fn spot_new_order_and_cancel_order_use_the_spot_contract_and_fixtures() {
    let market = spot_market();
    let order = OrderRequest::limit(
        market.clone(),
        Side::Buy,
        Size::Base(Decimal::new(1, 2)),
        Decimal::from(100_000),
    );
    assert_eq!(
        signed_params(&private::place_order_request(&spot(), &order).expect("a Spot order")),
        "/api/v3/order?symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=0.01&price=100000&timeInForce=GTC&newOrderRespType=RESULT"
    );
    assert_eq!(
        signed_params(
            &private::cancel_order_request(&spot(), &market, "28").expect("a Spot cancellation")
        ),
        "/api/v3/order?symbol=BTCUSDT&orderId=28"
    );

    for (fixture, expected_status) in [
        (
            r#"{"symbol":"BTCUSDT","orderId":28,"side":"BUY","status":"NEW","price":"100000","origQty":"0.01","executedQty":"0","transactTime":1499827319559}"#,
            crate::types::OrderStatus::Open,
        ),
        (
            r#"{"symbol":"BTCUSDT","orderId":28,"side":"BUY","status":"CANCELED","price":"100000","origQty":"0.01","executedQty":"0","transactTime":1499827319559}"#,
            crate::types::OrderStatus::Cancelled,
        ),
    ] {
        let raw: super::parse::RawOrder =
            super::parse::json(fixture, "Spot order").expect("the Spot order fixture");
        assert_eq!(
            super::parse::order(&market, &raw)
                .expect("a Spot order response")
                .status,
            expected_status
        );
    }
}

#[test]
fn usd_m_new_order_and_cancel_order_use_the_futures_contract_and_fixtures() {
    let market = usd_m_market();
    let order = OrderRequest::limit(
        market.clone(),
        Side::Sell,
        Size::Base(Decimal::new(1, 2)),
        Decimal::from(100_000),
    );
    assert_eq!(
        signed_params(&private::place_order_request(&usd_m(), &order).expect("a USD-M order")),
        "/fapi/v1/order?symbol=BTCUSDT&side=SELL&type=LIMIT&quantity=0.01&price=100000&timeInForce=GTC&newOrderRespType=RESULT"
    );
    assert_eq!(
        signed_params(
            &private::cancel_order_request(&usd_m(), &market, "28").expect("a USD-M cancellation")
        ),
        "/fapi/v1/order?symbol=BTCUSDT&orderId=28"
    );

    for (fixture, expected_status) in [
        (
            r#"{"symbol":"BTCUSDT","orderId":28,"side":"SELL","status":"NEW","price":"100000","origQty":"0.01","executedQty":"0","updateTime":1499827319559}"#,
            crate::types::OrderStatus::Open,
        ),
        (
            r#"{"symbol":"BTCUSDT","orderId":28,"side":"SELL","status":"CANCELED","price":"100000","origQty":"0.01","executedQty":"0","updateTime":1499827319559}"#,
            crate::types::OrderStatus::Cancelled,
        ),
    ] {
        let raw: super::parse::RawOrder =
            super::parse::json(fixture, "USD-M order").expect("the USD-M order fixture");
        assert_eq!(
            super::parse::order(&market, &raw)
                .expect("a USD-M order response")
                .status,
            expected_status
        );
    }
}

#[test]
fn spot_stream_units_use_the_documented_names_and_fixtures() {
    let subscription = Subscription::new()
        .market(spot_market())
        .feed(Feed::Trades)
        .feed(Feed::OrderBook)
        .feed(Feed::Ticker)
        .feed(Feed::Candles(Interval::Min1));
    let frames = stream::subscribe_frames(&spot(), &subscription).expect("one Spot subscription");
    let [(url, frame)] = frames.as_slice() else {
        panic!("Spot feeds must share one socket")
    };
    assert_eq!(*url, super::SPOT_WEBSOCKET_URL);
    let frame: Value = serde_json::from_str(frame).expect("a subscription frame");
    assert_eq!(
        frame["params"],
        serde_json::json!([
            "btcusdt@trade",
            "btcusdt@depth20@100ms",
            "btcusdt@ticker",
            "btcusdt@kline_1m"
        ])
    );

    let fixtures = [
        (
            r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1672515782136,"s":"BTCUSDT","t":12345,"p":"0.001","q":"100","T":1672515782136,"m":true,"M":true}}"#,
            "trade",
        ),
        (
            r#"{"stream":"btcusdt@depth20@100ms","data":{"lastUpdateId":160,"bids":[["0.0024","10"]],"asks":[["0.0026","100"]]}}"#,
            "depth",
        ),
        (
            r#"{"stream":"btcusdt@ticker","data":{"e":"24hrTicker","E":1672515782136,"s":"BTCUSDT","p":"0.0015","P":"250.00","c":"0.0025","h":"0.0025","l":"0.0010","v":"10000","q":"18","C":86400000}}"#,
            "ticker",
        ),
        (
            r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1672515782136,"s":"BTCUSDT","k":{"t":1672515780000,"i":"1m","o":"0.0010","c":"0.0020","h":"0.0025","l":"0.0015","v":"1000","q":"1.0000","x":false}}}"#,
            "kline",
        ),
    ];
    for (fixture, kind) in fixtures {
        let event = decode(&spot(), spot_market(), fixture).expect("a Spot stream fixture");
        assert!(
            matches!(
                (kind, event),
                ("trade", MarketEvent::Trade(_))
                    | ("depth", MarketEvent::OrderBook(_))
                    | ("ticker", MarketEvent::Ticker(_))
                    | ("kline", MarketEvent::Candle(_))
            ),
            "wrong event for {kind}"
        );
    }
}

#[test]
fn spot_user_data_stream_signature_uses_the_signed_websocket_method() {
    let frame = private::spot_user_data_subscribe_frame(&spot()).expect("a signed frame");
    let frame: Value = serde_json::from_str(&frame).expect("a JSON frame");
    assert_eq!(frame["method"], "userDataStream.subscribe.signature");
    assert_eq!(frame["params"]["apiKey"], "key");
    assert_eq!(frame["params"]["recvWindow"], 60_000);
    assert!(frame["params"]["timestamp"].as_i64().is_some());
    let signature = frame["params"]["signature"]
        .as_str()
        .expect("an HMAC signature");
    assert_eq!(signature.len(), 64);
    assert!(signature.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn usd_m_stream_units_use_the_split_entry_points_and_fixtures() {
    let subscription = Subscription::new()
        .market(usd_m_market())
        .feed(Feed::OrderBook)
        .feed(Feed::Ticker)
        .feed(Feed::Candles(Interval::Min1));
    let frames = stream::subscribe_frames(&usd_m(), &subscription).expect("USD-M subscriptions");
    assert_eq!(frames.len(), 2);
    let public: Value = serde_json::from_str(&frames[0].1).expect("a public frame");
    let market: Value = serde_json::from_str(&frames[1].1).expect("a market frame");
    assert_eq!(frames[0].0, super::USD_M_PUBLIC_WEBSOCKET_URL);
    assert_eq!(
        public["params"],
        serde_json::json!(["btcusdt@depth20@100ms"])
    );
    assert_eq!(frames[1].0, super::USD_M_MARKET_WEBSOCKET_URL);
    assert_eq!(
        market["params"],
        serde_json::json!(["btcusdt@ticker", "btcusdt@kline_1m"])
    );

    let fixtures = [
        (
            r#"{"stream":"btcusdt@depth20@100ms","data":{"e":"depthUpdate","E":1571889248277,"T":1571889248276,"s":"BTCUSDT","U":390497796,"u":390497878,"pu":390497794,"b":[["7403.89","0.002"]],"a":[["7405.96","3.4"]]}}"#,
            "depth",
        ),
        (
            r#"{"stream":"btcusdt@ticker","data":{"e":"24hrTicker","E":123456789,"s":"BTCUSDT","p":"0.0015","P":"250.00","c":"0.0025","h":"0.0025","l":"0.0010","v":"10000","q":"18","C":86400000}}"#,
            "ticker",
        ),
        (
            r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1499404907056,"s":"BTCUSDT","k":{"t":1499404860000,"i":"1m","o":"0.0010","c":"0.0020","h":"0.0025","l":"0.0015","v":"1000","q":"1.0000","x":true}}}"#,
            "kline",
        ),
    ];
    for (fixture, kind) in fixtures {
        let event = decode(&usd_m(), usd_m_market(), fixture).expect("a USD-M stream fixture");
        assert!(
            matches!(
                (kind, event),
                ("depth", MarketEvent::OrderBook(_))
                    | ("ticker", MarketEvent::Ticker(_))
                    | ("kline", MarketEvent::Candle(_))
            ),
            "wrong event for {kind}"
        );
    }
}

#[test]
fn usd_m_aggregate_trade_stream_stays_blocked_at_the_common_trade_boundary() {
    let subscription = Subscription::new()
        .market(usd_m_market())
        .feed(Feed::Trades);
    assert!(matches!(
        stream::subscribe_frames(&usd_m(), &subscription),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        BinanceMarket::UsdMFutures.market_kind(),
        crate::types::MarketKind::Perpetual
    );
}
