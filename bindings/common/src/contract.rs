use maxt::{
    AccountStream, Balance, BoxFuture, Candle, CandleRequest, FundingPayment, FundingRate,
    HistoryRequest, MarginRequest, MarginSummary, Market, MarketInfo, MarketKind, MarketStream,
    Order, OrderBook, OrderRequest, Page, Position, Result, StreamConfig, Subscription, Ticker,
    Trade,
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
    /// Reads open orders, optionally for one market.
    OpenOrders {
        /// The optional market filter.
        market: Option<Market>,
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
    /// Result of [`AdapterCall::OpenOrders`].
    OpenOrders(Vec<Order>),
    /// Result of [`AdapterCall::SubscribeAccount`].
    AccountStream(AccountStream),
    /// Result of an order placement or cancellation.
    Order(Order),
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
    fn dispatch(&self, call: AdapterCall) -> BoxFuture<'static, Result<AdapterReply>>;
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
            Self::OpenOrders(_) => "OpenOrders",
            Self::AccountStream(_) => "AccountStream",
            Self::Order(_) => "Order",
            Self::Positions(_) => "Positions",
            Self::MarginSummary(_) => "MarginSummary",
            Self::FundingRates(_) => "FundingRates",
            Self::FundingPayments(_) => "FundingPayments",
            Self::Unit => "Unit",
        }
    }
}
