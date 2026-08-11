//! The common API.

use crate::adapter::Adapter;
use crate::error::Result;
use crate::feature::Feature;
use crate::request::{
    CancelOrdersRequest, CandleRequest, DepositAddressRequest, HistoryRequest, MarginRequest,
    OrderHistoryRequest, OrderLookupRequest, OrderRequest, TransferHistoryRequest, WithdrawRequest,
};
use crate::stream::{AccountStream, MarketStream};
use crate::types::{
    AssetNetwork, Balance, CancelOrdersResult, Candle, Deposit, DepositAddress,
    DepositAddressEntry, Exchange, FundingPayment, FundingRate, MarginSummary, Market, MarketInfo,
    MarketKind, Order, OrderBook, OrderRules, Page, Position, StreamConfig, Subscription, Ticker,
    Trade, Withdrawal, WithdrawalQuote,
};

/// The common API over one exchange adapter.
///
/// Exchange-specific operations are available through [`Client::adapter`].
#[derive(Debug, Clone)]
pub struct Client<A> {
    adapter: A,
}

impl<A: Adapter> Client<A> {
    /// Wraps an adapter.
    ///
    /// Credentials are configured on the adapter before it is wrapped.
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    /// Which exchange this client talks to.
    ///
    /// This identifies the exchange, not an exchange-specific venue.
    pub fn exchange(&self) -> Exchange {
        self.adapter.exchange()
    }

    /// Whether this configured client offers a feature.
    ///
    /// Credential-dependent features return `false` until credentials are
    /// configured. A call may still fail for request-specific validation or an
    /// exchange-side rejection.
    pub fn supports(&self, feature: Feature) -> bool {
        self.adapter.supports(feature)
    }

    /// The underlying adapter, for exchange-specific operations.
    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    /// Unwraps this client and returns its adapter.
    pub fn into_adapter(self) -> A {
        self.adapter
    }

    /// Lists the exchange's markets of one kind.
    ///
    /// A venue that lists none of the requested kind returns an empty vector.
    pub async fn markets(&self, kind: MarketKind) -> Result<Vec<MarketInfo>> {
        self.adapter.markets(kind).await
    }

    /// Reads the most recent trades on a market, newest first.
    ///
    /// `limit` caps the number returned. `None` uses the exchange default.
    ///
    /// # Errors
    ///
    /// Built-in adapters return
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) when `limit` is
    /// zero or exceeds the exchange's per-request limit.
    pub async fn trades(&self, market: &Market, limit: Option<u32>) -> Result<Vec<Trade>> {
        self.adapter.trades(market, limit).await
    }

    /// Reads an order book snapshot.
    ///
    /// `depth` is the maximum number of levels per side. `None` uses the
    /// exchange default.
    ///
    /// # Errors
    ///
    /// Built-in adapters return
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) when `depth` is
    /// zero or unsupported by the exchange.
    ///
    /// Returned bids and asks are best-first; see [`OrderBook`](crate::OrderBook).
    pub async fn order_book(&self, market: &Market, depth: Option<u32>) -> Result<OrderBook> {
        self.adapter.order_book(market, depth).await
    }

    /// Reads a provider ticker summary for one market.
    ///
    /// Fields the exchange does not publish are `None`; see [`Ticker`].
    pub async fn ticker(&self, market: &Market) -> Result<Ticker> {
        self.adapter.ticker(market).await
    }

    /// Reads historical candles, oldest first.
    ///
    /// [`CandleRequest::limit`] may span multiple responses. One request makes
    /// at most 100 exchange calls.
    ///
    /// A request estimated to exceed that bound returns
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) before the first
    /// exchange call. Use bounded `from`/`limit` batches for longer histories.
    pub async fn candles(&self, request: &CandleRequest) -> Result<Vec<Candle>> {
        self.adapter.candles(request).await
    }

    /// Opens a live market data subscription with default connection settings.
    ///
    /// Browser WebAssembly uses [`Overflow::DropNewest`](crate::Overflow::DropNewest)
    /// because the browser WebSocket API cannot pause inbound delivery. Native
    /// clients retain the default backpressure policy. Use
    /// [`Client::subscribe_with`] for an explicit policy; browser WebSockets
    /// reject [`Overflow::Backpressure`](crate::Overflow::Backpressure).
    ///
    /// Use [`Client::subscribe_with`] to change reconnect and buffering
    /// behaviour. See [`MarketStream`] for item and termination semantics.
    pub async fn subscribe(&self, subscription: &Subscription) -> Result<MarketStream> {
        self.subscribe_with(subscription, &default_stream_config())
            .await
    }

    /// Opens a live market data subscription with explicit connection settings.
    ///
    /// One subscription may use more than one underlying connection when an
    /// adapter must split feeds across endpoints. Reconnect budgets and notices
    /// then apply per connection.
    pub async fn subscribe_with(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> Result<MarketStream> {
        self.adapter.subscribe(subscription, config).await
    }

    /// Reads the account's balances.
    ///
    /// Requires credentials.
    ///
    /// # Errors
    ///
    /// An adapter built without credentials fails with
    /// [`Error::Auth`](crate::Error::Auth) before anything is sent, on every
    /// exchange.
    pub async fn balances(&self) -> Result<Vec<Balance>> {
        self.adapter.balances().await
    }

    /// Reads current order fees, limits, supported combinations, and balances.
    ///
    /// Requires credentials.
    pub async fn order_rules(&self, market: &Market) -> Result<OrderRules> {
        self.adapter.order_rules(market).await
    }

    /// Reads live deposit and withdrawal rules for one asset.
    pub async fn asset_networks(&self, asset: &str) -> Result<Vec<AssetNetwork>> {
        if asset.trim().is_empty() {
            return Err(crate::Error::invalid_request(
                "asset",
                "asset must not be empty",
            ));
        }
        self.adapter.asset_networks(asset).await
    }

    /// Lists all exchange-issued deposit addresses for this account.
    ///
    /// A provider can omit network metadata, and an address can be absent
    /// while issuance is still pending.
    pub async fn deposit_addresses(&self) -> Result<Vec<DepositAddressEntry>> {
        self.adapter.deposit_addresses().await
    }

    /// Reads an exchange-issued deposit address for one asset and network.
    pub async fn deposit_address(&self, request: &DepositAddressRequest) -> Result<DepositAddress> {
        self.adapter.deposit_address(request).await
    }

    /// Requests creation of an exchange-issued deposit address.
    ///
    /// Exchanges can issue an address asynchronously. A successful request may
    /// therefore return an address with [`DepositAddress::address`] set to
    /// `None`; query [`Self::deposit_address`] until it is available.
    pub async fn create_deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> Result<DepositAddress> {
        self.adapter.create_deposit_address(request).await
    }

    /// Checks a withdrawal against current account and network rules.
    pub async fn prepare_withdrawal(&self, request: &WithdrawRequest) -> Result<WithdrawalQuote> {
        validate_withdraw_request(request)?;
        self.adapter.prepare_withdrawal(request).await
    }

    /// Submits one withdrawal without automatic retry.
    pub async fn withdraw(&self, request: &WithdrawRequest) -> Result<Withdrawal> {
        validate_withdraw_request(request)?;
        self.adapter.withdraw(request).await
    }

    /// Reads one page of deposit history.
    pub async fn deposits(&self, request: &TransferHistoryRequest) -> Result<Page<Deposit>> {
        self.adapter.deposits(request).await
    }

    /// Reads one page of withdrawal history.
    pub async fn withdrawals(&self, request: &TransferHistoryRequest) -> Result<Page<Withdrawal>> {
        self.adapter.withdrawals(request).await
    }

    /// Reads the account's open orders across every market.
    ///
    /// Requires credentials. A returned order may have completed between the
    /// exchange snapshot and receipt; inspect [`Order::status`](crate::Order::status).
    pub async fn open_orders(&self) -> Result<Vec<Order>> {
        self.adapter.open_orders(None).await
    }

    /// Reads the account's open orders on one market.
    ///
    /// Requires credentials. Results are scoped to `market`; the provider or
    /// adapter applies the market filter.
    pub async fn open_orders_on(&self, market: &Market) -> Result<Vec<Order>> {
        self.adapter.open_orders(Some(market)).await
    }

    /// Reads one order by the exchange's own identifier.
    ///
    /// Requires credentials. `market` prevents an identifier from silently
    /// resolving to an order on a different market.
    pub async fn order(&self, market: &Market, order_id: &str) -> Result<Order> {
        self.adapter.order(market, order_id).await
    }

    /// Reads one order by the caller-assigned identifier used at placement.
    ///
    /// Requires credentials.
    pub async fn order_by_client_id(&self, market: &Market, client_id: &str) -> Result<Order> {
        self.adapter.order_by_client_id(market, client_id).await
    }

    /// Looks up up to 100 orders by exchange or caller-assigned identifiers.
    ///
    /// Requires credentials. Providers may omit unknown, malformed, or
    /// account-inaccessible identifiers instead of returning one result per ID.
    pub async fn orders_by_ids(&self, request: &OrderLookupRequest) -> Result<Vec<Order>> {
        crate::adapters::validate_order_lookup(request)?;
        self.adapter.orders_by_ids(request).await
    }

    /// Reads one newest-first page of completed or cancelled orders.
    ///
    /// Requires credentials. Feed [`Page::next`](crate::Page::next) back into
    /// the request until it is `None`.
    pub async fn order_history(&self, request: &OrderHistoryRequest) -> Result<Page<Order>> {
        self.adapter.order_history(request).await
    }

    /// Opens a live private account subscription with default settings.
    ///
    /// Requires credentials.
    ///
    /// Browser WebAssembly uses [`Overflow::DropNewest`](crate::Overflow::DropNewest)
    /// for the same browser WebSocket limitation as [`Client::subscribe`].
    /// Native clients retain the default backpressure policy.
    ///
    /// See [`AccountStream`] for error, reconnect, and termination semantics.
    pub async fn subscribe_account(&self) -> Result<AccountStream> {
        self.subscribe_account_with(&default_stream_config()).await
    }

    /// Opens a live private account subscription with explicit connection settings.
    ///
    /// Requires credentials. An adapter may raise an idle timeout below the
    /// minimum its exchange can satisfy.
    pub async fn subscribe_account_with(&self, config: &StreamConfig) -> Result<AccountStream> {
        self.adapter.subscribe_account(config).await
    }

    /// Places an order.
    ///
    /// Requires credentials. The returned [`Order`] carries the exchange's own
    /// identifier, which is what [`Client::cancel_order`] takes.
    pub async fn place_order(&self, request: &OrderRequest) -> Result<Order> {
        self.adapter.place_order(request).await
    }

    /// Cancels an order.
    ///
    /// Requires credentials. `Ok(())` confirms that the provider accepted or
    /// confirmed the cancellation response; it is not a final fill snapshot.
    /// Cancellation races execution, so query the order when the final outcome
    /// matters.
    pub async fn cancel_order(&self, market: &Market, order_id: &str) -> Result<()> {
        self.adapter.cancel_order(market, order_id).await
    }

    /// Cancels an order by the caller-assigned identifier used at placement.
    ///
    /// Requires credentials. Cancellation has the same race and reconciliation
    /// requirements as [`Client::cancel_order`].
    pub async fn cancel_order_by_client_id(&self, market: &Market, client_id: &str) -> Result<()> {
        self.adapter
            .cancel_order_by_client_id(market, client_id)
            .await
    }

    /// Cancels multiple orders without treating a partial failure as an atomic failure.
    ///
    /// Requires credentials. Provider limits differ; Upbit accepts at most 20
    /// identifiers and Bithumb accepts at most 30.
    pub async fn cancel_orders(&self, request: &CancelOrdersRequest) -> Result<CancelOrdersResult> {
        crate::adapters::validate_cancel_orders(request)?;
        self.adapter.cancel_orders(request).await
    }

    /// Reads every open position.
    ///
    /// Requires credentials. Derivatives markets only.
    ///
    /// Rows with `quantity == 0` are removed.
    pub async fn positions(&self) -> Result<Vec<Position>> {
        Ok(open_positions(self.adapter.positions(None).await?))
    }

    /// Reads the open position on one market.
    ///
    /// Requires credentials. Derivatives markets only.
    ///
    /// A market the account holds nothing on answers an empty list rather than
    /// one flat position, on the same terms as [`Client::positions`].
    pub async fn positions_on(&self, market: &Market) -> Result<Vec<Position>> {
        Ok(open_positions(self.adapter.positions(Some(market)).await?))
    }

    /// Reads account-wide margin state.
    ///
    /// Requires credentials. Derivatives markets only. Values an exchange does
    /// not publish remain `None`.
    pub async fn margin_summary(&self) -> Result<MarginSummary> {
        self.adapter.margin_summary().await
    }

    /// Reads a market's funding-rate history, one page at a time.
    ///
    /// This public operation needs no account credentials. Continue only when
    /// [`Page::next`](crate::Page::next) is `Some`; item count does not mark the
    /// final page.
    pub async fn funding_rates(&self, request: &HistoryRequest) -> Result<Page<FundingRate>> {
        self.adapter.funding_rates(request).await
    }

    /// Reads the account's funding payment history, one page at a time.
    ///
    /// Requires credentials. Unlike [`Client::funding_rates`], this is what the
    /// account was actually charged or credited. Amounts are signed; negative
    /// means the account paid.
    pub async fn funding_payments(&self, request: &HistoryRequest) -> Result<Page<FundingPayment>> {
        self.adapter.funding_payments(request).await
    }

    /// Changes leverage and/or margin mode on a market.
    ///
    /// Requires credentials. Derivatives markets only. Provider requirements
    /// differ: some accept either field, while others require both. When both
    /// are accepted, the change is not guaranteed to be atomic; one provider
    /// operation may succeed before another fails.
    pub async fn set_margin(&self, request: &MarginRequest) -> Result<()> {
        self.adapter.set_margin(request).await
    }
}

fn default_stream_config() -> StreamConfig {
    #[cfg(target_arch = "wasm32")]
    {
        StreamConfig {
            overflow: crate::types::Overflow::DropNewest,
            ..StreamConfig::default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        StreamConfig::default()
    }
}

fn validate_withdraw_request(request: &WithdrawRequest) -> Result<()> {
    if request.asset.trim().is_empty() {
        return Err(crate::Error::invalid_request(
            "asset",
            "asset must not be empty",
        ));
    }
    if request.amount <= crate::Decimal::ZERO {
        return Err(crate::Error::invalid_request(
            "amount",
            "amount must be greater than zero",
        ));
    }
    if !request.network.same_chain(request.destination.network()) {
        return Err(crate::Error::transfer(
            crate::TransferErrorKind::NetworkMismatch,
            format!(
                "withdrawal network {} differs from destination network {}",
                request.network,
                request.destination.network(),
            ),
        ));
    }
    if !request
        .asset
        .eq_ignore_ascii_case(request.destination.asset())
    {
        return Err(crate::Error::transfer(
            crate::TransferErrorKind::AssetMismatch,
            format!(
                "withdrawal asset {} differs from destination asset {}",
                request.asset,
                request.destination.asset(),
            ),
        ));
    }
    if request.destination.address().trim().is_empty() {
        return Err(crate::Error::invalid_request(
            "destination.address",
            "address must not be empty",
        ));
    }
    Ok(())
}

impl<A: Adapter> From<A> for Client<A> {
    fn from(adapter: A) -> Self {
        Self::new(adapter)
    }
}

/// Keeps the common positions API limited to non-flat rows.
pub(crate) fn open_positions(mut positions: Vec<Position>) -> Vec<Position> {
    positions.retain(|position| !position.is_flat());
    positions
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::adapter::BoxFuture;
    use crate::{Decimal, Error, Feed, MarketEvent, Overflow, Side};

    #[derive(Debug, Clone)]
    struct PublicOnly;

    impl Adapter for PublicOnly {
        fn exchange(&self) -> Exchange {
            Exchange::Bithumb
        }

        fn supports(&self, feature: Feature) -> bool {
            !feature.needs_credentials()
        }

        fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
            let empty = matches!(kind, MarketKind::Perpetual);
            Box::pin(async move {
                Ok(if empty {
                    vec![]
                } else {
                    vec![MarketInfo {
                        market: Market::spot(Exchange::Bithumb, "BTC", "KRW"),
                        native_symbol: "BTC_KRW".to_string(),
                        status: crate::MarketStatus::Active,
                        korean_name: None,
                        english_name: None,
                    }]
                })
            })
        }
    }

    #[test]
    fn supports_answers_without_a_network_call() {
        let client = Client::new(PublicOnly);

        assert!(client.supports(Feature::Ticker));
        assert!(!client.supports(Feature::Trading));
        assert_eq!(client.exchange(), Exchange::Bithumb);
    }

    #[tokio::test]
    async fn a_spot_exchange_reports_no_perpetuals_rather_than_an_error() {
        let client = Client::new(PublicOnly);

        assert_eq!(client.markets(MarketKind::Spot).await.unwrap().len(), 1);
        assert!(
            client
                .markets(MarketKind::Perpetual)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn private_calls_on_a_public_client_name_the_missing_feature() {
        let client = Client::new(PublicOnly);

        let error = client.balances().await.unwrap_err();
        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::Balances,
                exchange: "bithumb",
                ..
            }
        ));
    }

    /// An adapter fixture that includes a flat position.
    #[derive(Debug, Clone)]
    struct ReportsWhatTheVenueSaid;

    impl ReportsWhatTheVenueSaid {
        fn market(quote: &str) -> Market {
            Market::perpetual(Exchange::Binance, "BTC", quote)
        }

        fn position(quantity: Decimal, quote: &str) -> Position {
            Position {
                market: Self::market(quote),
                side: if quantity.is_zero() {
                    None
                } else {
                    Some(Side::Buy)
                },
                quantity,
                entry_price: None,
                mark_price: None,
                notional: Some(Decimal::from(30_000)),
                unrealized_pnl: None,
                leverage: None,
                margin_mode: None,
            }
        }
    }

    impl Adapter for ReportsWhatTheVenueSaid {
        fn exchange(&self) -> Exchange {
            Exchange::Binance
        }

        fn supports(&self, _feature: Feature) -> bool {
            true
        }

        fn positions(&self, _market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
            Box::pin(async move {
                Ok(vec![
                    Self::position(Decimal::ZERO, "USDT"),
                    Self::position(Decimal::ONE, "USDC"),
                ])
            })
        }
    }

    /// The common client removes flat rows from both position queries.
    #[tokio::test]
    async fn a_flat_row_an_adapter_reports_is_not_answered_as_an_open_position() {
        let client = Client::new(ReportsWhatTheVenueSaid);

        assert_eq!(client.adapter().positions(None).await.unwrap().len(), 2);

        let open = client.positions().await.unwrap();
        assert_eq!(open.len(), 1, "a flat row was answered as an open position");
        assert_eq!(open[0].quantity, Decimal::ONE);

        let narrowed = client
            .positions_on(&ReportsWhatTheVenueSaid::market("USDT"))
            .await
            .unwrap();
        assert_eq!(narrowed.len(), 1, "{narrowed:?}");
        assert!(!narrowed[0].is_flat(), "{narrowed:?}");
    }

    #[tokio::test]
    async fn clients_over_boxed_adapters_share_one_type() {
        let clients: Vec<Client<Box<dyn Adapter>>> = vec![Client::new(Box::new(PublicOnly) as _)];

        for client in &clients {
            assert!(!client.markets(MarketKind::Spot).await.unwrap().is_empty());
        }
    }

    #[derive(Debug, Clone, Default)]
    struct RecordsStreamConfig(Arc<Mutex<Vec<StreamConfig>>>);

    impl Adapter for RecordsStreamConfig {
        fn exchange(&self) -> Exchange {
            Exchange::Bithumb
        }

        fn supports(&self, _feature: Feature) -> bool {
            true
        }

        fn subscribe(
            &self,
            _subscription: &Subscription,
            config: &StreamConfig,
        ) -> BoxFuture<'_, Result<MarketStream>> {
            self.0.lock().unwrap().push(config.clone());
            Box::pin(async {
                Ok(MarketStream::new(futures_util::stream::empty::<
                    Result<MarketEvent>,
                >()))
            })
        }

        fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
            self.0.lock().unwrap().push(config.clone());
            Box::pin(async {
                Ok(AccountStream::new(futures_util::stream::empty::<
                    Result<crate::AccountEvent>,
                >()))
            })
        }
    }

    #[tokio::test]
    async fn default_stream_methods_use_the_platform_supported_overflow() {
        let adapter = RecordsStreamConfig::default();
        let seen = Arc::clone(&adapter.0);
        let client = Client::new(adapter);
        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .feed(Feed::Trades);

        client.subscribe(&subscription).await.unwrap();
        client.subscribe_account().await.unwrap();

        #[cfg(target_arch = "wasm32")]
        let expected = Overflow::DropNewest;
        #[cfg(not(target_arch = "wasm32"))]
        let expected = Overflow::Backpressure;
        assert_eq!(
            seen.lock()
                .unwrap()
                .iter()
                .map(|config| config.overflow)
                .collect::<Vec<_>>(),
            vec![expected, expected]
        );
    }

    #[tokio::test]
    async fn explicit_stream_methods_preserve_backpressure() {
        let adapter = RecordsStreamConfig::default();
        let seen = Arc::clone(&adapter.0);
        let client = Client::new(adapter);
        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .feed(Feed::Trades);
        let config = StreamConfig::default();

        client.subscribe_with(&subscription, &config).await.unwrap();
        client.subscribe_account_with(&config).await.unwrap();

        assert!(
            seen.lock()
                .unwrap()
                .iter()
                .all(|config| config.overflow == Overflow::Backpressure)
        );
    }
}
