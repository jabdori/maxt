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
include!("generated_provider_methods.rs");

pub use crate::adapter::{
    AdapterCall, AdapterReply, AdapterResult, DartAdapter, WireFeed, WireOverflow,
    WireStreamConfig, WireSubscription,
};
pub use crate::convert::*;
pub use crate::stream::{
    AccountStreamItem, AccountStreamSink, MarketStreamItem, MarketStreamSink,
    NativeAccountSubscription, NativeHyperliquidAccountSubscription,
    NativeHyperliquidMarketSubscription, NativeMarketSubscription, WireAccountEvent,
    WireAccountStreamItem, WireHyperliquidAccountStreamItem, WireHyperliquidMarketStreamItem,
    WireMarketEvent, WireMarketStreamItem,
};

/// Version of the installed Dart/Rust boundary.
#[flutter_rust_bridge::frb(sync)]
pub fn bridge_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Configures the relay origin for browser HTTP and WebSocket requests.
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

/// Creates a pending subscription for Dart native-stream termination tests.
#[doc(hidden)]
#[flutter_rust_bridge::frb(sync)]
pub fn pending_market_subscription_for_test() -> NativeMarketSubscription {
    crate::stream::pending_market_subscription_for_test()
}

/// Delivers one item to a market stream created by a Dart Adapter.
pub async fn market_stream_sink_add(sink: &MarketStreamSink, item: MarketStreamItem) -> bool {
    sink.add(item).await
}

/// Delivers one item to an account stream created by a Dart Adapter.
pub async fn account_stream_sink_add(sink: &AccountStreamSink, item: AccountStreamItem) -> bool {
    sink.add(item).await
}

/// Reads the next event, error, or end item from a native market subscription.
pub async fn native_market_subscription_next(
    subscription: &NativeMarketSubscription,
) -> WireMarketStreamItem {
    subscription.next().await
}

/// Reads the next event, error, or end item from a native account subscription.
pub async fn native_account_subscription_next(
    subscription: &NativeAccountSubscription,
) -> WireAccountStreamItem {
    subscription.next().await
}

/// Stops a native market subscription and waits for source Rust stream cleanup.
pub async fn native_market_subscription_close(
    subscription: &NativeMarketSubscription,
) -> Result<(), NativeError> {
    subscription.close().await
}

/// Stops a native account subscription and waits for source Rust stream cleanup.
pub async fn native_account_subscription_close(
    subscription: &NativeAccountSubscription,
) -> Result<(), NativeError> {
    subscription.close().await
}

/// Reads the next event, error, or end item from a native Hyperliquid market subscription.
pub async fn native_hyperliquid_market_subscription_next(
    subscription: &NativeHyperliquidMarketSubscription,
) -> WireHyperliquidMarketStreamItem {
    subscription.next().await
}

/// Reads the next event, error, or end item from a native Hyperliquid account subscription.
pub async fn native_hyperliquid_account_subscription_next(
    subscription: &NativeHyperliquidAccountSubscription,
) -> WireHyperliquidAccountStreamItem {
    subscription.next().await
}

/// Stops a native Hyperliquid market subscription and waits for source Rust stream cleanup.
pub async fn native_hyperliquid_market_subscription_close(
    subscription: &NativeHyperliquidMarketSubscription,
) -> Result<(), NativeError> {
    subscription.close().await
}

/// Stops a native Hyperliquid account subscription and waits for source Rust stream cleanup.
pub async fn native_hyperliquid_account_subscription_close(
    subscription: &NativeHyperliquidAccountSubscription,
) -> Result<(), NativeError> {
    subscription.close().await
}

/// Registers a Dart callback as a common Adapter implementation.
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

/// Value used to select an Upbit API region from Dart.
///
/// Pocket and KRW transfer APIs are available only in `Korea`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireUpbitRegion {
    /// Upbit Korea API.
    Korea,
    /// Upbit Singapore API.
    Singapore,
    /// Upbit Indonesia API.
    Indonesia,
    /// Upbit Thailand API.
    Thailand,
}

/// Binance venue configured from Dart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireBinanceVenue {
    /// Binance Spot API.
    Spot,
    /// Binance USD-M Futures API.
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

/// Native handle shared by Dart's common Client and provider-specific Adapters.
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

    /// Wraps a Dart-implemented Adapter as a common native Client.
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

    /// Configures an Upbit region and optional credentials.
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

    /// Configures optional Bithumb credentials.
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

    /// Configures optional Binance Spot credentials.
    #[flutter_rust_bridge::frb(sync)]
    pub fn binance_spot(
        api_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, NativeError> {
        Self::binance(BinanceAdapter::spot(), api_key, secret_key).map_err(Into::into)
    }

    /// Configures optional Binance USD-M credentials.
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

    /// Configures Hyperliquid mainnet or testnet and an optional wallet.
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

    /// Returns the exchange connected by this native handle.
    #[flutter_rust_bridge::frb(sync)]
    pub fn exchange(&self) -> WireExchange {
        self.adapter.exchange().into()
    }

    /// Returns whether this native handle supports a common feature.
    #[flutter_rust_bridge::frb(sync)]
    pub fn supports(&self, feature: WireFeature) -> bool {
        self.adapter.supports(feature.into())
    }

    /// Returns the selected region for an Upbit handle, otherwise null.
    #[flutter_rust_bridge::frb(sync)]
    pub fn upbit_region(&self) -> Option<WireUpbitRegion> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Upbit(adapter) => Some(adapter.region().into()),
            _ => None,
        }
    }

    /// Returns the selected venue for a Binance handle, otherwise null.
    #[flutter_rust_bridge::frb(sync)]
    pub fn binance_venue(&self) -> Option<WireBinanceVenue> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Binance(adapter) => Some(adapter.venue().into()),
            _ => None,
        }
    }

    /// Returns whether a Hyperliquid handle uses testnet, otherwise null.
    #[flutter_rust_bridge::frb(sync)]
    pub fn is_testnet(&self) -> Option<bool> {
        match self.built_in.as_deref()? {
            BuiltInAdapter::Hyperliquid(adapter) => Some(adapter.is_testnet()),
            _ => None,
        }
    }

    /// Returns a market subscription that Dart can read item by item after connecting.
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

    /// Returns an account subscription that Dart can read item by item after connecting.
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

    /// Returns a Hyperliquid-native market subscription that Dart can read after connecting.
    pub async fn hyperliquid_subscribe_detailed(
        &self,
        subscription: WireSubscription,
    ) -> Result<NativeHyperliquidMarketSubscription, NativeError> {
        let subscription = subscription.into();
        let adapter = match self.built_in("hyperliquid_subscribe_detailed")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .subscribe_detailed(&subscription)
            .await
            .map(NativeHyperliquidMarketSubscription::new)
            .map_err(Into::into)
    }

    /// Returns a configured Hyperliquid-native market subscription that Dart can read after connecting.
    pub async fn hyperliquid_subscribe_detailed_with(
        &self,
        subscription: WireSubscription,
        config: WireStreamConfig,
    ) -> Result<NativeHyperliquidMarketSubscription, NativeError> {
        let subscription = subscription.into();
        let config = config.into();
        let adapter = match self.built_in("hyperliquid_subscribe_detailed_with")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .subscribe_detailed_with(&subscription, &config)
            .await
            .map(NativeHyperliquidMarketSubscription::new)
            .map_err(Into::into)
    }

    /// Returns a Hyperliquid-native account subscription that Dart can read after connecting.
    pub async fn hyperliquid_subscribe_detailed_account(
        &self,
    ) -> Result<NativeHyperliquidAccountSubscription, NativeError> {
        let adapter = match self.built_in("hyperliquid_subscribe_detailed_account")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .subscribe_detailed_account()
            .await
            .map(NativeHyperliquidAccountSubscription::new)
            .map_err(Into::into)
    }

    /// Returns a configured Hyperliquid-native account subscription that Dart can read after connecting.
    pub async fn hyperliquid_subscribe_detailed_account_with(
        &self,
        config: WireStreamConfig,
    ) -> Result<NativeHyperliquidAccountSubscription, NativeError> {
        let config = config.into();
        let adapter = match self.built_in("hyperliquid_subscribe_detailed_account_with")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .subscribe_detailed_account_with(&config)
            .await
            .map(NativeHyperliquidAccountSubscription::new)
            .map_err(Into::into)
    }

    /// Returns order-book snapshots for multiple Upbit Spot markets.
    ///
    /// `depth` is the maximum number of levels on each bid and ask side.
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

    /// Returns Upbit order-book snapshots grouped by a specified price unit.
    ///
    /// The price unit must be currently supported by each market.
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

    /// Returns the ticker summary for a specified Upbit Spot market.
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

    /// Queries Upbit tickers for one or more quote assets.
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

    /// Returns yearly candles for one Upbit market, oldest first.
    ///
    /// `count` is 1 through 200 and `toNs` is an exclusive end time.
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

    /// Returns the current order-book unit and supported price units for an Upbit market.
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

    /// Returns investment-warning and caution information for Upbit markets.
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

    /// Returns items actually subscribed by a matching active Upbit connection.
    ///
    /// Start a subscription with the same selector first and keep its returned stream running.
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

    /// Returns withdrawal allowlist addresses registered on the Upbit account.
    pub async fn upbit_withdrawal_addresses(
        &self,
    ) -> Result<Vec<WireUpbitWithdrawalAddress>, NativeError> {
        let adapter = match self.built_in("upbit_withdrawal_addresses")? {
            BuiltInAdapter::Upbit(adapter) => adapter,
            _ => return Err(provider_mismatch("Upbit")),
        };
        adapter
            .withdrawal_addresses()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Validates an Upbit order without submitting it.
    ///
    /// Returned order ID and status do not represent a live order and cannot be queried or cancelled.
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

    /// Returns Upbit order details including fills, fees, and self-match prevention information.
    ///
    /// When both UUID and client order identifier are supplied, Upbit prioritizes UUID.
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

    /// Returns closed Upbit orders that match the request.
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

    /// Returns Upbit deposit availability for one asset and network.
    ///
    /// This information is not real-time service state and can lag by minutes.
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

    /// Returns Travel Rule VASPs eligible for verification in Upbit Korea or Singapore.
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

    /// Requests Upbit Travel Rule verification by deposit UUID.
    ///
    /// This is a financial write; Upbit limits repeated requests for the same deposit.
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

    /// Requests Upbit Travel Rule verification by transaction ID.
    ///
    /// This is a financial write; Upbit limits repeated requests for the same deposit.
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

    /// Cancels matching pending Upbit orders in one request.
    ///
    /// The result distinguishes completed cancellations from orders no longer cancellable after state changes.
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

    /// Cancels an Upbit order and requests its replacement.
    ///
    /// A successful response can omit the replacement when the original order fills first.
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

    /// Requests a KRW deposit from a registered Upbit Korea account.
    ///
    /// This is a Korea-region-only financial write.
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

    /// Requests a KRW withdrawal to a registered Upbit Korea account.
    ///
    /// This Korea-only financial write can be rejected by a withdrawal safety lock.
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

    /// Returns API-key identifiers and expiration times registered for this Upbit Korea account.
    ///
    /// Secret-key material is never returned.
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

    /// Returns pockets visible to the Upbit Korea API key.
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

    /// Returns API keys for an Upbit Korea pocket.
    ///
    /// The request can filter by pocket UUID and whether to include expired keys.
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

    /// Returns asset balances for one Upbit Korea sub-pocket.
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

    /// Requests an asset transfer between Upbit Korea main pockets.
    ///
    /// This financial write requires destination pocket `to` under the current OpenAPI contract.
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

    /// Returns Upbit Korea main-pocket transfer history.
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

    /// Requests an asset transfer from the current Upbit Korea sub-pocket to another pocket.
    ///
    /// This financial write requires destination pocket `to` under the current OpenAPI contract.
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

    /// Returns transfer history for the current Upbit Korea sub-pocket.
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

    /// Returns native investment-caution flags for Bithumb markets.
    ///
    /// A caution-marked market can still trade; this is separate from common market status.
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

    /// Returns market-specific alerts enabled by Bithumb.
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

    /// Returns the newest Bithumb exchange notices first.
    ///
    /// `count` is 1 through 20; omission uses the Bithumb default.
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

    /// Returns Bithumb transfer-fee rules for one asset or `ALL`.
    ///
    /// This response does not include account-specific withdrawal availability or limits.
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

    /// Returns API-key information registered on this Bithumb account.
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

    /// Returns Bithumb withdrawal-address information registered on this account.
    ///
    /// This is a provider-registered address list, not a withdrawal quote or preflight result.
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

    /// Returns one order with Bithumb-specific details such as fills, fees, and STP.
    ///
    /// When UUID and client order ID are both supplied, Bithumb prioritizes UUID;
    /// the requested market is checked locally against the returned order.
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

    /// Returns Bithumb orders matching state, identifier, and page conditions.
    ///
    /// `state` and `states` are mutually exclusive; UUIDs take priority over client order IDs.
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

    /// Returns Bithumb KRW withdrawal history.
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

    /// Requests a Bithumb KRW withdrawal.
    ///
    /// This financial write can require a registered account and provider-side second factor.
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

    /// Returns Bithumb KRW deposit history.
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

    /// Requests a Bithumb KRW deposit.
    ///
    /// This financial write validates bank-account and second-factor requirements at the provider.
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

    /// Returns a Bithumb page of orders in `wait` or `watch` state.
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

    /// Returns a page of closed Bithumb orders matching the request.
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

    /// Submits up to 20 Bithumb orders together and returns per-item results.
    ///
    /// This is a financial write; some items can be rejected despite HTTP success.
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

    /// Returns a page of Bithumb TWAP orders.
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

    /// Creates a Bithumb TWAP order and returns its provider order identifier.
    ///
    /// This is a financial write request.
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

    /// Cancels a Bithumb TWAP order and returns its cancelled identifier.
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

    /// Returns price, quantity, and notional constraints for one Binance Spot market.
    ///
    /// Not supported by a USD-M Futures handle.
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

    /// Returns Binance Spot order details by numeric order ID.
    ///
    /// Includes filled and cancelled orders; not supported by a USD-M Futures handle.
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

    /// Returns the current average price for one Binance Spot market.
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

    /// Returns permissions, commissions, and balances for a Binance Spot account.
    pub async fn binance_spot_account_information(
        &self,
    ) -> Result<WireBinanceSpotAccountInformation, NativeError> {
        let adapter = match self.built_in("binance_spot_account_information")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .spot_account_information()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Cancels all open orders in a Binance Spot market and returns a report.
    pub async fn binance_spot_cancel_all_open_orders(
        &self,
        market: WireMarket,
    ) -> Result<WireBinanceSpotCancelAllOpenOrders, NativeError> {
        let adapter = match self.built_in("binance_spot_cancel_all_open_orders")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .spot_cancel_all_open_orders(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Binance Spot exchange metadata.
    pub async fn binance_spot_exchange_info(&self) -> Result<WireBinanceExchangeInfo, NativeError> {
        let adapter = match self.built_in("binance_spot_exchange_info")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .spot_exchange_info()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns margin, asset, and position information for a Binance USD-M account.
    pub async fn binance_usd_m_account_information(
        &self,
    ) -> Result<WireBinanceUsdMAccountInformation, NativeError> {
        let adapter = match self.built_in("binance_usd_m_account_information")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .usd_m_account_information()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Binance USD-M exchange metadata.
    pub async fn binance_usd_m_exchange_info(
        &self,
    ) -> Result<WireBinanceExchangeInfo, NativeError> {
        let adapter = match self.built_in("binance_usd_m_exchange_info")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .usd_m_exchange_info()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Binance USD-M position risk information.
    pub async fn binance_usd_m_position_information(
        &self,
        market: Option<WireMarket>,
    ) -> Result<Vec<WireBinanceUsdMPositionInformation>, NativeError> {
        let adapter = match self.built_in("binance_usd_m_position_information")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        let market = market.map(Into::into);
        adapter
            .usd_m_position_information(market.as_ref())
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns Binance Wallet coin and network configuration.
    pub async fn binance_all_coins_information(
        &self,
    ) -> Result<Vec<WireBinanceCoinInformation>, NativeError> {
        let adapter = match self.built_in("binance_all_coins_information")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .all_coins_information()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns permission state for the Binance API key.
    pub async fn binance_api_key_permissions(
        &self,
    ) -> Result<WireBinanceApiKeyPermissions, NativeError> {
        let adapter = match self.built_in("binance_api_key_permissions")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .api_key_permissions()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Binance deposit history.
    pub async fn binance_deposit_history(
        &self,
        request: WireBinanceDepositHistoryRequest,
    ) -> Result<WireBinanceDepositHistory, NativeError> {
        let request: maxt::BinanceDepositHistoryRequest = request.try_into()?;
        let adapter = match self.built_in("binance_deposit_history")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .deposit_history(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Binance Travel Rule questionnaire country requirements.
    pub async fn binance_questionnaire_requirements(
        &self,
    ) -> Result<WireBinanceQuestionnaireRequirements, NativeError> {
        let adapter = match self.built_in("binance_questionnaire_requirements")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .questionnaire_requirements()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns withdrawal addresses registered with Binance.
    pub async fn binance_withdraw_address_list(
        &self,
    ) -> Result<Vec<WireBinanceWithdrawalAddress>, NativeError> {
        let adapter = match self.built_in("binance_withdraw_address_list")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .withdraw_address_list()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns Binance withdrawal history.
    pub async fn binance_withdraw_history(
        &self,
        request: WireBinanceWithdrawHistoryRequest,
    ) -> Result<WireBinanceWithdrawHistory, NativeError> {
        let request: maxt::BinanceWithdrawHistoryRequest = request.try_into()?;
        let adapter = match self.built_in("binance_withdraw_history")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter
            .withdraw_history(&request)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns current mark price and funding information for one Binance USD-M market.
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

    /// Returns current mark prices for all Binance USD-M perpetual markets.
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

    /// Returns current open interest for one Binance USD-M market.
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

    /// Returns price- and direction-aggregated Binance USD-M trades.
    ///
    /// This is not an individual-trade list and needs no credentials because it is public.
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

    /// Returns a page of Binance account trades.
    ///
    /// It does not create a common cursor, even for a full page; use the provider trade ID to continue.
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

    /// Returns the Binance Spot Funding Wallet C2C trade-history response unchanged.
    ///
    /// Increase `page` to continue; this preserves the provider envelope rather than creating a common cursor.
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

    /// Validates a Binance order without creating it.
    ///
    /// Spot commonly returns an empty object; USD-M can return an order-shaped object.
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

    /// Cancels all open orders in one Binance market.
    ///
    /// Spot and USD-M response shapes differ, so only success is returned.
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

    /// Creates a Binance USD-M user-data-stream listen key.
    ///
    /// [subscribe_account] normally manages this lifecycle.
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

    /// Extends the Binance USD-M listen key owned by the current API key.
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

    /// Closes the Binance USD-M listen key owned by the current API key.
    pub async fn binance_usd_m_close_listen_key(&self) -> Result<(), NativeError> {
        let adapter = match self.built_in("binance_usd_m_close_listen_key")? {
            BuiltInAdapter::Binance(adapter) => adapter,
            _ => return Err(provider_mismatch("Binance")),
        };
        adapter.usd_m_close_listen_key().await.map_err(Into::into)
    }

    /// Returns a page of non-funding ledger entries for the configured Hyperliquid account.
    ///
    /// It needs only an address, does not use a signature, and keeps provider cursor and time-range semantics.
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

    /// Returns recent fills for the configured Hyperliquid address.
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

    /// Returns fills in a specified time range for the configured Hyperliquid address.
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

    /// Returns compact current open orders for the configured Hyperliquid address.
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

    /// Returns Hyperliquid order status by server order ID or client order ID.
    ///
    /// The adapter validates the address first, so a missing-address error is returned before an invalid client order ID.
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

    /// Returns recent order history for the configured Hyperliquid address.
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

    /// Returns current price, funding, and precision information for one Hyperliquid market.
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

    /// Returns a Hyperliquid candle snapshot and trade count.
    pub async fn hyperliquid_candle_snapshot(
        &self,
        market: WireMarket,
        interval: String,
        from_ns: i64,
        to_ns: Option<i64>,
    ) -> Result<Vec<WireHyperliquidCandleSnapshot>, NativeError> {
        let adapter = match self.built_in("hyperliquid_candle_snapshot")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .candle_snapshot(
                &market.into(),
                &interval,
                maxt::Timestamp::from_nanos(from_ns),
                to_ns.map(maxt::Timestamp::from_nanos),
            )
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns Hyperliquid L2 book data and order count per level.
    pub async fn hyperliquid_l2_book(
        &self,
        market: WireMarket,
    ) -> Result<WireHyperliquidL2Book, NativeError> {
        let adapter = match self.built_in("hyperliquid_l2_book")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .l2_book(&market.into())
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns recent Hyperliquid fills with hash and participant information.
    pub async fn hyperliquid_recent_trades(
        &self,
        market: WireMarket,
    ) -> Result<Vec<WireHyperliquidRecentTrade>, NativeError> {
        let adapter = match self.built_in("hyperliquid_recent_trades")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .recent_trades(&market.into())
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns Hyperliquid funding history and premium values.
    pub async fn hyperliquid_funding_history(
        &self,
        market: WireMarket,
        from_ns: i64,
        to_ns: Option<i64>,
    ) -> Result<Vec<WireHyperliquidFundingHistoryEntry>, NativeError> {
        let adapter = match self.built_in("hyperliquid_funding_history")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .funding_history(
                &market.into(),
                maxt::Timestamp::from_nanos(from_ns),
                to_ns.map(maxt::Timestamp::from_nanos),
            )
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns funding entries for the Hyperliquid account.
    pub async fn hyperliquid_user_funding(
        &self,
        from_ns: i64,
        to_ns: Option<i64>,
    ) -> Result<Vec<WireHyperliquidUserFunding>, NativeError> {
        let adapter = match self.built_in("hyperliquid_user_funding")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .user_funding(
                maxt::Timestamp::from_nanos(from_ns),
                to_ns.map(maxt::Timestamp::from_nanos),
            )
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Returns Hyperliquid Spot account balance state.
    pub async fn hyperliquid_spot_clearinghouse_state(
        &self,
    ) -> Result<WireHyperliquidSpotClearinghouseState, NativeError> {
        let adapter = match self.built_in("hyperliquid_spot_clearinghouse_state")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .spot_clearinghouse_state()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Hyperliquid Spot token and pair metadata.
    pub async fn hyperliquid_spot_meta(&self) -> Result<WireHyperliquidSpotMeta, NativeError> {
        let adapter = match self.built_in("hyperliquid_spot_meta")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .spot_meta()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns Hyperliquid Spot metadata together with asset contexts.
    pub async fn hyperliquid_spot_meta_and_asset_contexts(
        &self,
    ) -> Result<WireHyperliquidSpotMetaAndAssetContexts, NativeError> {
        let adapter = match self.built_in("hyperliquid_spot_meta_and_asset_contexts")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter
            .spot_meta_and_asset_contexts()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns current mid prices for default Hyperliquid Spot and perpetual markets.
    ///
    /// HIP-3 DEX markets are not included in this adapter's market table.
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

    /// Returns the current Info API request limit for the configured Hyperliquid address.
    ///
    /// This is a public account read that requires only an address.
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

    /// Returns provider roles for the configured Hyperliquid address.
    ///
    /// Unknown role names are preserved as raw values.
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

    /// Returns referral-program state for the configured Hyperliquid address.
    pub async fn hyperliquid_referral(&self) -> Result<WireHyperliquidReferral, NativeError> {
        let adapter = match self.built_in("hyperliquid_referral")? {
            BuiltInAdapter::Hyperliquid(adapter) => adapter,
            _ => return Err(provider_mismatch("Hyperliquid")),
        };
        adapter.referral().await.map(Into::into).map_err(Into::into)
    }

    /// Returns provider fee schedule for the configured Hyperliquid address.
    ///
    /// Also preserves detailed tier information that the provider can extend.
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

    /// Returns provider period-based portfolio history for the configured Hyperliquid address.
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

    /// Returns subaccounts for the configured Hyperliquid address.
    ///
    /// Returns an empty list when the provider sends null for no account.
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

    /// Returns current vault equity for the configured Hyperliquid address.
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

/// Handle that reads a public value while retaining the Rust lifetime of a Binance listen key.
#[flutter_rust_bridge::frb(opaque)]
pub struct WireBinanceListenKey {
    inner: BinanceListenKey,
}

impl WireBinanceListenKey {
    /// Returns the owned Binance listen-key string.
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
