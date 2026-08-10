//! Foreign adapter forwarding and stream lifecycle contracts.

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use futures_util::stream;
use futures_util::{Stream, StreamExt};
use maxt::{
    AccountEvent, AccountStream, Adapter, BoxFuture, CandleRequest, Client, Decimal, Error,
    Exchange, Feature, Feed, HistoryRequest, Interval, MarginRequest, MarginSummary, Market,
    MarketEvent, MarketKind, MarketStream, Order, OrderBook, OrderHistoryRequest,
    OrderLookupRequest, OrderRequest, OrderStatus, Page, Result, Side, Size, StreamConfig,
    Subscription, Ticker, Timestamp,
};
use maxt_bindings_common::{AdapterCall, AdapterReply, ForeignAdapter, ForeignDispatcher};

struct RecordingDispatcher {
    calls: Mutex<Vec<AdapterCall>>,
    replies: Mutex<VecDeque<Result<AdapterReply>>>,
}

impl RecordingDispatcher {
    fn new(replies: impl IntoIterator<Item = AdapterReply>) -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(Vec::new()),
            replies: Mutex::new(replies.into_iter().map(Ok).collect()),
        })
    }

    fn calls(&self) -> Vec<AdapterCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl ForeignDispatcher for RecordingDispatcher {
    fn dispatch(&self, call: AdapterCall) -> BoxFuture<'_, Result<AdapterReply>> {
        self.calls.lock().unwrap().push(call);
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("테스트 응답이 호출마다 있어야 합니다");
        Box::pin(async move { reply })
    }
}

struct BorrowingDispatcher {
    reply: Mutex<Option<AdapterReply>>,
}

impl ForeignDispatcher for BorrowingDispatcher {
    fn dispatch(&self, _call: AdapterCall) -> BoxFuture<'_, Result<AdapterReply>> {
        Box::pin(async move {
            Ok(self
                .reply
                .lock()
                .unwrap()
                .take()
                .expect("빌린 디스패처 응답은 한 번 존재해야 합니다"))
        })
    }
}

struct DropAware<S> {
    inner: S,
    dropped: Arc<AtomicBool>,
}

impl<S: Stream + Unpin> Stream for DropAware<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}

impl<S> Drop for DropAware<S> {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

fn assert_pending(stream: &mut (impl Stream + Unpin)) {
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        Pin::new(stream).poll_next(&mut context),
        Poll::Pending
    ));
}

fn market() -> Market {
    Market::perpetual(Exchange::Binance, "BTC", "USDT")
}

fn order_book() -> OrderBook {
    OrderBook {
        market: market(),
        timestamp: Timestamp::from_nanos(1),
        bids: vec![],
        asks: vec![],
    }
}

fn ticker() -> Ticker {
    Ticker {
        market: market(),
        timestamp: Timestamp::from_nanos(1),
        last_trade_time: None,
        last_price: Decimal::ONE,
        change: None,
        change_rate: None,
        high: None,
        low: None,
        volume: None,
        quote_volume: None,
    }
}

fn order(id: &str) -> Order {
    Order {
        id: id.to_string(),
        market: market(),
        side: Side::Buy,
        status: OrderStatus::Open,
        filled_quantity: Decimal::ZERO,
        remaining_quantity: Decimal::ONE,
        price: Some(Decimal::ONE),
        created_at: None,
    }
}

fn adapter(
    dispatcher: Arc<RecordingDispatcher>,
    features: impl IntoIterator<Item = Feature>,
) -> ForeignAdapter {
    ForeignAdapter::new(Exchange::Binance, features, dispatcher)
}

#[tokio::test]
async fn dispatcher_can_borrow_self_behind_a_trait_object() {
    let dispatcher: Arc<dyn ForeignDispatcher> = Arc::new(BorrowingDispatcher {
        reply: Mutex::new(Some(AdapterReply::Trades(vec![]))),
    });
    let client = Client::new(ForeignAdapter::new(
        Exchange::Binance,
        [Feature::Trades],
        dispatcher,
    ));

    assert!(client.trades(&market(), Some(1)).await.unwrap().is_empty());
}

#[tokio::test]
async fn every_current_adapter_method_forwards_an_owned_call() {
    let dispatcher = RecordingDispatcher::new([
        AdapterReply::Markets(vec![]),
        AdapterReply::Trades(vec![]),
        AdapterReply::OrderBook(order_book()),
        AdapterReply::Ticker(ticker()),
        AdapterReply::Candles(vec![]),
        AdapterReply::MarketStream(MarketStream::new(stream::empty::<Result<MarketEvent>>())),
        AdapterReply::Balances(vec![]),
        AdapterReply::OpenOrders(vec![]),
        AdapterReply::Order(order("by-id")),
        AdapterReply::Order(order("by-client-id")),
        AdapterReply::OrdersByIds(vec![order("by-ids")]),
        AdapterReply::OrderHistory(Page {
            items: vec![],
            next: None,
        }),
        AdapterReply::AccountStream(AccountStream::new(stream::empty::<Result<AccountEvent>>())),
        AdapterReply::PlaceOrder(order("placed")),
        AdapterReply::Unit,
        AdapterReply::Unit,
        AdapterReply::Positions(vec![]),
        AdapterReply::MarginSummary(MarginSummary {
            asset: "USDT".to_string(),
            equity: None,
            margin_balance: None,
            available_balance: None,
        }),
        AdapterReply::FundingRates(Page {
            items: vec![],
            next: None,
        }),
        AdapterReply::FundingPayments(Page {
            items: vec![],
            next: None,
        }),
        AdapterReply::Unit,
    ]);
    let adapter = adapter(
        dispatcher.clone(),
        [Feature::Trades, Feature::Trades, Feature::Trading],
    );
    let market = market();
    let candle_request = CandleRequest::new(market.clone(), Interval::Min1).limit(3);
    let subscription = Subscription::new()
        .market(market.clone())
        .feed(Feed::Trades);
    let config = StreamConfig::default();
    let order_request = OrderRequest::market(market.clone(), Side::Buy, Size::Base(Decimal::ONE));
    let order_lookup_request = OrderLookupRequest::exchange(["order-1"]).market(market.clone());
    let history_request = HistoryRequest::new(market.clone()).limit(7);
    let order_history_request = OrderHistoryRequest::new().market(market.clone()).limit(7);
    let margin_request = MarginRequest::new(market.clone()).leverage(Decimal::from(2));

    assert_eq!(adapter.exchange(), Exchange::Binance);
    assert!(adapter.supports(Feature::Trades));
    assert!(adapter.supports(Feature::Trading));
    assert!(!adapter.supports(Feature::Ticker));
    assert_eq!(adapter.features().len(), 2);
    assert!(adapter.features().contains(&Feature::Trades));
    assert!(adapter.features().contains(&Feature::Trading));

    adapter.markets(MarketKind::Perpetual).await.unwrap();
    adapter.trades(&market, Some(3)).await.unwrap();
    adapter.order_book(&market, Some(20)).await.unwrap();
    adapter.ticker(&market).await.unwrap();
    adapter.candles(&candle_request).await.unwrap();
    let _market_stream = adapter.subscribe(&subscription, &config).await.unwrap();
    adapter.balances().await.unwrap();
    adapter.open_orders(Some(&market)).await.unwrap();
    adapter.order(&market, "order-1").await.unwrap();
    adapter
        .order_by_client_id(&market, "client-1")
        .await
        .unwrap();
    adapter.orders_by_ids(&order_lookup_request).await.unwrap();
    adapter.order_history(&order_history_request).await.unwrap();
    let _account_stream = adapter.subscribe_account(&config).await.unwrap();
    adapter.place_order(&order_request).await.unwrap();
    adapter.cancel_order(&market, "order-1").await.unwrap();
    adapter
        .cancel_order_by_client_id(&market, "client-1")
        .await
        .unwrap();
    adapter.positions(Some(&market)).await.unwrap();
    adapter.margin_summary().await.unwrap();
    adapter.funding_rates(&history_request).await.unwrap();
    adapter.funding_payments(&history_request).await.unwrap();
    adapter.set_margin(&margin_request).await.unwrap();

    assert_eq!(
        dispatcher.calls(),
        vec![
            AdapterCall::Markets {
                kind: MarketKind::Perpetual,
            },
            AdapterCall::Trades {
                market: market.clone(),
                limit: Some(3),
            },
            AdapterCall::OrderBook {
                market: market.clone(),
                depth: Some(20),
            },
            AdapterCall::Ticker {
                market: market.clone(),
            },
            AdapterCall::Candles {
                request: candle_request,
            },
            AdapterCall::Subscribe {
                subscription,
                config: config.clone(),
            },
            AdapterCall::Balances,
            AdapterCall::OpenOrders {
                market: Some(market.clone()),
            },
            AdapterCall::Order {
                market: market.clone(),
                order_id: "order-1".to_string(),
            },
            AdapterCall::OrderByClientId {
                market: market.clone(),
                client_id: "client-1".to_string(),
            },
            AdapterCall::OrdersByIds {
                request: order_lookup_request,
            },
            AdapterCall::OrderHistory {
                request: order_history_request,
            },
            AdapterCall::SubscribeAccount {
                config: config.clone(),
            },
            AdapterCall::PlaceOrder {
                request: order_request,
            },
            AdapterCall::CancelOrder {
                market: market.clone(),
                order_id: "order-1".to_string(),
            },
            AdapterCall::CancelOrderByClientId {
                market: market.clone(),
                client_id: "client-1".to_string(),
            },
            AdapterCall::Positions {
                market: Some(market.clone()),
            },
            AdapterCall::MarginSummary,
            AdapterCall::FundingRates {
                request: history_request.clone(),
            },
            AdapterCall::FundingPayments {
                request: history_request,
            },
            AdapterCall::SetMargin {
                request: margin_request,
            },
        ]
    );
}

#[tokio::test]
async fn a_reply_variant_mismatch_is_an_adapter_error() {
    let dispatcher = RecordingDispatcher::new([AdapterReply::Ticker(ticker())]);
    let client = Client::new(adapter(dispatcher, [Feature::Trades]));

    let error = client.trades(&market(), Some(1)).await.unwrap_err();

    assert!(matches!(error, Error::Adapter { .. }));
    assert_eq!(
        error.to_string(),
        "adapter failed: foreign dispatcher returned Ticker where Trades was required"
    );
}

#[tokio::test]
async fn place_order_and_unit_replies_are_not_interchangeable() {
    let request = OrderRequest::market(market(), Side::Buy, Size::Base(Decimal::ONE));
    let place_dispatcher = RecordingDispatcher::new([AdapterReply::Unit]);
    let place_client = Client::new(adapter(place_dispatcher, [Feature::Trading]));

    let place_error = place_client.place_order(&request).await.unwrap_err();
    assert!(matches!(place_error, Error::Adapter { .. }));

    let cancel_dispatcher = RecordingDispatcher::new([AdapterReply::PlaceOrder(order("wrong"))]);
    let cancel_client = Client::new(adapter(cancel_dispatcher, [Feature::Trading]));

    let cancel_error = cancel_client
        .cancel_order(&market(), "order-1")
        .await
        .unwrap_err();
    assert!(matches!(cancel_error, Error::Adapter { .. }));
}

#[tokio::test]
async fn market_stream_continues_after_error_and_drop_cancels_the_source() {
    let dropped = Arc::new(AtomicBool::new(false));
    let source = DropAware {
        inner: stream::iter([
            Ok(MarketEvent::Reconnected),
            Err(Error::adapter("bad market item")),
            Ok(MarketEvent::Reconnected),
        ])
        .chain(stream::pending()),
        dropped: dropped.clone(),
    };
    let dispatcher =
        RecordingDispatcher::new([AdapterReply::MarketStream(MarketStream::new(source))]);
    let adapter = adapter(dispatcher, [Feature::TradeStream]);
    let subscription = Subscription::new().market(market()).feed(Feed::Trades);
    let mut events = adapter
        .subscribe(&subscription, &StreamConfig::default())
        .await
        .unwrap();

    assert!(matches!(
        events.next().await,
        Some(Ok(MarketEvent::Reconnected))
    ));
    assert!(matches!(
        events.next().await,
        Some(Err(Error::Adapter { .. }))
    ));
    assert!(matches!(
        events.next().await,
        Some(Ok(MarketEvent::Reconnected))
    ));
    assert_pending(&mut events);
    assert!(!dropped.load(Ordering::SeqCst));

    drop(events);
    assert!(dropped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn account_stream_continues_after_error_and_drop_cancels_the_source() {
    let dropped = Arc::new(AtomicBool::new(false));
    let source = DropAware {
        inner: stream::iter([
            Ok(AccountEvent::Reconnected),
            Err(Error::adapter("bad account item")),
            Ok(AccountEvent::Reconnected),
        ])
        .chain(stream::pending()),
        dropped: dropped.clone(),
    };
    let dispatcher =
        RecordingDispatcher::new([AdapterReply::AccountStream(AccountStream::new(source))]);
    let adapter = adapter(dispatcher, [Feature::AccountStream]);
    let mut events = adapter
        .subscribe_account(&StreamConfig::default())
        .await
        .unwrap();

    assert!(matches!(
        events.next().await,
        Some(Ok(AccountEvent::Reconnected))
    ));
    assert!(matches!(
        events.next().await,
        Some(Err(Error::Adapter { .. }))
    ));
    assert!(matches!(
        events.next().await,
        Some(Ok(AccountEvent::Reconnected))
    ));
    assert_pending(&mut events);
    assert!(!dropped.load(Ordering::SeqCst));

    drop(events);
    assert!(dropped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn finite_foreign_streams_end_with_none() {
    let dispatcher = RecordingDispatcher::new([
        AdapterReply::MarketStream(MarketStream::new(stream::iter([Ok(
            MarketEvent::Reconnected,
        )]))),
        AdapterReply::AccountStream(AccountStream::new(stream::iter([Ok(
            AccountEvent::Reconnected,
        )]))),
    ]);
    let adapter = adapter(dispatcher, [Feature::TradeStream, Feature::AccountStream]);
    let subscription = Subscription::new().market(market()).feed(Feed::Trades);

    let mut market_events = adapter
        .subscribe(&subscription, &StreamConfig::default())
        .await
        .unwrap();
    assert!(market_events.next().await.is_some());
    assert!(market_events.next().await.is_none());

    let mut account_events = adapter
        .subscribe_account(&StreamConfig::default())
        .await
        .unwrap();
    assert!(account_events.next().await.is_some());
    assert!(account_events.next().await.is_none());
}
