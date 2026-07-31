//! Verifies that external crates can implement [`Adapter`].
//!
//! Integration tests compile as a separate crate. `Fictional` exercises every
//! method and `BareMinimum` exercises the defaults.

use futures_util::StreamExt;
use futures_util::stream;
use maxt::adapters::UpbitAdapter;
use maxt::{
    AccountEvent, AccountStream, Adapter, Balance, BoxFuture, Candle, CandleRequest, Client,
    Cursor, Decimal, Error, Exchange, ExchangeErrorKind, Feature, Feed, FundingPayment,
    FundingRate, HistoryRequest, Level, MarginMode, MarginRequest, MarginSummary, Market,
    MarketEvent, MarketInfo, MarketKind, MarketStatus, MarketStream, Order, OrderBook,
    OrderRequest, OrderStatus, Overflow, Page, Position, Result, Side, Size, StreamConfig,
    Subscription, Ticker, Timestamp, Trade,
};

/// A complete external adapter fixture.
struct Fictional;

impl Fictional {
    /// The market used by every returned value.
    fn market() -> Market {
        // External adapters currently reuse an existing `Exchange` identity.
        Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC")
    }

    /// One trade returned over REST and the live stream.
    fn trade() -> Trade {
        Trade {
            market: Self::market(),
            timestamp: Timestamp::from_millis(1_700_000_000_000),
            price: Decimal::from(30_000),
            quantity: Decimal::new(5, 1),
            taker_side: Side::Buy,
            id: Some("1".to_string()),
        }
    }

    /// One resting order, under whichever identifier it is asked about.
    fn order(id: &str) -> Order {
        Order {
            id: id.to_string(),
            market: Self::market(),
            side: Side::Buy,
            status: OrderStatus::Open,
            filled_quantity: Decimal::ZERO,
            remaining_quantity: Decimal::ONE,
            price: Some(Decimal::from(29_000)),
            created_at: Some(Timestamp::from_millis(1_700_000_000_000)),
        }
    }
}

impl Adapter for Fictional {
    fn exchange(&self) -> Exchange {
        Exchange::Hyperliquid
    }

    fn supports(&self, _feature: Feature) -> bool {
        true
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move {
            if kind != MarketKind::Perpetual {
                return Ok(Vec::new());
            }
            Ok(vec![MarketInfo {
                market: Self::market(),
                native_symbol: "BTC-PERP".to_string(),
                status: MarketStatus::Active,
                korean_name: None,
                english_name: Some("Bitcoin".to_string()),
            }])
        })
    }

    fn trades(&self, _market: &Market, _limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        Box::pin(async move { Ok(vec![Self::trade()]) })
    }

    fn order_book(
        &self,
        _market: &Market,
        _depth: Option<u32>,
    ) -> BoxFuture<'_, Result<OrderBook>> {
        Box::pin(async move {
            Ok(OrderBook {
                market: Self::market(),
                timestamp: Timestamp::from_millis(1_700_000_000_000),
                bids: vec![Level {
                    price: Decimal::from(29_999),
                    quantity: Decimal::ONE,
                }],
                asks: vec![Level {
                    price: Decimal::from(30_001),
                    quantity: Decimal::ONE,
                }],
            })
        })
    }

    fn ticker(&self, _market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        Box::pin(async move {
            Ok(Ticker {
                market: Self::market(),
                timestamp: Timestamp::from_millis(1_700_000_000_000),
                last_trade_time: None,
                last_price: Decimal::from(30_000),
                change: None,
                change_rate: None,
                high: None,
                low: None,
                volume: None,
                quote_volume: None,
            })
        })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let interval = request.interval;
        Box::pin(async move {
            Ok(vec![Candle {
                market: Self::market(),
                interval,
                open_time: Timestamp::from_millis(1_700_000_000_000),
                open: Decimal::from(30_000),
                high: Decimal::from(30_100),
                low: Decimal::from(29_900),
                close: Decimal::from(30_050),
                volume: Decimal::from(12),
                quote_volume: None,
                closed: true,
            }])
        })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move {
            Ok(vec![Balance {
                asset: "USDC".to_string(),
                available: Decimal::from(1_000),
                locked: Decimal::ZERO,
            }])
        })
    }

    fn open_orders(&self, _market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        Box::pin(async move { Ok(vec![Self::order("order-1")]) })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        // Use both inputs to exercise the full subscription signature.
        let wanted = subscription.feeds().contains(&Feed::Trades);
        let overflow = config.overflow;

        Box::pin(async move {
            if !wanted {
                return Err(Error::InvalidRequest {
                    field: "feeds",
                    detail: "this exchange publishes trades and nothing else".to_string(),
                });
            }
            Ok(MarketStream::new(stream::iter(vec![
                Ok(MarketEvent::Reconnected),
                Ok(MarketEvent::Trade(Self::trade())),
                // A following event proves stream errors are non-terminal items.
                Err(Error::Decode {
                    detail: format!("a frame this adapter could not read under {overflow:?}"),
                }),
                Ok(MarketEvent::Trade(Self::trade())),
            ])))
        })
    }

    fn subscribe_account(&self, _config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        Box::pin(async move {
            Ok(AccountStream::new(stream::iter(vec![
                Ok(AccountEvent::Reconnected),
                Ok(AccountEvent::Balance(Balance {
                    asset: "USDC".to_string(),
                    available: Decimal::from(900),
                    locked: Decimal::from(100),
                })),
                Ok(AccountEvent::Order(Self::order("order-1"))),
            ])))
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let side = request.side;
        Box::pin(async move {
            Ok(Order {
                side,
                ..Self::order("order-2")
            })
        })
    }

    fn cancel_order(&self, _market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let id = order_id.to_string();
        Box::pin(async move {
            Ok(Order {
                status: OrderStatus::Cancelled,
                ..Self::order(&id)
            })
        })
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        let leverage = request.leverage;
        Box::pin(async move {
            match leverage {
                Some(leverage) if leverage > Decimal::from(20) => Err(Error::Exchange {
                    exchange: "fictional",
                    code: "1001".to_string(),
                    message: "leverage above the market maximum".to_string(),
                    status: Some(400),
                    kind: ExchangeErrorKind::Rejected,
                }),
                _ => Ok(()),
            }
        })
    }

    fn positions(&self, _market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        Box::pin(async move {
            Ok(vec![
                Position {
                    market: Self::market(),
                    side: Some(Side::Buy),
                    quantity: Decimal::ONE,
                    entry_price: Some(Decimal::from(29_500)),
                    mark_price: Some(Decimal::from(30_000)),
                    notional: Some(Decimal::from(30_000)),
                    unrealized_pnl: Some(Decimal::from(500)),
                    leverage: Some(Decimal::from(3)),
                    margin_mode: Some(MarginMode::Cross),
                },
                // `Client` owns the zero-size filter for external adapters.
                Position {
                    market: Self::market(),
                    side: None,
                    quantity: Decimal::ZERO,
                    entry_price: None,
                    mark_price: Some(Decimal::from(30_000)),
                    notional: Some(Decimal::ZERO),
                    unrealized_pnl: Some(Decimal::ZERO),
                    leverage: None,
                    margin_mode: None,
                },
            ])
        })
    }

    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        Box::pin(async move {
            Ok(MarginSummary {
                asset: "USDC".to_string(),
                equity: Some(Decimal::from(1_500)),
                margin_balance: Some(Decimal::from(1_000)),
                available_balance: Some(Decimal::from(500)),
            })
        })
    }

    fn funding_rates(&self, _request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        Box::pin(async move {
            Ok(Page {
                items: vec![FundingRate {
                    market: Self::market(),
                    timestamp: Timestamp::from_millis(1_700_000_000_000),
                    rate: Decimal::new(1, 4),
                    mark_price: Some(Decimal::from(30_000)),
                }],
                next: None,
            })
        })
    }

    fn funding_payments(
        &self,
        _request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        Box::pin(async move {
            Ok(Page {
                items: vec![FundingPayment {
                    market: Self::market(),
                    timestamp: Timestamp::from_millis(1_700_000_000_000),
                    amount: Decimal::new(-25, 2),
                    rate: Some(Decimal::new(1, 4)),
                    id: Some("payment-1".to_string()),
                }],
                // External adapters must be able to construct resume cursors.
                next: Some(Cursor::new("page-2")),
            })
        })
    }
}

#[tokio::test]
async fn a_crate_that_is_not_maxt_can_implement_an_adapter_end_to_end() {
    let client = Client::new(Fictional);

    assert_eq!(
        client.markets(MarketKind::Perpetual).await.unwrap().len(),
        1
    );
    assert!(client.markets(MarketKind::Spot).await.unwrap().is_empty());

    let book = client.order_book(&Fictional::market(), None).await.unwrap();
    assert_eq!(book.spread().unwrap(), Decimal::from(2));

    assert_eq!(
        client
            .trades(&Fictional::market(), None)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(client.balances().await.unwrap().len(), 1);
    assert_eq!(
        client.positions().await.unwrap().len(),
        1,
        "a flat row an outside adapter reported was answered as an open position"
    );
    assert!(
        client
            .funding_payments(&HistoryRequest::new(Fictional::market()))
            .await
            .unwrap()
            .has_more()
    );
}

#[tokio::test]
async fn an_outside_adapter_can_return_a_market_stream_that_yields_events() {
    let client = Client::new(Fictional);
    let subscription = Subscription::new()
        .market(Fictional::market())
        .feed(Feed::Trades);

    let stream = client.subscribe(&subscription).await.unwrap();
    let events: Vec<_> = stream.collect().await;

    assert_eq!(events.len(), 4);
    assert!(matches!(events[0], Ok(MarketEvent::Reconnected)));
    assert!(matches!(events[1], Ok(MarketEvent::Trade(_))));
    // An error item does not terminate the stream.
    assert!(matches!(events[2], Err(Error::Decode { .. })));
    assert!(matches!(events[3], Ok(MarketEvent::Trade(_))));

    // Reject an unsatisfiable subscription instead of returning an empty stream.
    let error = client
        .subscribe(&Subscription::new().feed(Feed::Ticker))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidRequest { field: "feeds", .. }
    ));
}

#[tokio::test]
async fn an_outside_adapter_can_return_an_account_stream_that_yields_events() {
    let config = StreamConfig {
        max_reconnect_attempts: Some(3),
        ..StreamConfig::default()
    };
    let events: Vec<_> = Client::new(Fictional)
        .subscribe_account_with(&config)
        .await
        .unwrap()
        .collect()
        .await;

    assert_eq!(events.len(), 3);
    assert!(matches!(events[0], Ok(AccountEvent::Reconnected)));
    assert!(matches!(events[1], Ok(AccountEvent::Balance(_))));
    assert!(matches!(events[2], Ok(AccountEvent::Order(_))));
}

#[tokio::test]
async fn an_outside_adapter_can_answer_and_refuse_the_order_calls() {
    let client = Client::new(Fictional);

    let placed = client
        .place_order(&OrderRequest::market(
            Fictional::market(),
            Side::Sell,
            Size::Base(Decimal::ONE),
        ))
        .await
        .unwrap();
    assert_eq!(placed.side, Side::Sell);

    let cancelled = client
        .cancel_order(&Fictional::market(), "order-1")
        .await
        .unwrap();
    assert_eq!(cancelled.status, OrderStatus::Cancelled);

    client
        .set_margin(&MarginRequest::new(Fictional::market()).leverage(Decimal::from(3)))
        .await
        .unwrap();

    // External adapters must be able to preserve exchange rejections.
    let refused = client
        .set_margin(&MarginRequest::new(Fictional::market()).leverage(Decimal::from(50)))
        .await
        .unwrap_err();
    assert!(matches!(
        refused,
        Error::Exchange {
            exchange: "fictional",
            kind: ExchangeErrorKind::Rejected,
            ..
        }
    ));
    assert!(!refused.is_retryable());
}

/// An adapter that inherits every optional default.
struct BareMinimum;

impl Adapter for BareMinimum {
    fn exchange(&self) -> Exchange {
        Exchange::Hyperliquid
    }

    fn supports(&self, _feature: Feature) -> bool {
        false
    }
}

#[tokio::test]
async fn an_outside_adapter_that_implements_nothing_optional_inherits_the_defaults() {
    let client = Client::new(BareMinimum);

    let error = client
        .cancel_order(&Fictional::market(), "order-1")
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Unsupported {
            feature: Feature::Trading,
            ..
        }
    ));

    // Empty subscriptions are invalid before feature support is considered.
    assert!(matches!(
        client.subscribe(&Subscription::new()).await.unwrap_err(),
        Error::InvalidRequest {
            field: "markets",
            ..
        }
    ));
    assert!(matches!(
        client
            .subscribe(&Subscription::new().market(Fictional::market()))
            .await
            .unwrap_err(),
        Error::InvalidRequest { field: "feeds", .. }
    ));
    for (feed, expected) in [
        (Feed::Trades, Feature::TradeStream),
        (Feed::OrderBook, Feature::OrderBookStream),
        (Feed::Ticker, Feature::TickerStream),
        (Feed::Candles(maxt::Interval::Min1), Feature::CandleStream),
    ] {
        let error = client
            .subscribe(&Subscription::new().market(Fictional::market()).feed(feed))
            .await
            .unwrap_err();
        assert!(
            matches!(error, Error::Unsupported { feature, .. } if feature == expected),
            "{feed:?} returned {error:?}"
        );
    }
    assert!(matches!(
        client.subscribe_account().await.unwrap_err(),
        Error::Unsupported {
            feature: Feature::AccountStream,
            ..
        }
    ));
}

#[test]
fn an_outside_crate_can_configure_a_stream() {
    let config = StreamConfig {
        idle_timeout_ms: 120_000,
        overflow: Overflow::DropNewest,
        buffer_size: 256,
        max_reconnect_attempts: Some(5),
        ..StreamConfig::default()
    };

    assert_eq!(config.idle_timeout_ms, 120_000);
    assert_eq!(config.max_reconnect_attempts, Some(5));
    // Untouched fields keep the defaults.
    assert_eq!(config.initial_reconnect_delay_ms, 1_000);
}

#[test]
fn every_field_of_a_stream_config_can_be_named_from_outside() {
    let config = StreamConfig {
        max_reconnect_attempts: None,
        initial_reconnect_delay_ms: 250,
        max_reconnect_delay_ms: 10_000,
        idle_timeout_ms: 45_000,
        buffer_size: 1_024,
        overflow: Overflow::Backpressure,
    };

    assert_eq!(config.buffer_size, 1_024);
}

#[test]
fn adapters_from_this_crate_and_from_outside_share_one_type() {
    let mixed: Vec<Box<dyn Adapter>> = vec![Box::new(UpbitAdapter::new()), Box::new(Fictional)];

    assert_eq!(mixed.len(), 2);
    assert!(mixed[0].supports(Feature::Candles));
}
