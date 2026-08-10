use std::collections::HashSet;
use std::sync::Arc;

use maxt::{
    AccountStream, Adapter, AssetNetwork, Balance, BoxFuture, Candle, CandleRequest, Deposit,
    DepositAddress, DepositAddressRequest, Exchange, Feature, FundingPayment, FundingRate,
    HistoryRequest, MarginRequest, MarginSummary, Market, MarketInfo, MarketKind, MarketStream,
    Order, OrderBook, OrderRequest, Page, Position, Result, StreamConfig, Subscription, Ticker,
    Trade, TransferHistoryRequest, WithdrawRequest, Withdrawal, WithdrawalQuote,
};

use crate::{AdapterCall, AdapterReply, ForeignDispatcher};

/// A Rust [`Adapter`] backed by a [`ForeignDispatcher`].
pub struct ForeignAdapter {
    exchange: Exchange,
    features: HashSet<Feature>,
    dispatcher: Arc<dyn ForeignDispatcher>,
}

impl ForeignAdapter {
    /// Creates an adapter with binding-owned exchange and feature metadata.
    pub fn new(
        exchange: Exchange,
        features: impl IntoIterator<Item = Feature>,
        dispatcher: Arc<dyn ForeignDispatcher>,
    ) -> Self {
        Self {
            exchange,
            features: features.into_iter().collect(),
            dispatcher,
        }
    }

    /// The configured features without duplicates.
    pub fn features(&self) -> &HashSet<Feature> {
        &self.features
    }

    /// The binding-specific dispatcher.
    pub fn dispatcher(&self) -> &dyn ForeignDispatcher {
        self.dispatcher.as_ref()
    }
}

macro_rules! dispatch {
    ($self:expr, $call:expr, $variant:path, $expected:literal) => {{
        let future = $self.dispatcher.dispatch($call);
        Box::pin(async move {
            match future.await? {
                $variant(value) => Ok(value),
                reply => Err(unexpected_reply($expected, &reply)),
            }
        })
    }};
}

impl Adapter for ForeignAdapter {
    fn exchange(&self) -> Exchange {
        self.exchange
    }

    fn supports(&self, feature: Feature) -> bool {
        self.features.contains(&feature)
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        dispatch!(
            self,
            AdapterCall::Markets { kind },
            AdapterReply::Markets,
            "Markets"
        )
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        dispatch!(
            self,
            AdapterCall::Trades {
                market: market.clone(),
                limit,
            },
            AdapterReply::Trades,
            "Trades"
        )
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        dispatch!(
            self,
            AdapterCall::OrderBook {
                market: market.clone(),
                depth,
            },
            AdapterReply::OrderBook,
            "OrderBook"
        )
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        dispatch!(
            self,
            AdapterCall::Ticker {
                market: market.clone(),
            },
            AdapterReply::Ticker,
            "Ticker"
        )
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        dispatch!(
            self,
            AdapterCall::Candles {
                request: request.clone(),
            },
            AdapterReply::Candles,
            "Candles"
        )
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        dispatch!(
            self,
            AdapterCall::Subscribe {
                subscription: subscription.clone(),
                config: config.clone(),
            },
            AdapterReply::MarketStream,
            "MarketStream"
        )
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        dispatch!(
            self,
            AdapterCall::Balances,
            AdapterReply::Balances,
            "Balances"
        )
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        dispatch!(
            self,
            AdapterCall::AssetNetworks {
                asset: asset.to_owned(),
            },
            AdapterReply::AssetNetworks,
            "AssetNetworks"
        )
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        dispatch!(
            self,
            AdapterCall::DepositAddress {
                request: request.clone(),
            },
            AdapterReply::DepositAddress,
            "DepositAddress"
        )
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        dispatch!(
            self,
            AdapterCall::PrepareWithdrawal {
                request: request.clone(),
            },
            AdapterReply::WithdrawalQuote,
            "WithdrawalQuote"
        )
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        dispatch!(
            self,
            AdapterCall::Withdraw {
                request: request.clone(),
            },
            AdapterReply::Withdrawal,
            "Withdrawal"
        )
    }

    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        dispatch!(
            self,
            AdapterCall::Deposits {
                request: request.clone(),
            },
            AdapterReply::Deposits,
            "Deposits"
        )
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        dispatch!(
            self,
            AdapterCall::Withdrawals {
                request: request.clone(),
            },
            AdapterReply::Withdrawals,
            "Withdrawals"
        )
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        dispatch!(
            self,
            AdapterCall::OpenOrders {
                market: market.cloned(),
            },
            AdapterReply::OpenOrders,
            "OpenOrders"
        )
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        dispatch!(
            self,
            AdapterCall::SubscribeAccount {
                config: config.clone(),
            },
            AdapterReply::AccountStream,
            "AccountStream"
        )
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        dispatch!(
            self,
            AdapterCall::PlaceOrder {
                request: request.clone(),
            },
            AdapterReply::PlaceOrder,
            "PlaceOrder"
        )
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        let future = self.dispatcher.dispatch(AdapterCall::CancelOrder {
            market: market.clone(),
            order_id: order_id.to_owned(),
        });
        Box::pin(async move {
            match future.await? {
                AdapterReply::Unit => Ok(()),
                reply => Err(unexpected_reply("Unit", &reply)),
            }
        })
    }

    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, Result<()>> {
        let future = self
            .dispatcher
            .dispatch(AdapterCall::CancelOrderByClientId {
                market: market.clone(),
                client_id: client_id.to_owned(),
            });
        Box::pin(async move {
            match future.await? {
                AdapterReply::Unit => Ok(()),
                reply => Err(unexpected_reply("Unit", &reply)),
            }
        })
    }

    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        dispatch!(
            self,
            AdapterCall::Positions {
                market: market.cloned(),
            },
            AdapterReply::Positions,
            "Positions"
        )
    }

    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        dispatch!(
            self,
            AdapterCall::MarginSummary,
            AdapterReply::MarginSummary,
            "MarginSummary"
        )
    }

    fn funding_rates(&self, request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        dispatch!(
            self,
            AdapterCall::FundingRates {
                request: request.clone(),
            },
            AdapterReply::FundingRates,
            "FundingRates"
        )
    }

    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        dispatch!(
            self,
            AdapterCall::FundingPayments {
                request: request.clone(),
            },
            AdapterReply::FundingPayments,
            "FundingPayments"
        )
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        let future = self.dispatcher.dispatch(AdapterCall::SetMargin {
            request: request.clone(),
        });
        Box::pin(async move {
            match future.await? {
                AdapterReply::Unit => Ok(()),
                reply => Err(unexpected_reply("Unit", &reply)),
            }
        })
    }
}

fn unexpected_reply(expected: &str, reply: &AdapterReply) -> maxt::Error {
    maxt::Error::adapter(format!(
        "foreign dispatcher returned {} where {expected} was required",
        reply.kind()
    ))
}
