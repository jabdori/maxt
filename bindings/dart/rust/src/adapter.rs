use std::sync::Arc;
use std::sync::Mutex;

use flutter_rust_bridge::DartFnFuture;
use maxt::{Error, Feature, Feed, Overflow, Result, StreamConfig, Subscription};
use maxt_bindings_common::{
    AdapterCall as CommonAdapterCall, AdapterReply as CommonAdapterReply, ForeignAdapter,
    ForeignDispatcher,
};

use crate::convert::{
    NativeError, WireBalance, WireCandle, WireCandleRequest, WireExchange, WireFeature,
    WireFundingPaymentPage, WireFundingRatePage, WireHistoryRequest, WireMarginRequest,
    WireMarginSummary, WireMarket, WireMarketInfo, WireMarketKind, WireOrder, WireOrderBook,
    WireOrderRequest, WirePosition, WireTicker, WireTrade,
};
use crate::stream::{
    AccountStreamSink, CancelCallback, CancelFuture, MarketStreamSink, account_stream_channel,
    market_stream_channel,
};

/// Dart Adapter가 받을 market feed입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireFeed {
    Trades,
    OrderBook,
    Ticker,
    Candles(crate::convert::WireInterval),
}

/// Dart Adapter가 받을 owned subscription입니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSubscription {
    pub markets: Vec<WireMarket>,
    pub feeds: Vec<WireFeed>,
}

/// Dart Adapter가 받을 overflow 정책입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireOverflow {
    Backpressure,
    DropNewest,
}

/// Dart Adapter가 받을 owned stream 설정입니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStreamConfig {
    pub max_reconnect_attempts: Option<u32>,
    pub initial_reconnect_delay_ms: u64,
    pub max_reconnect_delay_ms: u64,
    pub idle_timeout_ms: u64,
    pub buffer_size: usize,
    pub overflow: WireOverflow,
}

impl From<Subscription> for WireSubscription {
    fn from(value: Subscription) -> Self {
        Self {
            markets: value.markets().iter().cloned().map(Into::into).collect(),
            feeds: value.feeds().iter().copied().map(Into::into).collect(),
        }
    }
}

impl From<Feed> for WireFeed {
    fn from(value: Feed) -> Self {
        match value {
            Feed::Trades => Self::Trades,
            Feed::OrderBook => Self::OrderBook,
            Feed::Ticker => Self::Ticker,
            Feed::Candles(interval) => Self::Candles(interval.into()),
            _ => unreachable!("새 maxt feed에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<StreamConfig> for WireStreamConfig {
    fn from(value: StreamConfig) -> Self {
        Self {
            max_reconnect_attempts: value.max_reconnect_attempts,
            initial_reconnect_delay_ms: value.initial_reconnect_delay_ms,
            max_reconnect_delay_ms: value.max_reconnect_delay_ms,
            idle_timeout_ms: value.idle_timeout_ms,
            buffer_size: value.buffer_size,
            overflow: match value.overflow {
                Overflow::Backpressure => WireOverflow::Backpressure,
                Overflow::DropNewest => WireOverflow::DropNewest,
                _ => unreachable!("새 maxt overflow에는 새 Dart wire variant가 필요합니다"),
            },
        }
    }
}

/// Dart에 전달되는 owned Adapter 호출입니다.
#[derive(Debug)]
#[flutter_rust_bridge::frb(non_opaque)]
pub enum AdapterCall {
    /// 한 종류의 시장 목록을 요청합니다.
    Markets { kind: WireMarketKind },
    /// 최근 체결을 요청합니다.
    Trades {
        market: WireMarket,
        limit: Option<u32>,
    },
    /// 호가 스냅샷을 요청합니다.
    OrderBook {
        market: WireMarket,
        depth: Option<u32>,
    },
    /// ticker 요약을 요청합니다.
    Ticker { market: WireMarket },
    /// 과거 candle을 요청합니다.
    Candles { request: WireCandleRequest },
    /// 계정 잔고를 요청합니다.
    Balances,
    /// 미체결 주문을 요청합니다.
    OpenOrders { market: Option<WireMarket> },
    /// 주문을 제출합니다.
    PlaceOrder { request: WireOrderRequest },
    /// 주문을 취소합니다.
    CancelOrder {
        market: WireMarket,
        order_id: String,
    },
    /// 미결제 포지션을 요청합니다.
    Positions { market: Option<WireMarket> },
    /// 계정 증거금 요약을 요청합니다.
    MarginSummary,
    /// 펀딩 비율 이력을 요청합니다.
    FundingRates { request: WireHistoryRequest },
    /// 펀딩 지급 이력을 요청합니다.
    FundingPayments { request: WireHistoryRequest },
    /// 레버리지 또는 증거금 모드를 설정합니다.
    SetMargin { request: WireMarginRequest },
    /// Dart market subscription을 시작합니다.
    Subscribe {
        stream_id: String,
        subscription: WireSubscription,
        config: WireStreamConfig,
        sink: MarketStreamSink,
    },
    /// Dart account subscription을 시작합니다.
    SubscribeAccount {
        stream_id: String,
        config: WireStreamConfig,
        sink: AccountStreamSink,
    },
    /// Rust consumer가 먼저 닫은 Dart subscription을 취소합니다.
    CancelStream { stream_id: String },
}

/// Dart dispatcher가 반환하는 Adapter 응답입니다.
#[derive(Debug)]
#[flutter_rust_bridge::frb(non_opaque)]
pub enum AdapterReply {
    /// 시장 목록 응답입니다.
    Markets(Vec<WireMarketInfo>),
    /// 최근 체결 응답입니다.
    Trades(Vec<WireTrade>),
    /// 호가 스냅샷 응답입니다.
    OrderBook(WireOrderBook),
    /// ticker 응답입니다.
    Ticker(WireTicker),
    /// candle 응답입니다.
    Candles(Vec<WireCandle>),
    /// 잔고 응답입니다.
    Balances(Vec<WireBalance>),
    /// 미체결 주문 응답입니다.
    OpenOrders(Vec<WireOrder>),
    /// 주문 제출 응답입니다.
    PlaceOrder(WireOrder),
    /// 주문 취소 응답입니다.
    CancelOrder(WireOrder),
    /// 포지션 응답입니다.
    Positions(Vec<WirePosition>),
    /// 증거금 요약 응답입니다.
    MarginSummary(WireMarginSummary),
    /// 펀딩 비율 페이지 응답입니다.
    FundingRates(WireFundingRatePage),
    /// 펀딩 지급 페이지 응답입니다.
    FundingPayments(WireFundingPaymentPage),
    /// 값이 없는 성공 응답입니다.
    Unit,
}

impl AdapterReply {
    fn kind(&self) -> &'static str {
        match self {
            Self::Markets(_) => "Markets",
            Self::Trades(_) => "Trades",
            Self::OrderBook(_) => "OrderBook",
            Self::Ticker(_) => "Ticker",
            Self::Candles(_) => "Candles",
            Self::Balances(_) => "Balances",
            Self::OpenOrders(_) => "OpenOrders",
            Self::PlaceOrder(_) => "PlaceOrder",
            Self::CancelOrder(_) => "CancelOrder",
            Self::Positions(_) => "Positions",
            Self::MarginSummary(_) => "MarginSummary",
            Self::FundingRates(_) => "FundingRates",
            Self::FundingPayments(_) => "FundingPayments",
            Self::Unit => "Unit",
        }
    }
}

/// Dart dispatcher의 구조화된 성공 또는 오류입니다.
#[derive(Debug)]
#[flutter_rust_bridge::frb(non_opaque)]
pub enum AdapterResult {
    /// 정상 Adapter 응답입니다.
    Success(AdapterReply),
    /// `maxt::Error` 의미를 보존하는 오류입니다.
    Error(NativeError),
}

type DartCallback =
    dyn Fn(AdapterCall) -> DartFnFuture<anyhow::Result<AdapterResult>> + Send + Sync;

struct DartDispatcher {
    callback: Arc<DartCallback>,
    next_stream_id: Mutex<Option<u64>>,
}

fn allocate_stream_id(next: &Mutex<Option<u64>>) -> Result<String> {
    let mut next = next.lock().unwrap();
    let stream_id = next
        .ok_or_else(|| Error::adapter("Dart adapter stream identifier space has been exhausted"))?;
    *next = stream_id.checked_add(1);
    Ok(stream_id.to_string())
}

impl DartDispatcher {
    fn next_stream_id(&self) -> Result<String> {
        allocate_stream_id(&self.next_stream_id)
    }

    fn cancellation(&self) -> CancelCallback {
        let callback = self.callback.clone();
        Arc::new(move |stream_id| -> CancelFuture {
            let future = callback(AdapterCall::CancelStream {
                stream_id: stream_id.clone(),
            });
            Box::pin(async move { wait_for_unit(future).await })
        })
    }

    fn subscribe_market(
        &self,
        subscription: Subscription,
        config: StreamConfig,
    ) -> maxt::BoxFuture<'static, Result<CommonAdapterReply>> {
        let stream_id = match self.next_stream_id() {
            Ok(stream_id) => stream_id,
            Err(error) => return Box::pin(async move { Err(error) }),
        };
        let (sink, stream) = market_stream_channel(stream_id.clone(), &config, self.cancellation());
        let future = (self.callback)(AdapterCall::Subscribe {
            stream_id,
            subscription: subscription.into(),
            config: config.into(),
            sink,
        });
        Box::pin(async move {
            wait_for_unit(future).await?;
            Ok(CommonAdapterReply::MarketStream(stream))
        })
    }

    fn subscribe_account(
        &self,
        config: StreamConfig,
    ) -> maxt::BoxFuture<'static, Result<CommonAdapterReply>> {
        let stream_id = match self.next_stream_id() {
            Ok(stream_id) => stream_id,
            Err(error) => return Box::pin(async move { Err(error) }),
        };
        let (sink, stream) =
            account_stream_channel(stream_id.clone(), &config, self.cancellation());
        let future = (self.callback)(AdapterCall::SubscribeAccount {
            stream_id,
            config: config.into(),
            sink,
        });
        Box::pin(async move {
            wait_for_unit(future).await?;
            Ok(CommonAdapterReply::AccountStream(stream))
        })
    }
}

async fn wait_for_unit(future: DartFnFuture<anyhow::Result<AdapterResult>>) -> Result<()> {
    match future.await {
        Ok(AdapterResult::Success(AdapterReply::Unit)) => Ok(()),
        Ok(AdapterResult::Success(reply)) => Err(Error::adapter(format!(
            "Dart dispatcher returned {} where Unit was required",
            reply.kind(),
        ))),
        Ok(AdapterResult::Error(error)) => Err(structured_error(error)),
        Err(error) => Err(Error::adapter(format!(
            "Dart adapter dispatcher failed: {error:#}"
        ))),
    }
}

#[derive(Clone, Copy)]
enum ExpectedReply {
    Markets,
    Trades,
    OrderBook,
    Ticker,
    Candles,
    Balances,
    OpenOrders,
    PlaceOrder,
    CancelOrder,
    Positions,
    MarginSummary,
    FundingRates,
    FundingPayments,
    Unit,
}

impl ExpectedReply {
    const fn kind(self) -> &'static str {
        match self {
            Self::Markets => "Markets",
            Self::Trades => "Trades",
            Self::OrderBook => "OrderBook",
            Self::Ticker => "Ticker",
            Self::Candles => "Candles",
            Self::Balances => "Balances",
            Self::OpenOrders => "OpenOrders",
            Self::PlaceOrder => "PlaceOrder",
            Self::CancelOrder => "CancelOrder",
            Self::Positions => "Positions",
            Self::MarginSummary => "MarginSummary",
            Self::FundingRates => "FundingRates",
            Self::FundingPayments => "FundingPayments",
            Self::Unit => "Unit",
        }
    }
}

impl ForeignDispatcher for DartDispatcher {
    fn dispatch(&self, call: CommonAdapterCall) -> maxt::BoxFuture<'_, Result<CommonAdapterReply>> {
        let call = match call {
            CommonAdapterCall::Subscribe {
                subscription,
                config,
            } => return self.subscribe_market(subscription, config),
            CommonAdapterCall::SubscribeAccount { config } => {
                return self.subscribe_account(config);
            }
            call => call,
        };
        let (call, expected) = match call {
            CommonAdapterCall::Markets { kind } => (
                AdapterCall::Markets { kind: kind.into() },
                ExpectedReply::Markets,
            ),
            CommonAdapterCall::Trades { market, limit } => (
                AdapterCall::Trades {
                    market: market.into(),
                    limit,
                },
                ExpectedReply::Trades,
            ),
            CommonAdapterCall::OrderBook { market, depth } => (
                AdapterCall::OrderBook {
                    market: market.into(),
                    depth,
                },
                ExpectedReply::OrderBook,
            ),
            CommonAdapterCall::Ticker { market } => (
                AdapterCall::Ticker {
                    market: market.into(),
                },
                ExpectedReply::Ticker,
            ),
            CommonAdapterCall::Candles { request } => (
                AdapterCall::Candles {
                    request: request.into(),
                },
                ExpectedReply::Candles,
            ),
            CommonAdapterCall::Balances => (AdapterCall::Balances, ExpectedReply::Balances),
            CommonAdapterCall::OpenOrders { market } => (
                AdapterCall::OpenOrders {
                    market: market.map(Into::into),
                },
                ExpectedReply::OpenOrders,
            ),
            CommonAdapterCall::PlaceOrder { request } => (
                AdapterCall::PlaceOrder {
                    request: request.into(),
                },
                ExpectedReply::PlaceOrder,
            ),
            CommonAdapterCall::CancelOrder { market, order_id } => (
                AdapterCall::CancelOrder {
                    market: market.into(),
                    order_id,
                },
                ExpectedReply::CancelOrder,
            ),
            CommonAdapterCall::Positions { market } => (
                AdapterCall::Positions {
                    market: market.map(Into::into),
                },
                ExpectedReply::Positions,
            ),
            CommonAdapterCall::MarginSummary => {
                (AdapterCall::MarginSummary, ExpectedReply::MarginSummary)
            }
            CommonAdapterCall::FundingRates { request } => (
                AdapterCall::FundingRates {
                    request: request.into(),
                },
                ExpectedReply::FundingRates,
            ),
            CommonAdapterCall::FundingPayments { request } => (
                AdapterCall::FundingPayments {
                    request: request.into(),
                },
                ExpectedReply::FundingPayments,
            ),
            CommonAdapterCall::SetMargin { request } => (
                AdapterCall::SetMargin {
                    request: request.into(),
                },
                ExpectedReply::Unit,
            ),
            _ => {
                return Box::pin(async {
                    Err(Error::adapter(
                        "Dart dispatcher forwarding is not implemented for this Adapter call",
                    ))
                });
            }
        };
        let future = (self.callback)(call);

        Box::pin(async move {
            match future.await {
                Ok(AdapterResult::Success(reply)) => reply.into_common(expected),
                Ok(AdapterResult::Error(error)) => Err(structured_error(error)),
                Err(error) => Err(Error::adapter(format!(
                    "Dart adapter dispatcher failed: {error:#}"
                ))),
            }
        })
    }
}

impl AdapterReply {
    fn into_common(self, expected: ExpectedReply) -> Result<CommonAdapterReply> {
        match (expected, self) {
            (ExpectedReply::Markets, Self::Markets(values)) => {
                convert_vec(values, "Markets").map(CommonAdapterReply::Markets)
            }
            (ExpectedReply::Trades, Self::Trades(values)) => {
                convert_vec(values, "Trades").map(CommonAdapterReply::Trades)
            }
            (ExpectedReply::OrderBook, Self::OrderBook(value)) => value
                .try_into()
                .map(CommonAdapterReply::OrderBook)
                .map_err(|error| invalid_reply("OrderBook", error)),
            (ExpectedReply::Ticker, Self::Ticker(value)) => value
                .try_into()
                .map(CommonAdapterReply::Ticker)
                .map_err(|error| invalid_reply("Ticker", error)),
            (ExpectedReply::Candles, Self::Candles(values)) => {
                convert_vec(values, "Candles").map(CommonAdapterReply::Candles)
            }
            (ExpectedReply::Balances, Self::Balances(values)) => {
                convert_vec(values, "Balances").map(CommonAdapterReply::Balances)
            }
            (ExpectedReply::OpenOrders, Self::OpenOrders(values)) => {
                convert_vec(values, "OpenOrders").map(CommonAdapterReply::OpenOrders)
            }
            (ExpectedReply::PlaceOrder, Self::PlaceOrder(value)) => value
                .try_into()
                .map(CommonAdapterReply::PlaceOrder)
                .map_err(|error| invalid_reply("PlaceOrder", error)),
            (ExpectedReply::CancelOrder, Self::CancelOrder(value)) => value
                .try_into()
                .map(CommonAdapterReply::CancelOrder)
                .map_err(|error| invalid_reply("CancelOrder", error)),
            (ExpectedReply::Positions, Self::Positions(values)) => {
                convert_vec(values, "Positions").map(CommonAdapterReply::Positions)
            }
            (ExpectedReply::MarginSummary, Self::MarginSummary(value)) => value
                .try_into()
                .map(CommonAdapterReply::MarginSummary)
                .map_err(|error| invalid_reply("MarginSummary", error)),
            (ExpectedReply::FundingRates, Self::FundingRates(value)) => value
                .try_into()
                .map(CommonAdapterReply::FundingRates)
                .map_err(|error| invalid_reply("FundingRates", error)),
            (ExpectedReply::FundingPayments, Self::FundingPayments(value)) => value
                .try_into()
                .map(CommonAdapterReply::FundingPayments)
                .map_err(|error| invalid_reply("FundingPayments", error)),
            (ExpectedReply::Unit, Self::Unit) => Ok(CommonAdapterReply::Unit),
            (expected, reply) => Err(Error::adapter(format!(
                "Dart dispatcher returned {} where {} was required",
                reply.kind(),
                expected.kind(),
            ))),
        }
    }
}

fn convert_vec<W, T>(values: Vec<W>, kind: &str) -> Result<Vec<T>>
where
    T: TryFrom<W, Error = NativeError>,
{
    values
        .into_iter()
        .map(TryInto::try_into)
        .collect::<std::result::Result<_, _>>()
        .map_err(|error| invalid_reply(kind, error))
}

fn invalid_reply(kind: &str, error: NativeError) -> Error {
    Error::adapter(format!("Dart adapter returned invalid {kind}: {error}"))
}

fn structured_error(error: NativeError) -> Error {
    Error::try_from(error).unwrap_or_else(|error| error)
}

/// Dart 구현을 Rust `Adapter`로 등록합니다.
pub fn register_dart_adapter(
    exchange: WireExchange,
    features: Vec<WireFeature>,
    dispatcher: impl Fn(AdapterCall) -> DartFnFuture<anyhow::Result<AdapterResult>>
    + Send
    + Sync
    + 'static,
) -> DartAdapter {
    let dispatcher = Arc::new(DartDispatcher {
        callback: Arc::new(dispatcher),
        next_stream_id: Mutex::new(Some(1)),
    });
    DartAdapter {
        inner: registered_adapter(exchange, features, dispatcher),
    }
}

/// Dart 구현을 소유하는 Rust Adapter handle입니다.
#[flutter_rust_bridge::frb(opaque)]
pub struct DartAdapter {
    inner: ForeignAdapter,
}

impl DartAdapter {
    pub(crate) fn into_adapter(self) -> Arc<dyn maxt::Adapter> {
        Arc::new(self.inner)
    }
}

fn registered_adapter(
    exchange: WireExchange,
    features: Vec<WireFeature>,
    dispatcher: Arc<dyn ForeignDispatcher>,
) -> ForeignAdapter {
    ForeignAdapter::new(
        exchange.into(),
        features.into_iter().map(Feature::from),
        dispatcher,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use futures_util::StreamExt;
    use maxt::{Adapter, BoxFuture, Error, Exchange, Feature, Result};
    use maxt_bindings_common::{
        AdapterCall as CommonAdapterCall, AdapterReply as CommonAdapterReply, ForeignDispatcher,
    };

    use crate::convert::{
        NativeError, WireExchange, WireFeature, WireFundingPaymentPage, WireFundingRatePage,
        WireInterval, WireMarginSummary, WireOrder, WireOrderBook, WireOrderStatus, WireTicker,
    };
    use crate::stream::NativeMarketSubscription;

    use super::{
        AdapterCall, AdapterReply, AdapterResult, allocate_stream_id, register_dart_adapter,
        registered_adapter,
    };

    #[test]
    fn stream_identifier_exhaustion_never_wraps_or_reuses_zero() {
        let next = Mutex::new(Some(u64::MAX));

        assert_eq!(allocate_stream_id(&next).unwrap(), u64::MAX.to_string());
        assert!(matches!(
            allocate_stream_id(&next),
            Err(Error::Adapter { detail }) if detail.contains("identifier space has been exhausted"),
        ));
    }

    struct UnusedDispatcher;

    impl ForeignDispatcher for UnusedDispatcher {
        fn dispatch(
            &self,
            _call: CommonAdapterCall,
        ) -> BoxFuture<'static, Result<CommonAdapterReply>> {
            Box::pin(async { Err(Error::adapter("registration test does not dispatch")) })
        }
    }

    #[test]
    fn registration_caches_validated_exchange_and_unique_features() {
        let adapter = registered_adapter(
            WireExchange::Binance,
            vec![
                WireFeature::Trades,
                WireFeature::Trades,
                WireFeature::TradeStream,
            ],
            Arc::new(UnusedDispatcher),
        );

        assert_eq!(adapter.exchange(), Exchange::Binance);
        assert!(adapter.supports(Feature::Trades));
        assert!(adapter.supports(Feature::TradeStream));
        assert!(!adapter.supports(Feature::Trading));
        assert_eq!(adapter.features().len(), 2);
    }

    #[tokio::test]
    async fn one_async_dispatcher_receives_an_owned_call_and_forwards_its_reply() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let observed = calls.clone();
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::Markets],
            move |call| {
                let observed = observed.clone();
                Box::pin(async move {
                    observed.lock().unwrap().push(call);
                    Ok(AdapterResult::Success(AdapterReply::Markets(Vec::new())))
                })
            },
        );

        let markets = adapter
            .inner
            .markets(maxt::MarketKind::Perpetual)
            .await
            .unwrap();

        assert!(markets.is_empty());
        assert!(matches!(
            calls.lock().unwrap().as_slice(),
            [AdapterCall::Markets {
                kind: crate::convert::WireMarketKind::Perpetual,
            }]
        ));
    }

    #[tokio::test]
    async fn public_rest_methods_all_forward_through_the_same_dispatcher() {
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![
                WireFeature::Trades,
                WireFeature::OrderBook,
                WireFeature::Ticker,
                WireFeature::Candles,
            ],
            |call| {
                Box::pin(async move {
                    let reply = match call {
                        AdapterCall::Trades { market, limit } => {
                            assert_eq!(market.base, "BTC");
                            assert_eq!(limit, Some(7));
                            AdapterReply::Trades(Vec::new())
                        }
                        AdapterCall::OrderBook { market, depth } => {
                            assert_eq!(depth, Some(20));
                            AdapterReply::OrderBook(WireOrderBook {
                                market,
                                timestamp_ns: 42,
                                bids: Vec::new(),
                                asks: Vec::new(),
                            })
                        }
                        AdapterCall::Ticker { market } => AdapterReply::Ticker(WireTicker {
                            market,
                            timestamp_ns: 43,
                            last_trade_time_ns: None,
                            last_price: "100.25".to_owned(),
                            change: None,
                            change_rate: None,
                            high: None,
                            low: None,
                            volume: None,
                            quote_volume: None,
                        }),
                        AdapterCall::Candles { request } => {
                            assert_eq!(request.interval, WireInterval::Min1);
                            assert_eq!(request.limit, Some(3));
                            AdapterReply::Candles(Vec::new())
                        }
                        other => panic!("unexpected call: {other:?}"),
                    };
                    Ok(AdapterResult::Success(reply))
                })
            },
        );
        let market = maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT");

        assert!(
            adapter
                .inner
                .trades(&market, Some(7))
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            adapter
                .inner
                .order_book(&market, Some(20))
                .await
                .unwrap()
                .timestamp,
            maxt::Timestamp::from_nanos(42),
        );
        assert_eq!(
            adapter.inner.ticker(&market).await.unwrap().last_price,
            maxt::Decimal::new(10_025, 2),
        );
        assert!(
            adapter
                .inner
                .candles(&maxt::CandleRequest::new(market, maxt::Interval::Min1).limit(3))
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn account_and_order_methods_all_forward_through_the_same_dispatcher() {
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![
                WireFeature::Balances,
                WireFeature::OpenOrders,
                WireFeature::Trading,
            ],
            |call| {
                Box::pin(async move {
                    let reply = match call {
                        AdapterCall::Balances => AdapterReply::Balances(Vec::new()),
                        AdapterCall::OpenOrders { market } => {
                            assert_eq!(market.unwrap().base, "BTC");
                            AdapterReply::OpenOrders(Vec::new())
                        }
                        AdapterCall::PlaceOrder { request } => {
                            assert_eq!(request.size.value, "1");
                            AdapterReply::PlaceOrder(wire_order("placed", request.market))
                        }
                        AdapterCall::CancelOrder { market, order_id } => {
                            assert_eq!(order_id, "order-1");
                            AdapterReply::CancelOrder(wire_order("cancelled", market))
                        }
                        other => panic!("unexpected call: {other:?}"),
                    };
                    Ok(AdapterResult::Success(reply))
                })
            },
        );
        let market = maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT");
        let request = maxt::OrderRequest::market(
            market.clone(),
            maxt::Side::Buy,
            maxt::Size::Base(maxt::Decimal::ONE),
        );

        assert!(adapter.inner.balances().await.unwrap().is_empty());
        assert!(
            adapter
                .inner
                .open_orders(Some(&market))
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            adapter.inner.place_order(&request).await.unwrap().id,
            "placed",
        );
        assert_eq!(
            adapter
                .inner
                .cancel_order(&market, "order-1")
                .await
                .unwrap()
                .id,
            "cancelled",
        );
    }

    fn wire_order(id: &str, market: crate::convert::WireMarket) -> WireOrder {
        WireOrder {
            id: id.to_owned(),
            market,
            side: crate::convert::WireSide::Buy,
            status: WireOrderStatus::Open,
            filled_quantity: "0".to_owned(),
            remaining_quantity: "1".to_owned(),
            price: None,
            created_at_ns: None,
        }
    }

    #[tokio::test]
    async fn derivatives_methods_all_forward_through_the_same_dispatcher() {
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![
                WireFeature::Positions,
                WireFeature::Margin,
                WireFeature::FundingRates,
                WireFeature::FundingPayments,
                WireFeature::MarginConfig,
            ],
            |call| {
                Box::pin(async move {
                    let reply = match call {
                        AdapterCall::Positions { market } => {
                            assert_eq!(market.unwrap().quote, "USDT");
                            AdapterReply::Positions(Vec::new())
                        }
                        AdapterCall::MarginSummary => {
                            AdapterReply::MarginSummary(WireMarginSummary {
                                asset: "USDT".to_owned(),
                                equity: Some("10".to_owned()),
                                margin_balance: None,
                                available_balance: None,
                            })
                        }
                        AdapterCall::FundingRates { request } => {
                            assert_eq!(request.limit, Some(5));
                            AdapterReply::FundingRates(WireFundingRatePage {
                                items: Vec::new(),
                                next: Some("rates-next".to_owned()),
                            })
                        }
                        AdapterCall::FundingPayments { request } => {
                            assert_eq!(request.limit, Some(5));
                            AdapterReply::FundingPayments(WireFundingPaymentPage {
                                items: Vec::new(),
                                next: None,
                            })
                        }
                        AdapterCall::SetMargin { request } => {
                            assert_eq!(request.leverage.as_deref(), Some("3"));
                            AdapterReply::Unit
                        }
                        other => panic!("unexpected call: {other:?}"),
                    };
                    Ok(AdapterResult::Success(reply))
                })
            },
        );
        let market = maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT");
        let history = maxt::HistoryRequest::new(market.clone()).limit(5);

        assert!(
            adapter
                .inner
                .positions(Some(&market))
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            adapter.inner.margin_summary().await.unwrap().equity,
            Some(maxt::Decimal::TEN),
        );
        assert!(
            adapter
                .inner
                .funding_rates(&history)
                .await
                .unwrap()
                .has_more()
        );
        assert!(
            !adapter
                .inner
                .funding_payments(&history)
                .await
                .unwrap()
                .has_more()
        );
        adapter
            .inner
            .set_margin(&maxt::MarginRequest::new(market).leverage(maxt::Decimal::from(3)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn market_subscription_uses_a_rust_owned_sink_and_preserves_error_items() {
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::TradeStream],
            |call| {
                Box::pin(async move {
                    match call {
                        AdapterCall::Subscribe {
                            stream_id,
                            subscription,
                            sink,
                            ..
                        } => {
                            assert_eq!(stream_id, "1");
                            assert_eq!(subscription.markets.len(), 1);
                            assert!(
                                sink.add(crate::stream::MarketStreamItem::Event(
                                    crate::stream::WireMarketEvent::Reconnected,
                                ))
                                .await
                            );
                            assert!(
                                sink.add(crate::stream::MarketStreamItem::Error(
                                    NativeError::from(Error::Decode {
                                        detail: "bad Dart frame".to_owned(),
                                    }),
                                ))
                                .await
                            );
                            assert!(sink.add(crate::stream::MarketStreamItem::End).await);
                            Ok(AdapterResult::Success(AdapterReply::Unit))
                        }
                        other => panic!("unexpected call: {other:?}"),
                    }
                })
            },
        );
        let subscription = maxt::Subscription::new()
            .market(maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(maxt::Feed::Trades);
        let mut stream = adapter
            .inner
            .subscribe(&subscription, &maxt::StreamConfig::default())
            .await
            .unwrap();

        assert!(matches!(
            stream.next().await,
            Some(Ok(maxt::MarketEvent::Reconnected)),
        ));
        assert!(matches!(
            stream.next().await,
            Some(Err(Error::Decode { detail })) if detail == "bad Dart frame",
        ));
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn account_subscription_forwards_through_its_rust_owned_sink() {
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::AccountStream],
            |call| {
                Box::pin(async move {
                    match call {
                        AdapterCall::SubscribeAccount {
                            stream_id,
                            config,
                            sink,
                        } => {
                            assert_eq!(stream_id, "1");
                            assert_eq!(config.buffer_size, 32);
                            assert!(
                                sink.add(crate::stream::AccountStreamItem::Event(
                                    crate::stream::WireAccountEvent::Reconnected,
                                ))
                                .await
                            );
                            assert!(sink.add(crate::stream::AccountStreamItem::End).await);
                            Ok(AdapterResult::Success(AdapterReply::Unit))
                        }
                        other => panic!("unexpected call: {other:?}"),
                    }
                })
            },
        );
        let config = maxt::StreamConfig {
            buffer_size: 32,
            ..maxt::StreamConfig::default()
        };
        let mut stream = adapter.inner.subscribe_account(&config).await.unwrap();

        assert!(matches!(
            stream.next().await,
            Some(Ok(maxt::AccountEvent::Reconnected)),
        ));
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn dropping_a_rust_adapter_stream_dispatches_one_dart_cancellation() {
        use std::sync::mpsc;
        use std::time::Duration;

        let retained_sink = Arc::new(Mutex::new(None));
        let retained_by_dispatcher = retained_sink.clone();
        let (cancelled_tx, cancelled_rx) = mpsc::channel();
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::TradeStream],
            move |call| {
                let retained_by_dispatcher = retained_by_dispatcher.clone();
                let cancelled_tx = cancelled_tx.clone();
                Box::pin(async move {
                    match call {
                        AdapterCall::Subscribe { sink, .. } => {
                            *retained_by_dispatcher.lock().unwrap() = Some(sink);
                        }
                        AdapterCall::CancelStream { stream_id } => {
                            retained_by_dispatcher.lock().unwrap().take();
                            cancelled_tx.send(stream_id).unwrap();
                        }
                        other => panic!("unexpected call: {other:?}"),
                    }
                    Ok(AdapterResult::Success(AdapterReply::Unit))
                })
            },
        );
        let subscription = maxt::Subscription::new()
            .market(maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(maxt::Feed::Trades);
        let stream = adapter
            .inner
            .subscribe(&subscription, &maxt::StreamConfig::default())
            .await
            .unwrap();

        drop(stream);

        assert_eq!(
            cancelled_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            "1",
        );
        assert!(retained_sink.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn dropping_a_pending_subscribe_dispatches_cancellation_immediately() {
        use std::time::Duration;

        use futures_util::future::{Either, select};

        let (started_tx, started_rx) = futures_channel::oneshot::channel();
        let started_tx = Arc::new(Mutex::new(Some(started_tx)));
        let (_finish_tx, finish_rx) = futures_channel::oneshot::channel::<()>();
        let finish_rx = Arc::new(Mutex::new(Some(finish_rx)));
        let (cancelled_tx, cancelled_rx) = futures_channel::oneshot::channel();
        let cancelled_tx = Arc::new(Mutex::new(Some(cancelled_tx)));
        let retained_sink = Arc::new(Mutex::new(None));
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::TradeStream],
            move |call| {
                let started_tx = started_tx.clone();
                let finish_rx = finish_rx.clone();
                let cancelled_tx = cancelled_tx.clone();
                let retained_sink = retained_sink.clone();
                Box::pin(async move {
                    match call {
                        AdapterCall::Subscribe { sink, .. } => {
                            *retained_sink.lock().unwrap() = Some(sink);
                            started_tx.lock().unwrap().take().unwrap().send(()).unwrap();
                            let receiver = finish_rx.lock().unwrap().take().unwrap();
                            let _ = receiver.await;
                        }
                        AdapterCall::CancelStream { stream_id } => {
                            cancelled_tx
                                .lock()
                                .unwrap()
                                .take()
                                .unwrap()
                                .send(stream_id)
                                .unwrap();
                            retained_sink.lock().unwrap().take();
                        }
                        other => panic!("unexpected call: {other:?}"),
                    }
                    Ok(AdapterResult::Success(AdapterReply::Unit))
                })
            },
        )
        .into_adapter();
        let subscription = maxt::Subscription::new()
            .market(maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(maxt::Feed::Trades);
        let pending = tokio::spawn(async move {
            adapter
                .subscribe(&subscription, &maxt::StreamConfig::default())
                .await
        });
        started_rx.await.unwrap();

        pending.abort();
        let _ = pending.await;

        let (timeout_tx, timeout_rx) = futures_channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));
            let _ = timeout_tx.send(());
        });
        match select(Box::pin(cancelled_rx), Box::pin(timeout_rx)).await {
            Either::Left((Ok(stream_id), _)) => assert_eq!(stream_id, "1"),
            Either::Left((Err(error), _)) => panic!("cancellation channel failed: {error}"),
            Either::Right(_) => panic!("CancelStream was not dispatched within one second"),
        }
    }

    #[tokio::test]
    async fn failed_subscription_ends_its_sink_without_dispatching_late_cancellation() {
        let cancellations = Arc::new(Mutex::new(0_u32));
        let observed = cancellations.clone();
        let adapter = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::TradeStream],
            move |call| {
                let observed = observed.clone();
                Box::pin(async move {
                    match call {
                        AdapterCall::Subscribe { sink, .. } => {
                            assert!(sink.add(crate::stream::MarketStreamItem::End).await);
                            Ok(AdapterResult::Error(NativeError::from(Error::Transport {
                                detail: "registration failed".to_owned(),
                            })))
                        }
                        AdapterCall::CancelStream { .. } => {
                            *observed.lock().unwrap() += 1;
                            Ok(AdapterResult::Success(AdapterReply::Unit))
                        }
                        other => panic!("unexpected call: {other:?}"),
                    }
                })
            },
        );
        let subscription = maxt::Subscription::new()
            .market(maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(maxt::Feed::Trades);

        let error = adapter
            .inner
            .subscribe(&subscription, &maxt::StreamConfig::default())
            .await
            .unwrap_err();
        for _ in 0..32 {
            tokio::task::yield_now().await;
        }

        assert!(matches!(
            error,
            Error::Transport { detail } if detail == "registration failed",
        ));
        assert_eq!(*cancellations.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn native_close_waits_for_dart_cancel_stream_ack() {
        let retained_sink = Arc::new(Mutex::new(None));
        let (started_tx, started_rx) = futures_channel::oneshot::channel();
        let started_tx = Arc::new(Mutex::new(Some(started_tx)));
        let (release_tx, release_rx) = futures_channel::oneshot::channel();
        let release_rx = Arc::new(Mutex::new(Some(release_rx)));
        let adapter =
            register_dart_adapter(WireExchange::Binance, vec![WireFeature::TradeStream], {
                let retained_sink = retained_sink.clone();
                move |call| {
                    let retained_sink = retained_sink.clone();
                    let started_tx = started_tx.clone();
                    let release_rx = release_rx.clone();
                    Box::pin(async move {
                        match call {
                            AdapterCall::Subscribe { sink, .. } => {
                                *retained_sink.lock().unwrap() = Some(sink);
                            }
                            AdapterCall::CancelStream { .. } => {
                                started_tx.lock().unwrap().take().unwrap().send(()).unwrap();
                                let receiver = release_rx.lock().unwrap().take().unwrap();
                                let _ = receiver.await;
                                retained_sink.lock().unwrap().take();
                            }
                            other => panic!("unexpected call: {other:?}"),
                        }
                        Ok(AdapterResult::Success(AdapterReply::Unit))
                    })
                }
            });
        let request = maxt::Subscription::new()
            .market(maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(maxt::Feed::Trades);
        let stream = adapter
            .inner
            .subscribe(&request, &maxt::StreamConfig::default())
            .await
            .unwrap();
        let subscription = Arc::new(NativeMarketSubscription::new(stream));
        let closing = tokio::spawn({
            let subscription = subscription.clone();
            async move { subscription.close().await }
        });
        started_rx.await.unwrap();
        tokio::task::yield_now().await;

        assert!(!closing.is_finished());
        assert!(retained_sink.lock().unwrap().is_some());

        release_tx.send(()).unwrap();
        closing.await.unwrap().unwrap();
        assert!(retained_sink.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn dart_cancel_stream_error_is_returned_by_native_close() {
        let retained_sink = Arc::new(Mutex::new(None));
        let adapter =
            register_dart_adapter(WireExchange::Binance, vec![WireFeature::TradeStream], {
                let retained_sink = retained_sink.clone();
                move |call| {
                    let retained_sink = retained_sink.clone();
                    Box::pin(async move {
                        match call {
                            AdapterCall::Subscribe { sink, .. } => {
                                *retained_sink.lock().unwrap() = Some(sink);
                                Ok(AdapterResult::Success(AdapterReply::Unit))
                            }
                            AdapterCall::CancelStream { .. } => {
                                retained_sink.lock().unwrap().take();
                                Ok(AdapterResult::Error(NativeError::from(Error::Transport {
                                    detail: "Dart close failed".to_owned(),
                                })))
                            }
                            other => panic!("unexpected call: {other:?}"),
                        }
                    })
                }
            });
        let request = maxt::Subscription::new()
            .market(maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(maxt::Feed::Trades);
        let stream = adapter
            .inner
            .subscribe(&request, &maxt::StreamConfig::default())
            .await
            .unwrap();
        let subscription = NativeMarketSubscription::new(stream);

        let error = subscription.close().await.unwrap_err();

        assert_eq!(error.kind, crate::convert::NativeErrorKind::Transport);
        assert_eq!(error.detail.as_deref(), Some("Dart close failed"));
    }

    #[tokio::test]
    async fn callback_failures_and_structured_errors_keep_distinct_meanings() {
        let failed =
            register_dart_adapter(WireExchange::Binance, vec![WireFeature::Markets], |_| {
                Box::pin(async { Err(anyhow::anyhow!("callback broke")) })
            })
            .inner
            .markets(maxt::MarketKind::Spot)
            .await
            .unwrap_err();
        assert!(matches!(
            failed,
            Error::Adapter { ref detail } if detail.contains("callback broke"),
        ));

        let original = Error::Exchange {
            exchange: "binance",
            code: "-1003".to_owned(),
            message: "too many requests".to_owned(),
            status: Some(429),
            kind: maxt::ExchangeErrorKind::RateLimited,
        };
        let wire = NativeError::from(original.clone());
        let structured = register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::Markets],
            move |_| {
                let wire = wire.clone();
                Box::pin(async move { Ok(AdapterResult::Error(wire)) })
            },
        )
        .inner
        .markets(maxt::MarketKind::Spot)
        .await
        .unwrap_err();

        assert_eq!(structured, original);
    }

    #[tokio::test]
    async fn a_mismatched_reply_is_an_adapter_contract_error() {
        let error =
            register_dart_adapter(WireExchange::Binance, vec![WireFeature::Markets], |_| {
                Box::pin(async { Ok(AdapterResult::Success(AdapterReply::Trades(Vec::new()))) })
            })
            .inner
            .markets(maxt::MarketKind::Spot)
            .await
            .unwrap_err();

        assert!(matches!(error, Error::Adapter { .. }));
        assert!(
            error
                .to_string()
                .contains("Trades where Markets was required")
        );
    }
}
