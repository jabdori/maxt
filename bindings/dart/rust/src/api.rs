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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireUpbitRegion {
    Korea,
    Singapore,
    Indonesia,
    Thailand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireBinanceVenue {
    Spot,
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

    #[flutter_rust_bridge::frb(sync)]
    pub fn exchange(&self) -> WireExchange {
        self.adapter.exchange().into()
    }

    #[flutter_rust_bridge::frb(sync)]
    pub fn supports(&self, feature: WireFeature) -> bool {
        self.adapter.supports(feature.into())
    }

    #[flutter_rust_bridge::frb(sync)]
    pub fn upbit_region(&self) -> Option<WireUpbitRegion> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Upbit(adapter) => Some(adapter.region().into()),
            _ => None,
        }
    }

    #[flutter_rust_bridge::frb(sync)]
    pub fn binance_venue(&self) -> Option<WireBinanceVenue> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Binance(adapter) => Some(adapter.venue().into()),
            _ => None,
        }
    }

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

    pub async fn bithumb_pending_orders(
        &self,
        request: WireBithumbPendingOrdersRequest,
    ) -> Result<WireOrderPage, NativeError> {
        let request: maxt::BithumbPendingOrdersRequest = request.try_into()?;
        let adapter = match self.built_in("bithumb_pending_orders")? {
            BuiltInAdapter::Bithumb(adapter) => adapter,
            _ => return Err(provider_mismatch("Bithumb")),
        };
        adapter.pending_orders(&request).await.map(Into::into).map_err(Into::into)
    }

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

    pub async fn binance_usd_m_close_listen_key(&self) -> Result<(), NativeError> {
        let adapter = match self.built_in("binance_usd_m_close_listen_key")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter.usd_m_close_listen_key().await.map_err(Into::into)
    }

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
}

/// Binance listen key의 Rust 수명을 유지하면서 공개 값을 읽는 handle입니다.
#[flutter_rust_bridge::frb(opaque)]
pub struct WireBinanceListenKey {
    inner: BinanceListenKey,
}

impl WireBinanceListenKey {
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
        let upbit = NativeClient::upbit(WireUpbitRegion::Singapore, None, None).unwrap();
        assert_eq!(upbit.exchange(), WireExchange::Upbit);
        assert_eq!(upbit.upbit_region(), Some(WireUpbitRegion::Singapore));
        assert!(!upbit.supports(WireFeature::Trading));

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
