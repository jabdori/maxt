use std::sync::Arc;

use flutter_rust_bridge::DartFnFuture;
use maxt::adapters::{
    BinanceAdapter, BinanceListenKey, BinanceMarket, BithumbAdapter, HyperliquidAdapter,
    UpbitAdapter, UpbitRegion,
};
use maxt::{
    AccountStream, Adapter, AssetNetwork, Balance, BoxFuture, CancelOrdersRequest,
    CancelOrdersResult, Candle, CandleRequest, Deposit, DepositAddress, DepositAddressEntry,
    DepositAddressRequest, Error, Exchange, Feature, Feed, FundingPayment, FundingRate,
    HistoryRequest, MarginRequest, MarginSummary, Market, MarketInfo, MarketKind, MarketStream,
    Order, OrderBook, OrderHistoryRequest, OrderLookupRequest, OrderRequest, OrderRules, Overflow,
    Page, Position, StreamConfig, Subscription, Ticker, Trade, TransferHistoryRequest,
    TransferLookupRequest, WithdrawRequest, Withdrawal, WithdrawalQuote,
};

mod generated_native_client;

pub use crate::adapter::{
    AdapterCall, AdapterReply, AdapterResult, DartAdapter, WireFeed, WireOverflow,
    WireStreamConfig, WireSubscription,
};
pub use crate::convert::*;
pub use crate::stream::{
    AccountStreamItem, AccountStreamSink, MarketStreamItem, MarketStreamSink,
    NativeAccountSubscription, NativeMarketSubscription, WireAccountEvent, WireAccountStreamItem,
    WireMarketEvent, WireMarketStreamItem,
};

/// 설치된 Dart/Rust 경계의 버전입니다.
#[flutter_rust_bridge::frb(sync)]
pub fn bridge_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// 브라우저 HTTP와 WebSocket 요청에 사용할 relay origin을 설정합니다.
#[flutter_rust_bridge::frb(sync)]
pub fn configure_browser_relay(relay_url: String) -> Result<(), NativeError> {
    #[cfg(target_arch = "wasm32")]
    {
        maxt::configure_browser_relay(&relay_url).map_err(NativeError::from)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = relay_url;
        Ok(())
    }
}

/// Dart 네이티브 스트림 종료 테스트용으로 대기 중인 구독을 만듭니다.
#[doc(hidden)]
#[flutter_rust_bridge::frb(sync)]
pub fn pending_market_subscription_for_test() -> NativeMarketSubscription {
    crate::stream::pending_market_subscription_for_test()
}

/// Dart Adapter가 만든 시장 스트림에 한 항목을 전달합니다.
pub async fn market_stream_sink_add(sink: &MarketStreamSink, item: MarketStreamItem) -> bool {
    sink.add(item).await
}

/// Dart Adapter가 만든 계정 스트림에 한 항목을 전달합니다.
pub async fn account_stream_sink_add(sink: &AccountStreamSink, item: AccountStreamItem) -> bool {
    sink.add(item).await
}

/// 네이티브 시장 구독에서 다음 event/error/end 항목을 읽습니다.
pub async fn native_market_subscription_next(
    subscription: &NativeMarketSubscription,
) -> WireMarketStreamItem {
    subscription.next().await
}

/// 네이티브 계정 구독에서 다음 event/error/end 항목을 읽습니다.
pub async fn native_account_subscription_next(
    subscription: &NativeAccountSubscription,
) -> WireAccountStreamItem {
    subscription.next().await
}

/// 네이티브 시장 구독을 중단하고 원본 Rust stream 정리를 기다립니다.
pub async fn native_market_subscription_close(
    subscription: &NativeMarketSubscription,
) -> Result<(), NativeError> {
    subscription.close().await
}

/// 네이티브 계정 구독을 중단하고 원본 Rust stream 정리를 기다립니다.
pub async fn native_account_subscription_close(
    subscription: &NativeAccountSubscription,
) -> Result<(), NativeError> {
    subscription.close().await
}

/// Dart callback을 공통 Adapter 구현으로 등록합니다.
pub fn register_dart_adapter(
    exchange: WireExchange,
    features: Vec<WireFeature>,
    dispatcher: impl Fn(AdapterCall) -> DartFnFuture<anyhow::Result<AdapterResult>>
    + Send
    + Sync
    + 'static,
) -> DartAdapter {
    crate::adapter::register_dart_adapter(exchange, features, dispatcher)
}

/// Dart에서 Upbit API 지역을 선택할 때 쓰는 값입니다.
///
/// 포켓과 원화 입출금 API는 `Korea`에서만 사용할 수 있습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireUpbitRegion {
    /// 대한민국 Upbit API입니다.
    Korea,
    /// 싱가포르 Upbit API입니다.
    Singapore,
    /// 인도네시아 Upbit API입니다.
    Indonesia,
    /// 태국 Upbit API입니다.
    Thailand,
}

/// Dart에서 구성한 Binance 제품군입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireBinanceVenue {
    /// Binance Spot API입니다.
    Spot,
    /// Binance USD-M Futures API입니다.
    UsdMFutures,
}

impl From<WireUpbitRegion> for UpbitRegion {
    fn from(value: WireUpbitRegion) -> Self {
        match value {
            WireUpbitRegion::Korea => Self::Korea,
            WireUpbitRegion::Singapore => Self::Singapore,
            WireUpbitRegion::Indonesia => Self::Indonesia,
            WireUpbitRegion::Thailand => Self::Thailand,
        }
    }
}

impl From<UpbitRegion> for WireUpbitRegion {
    fn from(value: UpbitRegion) -> Self {
        match value {
            UpbitRegion::Korea => Self::Korea,
            UpbitRegion::Singapore => Self::Singapore,
            UpbitRegion::Indonesia => Self::Indonesia,
            UpbitRegion::Thailand => Self::Thailand,
            _ => unreachable!("새 Upbit region에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<BinanceMarket> for WireBinanceVenue {
    fn from(value: BinanceMarket) -> Self {
        match value {
            BinanceMarket::Spot => Self::Spot,
            BinanceMarket::UsdMFutures => Self::UsdMFutures,
            _ => unreachable!("새 Binance venue에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<WireSubscription> for Subscription {
    fn from(value: WireSubscription) -> Self {
        let subscription =
            Subscription::new().markets_iter(value.markets.into_iter().map(Into::into));
        value
            .feeds
            .into_iter()
            .fold(subscription, |subscription, feed| {
                subscription.feed(feed.into())
            })
    }
}

impl From<WireFeed> for Feed {
    fn from(value: WireFeed) -> Self {
        match value {
            WireFeed::Trades => Self::Trades,
            WireFeed::OrderBook => Self::OrderBook,
            WireFeed::Ticker => Self::Ticker,
            WireFeed::Candles(interval) => Self::Candles(interval.into()),
        }
    }
}

impl From<WireStreamConfig> for StreamConfig {
    fn from(value: WireStreamConfig) -> Self {
        Self {
            max_reconnect_attempts: value.max_reconnect_attempts,
            initial_reconnect_delay_ms: value.initial_reconnect_delay_ms,
            max_reconnect_delay_ms: value.max_reconnect_delay_ms,
            idle_timeout_ms: value.idle_timeout_ms,
            buffer_size: value.buffer_size,
            overflow: match value.overflow {
                WireOverflow::Backpressure => Overflow::Backpressure,
                WireOverflow::DropNewest => Overflow::DropNewest,
            },
        }
    }
}

enum BuiltInAdapter {
    Upbit(UpbitAdapter),
    Bithumb(BithumbAdapter),
    Binance(BinanceAdapter),
    Hyperliquid(HyperliquidAdapter),
}

impl BuiltInAdapter {
    fn as_adapter(&self) -> &dyn Adapter {
        match self {
            Self::Upbit(adapter) => adapter,
            Self::Bithumb(adapter) => adapter,
            Self::Binance(adapter) => adapter,
            Self::Hyperliquid(adapter) => adapter,
        }
    }
}

impl Adapter for BuiltInAdapter {
    fn exchange(&self) -> Exchange {
        self.as_adapter().exchange()
    }

    fn supports(&self, feature: Feature) -> bool {
        self.as_adapter().supports(feature)
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, maxt::Result<Vec<MarketInfo>>> {
        self.as_adapter().markets(kind)
    }

    fn trades(
        &self,
        market: &Market,
        limit: Option<u32>,
    ) -> BoxFuture<'_, maxt::Result<Vec<Trade>>> {
        self.as_adapter().trades(market, limit)
    }

    fn order_book(
        &self,
        market: &Market,
        depth: Option<u32>,
    ) -> BoxFuture<'_, maxt::Result<OrderBook>> {
        self.as_adapter().order_book(market, depth)
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, maxt::Result<Ticker>> {
        self.as_adapter().ticker(market)
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, maxt::Result<Vec<Candle>>> {
        self.as_adapter().candles(request)
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, maxt::Result<MarketStream>> {
        self.as_adapter().subscribe(subscription, config)
    }

    fn balances(&self) -> BoxFuture<'_, maxt::Result<Vec<Balance>>> {
        self.as_adapter().balances()
    }

    fn order_rules(&self, market: &Market) -> BoxFuture<'_, maxt::Result<OrderRules>> {
        self.as_adapter().order_rules(market)
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, maxt::Result<Vec<AssetNetwork>>> {
        self.as_adapter().asset_networks(asset)
    }

    fn deposit_addresses(&self) -> BoxFuture<'_, maxt::Result<Vec<DepositAddressEntry>>> {
        self.as_adapter().deposit_addresses()
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, maxt::Result<DepositAddress>> {
        self.as_adapter().deposit_address(request)
    }

    fn create_deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, maxt::Result<DepositAddress>> {
        self.as_adapter().create_deposit_address(request)
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, maxt::Result<WithdrawalQuote>> {
        self.as_adapter().prepare_withdrawal(request)
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, maxt::Result<Withdrawal>> {
        self.as_adapter().withdraw(request)
    }

    fn deposit(&self, request: &TransferLookupRequest) -> BoxFuture<'_, maxt::Result<Deposit>> {
        self.as_adapter().deposit(request)
    }

    fn withdrawal(
        &self,
        request: &TransferLookupRequest,
    ) -> BoxFuture<'_, maxt::Result<Withdrawal>> {
        self.as_adapter().withdrawal(request)
    }

    fn cancel_withdrawal(&self, withdrawal_id: &str) -> BoxFuture<'_, maxt::Result<()>> {
        self.as_adapter().cancel_withdrawal(withdrawal_id)
    }

    fn deposits(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, maxt::Result<Page<Deposit>>> {
        self.as_adapter().deposits(request)
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, maxt::Result<Page<Withdrawal>>> {
        self.as_adapter().withdrawals(request)
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, maxt::Result<Vec<Order>>> {
        self.as_adapter().open_orders(market)
    }

    fn order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, maxt::Result<Order>> {
        self.as_adapter().order(market, order_id)
    }

    fn order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, maxt::Result<Order>> {
        self.as_adapter().order_by_client_id(market, client_id)
    }

    fn orders_by_ids(
        &self,
        request: &OrderLookupRequest,
    ) -> BoxFuture<'_, maxt::Result<Vec<Order>>> {
        self.as_adapter().orders_by_ids(request)
    }

    fn order_history(
        &self,
        request: &OrderHistoryRequest,
    ) -> BoxFuture<'_, maxt::Result<Page<Order>>> {
        self.as_adapter().order_history(request)
    }

    fn subscribe_account(
        &self,
        config: &StreamConfig,
    ) -> BoxFuture<'_, maxt::Result<AccountStream>> {
        self.as_adapter().subscribe_account(config)
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, maxt::Result<Order>> {
        self.as_adapter().place_order(request)
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, maxt::Result<()>> {
        self.as_adapter().cancel_order(market, order_id)
    }

    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, maxt::Result<()>> {
        self.as_adapter()
            .cancel_order_by_client_id(market, client_id)
    }

    fn cancel_orders(
        &self,
        request: &CancelOrdersRequest,
    ) -> BoxFuture<'_, maxt::Result<CancelOrdersResult>> {
        self.as_adapter().cancel_orders(request)
    }

    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, maxt::Result<Vec<Position>>> {
        self.as_adapter().positions(market)
    }

    fn margin_summary(&self) -> BoxFuture<'_, maxt::Result<MarginSummary>> {
        self.as_adapter().margin_summary()
    }

    fn funding_rates(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, maxt::Result<Page<FundingRate>>> {
        self.as_adapter().funding_rates(request)
    }

    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, maxt::Result<Page<FundingPayment>>> {
        self.as_adapter().funding_payments(request)
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, maxt::Result<()>> {
        self.as_adapter().set_margin(request)
    }
}

/// Dart의 공통 Client와 제공자별 Adapter가 공유하는 native handle입니다.
#[flutter_rust_bridge::frb(opaque)]
pub struct NativeClient {
    adapter: Arc<dyn Adapter>,
    built_in: Option<Arc<BuiltInAdapter>>,
}

impl NativeClient {
    fn from_built_in(adapter: BuiltInAdapter) -> Self {
        let built_in = Arc::new(adapter);
        let adapter = built_in.clone();
        Self {
            adapter,
            built_in: Some(built_in),
        }
    }

    pub(crate) fn from_adapter(adapter: Arc<dyn Adapter>) -> Self {
        Self {
            adapter,
            built_in: None,
        }
    }

    /// Dart에서 구현한 Adapter를 공통 native Client로 감쌉니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn from_dart_adapter(adapter: DartAdapter) -> Self {
        Self::from_adapter(adapter.into_adapter())
    }

    fn built_in(&self, operation: &str) -> Result<&BuiltInAdapter, Error> {
        self.built_in
            .as_deref()
            .ok_or_else(|| Error::InvalidRequest {
                field: "adapter".to_owned(),
                detail: format!("{operation} is available only on a built-in adapter"),
            })
    }

    /// Upbit의 지역과 선택적 자격증명을 구성합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn upbit(
        region: WireUpbitRegion,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, NativeError> {
        let mut adapter = UpbitAdapter::with_region(region.into());
        if let Some((access_key, secret_key)) =
            credential_pair(access_key, secret_key, "access_key", "secret_key")?
        {
            adapter = adapter.with_credentials(access_key, secret_key);
        }
        Ok(Self::from_built_in(BuiltInAdapter::Upbit(adapter)))
    }

    /// Bithumb의 선택적 자격증명을 구성합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn bithumb(
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, NativeError> {
        let mut adapter = BithumbAdapter::new();
        if let Some((access_key, secret_key)) =
            credential_pair(access_key, secret_key, "access_key", "secret_key")?
        {
            adapter = adapter.with_credentials(access_key, secret_key);
        }
        Ok(Self::from_built_in(BuiltInAdapter::Bithumb(adapter)))
    }

    /// Binance Spot의 선택적 자격증명을 구성합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn binance_spot(
        api_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, NativeError> {
        Self::binance(BinanceAdapter::spot(), api_key, secret_key).map_err(Into::into)
    }

    /// Binance USD-M의 선택적 자격증명을 구성합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn binance_usd_m_futures(
        api_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, NativeError> {
        Self::binance(BinanceAdapter::usd_m_futures(), api_key, secret_key).map_err(Into::into)
    }

    fn binance(
        mut adapter: BinanceAdapter,
        api_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, Error> {
        if let Some((api_key, secret_key)) =
            credential_pair(api_key, secret_key, "api_key", "secret_key")?
        {
            adapter = adapter.with_credentials(api_key, secret_key);
        }
        Ok(Self::from_built_in(BuiltInAdapter::Binance(adapter)))
    }

    /// Hyperliquid mainnet/testnet과 선택적 지갑을 구성합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn hyperliquid(
        testnet: bool,
        address: Option<String>,
        private_key: Option<String>,
    ) -> Result<Self, NativeError> {
        let mut adapter = if testnet {
            HyperliquidAdapter::testnet()
        } else {
            HyperliquidAdapter::new()
        };
        if let Some(address) = address {
            adapter = adapter.with_query_address(address);
        }
        if let Some(private_key) = private_key {
            adapter = adapter.with_signer(private_key);
        }
        Ok(Self::from_built_in(BuiltInAdapter::Hyperliquid(adapter)))
    }

    /// 이 native handle이 연결한 거래소를 반환합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn exchange(&self) -> WireExchange {
        self.adapter.exchange().into()
    }

    /// 이 native handle이 지정한 공통 기능을 지원하는지 반환합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn supports(&self, feature: WireFeature) -> bool {
        self.adapter.supports(feature.into())
    }

    /// Upbit handle이면 선택된 지역을, 아니면 null을 반환합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn upbit_region(&self) -> Option<WireUpbitRegion> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Upbit(adapter) => Some(adapter.region().into()),
            _ => None,
        }
    }

    /// Binance handle이면 선택된 제품군을, 아니면 null을 반환합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn binance_venue(&self) -> Option<WireBinanceVenue> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Binance(adapter) => Some(adapter.venue().into()),
            _ => None,
        }
    }

    /// Hyperliquid handle이면 testnet 여부를, 아니면 null을 반환합니다.
    #[flutter_rust_bridge::frb(sync)]
    pub fn is_testnet(&self) -> Option<bool> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Hyperliquid(adapter) => Some(adapter.is_testnet()),
            _ => None,
        }
    }

    /// 초기 연결을 마친 뒤 Dart가 한 항목씩 읽을 수 있는 시장 구독을 반환합니다.
    pub async fn subscribe(
        &self,
        subscription: WireSubscription,
        config: WireStreamConfig,
    ) -> Result<NativeMarketSubscription, NativeError> {
        let subscription = subscription.into();
        let config = config.into();
        self.adapter
            .subscribe(&subscription, &config)
            .await
            .map(NativeMarketSubscription::new)
            .map_err(Into::into)
    }

    /// 초기 연결을 마친 뒤 Dart가 한 항목씩 읽을 수 있는 계정 구독을 반환합니다.
    pub async fn subscribe_account(
        &self,
        config: WireStreamConfig,
    ) -> Result<NativeAccountSubscription, NativeError> {
        let config = config.into();
        self.adapter
            .subscribe_account(&config)
            .await
            .map(NativeAccountSubscription::new)
            .map_err(Into::into)
    }

    /// 여러 Upbit 현물 시장의 호가 스냅샷을 반환합니다.
    ///
    /// `depth`는 각 매수·매도 측의 최대 단계 수입니다.
    pub async fn upbit_order_books(
        &self,
        markets: Vec<WireMarket>,
        depth: Option<u32>,
    ) -> Result<Vec<WireOrderBook>, NativeError> {
        let adapter = match self.built_in("upbit_order_books")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        let markets: Vec<_> = markets.into_iter().map(Into::into).collect();
        adapter
            .order_books(&markets, depth)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 지정 가격 단위로 묶은 Upbit 호가 스냅샷을 반환합니다.
    ///
    /// 가격 단위는 해당 시장의 현재 지원 단위여야 합니다.
    pub async fn upbit_order_books_at_level(
        &self,
        markets: Vec<WireMarket>,
        level: String,
        depth: Option<u32>,
    ) -> Result<Vec<WireOrderBook>, NativeError> {
        let adapter = match self.built_in("upbit_order_books_at_level")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        let markets: Vec<_> = markets.into_iter().map(Into::into).collect();
        let level = decimal_from_wire(&level, "level").map_err(NativeError::from)?;
        adapter
            .order_books_at_level(&markets, level, depth)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 지정한 Upbit 현물 시장의 ticker 요약을 반환합니다.
    pub async fn upbit_tickers(
        &self,
        markets: Vec<WireMarket>,
    ) -> Result<Vec<WireTicker>, NativeError> {
        let adapter = match self.built_in("upbit_tickers")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        let markets: Vec<_> = markets.into_iter().map(Into::into).collect();
        adapter
            .tickers(&markets)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 하나 이상의 호가 통화로 Upbit ticker를 조회합니다.
    pub async fn upbit_tickers_by_quote(
        &self,
        quote_currencies: Vec<String>,
    ) -> Result<Vec<WireTicker>, NativeError> {
        let adapter = match self.built_in("upbit_tickers_by_quote")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .tickers_by_quote(&quote_currencies)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 한 Upbit 시장의 연간 캔들을 오래된 순서로 반환합니다.
    ///
    /// `count`는 1부터 200까지이며, `toNs`는 포함되지 않는 종료 시각입니다.
    pub async fn upbit_year_candles(
        &self,
        market: WireMarket,
        to_ns: Option<i64>,
        count: Option<u32>,
    ) -> Result<Vec<WireUpbitYearCandle>, NativeError> {
        let adapter = match self.built_in("upbit_year_candles")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .year_candles(
                &market.into(),
                to_ns.map(maxt::Timestamp::from_nanos),
                count,
            )
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 지정한 Upbit 시장의 현재 호가 단위와 지원 가격 단위를 반환합니다.
    pub async fn upbit_orderbook_instruments(
        &self,
        markets: Vec<WireMarket>,
    ) -> Result<Vec<WireUpbitOrderBookInstrument>, NativeError> {
        let adapter = match self.built_in("upbit_orderbook_instruments")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        let markets: Vec<_> = markets.into_iter().map(Into::into).collect();
        adapter
            .orderbook_instruments(&markets)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Upbit 시장의 투자 경고·주의 정보를 반환합니다.
    pub async fn upbit_market_events(&self) -> Result<Vec<WireUpbitMarketEvent>, NativeError> {
        let adapter = match self.built_in("upbit_market_events")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .market_events()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 임시 Upbit 연결이 실제로 구독한 항목을 반환합니다.
    pub async fn upbit_list_subscriptions(
        &self,
        subscription: WireSubscription,
    ) -> Result<WireUpbitSubscriptionList, NativeError> {
        let adapter = match self.built_in("upbit_list_subscriptions")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .list_subscriptions(&subscription.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Upbit 주문을 실제 제출 없이 검증합니다.
    ///
    /// 반환된 주문 ID와 상태는 실주문을 뜻하지 않아 조회·취소에 사용할 수 없습니다.
    pub async fn upbit_test_order(
        &self,
        request: WireOrderRequest,
    ) -> Result<WireOrder, NativeError> {
        let request: OrderRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_test_order")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .test_order(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 체결·수수료·자전거래 방지 정보를 포함한 Upbit 주문 상세를 반환합니다.
    ///
    /// UUID와 사용자 주문 식별자를 모두 주면 Upbit는 UUID를 우선합니다.
    pub async fn upbit_order_detail(
        &self,
        request: WireUpbitOrderDetailRequest,
    ) -> Result<WireUpbitOrderDetail, NativeError> {
        let request: maxt::UpbitOrderDetailRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_order_detail")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .order_detail(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 조건에 맞는 Upbit 종료 주문 목록을 반환합니다.
    pub async fn upbit_closed_orders(
        &self,
        request: WireUpbitClosedOrdersRequest,
    ) -> Result<Vec<WireUpbitClosedOrder>, NativeError> {
        let request: maxt::UpbitClosedOrdersRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_closed_orders")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .closed_orders(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 한 자산·네트워크의 Upbit 입금 가능 정보를 반환합니다.
    ///
    /// 이 정보는 실시간 서비스 상태가 아니며 몇 분 지연될 수 있습니다.
    pub async fn upbit_deposit_info(
        &self,
        asset: String,
        network: String,
    ) -> Result<WireUpbitDepositInfo, NativeError> {
        let network = crate::convert::network_from_wire(network);
        let adapter = match self.built_in("upbit_deposit_info")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .deposit_info(&asset, &network)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Upbit Korea 또는 Singapore의 Travel Rule 검증 가능 VASP를 반환합니다.
    pub async fn upbit_travel_rule_vasps(
        &self,
    ) -> Result<Vec<WireUpbitTravelRuleVasp>, NativeError> {
        let adapter = match self.built_in("upbit_travel_rule_vasps")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .travel_rule_vasps()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 입금 UUID로 Upbit Travel Rule 검증을 요청합니다.
    ///
    /// 이는 금융 쓰기 요청이며 동일 입금에 대한 반복 요청 제한은 Upbit가 적용합니다.
    pub async fn upbit_verify_travel_rule_by_uuid(
        &self,
        deposit_uuid: String,
        vasp_uuid: String,
    ) -> Result<WireUpbitTravelRuleVerification, NativeError> {
        let adapter = match self.built_in("upbit_verify_travel_rule_by_uuid")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .verify_travel_rule_by_uuid(&deposit_uuid, &vasp_uuid)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 거래 ID로 Upbit Travel Rule 검증을 요청합니다.
    ///
    /// 이는 금융 쓰기 요청이며 동일 입금에 대한 반복 요청 제한은 Upbit가 적용합니다.
    pub async fn upbit_verify_travel_rule_by_txid(
        &self,
        txid: String,
        vasp_uuid: String,
        currency: String,
        net_type: String,
    ) -> Result<WireUpbitTravelRuleVerification, NativeError> {
        let adapter = match self.built_in("upbit_verify_travel_rule_by_txid")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .verify_travel_rule_by_txid(&txid, &vasp_uuid, &currency, &net_type)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 조건에 맞는 Upbit 대기 주문을 한 요청으로 취소합니다.
    ///
    /// 반환값은 취소 완료와 상태 변경으로 취소하지 못한 주문을 구분합니다.
    pub async fn upbit_batch_cancel_open_orders(
        &self,
        request: WireUpbitBatchCancelRequest,
    ) -> Result<WireCancelOrdersResult, NativeError> {
        let request: maxt::UpbitBatchCancelRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_batch_cancel_open_orders")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .batch_cancel_open_orders(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Upbit 주문을 취소하고 대체 주문을 요청합니다.
    ///
    /// 이전 주문이 먼저 체결되면 성공 응답이어도 대체 주문이 없을 수 있습니다.
    pub async fn upbit_cancel_and_new_order(
        &self,
        request: WireUpbitCancelAndNewOrderRequest,
    ) -> Result<WireUpbitCancelAndNewOrderResult, NativeError> {
        let request: maxt::UpbitCancelAndNewOrderRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_cancel_and_new_order")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .cancel_and_new_order(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Upbit Korea 등록 계좌에서 원화 입금을 요청합니다.
    ///
    /// 이는 Korea 지역 전용 금융 쓰기입니다.
    pub async fn upbit_deposit_krw(
        &self,
        request: WireUpbitKrwTransferRequest,
    ) -> Result<WireUpbitKrwDeposit, NativeError> {
        let request: maxt::UpbitKrwTransferRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_deposit_krw")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .deposit_krw(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Upbit Korea 등록 계좌로 원화 출금을 요청합니다.
    ///
    /// 이는 Korea 지역 전용 금융 쓰기이며 출금 안전 잠금으로 거절될 수 있습니다.
    pub async fn upbit_withdraw_krw(
        &self,
        request: WireUpbitKrwTransferRequest,
    ) -> Result<WireUpbitKrwWithdrawal, NativeError> {
        let request: maxt::UpbitKrwTransferRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_withdraw_krw")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .withdraw_krw(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 이 Upbit Korea 계정에 등록된 API 키 식별자와 만료 시각을 반환합니다.
    ///
    /// 비밀 키 자료는 반환하지 않습니다.
    pub async fn upbit_api_keys(&self) -> Result<Vec<WireUpbitApiKey>, NativeError> {
        let adapter = match self.built_in("upbit_api_keys")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .api_keys()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Upbit Korea API 키가 볼 수 있는 포켓 목록을 반환합니다.
    pub async fn upbit_list_pockets(&self) -> Result<Vec<WireUpbitPocket>, NativeError> {
        let adapter = match self.built_in("upbit_list_pockets")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .list_pockets()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Upbit Korea 포켓별 API 키를 반환합니다.
    ///
    /// 요청은 포켓 UUID와 만료 키 포함 여부를 선택적으로 제한합니다.
    pub async fn upbit_list_pocket_api_keys(
        &self,
        request: WireUpbitPocketApiKeysRequest,
    ) -> Result<Vec<WireUpbitPocketApiKeyGroup>, NativeError> {
        let request: maxt::UpbitPocketApiKeysRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_list_pocket_api_keys")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .list_pocket_api_keys(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 한 Upbit Korea 서브 포켓의 자산 잔고를 반환합니다.
    pub async fn upbit_sub_pocket_balances(
        &self,
        pocket_uuid: String,
    ) -> Result<Vec<WireUpbitPocketBalance>, NativeError> {
        let adapter = match self.built_in("upbit_sub_pocket_balances")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .sub_pocket_balances(&pocket_uuid)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Upbit Korea 메인 포켓 간 자산 이전을 요청합니다.
    ///
    /// 이는 금융 쓰기이며 현재 OpenAPI 계약상 대상 포켓 `to`가 필수입니다.
    pub async fn upbit_universal_transfer(
        &self,
        request: WireUpbitPocketUniversalTransferRequest,
    ) -> Result<WireUpbitPocketTransfer, NativeError> {
        let request: maxt::UpbitPocketUniversalTransferRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_universal_transfer")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .universal_transfer(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Upbit Korea 메인 포켓 이전 이력을 반환합니다.
    pub async fn upbit_universal_transfers(
        &self,
        request: WireUpbitPocketTransferQuery,
    ) -> Result<Vec<WireUpbitPocketTransfer>, NativeError> {
        let request: maxt::UpbitPocketTransferQuery = request.try_into()?;
        let adapter = match self.built_in("upbit_universal_transfers")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .universal_transfers(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 현재 Upbit Korea 서브 포켓에서 다른 포켓으로 자산 이전을 요청합니다.
    ///
    /// 이는 금융 쓰기이며 현재 OpenAPI 계약상 대상 포켓 `to`가 필수입니다.
    pub async fn upbit_sub_pocket_transfer(
        &self,
        request: WireUpbitPocketTransferRequest,
    ) -> Result<WireUpbitPocketTransfer, NativeError> {
        let request: maxt::UpbitPocketTransferRequest = request.try_into()?;
        let adapter = match self.built_in("upbit_sub_pocket_transfer")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .sub_pocket_transfer(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 현재 Upbit Korea 서브 포켓의 이전 이력을 반환합니다.
    pub async fn upbit_sub_pocket_transfers(
        &self,
        request: WireUpbitPocketTransferQuery,
    ) -> Result<Vec<WireUpbitPocketTransfer>, NativeError> {
        let request: maxt::UpbitPocketTransferQuery = request.try_into()?;
        let adapter = match self.built_in("upbit_sub_pocket_transfers")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .sub_pocket_transfers(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Bithumb 시장별 원본 투자 유의 플래그를 반환합니다.
    ///
    /// 유의 종목도 거래 가능할 수 있으며 공통 시장 상태와는 별개입니다.
    pub async fn bithumb_market_warnings(
        &self,
    ) -> Result<Vec<WireBithumbMarketWarning>, NativeError> {
        let adapter = match self.built_in("bithumb_market_warnings")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .market_warnings()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Bithumb에서 활성화한 시장별 경보를 반환합니다.
    pub async fn bithumb_market_alerts(&self) -> Result<Vec<WireBithumbMarketAlert>, NativeError> {
        let adapter = match self.built_in("bithumb_market_alerts")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .market_alerts()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 최신 Bithumb 거래소 공지를 먼저 반환합니다.
    ///
    /// `count`는 1부터 20까지이며 생략하면 Bithumb 기본값을 사용합니다.
    pub async fn bithumb_notices(
        &self,
        count: Option<u32>,
    ) -> Result<Vec<WireBithumbNotice>, NativeError> {
        let adapter = match self.built_in("bithumb_notices")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .notices(count)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 한 자산 또는 `ALL`에 대한 Bithumb 전송 수수료 규칙을 반환합니다.
    ///
    /// 계정별 출금 가능 여부와 한도는 이 응답에 포함되지 않습니다.
    pub async fn bithumb_transfer_fees(
        &self,
        currency: String,
    ) -> Result<Vec<WireBithumbAssetFee>, NativeError> {
        let adapter = match self.built_in("bithumb_transfer_fees")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .transfer_fees(&currency)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 이 Bithumb 계정에 등록된 API 키 정보를 반환합니다.
    pub async fn bithumb_api_keys(&self) -> Result<Vec<WireBithumbApiKey>, NativeError> {
        let adapter = match self.built_in("bithumb_api_keys")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .api_keys()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 이 계정에 등록된 Bithumb 출금 주소 정보를 반환합니다.
    ///
    /// 이는 출금 견적이나 사전 검증 결과가 아니라 제공자 등록 주소 목록입니다.
    pub async fn bithumb_withdrawal_addresses(
        &self,
    ) -> Result<Vec<WireBithumbWithdrawalAddress>, NativeError> {
        let adapter = match self.built_in("bithumb_withdrawal_addresses")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .withdrawal_addresses()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 체결·수수료·STP 등 Bithumb 전용 상세를 포함한 한 주문을 반환합니다.
    ///
    /// UUID와 client 주문 ID를 모두 주면 Bithumb는 UUID를 우선하며, 요청 시장은
    /// 반환 주문과 로컬에서 대조합니다.
    pub async fn bithumb_order_detail(
        &self,
        request: WireBithumbOrderDetailRequest,
    ) -> Result<WireBithumbOrderDetail, NativeError> {
        let request: maxt::BithumbOrderDetailRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_order_detail")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .order_detail(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 상태·식별자·페이지 조건에 맞는 Bithumb 주문 목록을 반환합니다.
    ///
    /// `state`와 `states`는 함께 설정할 수 없으며, UUID 목록은 client 주문 ID 목록보다
    /// 우선합니다.
    pub async fn bithumb_order_list(
        &self,
        request: WireBithumbOrderListRequest,
    ) -> Result<Vec<WireBithumbOrderListItem>, NativeError> {
        let request: maxt::BithumbOrderListRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_order_list")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .order_list(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Bithumb 원화 출금 이력을 반환합니다.
    pub async fn bithumb_krw_withdrawals(
        &self,
        request: WireBithumbKrwWithdrawalsRequest,
    ) -> Result<Vec<WireBithumbKrwWithdrawal>, NativeError> {
        let request: maxt::BithumbKrwWithdrawalsRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_krw_withdrawals")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .krw_withdrawals(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Bithumb 원화 출금을 요청합니다.
    ///
    /// 이는 금융 쓰기이며 등록 계좌와 제공자 측 2차 인증이 필요할 수 있습니다.
    pub async fn bithumb_withdraw_krw(
        &self,
        request: WireBithumbKrwTransferRequest,
    ) -> Result<WireBithumbKrwWithdrawal, NativeError> {
        let request: maxt::BithumbKrwTransferRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_withdraw_krw")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .withdraw_krw(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Bithumb 원화 입금 이력을 반환합니다.
    pub async fn bithumb_krw_deposits(
        &self,
        request: WireBithumbKrwDepositsRequest,
    ) -> Result<Vec<WireBithumbKrwDeposit>, NativeError> {
        let request: maxt::BithumbKrwDepositsRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_krw_deposits")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .krw_deposits(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Bithumb 원화 입금을 요청합니다.
    ///
    /// 이는 금융 쓰기이며 은행 계좌와 2차 인증 조건은 제공자 측에서 확인합니다.
    pub async fn bithumb_deposit_krw(
        &self,
        request: WireBithumbKrwTransferRequest,
    ) -> Result<WireBithumbKrwDeposit, NativeError> {
        let request: maxt::BithumbKrwTransferRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_deposit_krw")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .deposit_krw(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Bithumb `wait` 또는 `watch` 상태 주문 페이지를 반환합니다.
    pub async fn bithumb_pending_orders(
        &self,
        request: WireBithumbPendingOrdersRequest,
    ) -> Result<WireOrderPage, NativeError> {
        let request: maxt::BithumbPendingOrdersRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_pending_orders")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .pending_orders(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 조건에 맞는 Bithumb 종료 주문 페이지를 반환합니다.
    pub async fn bithumb_closed_orders(
        &self,
        request: WireBithumbClosedOrdersRequest,
    ) -> Result<WireBithumbClosedOrderPage, NativeError> {
        let request: maxt::BithumbClosedOrdersRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_closed_orders")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .closed_orders(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 최대 20개 Bithumb 주문을 함께 제출하고 항목별 결과를 반환합니다.
    ///
    /// 이는 금융 쓰기이며 HTTP 성공이어도 일부 항목은 거절될 수 있습니다.
    pub async fn bithumb_batch_orders(
        &self,
        request: WireBithumbBatchOrdersRequest,
    ) -> Result<WireBithumbBatchOrdersResult, NativeError> {
        let request: maxt::BithumbBatchOrdersRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_batch_orders")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .batch_orders(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Bithumb TWAP 주문 페이지를 반환합니다.
    pub async fn bithumb_twap_orders(
        &self,
        request: WireBithumbTwapOrdersRequest,
    ) -> Result<WireBithumbTwapOrderPage, NativeError> {
        let request: maxt::BithumbTwapOrdersRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_twap_orders")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .twap_orders(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Bithumb TWAP 주문을 생성하고 제공자 주문 식별자를 반환합니다.
    ///
    /// 이는 금융 쓰기 요청입니다.
    pub async fn bithumb_create_twap_order(
        &self,
        request: WireBithumbTwapOrderRequest,
    ) -> Result<String, NativeError> {
        let request: maxt::BithumbTwapOrderRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_create_twap_order")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .create_twap_order(&request)
            .await
            .map_err(Into::into)
    }

    /// Bithumb TWAP 주문을 취소하고 취소된 식별자를 반환합니다.
    pub async fn bithumb_cancel_twap_order(
        &self,
        algo_order_id: String,
    ) -> Result<String, NativeError> {
        let adapter = match self.built_in("bithumb_cancel_twap_order")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter
            .cancel_twap_order(&algo_order_id)
            .await
            .map_err(Into::into)
    }

    /// 한 Binance Spot 시장의 가격·수량·명목가 제약을 반환합니다.
    ///
    /// USD-M Futures handle에서는 지원되지 않습니다.
    pub async fn binance_spot_symbol_filters(
        &self,
        market: WireMarket,
    ) -> Result<WireBinanceSymbolFilters, NativeError> {
        let adapter = match self.built_in("binance_spot_symbol_filters")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .spot_symbol_filters(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 숫자 주문 ID로 Binance Spot 주문 상세를 반환합니다.
    ///
    /// 체결·취소 주문도 포함하며 USD-M Futures handle에서는 지원되지 않습니다.
    pub async fn binance_spot_order(
        &self,
        market: WireMarket,
        order_id: String,
    ) -> Result<WireBinanceSpotOrderDetail, NativeError> {
        let adapter = match self.built_in("binance_spot_order")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .spot_order(&market.into(), &order_id)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 한 Binance Spot 시장의 현재 평균 가격을 반환합니다.
    pub async fn binance_spot_average_price(
        &self,
        market: WireMarket,
    ) -> Result<WireBinanceSpotAveragePrice, NativeError> {
        let adapter = match self.built_in("binance_spot_average_price")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .spot_average_price(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 한 Binance USD-M 시장의 현재 mark price와 펀딩 정보를 반환합니다.
    pub async fn binance_mark_price(
        &self,
        market: WireMarket,
    ) -> Result<WireBinanceMarkPrice, NativeError> {
        let adapter = match self.built_in("binance_mark_price")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .mark_price(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 모든 Binance USD-M 영구 시장의 현재 mark price를 반환합니다.
    pub async fn binance_mark_prices(&self) -> Result<Vec<WireBinanceMarkPrice>, NativeError> {
        let adapter = match self.built_in("binance_mark_prices")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .mark_prices()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 한 Binance USD-M 시장의 현재 미결제약정을 반환합니다.
    pub async fn binance_open_interest(
        &self,
        market: WireMarket,
    ) -> Result<WireBinanceOpenInterest, NativeError> {
        let adapter = match self.built_in("binance_open_interest")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .open_interest(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Binance USD-M의 가격·방향 기준 집계 체결을 반환합니다.
    ///
    /// 개별 체결 목록이 아니며 공개 API라 자격증명이 필요 없습니다.
    pub async fn binance_aggregate_trades(
        &self,
        request: WireBinanceAggregateTradesRequest,
    ) -> Result<Vec<WireBinanceAggregateTrade>, NativeError> {
        let request: maxt::BinanceAggregateTradesRequest = request.try_into()?;
        let adapter = match self.built_in("binance_aggregate_trades")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .aggregate_trades(&request)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Binance 계정 체결 페이지를 반환합니다.
    ///
    /// 전체 페이지여도 일반 cursor를 만들지 않으므로 다음 조회에는 제공자 체결 ID를
    /// 사용해야 합니다.
    pub async fn binance_account_trades(
        &self,
        request: WireHistoryRequest,
    ) -> Result<WireBinanceAccountTradePage, NativeError> {
        let request: HistoryRequest = request.into();
        let adapter = match self.built_in("binance_account_trades")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .account_trades(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Binance Spot Funding Wallet의 C2C 거래 이력 응답을 그대로 반환합니다.
    ///
    /// `page`를 증가시켜 조회하며, 일반 cursor를 만들지 않고 제공자 envelope을 보존합니다.
    pub async fn binance_c_2_c_trade_history(
        &self,
        request: WireBinanceC2cTradeHistoryRequest,
    ) -> Result<WireBinanceC2cTradeHistoryPage, NativeError> {
        let request: maxt::BinanceC2cTradeHistoryRequest = request.try_into()?;
        let adapter = match self.built_in("binance_c2c_trade_history")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .c2c_trade_history(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Binance 주문을 실제 생성 없이 검증합니다.
    ///
    /// Spot은 보통 빈 객체를, USD-M은 주문 형태 객체를 반환할 수 있습니다.
    pub async fn binance_test_order(
        &self,
        request: WireBinanceTestOrderRequest,
    ) -> Result<WireBinanceTestOrder, NativeError> {
        let request: maxt::BinanceTestOrderRequest = request.try_into()?;
        let adapter = match self.built_in("binance_test_order")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .test_order(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 한 Binance 시장의 모든 미체결 주문을 취소합니다.
    ///
    /// Spot과 USD-M의 응답 형태가 달라 성공 여부만 반환합니다.
    pub async fn binance_cancel_all_open_orders(
        &self,
        market: WireMarket,
    ) -> Result<(), NativeError> {
        let adapter = match self.built_in("binance_cancel_all_open_orders")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .cancel_all_open_orders(&market.into())
            .await
            .map_err(Into::into)
    }

    /// Binance USD-M 사용자 데이터 스트림 listen key를 생성합니다.
    ///
    /// 일반적으로 [subscribe_account]가 이 수명을 관리합니다.
    pub async fn binance_usd_m_create_listen_key(
        &self,
    ) -> Result<WireBinanceListenKey, NativeError> {
        let adapter = match self.built_in("binance_usd_m_create_listen_key")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .usd_m_create_listen_key()
            .await
            .map(|inner| WireBinanceListenKey { inner })
            .map_err(Into::into)
    }

    /// 현재 API 키가 소유한 Binance USD-M listen key를 연장합니다.
    pub async fn binance_usd_m_keepalive_listen_key(&self) -> Result<(), NativeError> {
        let adapter = match self.built_in("binance_usd_m_keepalive_listen_key")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .usd_m_keepalive_listen_key()
            .await
            .map_err(Into::into)
    }

    /// 현재 API 키가 소유한 Binance USD-M listen key를 닫습니다.
    pub async fn binance_usd_m_close_listen_key(&self) -> Result<(), NativeError> {
        let adapter = match self.built_in("binance_usd_m_close_listen_key")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter.usd_m_close_listen_key().await.map_err(Into::into)
    }

    /// 구성된 Hyperliquid 계정의 비펀딩 원장 페이지를 반환합니다.
    ///
    /// 주소만 필요하고 서명은 사용하지 않으며, cursor와 시간 범위는 제공자 기준입니다.
    pub async fn hyperliquid_non_funding_ledger(
        &self,
        from_ns: Option<i64>,
        to_ns: Option<i64>,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> Result<WireHyperliquidLedgerPage, NativeError> {
        let adapter = match self.built_in("hyperliquid_non_funding_ledger")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        let cursor = cursor.map(maxt::Cursor::new);
        adapter
            .non_funding_ledger(
                from_ns.map(maxt::Timestamp::from_nanos),
                to_ns.map(maxt::Timestamp::from_nanos),
                cursor.as_ref(),
                limit,
            )
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 최근 체결을 반환합니다.
    pub async fn hyperliquid_user_fills(
        &self,
        aggregate_by_time: bool,
    ) -> Result<Vec<WireHyperliquidUserFill>, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_fills")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_fills(aggregate_by_time)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 지정 시간 범위 체결을 반환합니다.
    pub async fn hyperliquid_user_fills_by_time(
        &self,
        from_ns: i64,
        to_ns: Option<i64>,
        aggregate_by_time: bool,
    ) -> Result<Vec<WireHyperliquidUserFill>, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_fills_by_time")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_fills_by_time(
                maxt::Timestamp::from_nanos(from_ns),
                to_ns.map(maxt::Timestamp::from_nanos),
                aggregate_by_time,
            )
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 간략한 현재 미체결 주문을 반환합니다.
    pub async fn hyperliquid_basic_open_orders(
        &self,
    ) -> Result<Vec<WireHyperliquidOpenOrder>, NativeError> {
        let adapter = match self.built_in("hyperliquid_basic_open_orders")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .basic_open_orders()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 서버 주문 ID 또는 클라이언트 주문 ID로 Hyperliquid 주문 상태를 반환합니다.
    ///
    /// 주소 검증은 adapter가 먼저 수행하므로, 잘못된 클라이언트 주문 ID보다 누락된
    /// 주소 오류가 우선 반환됩니다.
    pub async fn hyperliquid_order_status(
        &self,
        reference: WireHyperliquidOrderReference,
    ) -> Result<WireHyperliquidOrderStatusResponse, NativeError> {
        let adapter = match self.built_in("hyperliquid_order_status")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        let reference: maxt::HyperliquidOrderReference = reference.try_into()?;
        adapter
            .order_status(reference)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 최근 주문 이력을 반환합니다.
    pub async fn hyperliquid_historical_orders(
        &self,
    ) -> Result<Vec<WireHyperliquidOrderInfo>, NativeError> {
        let adapter = match self.built_in("hyperliquid_historical_orders")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .historical_orders()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 한 Hyperliquid 시장의 현재 가격·펀딩·정밀도 정보를 반환합니다.
    pub async fn hyperliquid_asset_context(
        &self,
        market: WireMarket,
    ) -> Result<WireHyperliquidAssetContext, NativeError> {
        let adapter = match self.built_in("hyperliquid_asset_context")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .asset_context(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 기본 Hyperliquid 현물·영구 시장의 현재 중간 가격을 반환합니다.
    ///
    /// HIP-3 DEX 시장은 이 adapter의 시장 표에 포함되지 않습니다.
    pub async fn hyperliquid_all_mids(&self) -> Result<Vec<WireHyperliquidMidPrice>, NativeError> {
        let adapter = match self.built_in("hyperliquid_all_mids")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .all_mids()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 현재 Info API 요청 한도를 반환합니다.
    ///
    /// 이는 주소만 필요한 공개 계정 조회입니다.
    pub async fn hyperliquid_user_rate_limit(
        &self,
    ) -> Result<WireHyperliquidUserRateLimit, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_rate_limit")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_rate_limit()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 제공자 역할을 반환합니다.
    ///
    /// 알 수 없는 역할 이름은 raw 값으로 보존됩니다.
    pub async fn hyperliquid_user_role(&self) -> Result<WireHyperliquidUserRole, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_role")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_role()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 추천 프로그램 상태를 반환합니다.
    pub async fn hyperliquid_referral(&self) -> Result<WireHyperliquidReferral, NativeError> {
        let adapter = match self.built_in("hyperliquid_referral")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter.referral().await.map(Into::into).map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 제공자 수수료 일정을 반환합니다.
    ///
    /// 제공자가 확장할 수 있는 세부 tier 정보도 보존합니다.
    pub async fn hyperliquid_user_fees(&self) -> Result<WireHyperliquidUserFees, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_fees")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_fees()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 제공자 기간별 포트폴리오 이력을 반환합니다.
    pub async fn hyperliquid_portfolio(
        &self,
    ) -> Result<Vec<WireHyperliquidPortfolioPeriod>, NativeError> {
        let adapter = match self.built_in("hyperliquid_portfolio")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .portfolio()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 서브 계정을 반환합니다.
    ///
    /// 제공자가 계정이 없음을 null로 보내면 빈 목록으로 반환합니다.
    pub async fn hyperliquid_sub_accounts(
        &self,
    ) -> Result<Vec<WireHyperliquidSubAccount>, NativeError> {
        let adapter = match self.built_in("hyperliquid_sub_accounts")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .sub_accounts()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// 구성된 Hyperliquid 주소의 현재 vault 지분을 반환합니다.
    pub async fn hyperliquid_user_vault_equities(
        &self,
    ) -> Result<Vec<WireHyperliquidVaultEquity>, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_vault_equities")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_vault_equities()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}

/// Binance listen key의 Rust 수명을 유지하면서 공개 값을 읽는 handle입니다.
#[flutter_rust_bridge::frb(opaque)]
pub struct WireBinanceListenKey {
    inner: BinanceListenKey,
}

impl WireBinanceListenKey {
    /// 소유 중인 Binance listen key 문자열을 반환합니다.
    #[flutter_rust_bridge::frb(sync, getter)]
    pub fn value(&self) -> String {
        self.inner.as_str().to_owned()
    }
}

fn credential_pair(
    first: Option<String>,
    second: Option<String>,
    first_name: &str,
    second_name: &str,
) -> Result<Option<(String, String)>, Error> {
    match (first, second) {
        (None, None) => Ok(None),
        (Some(first), Some(second)) => Ok(Some((first, second))),
        _ => Err(Error::InvalidRequest {
            field: "credentials".to_owned(),
            detail: format!("{first_name} and {second_name} must be provided together"),
        }),
    }
}

fn provider_mismatch(provider: &str) -> NativeError {
    NativeError::invalid_request("adapter", format!("this operation requires {provider}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factories_preserve_region_venue_testnet_and_credential_features() {
        let upbit_korea = NativeClient::upbit(
            WireUpbitRegion::Korea,
            Some("key".to_owned()),
            Some("secret".to_owned()),
        )
        .unwrap();
        assert!(upbit_korea.supports(WireFeature::TravelRule));

        let upbit = NativeClient::upbit(WireUpbitRegion::Singapore, None, None).unwrap();
        assert_eq!(upbit.exchange(), WireExchange::Upbit);
        assert_eq!(upbit.upbit_region(), Some(WireUpbitRegion::Singapore));
        assert!(!upbit.supports(WireFeature::Trading));
        assert!(!upbit.supports(WireFeature::TravelRule));

        let upbit_with_credentials = NativeClient::upbit(
            WireUpbitRegion::Singapore,
            Some("key".to_owned()),
            Some("secret".to_owned()),
        )
        .unwrap();
        assert!(upbit_with_credentials.supports(WireFeature::TravelRule));

        let upbit_indonesia = NativeClient::upbit(
            WireUpbitRegion::Indonesia,
            Some("key".to_owned()),
            Some("secret".to_owned()),
        )
        .unwrap();
        assert!(!upbit_indonesia.supports(WireFeature::TravelRule));

        let binance =
            NativeClient::binance_usd_m_futures(Some("key".to_owned()), Some("secret".to_owned()))
                .unwrap();
        assert_eq!(binance.binance_venue(), Some(WireBinanceVenue::UsdMFutures));
        assert!(binance.supports(WireFeature::Trading));

        let hyperliquid = NativeClient::hyperliquid(true, None, None).unwrap();
        assert_eq!(hyperliquid.is_testnet(), Some(true));

        let address_only = NativeClient::hyperliquid(
            false,
            Some("0x14791697260e4c9a71f18484c9f997b308e59325".to_owned()),
            None,
        )
        .unwrap();
        assert!(address_only.supports(WireFeature::Balances));
        assert!(!address_only.supports(WireFeature::Trading));

        let signer_only = NativeClient::hyperliquid(
            false,
            None,
            Some("0x0123456789012345678901234567890123456789012345678901234567890123".to_owned()),
        )
        .unwrap();
        assert!(!signer_only.supports(WireFeature::Balances));
        assert!(signer_only.supports(WireFeature::Trading));
    }

    #[test]
    fn factories_reject_half_of_a_credential_pair() {
        let Err(error) = NativeClient::bithumb(Some("key".to_owned()), None) else {
            panic!("half of a credential pair must fail");
        };
        assert_eq!(error.kind, NativeErrorKind::InvalidRequest);
        assert_eq!(error.field.as_deref(), Some("credentials"));
    }

    #[tokio::test]
    async fn provider_specific_method_rejects_the_wrong_handle_before_network_io() {
        let client = NativeClient::upbit(WireUpbitRegion::Korea, None, None).unwrap();
        let error = client.bithumb_market_alerts().await.unwrap_err();
        assert_eq!(error.kind, NativeErrorKind::InvalidRequest);
        assert_eq!(error.field.as_deref(), Some("adapter"));
    }

    #[tokio::test]
    async fn built_in_wallet_calls_forward_to_the_provider_before_network_io() {
        let client = NativeClient::upbit(WireUpbitRegion::Korea, None, None).unwrap();
        let request = WireTransferLookupRequest {
            asset: "BTC".to_owned(),
            id: Some("deposit-1".to_owned()),
            tx_id: None,
        };

        for error in [
            client.deposit(request.clone()).await.unwrap_err(),
            client.withdrawal(request).await.unwrap_err(),
            client
                .cancel_withdrawal("withdrawal-1".to_owned())
                .await
                .unwrap_err(),
        ] {
            assert_eq!(error.kind, NativeErrorKind::Auth);
        }
    }

    #[tokio::test]
    async fn subscribe_reports_the_initial_adapter_error_before_returning_a_stream_handle() {
        let adapter = crate::adapter::register_dart_adapter(
            WireExchange::Binance,
            vec![WireFeature::TradeStream],
            |call| {
                Box::pin(async move {
                    match call {
                        AdapterCall::Subscribe { .. } => Ok(AdapterResult::Error(
                            NativeError::invalid_request("markets", "initial failure"),
                        )),
                        other => panic!("unexpected call: {other:?}"),
                    }
                })
            },
        );
        let client = NativeClient::from_dart_adapter(adapter);
        let subscription = WireSubscription {
            markets: vec![maxt::Market::perpetual(Exchange::Binance, "BTC", "USDT").into()],
            feeds: vec![WireFeed::Trades],
        };

        let Err(error) = client
            .subscribe(subscription, maxt::StreamConfig::default().into())
            .await
        else {
            panic!("initial subscription failure must not return a handle");
        };

        assert_eq!(error.kind, NativeErrorKind::InvalidRequest);
        assert_eq!(error.field.as_deref(), Some("markets"));
    }
}
