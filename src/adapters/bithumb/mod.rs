//! Bithumb spot adapter.

mod parse;
mod private;
mod rest;
mod stream;
mod wallet;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{
    CancelOrdersRequest, CandleRequest, DepositAddressRequest, OrderHistoryRequest,
    OrderLookupRequest, OrderRequest, TransferHistoryRequest, TransferLookupRequest,
    WithdrawRequest,
};
use crate::stream::{AccountStream, MarketStream};
use crate::transport::HttpTransport;
use crate::types::{
    AssetNetwork, Balance, CancelOrdersResult, Candle, Deposit, DepositAddress,
    DepositAddressEntry, Exchange, Market, MarketInfo, MarketKind, Network, Order, OrderBook,
    OrderRules, Page, StreamConfig, Subscription, Ticker, Timestamp, Trade, Withdrawal,
    WithdrawalFee, WithdrawalQuote,
};

pub(crate) const REST_BASE_URL: &str = "https://api.bithumb.com";
pub(crate) const WEBSOCKET_URL: &str = "wss://ws-api.bithumb.com/websocket/v1";
/// Private account events use a separate v2 WebSocket endpoint.
pub(crate) const PRIVATE_WEBSOCKET_URL: &str = "wss://ws-api.bithumb.com/websocket/v2/private";

pub(crate) fn network_from_provider(raw: &str) -> Network {
    match raw.trim().to_ascii_uppercase().as_str() {
        "BTC" | "BITCOIN" => Network::Bitcoin,
        "ETH" | "ETHEREUM" => Network::Ethereum,
        "ARB" | "ARBITRUM" | "ARBITRUM ONE" => Network::Arbitrum,
        "BSC" | "BEP20" | "BNB SMART CHAIN" => Network::BnbSmartChain,
        "TRX" | "TRON" => Network::Tron,
        "SOL" | "SOLANA" => Network::Solana,
        "MATIC" | "POLYGON" | "POLYGON POS" => Network::Polygon,
        "BASE" => Network::Base,
        "OP" | "OPTIMISM" => Network::Optimism,
        "AVAXC" | "AVAX-C" | "AVALANCHE C-CHAIN" => Network::AvalancheC,
        "XRP" | "XRP LEDGER" => Network::XrpLedger,
        "XLM" | "STELLAR" => Network::Stellar,
        "ATOM" | "COSMOS" | "COSMOS HUB" => Network::Cosmos,
        "APT" | "APTOS" => Network::Aptos,
        "SUI" => Network::Sui,
        "TON" => Network::Ton,
        "NEAR" => Network::Near,
        "DOT" | "POLKADOT" => Network::Polkadot,
        _ => Network::Other(raw.trim().to_owned()),
    }
}

/// Severity of a Bithumb market alert (경보제), ordered from least to most severe.
///
/// This is separate from the `CAUTION` investment-warning flag returned by
/// [`BithumbAdapter::market_warnings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BithumbAlertStep {
    /// Caution (주의), the lowest alert level.
    Caution,
    /// Warning (경고), the middle alert level.
    Warning,
    /// Danger (위험), the highest documented alert level.
    Danger,
    /// An unrecognized level, ordered above [`Self::Danger`] so thresholds surface it.
    Unknown,
}

/// An active Bithumb market alert for one market and criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbMarketAlert {
    /// Bithumb's criterion code, preserved verbatim for forward compatibility.
    pub kind: String,
    /// Alert severity.
    pub step: BithumbAlertStep,
    /// Alert expiry, converted from Bithumb's KST wall-clock value to UTC.
    pub ends_at: Timestamp,
}

/// One Bithumb exchange notice.
///
/// Bithumb publishes `published_at` and `modified_at` as Korea Standard Time
/// wall-clock values. They are converted to UTC timestamps at the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbNotice {
    /// Bithumb's notice categories, in the order supplied by the provider.
    pub categories: Vec<String>,
    /// Notice title.
    pub title: String,
    /// Provider-hosted notice URL.
    pub url: String,
    /// Publication time converted from KST to UTC.
    pub published_at: Timestamp,
    /// Most recent modification time converted from KST to UTC.
    pub modified_at: Timestamp,
}

/// One Bithumb asset's public transfer-fee catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbAssetFee {
    /// Bithumb's display name for the asset.
    pub display_name: String,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Fee rules for every network Bithumb returned for this asset.
    pub networks: Vec<BithumbNetworkFee>,
}

/// One Bithumb network's public transfer-fee rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbNetworkFee {
    /// Canonical network when maxt recognizes Bithumb's network name.
    pub network: Network,
    /// Bithumb's exact `net_name` value.
    pub provider_name: String,
    /// Deposit fee in the transferred asset.
    pub deposit_fee: rust_decimal::Decimal,
    /// Minimum deposit amount.
    pub minimum_deposit: rust_decimal::Decimal,
    /// Fixed or rate-based withdrawal fee rule.
    pub withdrawal_fee: WithdrawalFee,
    /// Minimum withdrawal amount.
    pub minimum_withdrawal: rust_decimal::Decimal,
}

/// Adapter for Bithumb spot markets.
///
/// Public REST supports markets, trades, order books, tickers, and candles.
/// Public WebSocket supports trades, order books, and tickers. Derivatives and
/// candle streams return [`Error::Unsupported`](crate::Error::Unsupported).
#[derive(Debug, Clone)]
pub struct BithumbAdapter {
    credentials: Option<BithumbCredentials>,
    // Stored as a result because `new` is infallible.
    http: Result<HttpTransport>,
}

#[derive(Debug, Clone)]
pub(crate) struct BithumbCredentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
}

impl BithumbAdapter {
    /// Creates an adapter for public market data.
    pub fn new() -> Self {
        Self {
            credentials: None,
            http: HttpTransport::new(REST_BASE_URL),
        }
    }

    /// Adds the access key and secret key required by private APIs.
    #[must_use]
    pub fn with_credentials(
        mut self,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(BithumbCredentials {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Returns every listed market with its raw investment-warning flag.
    ///
    /// The value is `NONE` or `CAUTION` (유의 종목). A warned market remains
    /// tradable and maps to [`MarketStatus::Unknown`](crate::MarketStatus::Unknown).
    /// This flag is separate from [`Self::market_alerts`].
    pub async fn market_warnings(&self) -> Result<Vec<(Market, String)>> {
        rest::market_warnings(self.http()?).await
    }

    /// Returns active alert-system rows, one per market and criterion.
    ///
    /// Markets without alerts are omitted. Alerts do not change the common
    /// [`MarketStatus`](crate::MarketStatus).
    pub async fn market_alerts(&self) -> Result<Vec<(Market, BithumbMarketAlert)>> {
        rest::market_alerts(self.http()?).await
    }

    /// Returns the newest Bithumb exchange notices first.
    ///
    /// `count` must be from 1 through 20. `None` uses Bithumb's documented
    /// five-notice default.
    pub async fn notices(&self, count: Option<u32>) -> Result<Vec<BithumbNotice>> {
        rest::notices(self.http()?, count).await
    }

    /// Returns Bithumb's public transfer-fee catalog for one asset or `ALL`.
    ///
    /// The returned values are fee rules, not account-specific withdrawal
    /// availability. Use [`Adapter::asset_networks`] for an authenticated
    /// account's current transfer status and limits.
    pub async fn transfer_fees(&self, currency: &str) -> Result<Vec<BithumbAssetFee>> {
        rest::transfer_fees(self.http()?, currency).await
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    /// Returns credentials or an authentication error before network I/O.
    fn credentials(&self) -> Result<&BithumbCredentials> {
        self.credentials
            .as_ref()
            .ok_or_else(|| Error::auth("bithumb needs both an access key and a secret key"))
    }

    pub(crate) fn http(&self) -> Result<&HttpTransport> {
        self.http.as_ref().map_err(Clone::clone)
    }
}

impl Default for BithumbAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for BithumbAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Bithumb
    }

    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() {
            return false;
        }
        // Bithumb does not publish a public candle stream.
        if matches!(feature, Feature::CandleStream) {
            return false;
        }
        if feature.needs_credentials() {
            return self.is_authenticated();
        }
        true
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move { rest::markets(self.http()?, kind).await })
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let market = market.clone();
        Box::pin(async move { rest::trades(self.http()?, &market, limit).await })
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let market = market.clone();
        Box::pin(async move { rest::order_book(self.http()?, &market, depth).await })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move { rest::ticker(self.http()?, &market).await })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let request = request.clone();
        Box::pin(async move { rest::candles(self.http()?, &request).await })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        let subscription = subscription.clone();
        let config = config.clone();
        Box::pin(async move { stream::subscribe(&subscription, &config).await })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let config = config.clone();
        Box::pin(async move { stream::subscribe_account(self.credentials()?, &config).await })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self.http()?, self.credentials()?).await })
    }

    fn order_rules(&self, market: &Market) -> BoxFuture<'_, Result<OrderRules>> {
        let market = market.clone();
        Box::pin(
            async move { private::order_rules(self.http()?, self.credentials()?, &market).await },
        )
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        let asset = asset.to_string();
        Box::pin(
            async move { wallet::asset_networks(self.http()?, self.credentials()?, &asset).await },
        )
    }

    fn deposit_addresses(&self) -> BoxFuture<'_, Result<Vec<DepositAddressEntry>>> {
        Box::pin(async move { wallet::deposit_addresses(self.http()?, self.credentials()?).await })
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::deposit_address(self.http()?, self.credentials()?, &request).await
        })
    }

    fn create_deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::create_deposit_address(self.http()?, self.credentials()?, &request).await
        })
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::prepare_withdrawal(self.http()?, self.credentials()?, &request).await
        })
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(async move { wallet::withdraw(self.http()?, self.credentials()?, &request).await })
    }

    fn deposit(&self, request: &TransferLookupRequest) -> BoxFuture<'_, Result<Deposit>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposit(self.http()?, self.credentials()?, &request).await })
    }

    fn withdrawal(&self, request: &TransferLookupRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(
            async move { wallet::withdrawal(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn cancel_withdrawal(&self, withdrawal_id: &str) -> BoxFuture<'_, Result<()>> {
        let withdrawal_id = withdrawal_id.to_owned();
        Box::pin(async move {
            wallet::cancel_withdrawal(self.http()?, self.credentials()?, &withdrawal_id).await
        })
    }

    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposits(self.http()?, self.credentials()?, &request).await })
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        let request = request.clone();
        Box::pin(
            async move { wallet::withdrawals(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move {
            private::open_orders(self.http()?, self.credentials()?, market.as_ref()).await
        })
    }

    fn order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::order(self.http()?, self.credentials()?, &market, &order_id).await
        })
    }

    fn order_by_client_id(&self, market: &Market, client_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let client_id = client_id.to_string();
        Box::pin(async move {
            private::order_by_client_id(self.http()?, self.credentials()?, &market, &client_id)
                .await
        })
    }

    fn orders_by_ids(&self, request: &OrderLookupRequest) -> BoxFuture<'_, Result<Vec<Order>>> {
        let request = request.clone();
        Box::pin(async move {
            private::orders_by_ids(self.http()?, self.credentials()?, &request).await
        })
    }

    fn order_history(&self, request: &OrderHistoryRequest) -> BoxFuture<'_, Result<Page<Order>>> {
        let request = request.clone();
        Box::pin(async move {
            private::order_history(self.http()?, self.credentials()?, &request).await
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(
            async move { private::place_order(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::cancel_order(self.http()?, self.credentials()?, &market, &order_id).await
        })
    }

    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let client_id = client_id.to_string();
        Box::pin(async move {
            private::cancel_order_by_client_id(
                self.http()?,
                self.credentials()?,
                &market,
                &client_id,
            )
            .await
        })
    }

    fn cancel_orders(
        &self,
        request: &CancelOrdersRequest,
    ) -> BoxFuture<'_, Result<CancelOrdersResult>> {
        let request = request.clone();
        Box::pin(async move {
            private::cancel_orders(self.http()?, self.credentials()?, &request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candles_are_available_over_rest_but_not_as_a_stream() {
        let adapter = BithumbAdapter::new();

        assert!(adapter.supports(Feature::Candles));
        assert!(!adapter.supports(Feature::CandleStream));
    }

    #[test]
    fn every_other_public_stream_is_available() {
        let adapter = BithumbAdapter::new();

        for feature in [
            Feature::TradeStream,
            Feature::OrderBookStream,
            Feature::TickerStream,
        ] {
            assert!(adapter.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn a_spot_exchange_never_claims_derivatives_features() {
        let adapter = BithumbAdapter::new().with_credentials("access", "secret");

        for feature in [Feature::Positions, Feature::Margin, Feature::FundingRates] {
            assert!(!adapter.supports(feature), "{feature:?}");
        }
    }

    #[tokio::test]
    async fn subscribing_to_candles_is_refused_before_a_socket_is_opened() {
        use crate::types::{Feed, Interval, Market, StreamConfig, Subscription};

        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .feed(Feed::Candles(Interval::Min1));

        let error = BithumbAdapter::new()
            .subscribe(&subscription, &StreamConfig::default())
            .await
            .expect_err("bithumb publishes no candle stream");

        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::CandleStream,
                exchange: "bithumb",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn a_private_call_without_credentials_is_an_auth_failure_not_a_missing_feature() {
        let error = BithumbAdapter::new()
            .balances()
            .await
            .expect_err("no credentials were supplied");

        // The feature exists, but the request lacks credentials.
        assert!(
            matches!(error, Error::Auth { .. }),
            "expected an auth failure, got {error:?}"
        );
    }

    #[test]
    fn credentials_are_what_unlock_the_private_half() {
        let public = BithumbAdapter::new();
        let private = BithumbAdapter::new().with_credentials("access", "secret");

        for feature in [
            Feature::Balances,
            Feature::AssetNetworks,
            Feature::DepositAddresses,
            Feature::DepositHistory,
            Feature::DepositLookup,
            Feature::WithdrawalQuotes,
            Feature::Withdrawals,
            Feature::WithdrawalHistory,
            Feature::WithdrawalLookup,
            Feature::WithdrawalCancellation,
            Feature::Trading,
            Feature::AccountStream,
        ] {
            assert!(!public.supports(feature), "{feature:?}");
            assert!(private.supports(feature), "{feature:?}");
        }
    }
}
