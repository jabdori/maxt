//! The contract every exchange adapter implements.

use std::future::Future;
use std::pin::Pin;

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{
    CancelOrdersRequest, CandleRequest, DepositAddressRequest, HistoryRequest, MarginRequest,
    OrderHistoryRequest, OrderLookupRequest, OrderRequest, TransferHistoryRequest, WithdrawRequest,
};
use crate::stream::{AccountStream, MarketStream};
use crate::types::{
    AssetNetwork, Balance, CancelOrdersResult, Candle, Deposit, DepositAddress, Exchange, Feed,
    FundingPayment, FundingRate, MarginSummary, Market, MarketInfo, MarketKind, Order, OrderBook,
    Page, Position, StreamConfig, Subscription, Ticker, Trade, Withdrawal, WithdrawalQuote,
};

/// A boxed future used to keep [`Adapter`] dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Exchange adapter contract used by [`Client`](crate::Client).
///
/// Implementations must preserve the ordering, validation, and normalization
/// documented by the corresponding [`Client`](crate::Client) methods. All
/// optional methods default to [`Error::Unsupported`].
///
/// External implementations can provide mocks, backtests, or recorded-data
/// adapters. A new real exchange also requires a new [`Exchange`] variant.
pub trait Adapter: Send + Sync + 'static {
    /// Which exchange this adapter talks to.
    fn exchange(&self) -> Exchange;

    /// Whether this configured adapter can use a feature.
    ///
    /// Structural absence is returned from the corresponding method as
    /// [`Error::Unsupported`]. A supported private endpoint without usable
    /// credentials returns [`Error::Auth`](crate::Error::Auth).
    fn supports(&self, feature: Feature) -> bool;

    /// Lists the exchange's markets of one kind.
    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        let _ = kind;
        unsupported(self.exchange(), Feature::Markets)
    }

    /// Reads the most recent trades on a market.
    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let _ = (market, limit);
        unsupported(self.exchange(), Feature::Trades)
    }

    /// Reads an order book snapshot.
    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let _ = (market, depth);
        unsupported(self.exchange(), Feature::OrderBook)
    }

    /// Reads a provider ticker summary for one market.
    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let _ = market;
        unsupported(self.exchange(), Feature::Ticker)
    }

    /// Reads historical candles.
    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::Candles)
    }

    /// Opens a live market-data subscription.
    ///
    /// The default rejects empty markets or feeds, then returns
    /// [`Error::Unsupported`] for the first requested feed. Implementors build
    /// streams with [`MarketStream::new`] and handle reconnects.
    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        if subscription.markets().is_empty() {
            return Box::pin(async {
                Err(Error::invalid_request(
                    "markets",
                    "a subscription needs at least one market",
                ))
            });
        }
        if subscription.feeds().is_empty() {
            return Box::pin(async {
                Err(Error::invalid_request(
                    "feeds",
                    "a subscription needs at least one feed",
                ))
            });
        }
        let _ = config;
        let feature = match subscription.feeds()[0] {
            Feed::Trades => Feature::TradeStream,
            Feed::OrderBook => Feature::OrderBookStream,
            Feed::Ticker => Feature::TickerStream,
            Feed::Candles(_) => Feature::CandleStream,
        };
        unsupported(self.exchange(), feature)
    }

    /// Reads the account's balances.
    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        unsupported(self.exchange(), Feature::Balances)
    }

    /// Reads live transfer rules for one asset.
    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        let _ = asset;
        unsupported(self.exchange(), Feature::AssetNetworks)
    }

    /// Reads an exchange-issued deposit address.
    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let _ = request;
        unsupported(self.exchange(), Feature::DepositAddresses)
    }

    /// Performs live source-account checks without submitting a withdrawal.
    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        let _ = request;
        unsupported(self.exchange(), Feature::WithdrawalQuotes)
    }

    /// Submits one withdrawal without automatic retry.
    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let _ = request;
        unsupported(self.exchange(), Feature::Withdrawals)
    }

    /// Reads one page of deposit history.
    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::DepositHistory)
    }

    /// Reads one page of withdrawal history.
    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::WithdrawalHistory)
    }

    /// Reads the account's open orders, optionally narrowed to one market.
    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let _ = market;
        unsupported(self.exchange(), Feature::OpenOrders)
    }

    /// Reads one order by the exchange's own identifier.
    fn order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let _ = (market, order_id);
        unsupported(self.exchange(), Feature::OrderHistory)
    }

    /// Reads one order by the caller-assigned identifier supplied at placement.
    fn order_by_client_id(&self, market: &Market, client_id: &str) -> BoxFuture<'_, Result<Order>> {
        let _ = (market, client_id);
        unsupported(self.exchange(), Feature::OrderHistory)
    }

    /// Looks up up to 100 orders by exchange or caller-assigned identifiers.
    fn orders_by_ids(&self, request: &OrderLookupRequest) -> BoxFuture<'_, Result<Vec<Order>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::OrderHistory)
    }

    /// Reads one newest-first page of completed or cancelled orders.
    fn order_history(&self, request: &OrderHistoryRequest) -> BoxFuture<'_, Result<Page<Order>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::OrderHistory)
    }

    /// Opens a live private account subscription.
    ///
    /// Implementors build streams with [`AccountStream::new`] and report
    /// credential-renewal failures as `Err` stream items.
    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let _ = config;
        unsupported(self.exchange(), Feature::AccountStream)
    }

    /// Places an order.
    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let _ = request;
        unsupported(self.exchange(), Feature::Trading)
    }

    /// Cancels an order by the exchange's own identifier.
    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        let _ = (market, order_id);
        unsupported(self.exchange(), Feature::Trading)
    }

    /// Cancels an order by the caller-assigned identifier supplied at placement.
    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, Result<()>> {
        let _ = (market, client_id);
        unsupported(self.exchange(), Feature::Trading)
    }

    /// Cancels multiple orders and returns every per-order outcome.
    fn cancel_orders(
        &self,
        request: &CancelOrdersRequest,
    ) -> BoxFuture<'_, Result<CancelOrdersResult>> {
        let _ = request;
        unsupported(self.exchange(), Feature::Trading)
    }

    /// Reads open positions, optionally narrowed to one market.
    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        let _ = market;
        unsupported(self.exchange(), Feature::Positions)
    }

    /// Reads account-wide margin state.
    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        unsupported(self.exchange(), Feature::Margin)
    }

    /// Reads a market's funding rate history.
    fn funding_rates(&self, request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::FundingRates)
    }

    /// Reads the account's funding payment history.
    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        let _ = request;
        unsupported(self.exchange(), Feature::FundingPayments)
    }

    /// Sets leverage or margin mode on a market.
    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        let _ = request;
        unsupported(self.exchange(), Feature::MarginConfig)
    }
}

/// Builds the failed future a missing feature produces.
///
/// A free function, because a generic trait method would cost [`Adapter`] its
/// `dyn` compatibility.
fn unsupported<'a, T: Send + 'a>(exchange: Exchange, feature: Feature) -> BoxFuture<'a, Result<T>> {
    let exchange = exchange.id();
    Box::pin(async move {
        Err(Error::unsupported(
            feature,
            exchange,
            format!("{exchange} has no endpoint for {feature}"),
        ))
    })
}

impl Adapter for Box<dyn Adapter> {
    fn exchange(&self) -> Exchange {
        (**self).exchange()
    }

    fn supports(&self, feature: Feature) -> bool {
        (**self).supports(feature)
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        (**self).markets(kind)
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        (**self).trades(market, limit)
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        (**self).order_book(market, depth)
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        (**self).ticker(market)
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        (**self).candles(request)
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        (**self).subscribe(subscription, config)
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        (**self).balances()
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        (**self).asset_networks(asset)
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        (**self).deposit_address(request)
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        (**self).prepare_withdrawal(request)
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        (**self).withdraw(request)
    }

    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        (**self).deposits(request)
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        (**self).withdrawals(request)
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        (**self).open_orders(market)
    }

    fn order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        (**self).order(market, order_id)
    }

    fn order_by_client_id(&self, market: &Market, client_id: &str) -> BoxFuture<'_, Result<Order>> {
        (**self).order_by_client_id(market, client_id)
    }

    fn orders_by_ids(&self, request: &OrderLookupRequest) -> BoxFuture<'_, Result<Vec<Order>>> {
        (**self).orders_by_ids(request)
    }

    fn order_history(&self, request: &OrderHistoryRequest) -> BoxFuture<'_, Result<Page<Order>>> {
        (**self).order_history(request)
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        (**self).subscribe_account(config)
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        (**self).place_order(request)
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        (**self).cancel_order(market, order_id)
    }

    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, Result<()>> {
        (**self).cancel_order_by_client_id(market, client_id)
    }

    fn cancel_orders(
        &self,
        request: &CancelOrdersRequest,
    ) -> BoxFuture<'_, Result<CancelOrdersResult>> {
        (**self).cancel_orders(request)
    }

    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        (**self).positions(market)
    }

    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        (**self).margin_summary()
    }

    fn funding_rates(&self, request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        (**self).funding_rates(request)
    }

    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        (**self).funding_payments(request)
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        (**self).set_margin(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MarketDataOnly;

    impl Adapter for MarketDataOnly {
        fn exchange(&self) -> Exchange {
            Exchange::Upbit
        }

        fn supports(&self, feature: Feature) -> bool {
            matches!(feature, Feature::Markets | Feature::Ticker)
        }
    }

    #[tokio::test]
    async fn unimplemented_methods_report_the_missing_feature_by_name() {
        let error = MarketDataOnly.balances().await.unwrap_err();

        let Error::Unsupported {
            feature, exchange, ..
        } = error
        else {
            panic!("expected an unsupported-feature error");
        };
        assert_eq!(feature, Feature::Balances);
        assert_eq!(exchange, "upbit");
    }

    #[tokio::test]
    async fn an_adapter_survives_being_held_behind_dyn() {
        let adapters: Vec<Box<dyn Adapter>> = vec![Box::new(MarketDataOnly)];

        for adapter in &adapters {
            assert_eq!(adapter.exchange(), Exchange::Upbit);
            assert!(adapter.supports(Feature::Ticker));
            assert!(adapter.positions(None).await.is_err());
        }
    }

    #[test]
    fn supports_reflects_the_adapter_not_the_trait_defaults() {
        assert!(MarketDataOnly.supports(Feature::Markets));
        assert!(!MarketDataOnly.supports(Feature::Trading));
    }
}
