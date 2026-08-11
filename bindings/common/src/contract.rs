use maxt::{
    AccountStream, AssetNetwork, Balance, BoxFuture, CancelOrdersRequest, CancelOrdersResult,
    Candle, CandleRequest, Deposit, DepositAddress, DepositAddressRequest, FundingPayment,
    FundingRate, HistoryRequest, MarginRequest, MarginSummary, Market, MarketInfo, MarketKind,
    MarketStream, Order, OrderBook, OrderHistoryRequest, OrderLookupRequest, OrderRequest,
    OrderRules, Page, Position, Result, StreamConfig, Subscription, Ticker, Trade,
    TransferHistoryRequest, WithdrawRequest, Withdrawal, WithdrawalQuote,
};

/// An owned call across a language binding boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AdapterCall {
    /// Lists markets of one kind.
    Markets {
        /// The requested instrument kind.
        kind: MarketKind,
    },
    /// Reads recent trades.
    Trades {
        /// The market to read.
        market: Market,
        /// The maximum requested row count.
        limit: Option<u32>,
    },
    /// Reads an order book snapshot.
    OrderBook {
        /// The market to read.
        market: Market,
        /// The requested levels per side.
        depth: Option<u32>,
    },
    /// Reads a ticker.
    Ticker {
        /// The market to read.
        market: Market,
    },
    /// Reads historical candles.
    Candles {
        /// The complete candle request.
        request: CandleRequest,
    },
    /// Opens a market-data stream.
    Subscribe {
        /// The requested markets and feeds.
        subscription: Subscription,
        /// Connection and buffering settings.
        config: StreamConfig,
    },
    /// Reads account balances.
    Balances,
    /// Reads dynamic order rules for one market.
    OrderRules {
        /// The market to inspect.
        market: Market,
    },
    /// Reads live asset-network rules.
    AssetNetworks {
        /// Asset symbol.
        asset: String,
    },
    /// Reads one deposit address.
    DepositAddress {
        /// Complete address request.
        request: DepositAddressRequest,
    },
    /// Requests creation of one deposit address.
    CreateDepositAddress {
        /// Complete address request.
        request: DepositAddressRequest,
    },
    /// Checks one withdrawal without submitting it.
    PrepareWithdrawal {
        /// Complete withdrawal request.
        request: WithdrawRequest,
    },
    /// Submits one withdrawal.
    Withdraw {
        /// Complete withdrawal request.
        request: WithdrawRequest,
    },
    /// Reads deposit history.
    Deposits {
        /// Complete history request.
        request: TransferHistoryRequest,
    },
    /// Reads withdrawal history.
    Withdrawals {
        /// Complete history request.
        request: TransferHistoryRequest,
    },
    /// Reads open orders, optionally for one market.
    OpenOrders {
        /// The optional market filter.
        market: Option<Market>,
    },
    /// Reads one order by exchange identifier.
    Order {
        /// The order's market.
        market: Market,
        /// The exchange's identifier.
        order_id: String,
    },
    /// Reads one order by caller-assigned identifier.
    OrderByClientId {
        /// The order's market.
        market: Market,
        /// The caller-assigned identifier.
        client_id: String,
    },
    /// Looks up multiple orders by one identifier namespace.
    OrdersByIds {
        /// Complete lookup request.
        request: OrderLookupRequest,
    },
    /// Reads final-order history.
    OrderHistory {
        /// Complete history request.
        request: OrderHistoryRequest,
    },
    /// Opens an account stream.
    SubscribeAccount {
        /// Connection and buffering settings.
        config: StreamConfig,
    },
    /// Places an order.
    PlaceOrder {
        /// The complete order request.
        request: OrderRequest,
    },
    /// Cancels an order.
    CancelOrder {
        /// The order's market.
        market: Market,
        /// The exchange's order identifier.
        order_id: String,
    },
    /// Cancels an order by its caller-assigned identifier.
    CancelOrderByClientId {
        /// The order's market.
        market: Market,
        /// The caller-assigned identifier.
        client_id: String,
    },
    /// Cancels multiple orders by one identifier namespace.
    CancelOrders {
        /// Complete batch-cancellation request.
        request: CancelOrdersRequest,
    },
    /// Reads open positions, optionally for one market.
    Positions {
        /// The optional market filter.
        market: Option<Market>,
    },
    /// Reads account-wide margin state.
    MarginSummary,
    /// Reads funding-rate history.
    FundingRates {
        /// The complete history request.
        request: HistoryRequest,
    },
    /// Reads funding-payment history.
    FundingPayments {
        /// The complete history request.
        request: HistoryRequest,
    },
    /// Changes leverage or margin mode.
    SetMargin {
        /// The complete margin request.
        request: MarginRequest,
    },
}

/// An owned reply returned by a foreign dispatcher.
#[derive(Debug)]
#[non_exhaustive]
pub enum AdapterReply {
    /// Result of [`AdapterCall::Markets`].
    Markets(Vec<MarketInfo>),
    /// Result of [`AdapterCall::Trades`].
    Trades(Vec<Trade>),
    /// Result of [`AdapterCall::OrderBook`].
    OrderBook(OrderBook),
    /// Result of [`AdapterCall::Ticker`].
    Ticker(Ticker),
    /// Result of [`AdapterCall::Candles`].
    Candles(Vec<Candle>),
    /// Result of [`AdapterCall::Subscribe`].
    MarketStream(MarketStream),
    /// Result of [`AdapterCall::Balances`].
    Balances(Vec<Balance>),
    /// Result of [`AdapterCall::OrderRules`].
    OrderRules(Box<OrderRules>),
    /// Result of [`AdapterCall::AssetNetworks`].
    AssetNetworks(Vec<AssetNetwork>),
    /// Result of [`AdapterCall::DepositAddress`].
    DepositAddress(DepositAddress),
    /// Result of [`AdapterCall::CreateDepositAddress`].
    CreateDepositAddress(DepositAddress),
    /// Result of [`AdapterCall::PrepareWithdrawal`].
    WithdrawalQuote(WithdrawalQuote),
    /// Result of [`AdapterCall::Withdraw`].
    Withdrawal(Withdrawal),
    /// Result of [`AdapterCall::Deposits`].
    Deposits(Page<Deposit>),
    /// Result of [`AdapterCall::Withdrawals`].
    Withdrawals(Page<Withdrawal>),
    /// Result of [`AdapterCall::OpenOrders`].
    OpenOrders(Vec<Order>),
    /// Result of [`AdapterCall::Order`] or [`AdapterCall::OrderByClientId`].
    Order(Order),
    /// Result of [`AdapterCall::OrdersByIds`].
    OrdersByIds(Vec<Order>),
    /// Result of [`AdapterCall::OrderHistory`].
    OrderHistory(Page<Order>),
    /// Result of [`AdapterCall::SubscribeAccount`].
    AccountStream(AccountStream),
    /// Result of [`AdapterCall::PlaceOrder`].
    PlaceOrder(Order),
    /// Result of [`AdapterCall::CancelOrders`].
    CancelOrdersResult(CancelOrdersResult),
    /// Result of [`AdapterCall::Positions`].
    Positions(Vec<Position>),
    /// Result of [`AdapterCall::MarginSummary`].
    MarginSummary(MarginSummary),
    /// Result of [`AdapterCall::FundingRates`].
    FundingRates(Page<FundingRate>),
    /// Result of [`AdapterCall::FundingPayments`].
    FundingPayments(Page<FundingPayment>),
    /// Result of [`AdapterCall::SetMargin`].
    Unit,
}

/// Executes owned adapter calls in a foreign runtime.
pub trait ForeignDispatcher: Send + Sync + 'static {
    /// Dispatches one call and returns its typed reply.
    fn dispatch(&self, call: AdapterCall) -> BoxFuture<'_, Result<AdapterReply>>;
}

impl AdapterReply {
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::Markets(_) => "Markets",
            Self::Trades(_) => "Trades",
            Self::OrderBook(_) => "OrderBook",
            Self::Ticker(_) => "Ticker",
            Self::Candles(_) => "Candles",
            Self::MarketStream(_) => "MarketStream",
            Self::Balances(_) => "Balances",
            Self::OrderRules(_) => "OrderRules",
            Self::AssetNetworks(_) => "AssetNetworks",
            Self::DepositAddress(_) => "DepositAddress",
            Self::CreateDepositAddress(_) => "CreateDepositAddress",
            Self::WithdrawalQuote(_) => "WithdrawalQuote",
            Self::Withdrawal(_) => "Withdrawal",
            Self::Deposits(_) => "Deposits",
            Self::Withdrawals(_) => "Withdrawals",
            Self::OpenOrders(_) => "OpenOrders",
            Self::Order(_) => "Order",
            Self::OrdersByIds(_) => "OrdersByIds",
            Self::OrderHistory(_) => "OrderHistory",
            Self::AccountStream(_) => "AccountStream",
            Self::PlaceOrder(_) => "PlaceOrder",
            Self::CancelOrdersResult(_) => "CancelOrdersResult",
            Self::Positions(_) => "Positions",
            Self::MarginSummary(_) => "MarginSummary",
            Self::FundingRates(_) => "FundingRates",
            Self::FundingPayments(_) => "FundingPayments",
            Self::Unit => "Unit",
        }
    }
}
