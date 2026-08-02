use std::sync::Arc;

use maxt::{Adapter, Client, Error};
#[cfg(not(test))]
use napi_derive::napi;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::convert::{
    WireBalance, WireCandle, WireCandleRequest, WireFundingPayment, WireFundingRate,
    WireHistoryRequest, WireMarginRequest, WireMarginSummary, WireMarket, WireMarketInfo,
    WireOrder, WireOrderBook, WireOrderRequest, WirePage, WirePosition, WireStreamConfig,
    WireSubscription, WireTicker, WireTrade, feature_from_id, from_wire_value,
    market_kind_from_wire, outcome,
};
use crate::stream::NativeStreamRegistry;

#[cfg_attr(not(test), napi)]
pub struct NativeClient {
    inner: Arc<Client<Box<dyn Adapter>>>,
    streams: Arc<NativeStreamRegistry>,
}

impl NativeClient {
    pub fn from_boxed(adapter: Box<dyn Adapter>) -> Self {
        Self {
            inner: Arc::new(Client::new(adapter)),
            streams: Arc::new(NativeStreamRegistry::default()),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(adapter: Box<dyn Adapter>) -> Self {
        Self::from_boxed(adapter)
    }
}

#[cfg_attr(not(test), napi)]
impl NativeClient {
    #[cfg_attr(not(test), napi(js_name = "exchange"))]
    pub fn exchange(&self) -> String {
        self.inner.exchange().id().to_owned()
    }

    #[cfg_attr(not(test), napi(js_name = "supports"))]
    pub fn supports(&self, feature: String) -> bool {
        feature_from_id(&feature).is_some_and(|feature| self.inner.supports(feature))
    }

    #[cfg_attr(not(test), napi(js_name = "markets"))]
    pub async fn markets(&self, kind: Value) -> Value {
        let kind = from_wire_value::<String>(kind, "kind")
            .and_then(|kind| market_kind_from_wire(&kind, "kind"));
        match kind {
            Ok(kind) => outcome(wire_vec::<_, WireMarketInfo>(
                self.inner.markets(kind).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "trades"))]
    pub async fn trades(&self, market: Value, limit: Value) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let limit = from_wire_value::<Option<u32>>(limit, "limit");
        match (market, limit) {
            (Ok(market), Ok(limit)) => outcome(wire_vec::<_, WireTrade>(
                self.inner.trades(&market, limit).await,
            )),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "orderBook"))]
    pub async fn order_book(&self, market: Value, depth: Value) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let depth = from_wire_value::<Option<u32>>(depth, "depth");
        match (market, depth) {
            (Ok(market), Ok(depth)) => outcome(wire_one::<_, WireOrderBook>(
                self.inner.order_book(&market, depth).await,
            )),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "ticker"))]
    pub async fn ticker(&self, market: Value) -> Value {
        match parse_wire::<maxt::Market, WireMarket>(market, "market") {
            Ok(market) => outcome(wire_one::<_, WireTicker>(self.inner.ticker(&market).await)),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "candles"))]
    pub async fn candles(&self, request: Value) -> Value {
        match parse_wire::<maxt::CandleRequest, WireCandleRequest>(request, "request") {
            Ok(request) => outcome(wire_vec::<_, WireCandle>(
                self.inner.candles(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "balances"))]
    pub async fn balances(&self) -> Value {
        outcome(wire_vec::<_, WireBalance>(self.inner.balances().await))
    }

    #[cfg_attr(not(test), napi(js_name = "openOrders"))]
    pub async fn open_orders(&self) -> Value {
        outcome(wire_vec::<_, WireOrder>(self.inner.open_orders().await))
    }

    #[cfg_attr(not(test), napi(js_name = "openOrdersOn"))]
    pub async fn open_orders_on(&self, market: Value) -> Value {
        let market = from_wire_value::<WireMarket>(market, "market").and_then(TryInto::try_into);
        match market {
            Ok(market) => outcome(wire_vec::<_, WireOrder>(
                self.inner.open_orders_on(&market).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "positions"))]
    pub async fn positions(&self) -> Value {
        outcome(wire_vec::<_, WirePosition>(self.inner.positions().await))
    }

    #[cfg_attr(not(test), napi(js_name = "placeOrder"))]
    pub async fn place_order(&self, request: Value) -> Value {
        match parse_wire::<maxt::OrderRequest, WireOrderRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WireOrder>(
                self.inner.place_order(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "cancelOrder"))]
    pub async fn cancel_order(&self, market: Value, order_id: Value) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let order_id = from_wire_value::<String>(order_id, "order_id");
        match (market, order_id) {
            (Ok(market), Ok(order_id)) => outcome(wire_one::<_, WireOrder>(
                self.inner.cancel_order(&market, &order_id).await,
            )),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "positionsOn"))]
    pub async fn positions_on(&self, market: Value) -> Value {
        match parse_wire::<maxt::Market, WireMarket>(market, "market") {
            Ok(market) => outcome(wire_vec::<_, WirePosition>(
                self.inner.positions_on(&market).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "marginSummary"))]
    pub async fn margin_summary(&self) -> Value {
        outcome(wire_one::<_, WireMarginSummary>(
            self.inner.margin_summary().await,
        ))
    }

    #[cfg_attr(not(test), napi(js_name = "fundingRates"))]
    pub async fn funding_rates(&self, request: Value) -> Value {
        match parse_wire::<maxt::HistoryRequest, WireHistoryRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WirePage<WireFundingRate>>(
                self.inner.funding_rates(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "fundingPayments"))]
    pub async fn funding_payments(&self, request: Value) -> Value {
        match parse_wire::<maxt::HistoryRequest, WireHistoryRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WirePage<WireFundingPayment>>(
                self.inner.funding_payments(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "setMargin"))]
    pub async fn set_margin(&self, request: Value) -> Value {
        match parse_wire::<maxt::MarginRequest, WireMarginRequest>(request, "request") {
            Ok(request) => outcome(self.inner.set_margin(&request).await.map(|()| Value::Null)),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(not(test), napi(js_name = "subscribe"))]
    pub async fn subscribe(&self, subscription: Value) -> Value {
        let subscription = from_wire_value::<WireSubscription>(subscription, "subscription")
            .and_then(TryInto::try_into);
        let result = match subscription {
            Ok(subscription) => match self.inner.subscribe(&subscription).await {
                Ok(stream) => self.streams.insert_market(stream).await,
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        outcome(result)
    }

    #[cfg_attr(not(test), napi(js_name = "subscribeWith"))]
    pub async fn subscribe_with(&self, subscription: Value, config: Value) -> Value {
        let subscription = from_wire_value::<WireSubscription>(subscription, "subscription")
            .and_then(TryInto::try_into);
        let config =
            from_wire_value::<WireStreamConfig>(config, "config").and_then(TryInto::try_into);
        let result = match (subscription, config) {
            (Ok(subscription), Ok(config)) => {
                match self.inner.subscribe_with(&subscription, &config).await {
                    Ok(stream) => self.streams.insert_market(stream).await,
                    Err(error) => Err(error),
                }
            }
            (Err(error), _) | (_, Err(error)) => Err(error),
        };
        outcome(result)
    }

    #[cfg_attr(not(test), napi(js_name = "subscribeAccount"))]
    pub async fn subscribe_account(&self) -> Value {
        let result = match self.inner.subscribe_account().await {
            Ok(stream) => self.streams.insert_account(stream).await,
            Err(error) => Err(error),
        };
        outcome(result)
    }

    #[cfg_attr(not(test), napi(js_name = "subscribeAccountWith"))]
    pub async fn subscribe_account_with(&self, config: Value) -> Value {
        let config =
            from_wire_value::<WireStreamConfig>(config, "config").and_then(TryInto::try_into);
        let result = match config {
            Ok(config) => match self.inner.subscribe_account_with(&config).await {
                Ok(stream) => self.streams.insert_account(stream).await,
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        outcome(result)
    }

    #[cfg_attr(not(test), napi(js_name = "streamNext"))]
    pub async fn stream_next(&self, id: Value) -> Value {
        let result = match from_wire_value::<String>(id, "stream_id") {
            Ok(id) => self.streams.next(&id).await,
            Err(error) => Err(error),
        };
        outcome(result)
    }

    #[cfg_attr(not(test), napi(js_name = "streamClose"))]
    pub async fn stream_close(&self, id: Value) -> Value {
        let result = match from_wire_value::<String>(id, "stream_id") {
            Ok(id) => self.streams.close(&id).await,
            Err(error) => Err(error),
        };
        outcome(result.map(|()| Value::Null))
    }
}

fn parse_wire<T, W>(value: Value, field: &str) -> maxt::Result<T>
where
    W: DeserializeOwned,
    T: TryFrom<W, Error = Error>,
{
    from_wire_value::<W>(value, field).and_then(TryInto::try_into)
}

fn wire_one<T, W>(result: maxt::Result<T>) -> maxt::Result<W>
where
    W: TryFrom<T, Error = Error> + Serialize,
{
    result.and_then(TryInto::try_into)
}

fn wire_vec<T, W>(result: maxt::Result<Vec<T>>) -> maxt::Result<Vec<W>>
where
    W: TryFrom<T, Error = Error> + Serialize,
{
    result.and_then(|values| {
        values
            .into_iter()
            .map(TryInto::try_into)
            .collect::<maxt::Result<_>>()
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use maxt::{
        AccountEvent, AccountStream, Adapter, Balance, BoxFuture, Candle, CandleRequest, Decimal,
        Exchange, Feature, FundingPayment, FundingRate, HistoryRequest, MarginRequest,
        MarginSummary, Market, MarketEvent, MarketInfo, MarketKind, MarketStream, Order, OrderBook,
        OrderRequest, OrderStatus, Page, Position, Side, StreamConfig, Subscription, Ticker,
        Timestamp, Trade,
    };

    use super::*;

    struct RecordingAdapter {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl RecordingAdapter {
        fn order(market: Market, side: Side) -> Order {
            Order {
                id: "order-1".to_owned(),
                market,
                side,
                status: OrderStatus::Open,
                filled_quantity: Decimal::ZERO,
                remaining_quantity: Decimal::ONE,
                price: None,
                created_at: None,
            }
        }
    }

    impl Adapter for RecordingAdapter {
        fn exchange(&self) -> Exchange {
            Exchange::Binance
        }

        fn supports(&self, _feature: Feature) -> bool {
            true
        }

        fn markets(&self, _kind: MarketKind) -> BoxFuture<'_, maxt::Result<Vec<MarketInfo>>> {
            self.calls.lock().unwrap().push("markets");
            Box::pin(async { Ok(vec![]) })
        }

        fn trades(
            &self,
            _market: &Market,
            _limit: Option<u32>,
        ) -> BoxFuture<'_, maxt::Result<Vec<Trade>>> {
            self.calls.lock().unwrap().push("trades");
            Box::pin(async { Ok(vec![]) })
        }

        fn order_book(
            &self,
            market: &Market,
            _depth: Option<u32>,
        ) -> BoxFuture<'_, maxt::Result<OrderBook>> {
            self.calls.lock().unwrap().push("order_book");
            let market = market.clone();
            Box::pin(async move {
                Ok(OrderBook {
                    market,
                    timestamp: Timestamp::from_nanos(1),
                    bids: vec![],
                    asks: vec![],
                })
            })
        }

        fn ticker(&self, market: &Market) -> BoxFuture<'_, maxt::Result<Ticker>> {
            self.calls.lock().unwrap().push("ticker");
            let market = market.clone();
            Box::pin(async move {
                Ok(Ticker {
                    market,
                    timestamp: Timestamp::from_nanos(1),
                    last_trade_time: None,
                    last_price: Decimal::ONE,
                    change: None,
                    change_rate: None,
                    high: None,
                    low: None,
                    volume: None,
                    quote_volume: None,
                })
            })
        }

        fn candles(&self, _request: &CandleRequest) -> BoxFuture<'_, maxt::Result<Vec<Candle>>> {
            self.calls.lock().unwrap().push("candles");
            Box::pin(async { Ok(vec![]) })
        }

        fn balances(&self) -> BoxFuture<'_, maxt::Result<Vec<Balance>>> {
            self.calls.lock().unwrap().push("balances");
            Box::pin(async { Ok(vec![]) })
        }

        fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, maxt::Result<Vec<Order>>> {
            self.calls.lock().unwrap().push(if market.is_some() {
                "open_orders:some"
            } else {
                "open_orders:none"
            });
            Box::pin(async { Ok(vec![]) })
        }

        fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, maxt::Result<Vec<Position>>> {
            self.calls.lock().unwrap().push(if market.is_some() {
                "positions:some"
            } else {
                "positions:none"
            });
            let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
            Box::pin(async move {
                Ok([Decimal::ZERO, Decimal::ONE]
                    .into_iter()
                    .map(|quantity| Position {
                        market: market.clone(),
                        side: (!quantity.is_zero()).then_some(Side::Buy),
                        quantity,
                        entry_price: None,
                        mark_price: None,
                        notional: None,
                        unrealized_pnl: None,
                        leverage: None,
                        margin_mode: None,
                    })
                    .collect())
            })
        }

        fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, maxt::Result<Order>> {
            self.calls.lock().unwrap().push("place_order");
            let order = Self::order(request.market.clone(), request.side);
            Box::pin(async move { Ok(order) })
        }

        fn cancel_order(
            &self,
            market: &Market,
            _order_id: &str,
        ) -> BoxFuture<'_, maxt::Result<Order>> {
            self.calls.lock().unwrap().push("cancel_order");
            let order = Self::order(market.clone(), Side::Sell);
            Box::pin(async move { Ok(order) })
        }

        fn margin_summary(&self) -> BoxFuture<'_, maxt::Result<MarginSummary>> {
            self.calls.lock().unwrap().push("margin_summary");
            Box::pin(async {
                Ok(MarginSummary {
                    asset: "USDT".to_owned(),
                    equity: None,
                    margin_balance: None,
                    available_balance: None,
                })
            })
        }

        fn funding_rates(
            &self,
            _request: &HistoryRequest,
        ) -> BoxFuture<'_, maxt::Result<Page<FundingRate>>> {
            self.calls.lock().unwrap().push("funding_rates");
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }

        fn funding_payments(
            &self,
            _request: &HistoryRequest,
        ) -> BoxFuture<'_, maxt::Result<Page<FundingPayment>>> {
            self.calls.lock().unwrap().push("funding_payments");
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }

        fn set_margin(&self, _request: &MarginRequest) -> BoxFuture<'_, maxt::Result<()>> {
            self.calls.lock().unwrap().push("set_margin");
            Box::pin(async { Ok(()) })
        }

        fn subscribe(
            &self,
            _subscription: &Subscription,
            config: &StreamConfig,
        ) -> BoxFuture<'_, maxt::Result<MarketStream>> {
            self.calls
                .lock()
                .unwrap()
                .push(if config.buffer_size == 4_096 {
                    "subscribe:default"
                } else {
                    "subscribe:custom"
                });
            Box::pin(async {
                Ok(MarketStream::new(futures_util::stream::iter([Ok(
                    MarketEvent::Reconnected,
                )])))
            })
        }

        fn subscribe_account(
            &self,
            config: &StreamConfig,
        ) -> BoxFuture<'_, maxt::Result<AccountStream>> {
            self.calls
                .lock()
                .unwrap()
                .push(if config.buffer_size == 4_096 {
                    "subscribe_account:default"
                } else {
                    "subscribe_account:custom"
                });
            Box::pin(async {
                Ok(AccountStream::new(futures_util::stream::iter([Ok(
                    AccountEvent::Reconnected,
                )])))
            })
        }
    }

    #[tokio::test]
    async fn native_client_uses_core_filtering_and_optional_market_calls() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));

        let value = client.positions().await;
        assert_eq!(value["ok"], true);
        assert_eq!(value["value"].as_array().unwrap().len(), 1);

        assert_eq!(client.open_orders().await["ok"], true);
        assert_eq!(client.open_orders_on(market_wire()).await["ok"], true);
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["positions:none", "open_orders:none", "open_orders:some"]
        );
    }

    fn market_wire() -> serde_json::Value {
        serde_json::json!({
            "exchange": "binance",
            "kind": "perpetual",
            "base": "BTC",
            "quote": "USDT"
        })
    }

    #[tokio::test]
    async fn native_client_uses_core_stream_defaults_and_its_own_registry() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));
        let subscription = serde_json::json!({
            "markets": [market_wire()],
            "feeds": [{ "kind": "trades" }]
        });
        let config = serde_json::json!({
            "max_reconnect_attempts": null,
            "initial_reconnect_delay_ms": 1,
            "max_reconnect_delay_ms": 2,
            "idle_timeout_ms": 3,
            "buffer_size": 1,
            "overflow": "backpressure"
        });

        let market_default = client.subscribe(subscription.clone()).await;
        let market_custom = client.subscribe_with(subscription, config.clone()).await;
        let account_default = client.subscribe_account().await;
        let account_custom = client.subscribe_account_with(config).await;

        for handle in [
            &market_default["value"],
            &market_custom["value"],
            &account_default["value"],
            &account_custom["value"],
        ] {
            assert_eq!(client.stream_next(handle["id"].clone()).await["ok"], true);
            assert_eq!(client.stream_close(handle["id"].clone()).await["ok"], true);
        }
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                "subscribe:default",
                "subscribe:custom",
                "subscribe_account:default",
                "subscribe_account:custom",
            ]
        );
    }

    #[tokio::test]
    async fn native_client_forwards_all_sixteen_non_stream_operations() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));
        let market = market_wire();
        let candle_request = serde_json::json!({
            "market": market,
            "interval": "min1",
            "from": null,
            "to": null,
            "limit": null
        });
        let history_request = serde_json::json!({
            "market": market,
            "from": null,
            "to": null,
            "cursor": null,
            "limit": null
        });
        let order_request = serde_json::json!({
            "market": market,
            "side": "buy",
            "order_type": "market",
            "size": { "kind": "base", "value": "1.00" },
            "price": null,
            "time_in_force": null,
            "reduce_only": false
        });
        let margin_request = serde_json::json!({
            "market": market,
            "leverage": "10.0",
            "margin_mode": null
        });

        assert_eq!(client.exchange(), "binance");
        assert!(client.supports("ticker".to_owned()));
        let results = [
            client.markets(serde_json::json!("spot")).await,
            client.trades(market.clone(), Value::Null).await,
            client.order_book(market.clone(), Value::Null).await,
            client.ticker(market.clone()).await,
            client.candles(candle_request).await,
            client.balances().await,
            client.open_orders().await,
            client.open_orders_on(market.clone()).await,
            client.place_order(order_request).await,
            client
                .cancel_order(market.clone(), serde_json::json!("order-1"))
                .await,
            client.positions().await,
            client.positions_on(market).await,
            client.margin_summary().await,
            client.funding_rates(history_request.clone()).await,
            client.funding_payments(history_request).await,
            client.set_margin(margin_request).await,
        ];
        assert!(results.iter().all(|value| value["ok"] == true));
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                "markets",
                "trades",
                "order_book",
                "ticker",
                "candles",
                "balances",
                "open_orders:none",
                "open_orders:some",
                "place_order",
                "cancel_order",
                "positions:none",
                "positions:some",
                "margin_summary",
                "funding_rates",
                "funding_payments",
                "set_margin",
            ]
        );
    }
}
