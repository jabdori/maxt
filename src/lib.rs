//! A common Rust API for Upbit, Bithumb, Binance, and Hyperliquid.
//!
//! [`Client`] exposes operations with common semantics. Exchange-specific
//! operations remain on the concrete types in [`adapters`], and unavailable
//! operations return [`Error::Unsupported`].
//!
//! # Quick start: Binance Spot
//!
//! Pick an adapter, wrap it in a [`Client`], and call the common API. This
//! credential-free example reads Binance Spot `BTC/USDT`; it does not submit
//! an order.
//!
//! ```no_run
//! use maxt::{Client, Exchange, Market, adapters::BinanceAdapter};
//!
//! # async fn run() -> maxt::Result<()> {
//! let client = Client::new(BinanceAdapter::spot());
//! let market = Market::spot(Exchange::Binance, "BTC", "USDT");
//! let ticker = client.ticker(&market).await?;
//! let average = client.adapter().spot_average_price(&market).await?;
//!
//! println!("last price: {}", ticker.last_price);
//! println!("{}-minute average: {}", average.minutes, average.price);
//! # Ok(())
//! # }
//! ```
//!
//! [`Client::ticker`] is a common operation. [`BinanceAdapter::spot_average_price`]
//! is a provider-specific operation, so it remains on [`Client::adapter`].
//! Public market data needs no credentials. Account and order operations do;
//! each adapter documents the credential form it accepts.
//!
//! # Supported adapter boundaries
//!
//! - [`BinanceAdapter`](adapters::BinanceAdapter): Spot and USD-M perpetual futures.
//! - [`UpbitAdapter`](adapters::UpbitAdapter): Spot across its supported regions.
//! - [`BithumbAdapter`](adapters::BithumbAdapter): Spot and KRW account APIs.
//! - [`HyperliquidAdapter`](adapters::HyperliquidAdapter): Spot and perpetual futures.
//!
//! Provider-only fields and endpoints remain on the concrete adapter. Check
//! [`Client::supports`] before an optional common operation when the configured
//! adapter or its credentials can vary at runtime.
//!
//! # Main types
//!
//! - [`Client`]: common market-data, account, order, and derivatives operations.
//! - [`adapters`]: exchange constructors and exchange-specific operations.
//! - [`Feature`]: capabilities reported by [`Client::supports`].
//! - [`Error`]: validation, capability, authentication, exchange, transport,
//!   and decoding failures.

#![doc(html_no_source)]

mod adapter;
mod client;
mod error;
mod feature;
mod request;
mod stream;
mod transport;
mod types;
mod wallet;

pub mod adapters;

pub use adapter::{Adapter, BoxFuture};
pub use adapters::{
    BinanceAccountTrade, BinanceAggregateTrade, BinanceAggregateTradesRequest, BinanceC2cTrade,
    BinanceC2cTradeHistoryPage, BinanceC2cTradeHistoryRequest, BinanceC2cTradeType,
    BinanceMarkPrice, BinanceOpenInterest, BinanceSpotAveragePrice, BinanceTestOrder,
    BinanceTestOrderRequest, BithumbApiKey, BithumbAssetFee, BithumbBatchOrder,
    BithumbBatchOrderFailure, BithumbBatchOrderOutcome, BithumbBatchOrdersRequest,
    BithumbBatchOrdersResult, BithumbClosedOrder, BithumbClosedOrderState,
    BithumbClosedOrdersRequest, BithumbKrwDeposit, BithumbKrwDepositsRequest,
    BithumbKrwTransferRequest, BithumbKrwWithdrawal, BithumbKrwWithdrawalsRequest,
    BithumbNetworkFee, BithumbNotice, BithumbOrderDetail, BithumbOrderDetailRequest,
    BithumbOrderDetailTrade, BithumbOrderDirection, BithumbOrderListItem, BithumbOrderListRequest,
    BithumbOrderListState, BithumbPendingOrderState, BithumbPendingOrdersRequest, BithumbTwapOrder,
    BithumbTwapOrderDirection, BithumbTwapOrderRequest, BithumbTwapOrdersRequest, BithumbTwapState,
    BithumbWithdrawalAddress, HyperliquidDailyVolume, HyperliquidMidPrice, HyperliquidOpenOrder,
    HyperliquidOrderDetail, HyperliquidOrderInfo, HyperliquidOrderReference,
    HyperliquidOrderStatusResponse, HyperliquidPortfolioPeriod, HyperliquidPortfolioPoint,
    HyperliquidReferral, HyperliquidReferrer, HyperliquidSubAccount, HyperliquidUserFees,
    HyperliquidUserFill, HyperliquidUserRateLimit, HyperliquidUserRole, HyperliquidVaultEquity,
    UpbitApiKey, UpbitBatchCancelRequest, UpbitBatchCancelScope, UpbitCancelAndNewOrder,
    UpbitCancelAndNewOrderRequest, UpbitCancelAndNewOrderResult, UpbitClosedOrder,
    UpbitClosedOrderState, UpbitClosedOrdersRequest, UpbitDepositInfo, UpbitKrwDeposit,
    UpbitKrwTransferRequest, UpbitKrwTwoFactorType, UpbitKrwWithdrawal, UpbitListedSubscription,
    UpbitOrderBookInstrument, UpbitOrderDetail, UpbitOrderDetailRequest, UpbitOrderDetailTrade,
    UpbitOrderDirection, UpbitOrderReference, UpbitOrderVolume, UpbitPocket, UpbitPocketApiKey,
    UpbitPocketApiKeyGroup, UpbitPocketApiKeysRequest, UpbitPocketBalance, UpbitPocketTransfer,
    UpbitPocketTransferDirection, UpbitPocketTransferOrder, UpbitPocketTransferQuery,
    UpbitPocketTransferRequest, UpbitPocketTransferState, UpbitPocketUniversalTransferRequest,
    UpbitSmpType, UpbitSubscriptionList, UpbitTravelRuleVasp, UpbitTravelRuleVerification,
    UpbitYearCandle,
};
pub use client::Client;
pub use error::{Error, ExchangeErrorKind, Result, TransferErrorKind};
pub use feature::Feature;
pub use request::{
    CancelOrdersRequest, CandleRequest, DepositAddressRequest, HistoryRequest, MarginRequest,
    OrderHistoryRequest, OrderIdKind, OrderLookupRequest, OrderRequest, TransferHistoryRequest,
    TransferLookupRequest, WithdrawRequest,
};
pub use stream::{AccountStream, MarketStream};
pub use types::{
    AccountEvent, AssetNetwork, Balance, CancelOrdersResult, CancelledOrder, Candle,
    ChainDestination, ChainTransferRequest, Cursor, Deposit, DepositAddress, DepositAddressEntry,
    DepositStatus, Exchange, ExchangeDestination, ExchangeTransferRequest, Feed, FundingPayment,
    FundingRate, Interval, Level, MarginMode, MarginSummary, Market, MarketEvent, MarketInfo,
    MarketKind, MarketStatus, Network, Order, OrderAccount, OrderBook, OrderCancelFailure,
    OrderOption, OrderRules, OrderStatus, OrderType, Overflow, Page, Position, Side, Size,
    StreamConfig, Subscription, Ticker, TimeInForce, Timestamp, Trade, TransferDestination,
    TravelRuleRequirement, Withdrawal, WithdrawalFee, WithdrawalQuote, WithdrawalStatus,
};
pub use wallet::{
    PreparedTransfer, TransferPlan, Wallet, execute_transfer_plan, prepare_chain_transfer,
    prepare_exchange_transfer,
};

/// The exact decimal type used for every price, quantity, and amount.
///
/// Import this re-export when constructing values for `maxt`; a direct
/// `rust_decimal` dependency is not required.
pub use rust_decimal::Decimal;

/// Configures the trusted relay used for credentialed browser HTTP and WebSocket calls.
///
/// The value must be an `http` or `https` origin without credentials, a path,
/// query, or fragment. Configuration is process-wide and may be repeated only
/// with the same normalized origin.
#[cfg(target_arch = "wasm32")]
pub fn configure_browser_relay(relay_url: &str) -> Result<()> {
    transport::configure_browser_relay(relay_url)
}

/// Parses decimal text without rounding or truncating it.
///
/// Plain and scientific notation are accepted. Values that [`Decimal`] cannot
/// represent exactly return an error.
pub fn parse_decimal_exact(text: &str) -> std::result::Result<Decimal, rust_decimal::Error> {
    adapters::decimal::exact(text)
}

/// Repository Markdown included only while running documentation tests.
///
/// This keeps Rust examples in the READMEs and `docs/` compiled without adding
/// them to the rendered API documentation.
#[cfg(doctest)]
mod markdown {
    macro_rules! compiled {
        ($($name:ident => $path:literal,)*) => {
            $(#[doc = include_str!($path)] pub mod $name {})*
        };
    }

    compiled! {
        readme => "../README.md",
        readme_ko => "../README.ko.md",
        contributing => "../CONTRIBUTING.md",
        contributing_ko => "../CONTRIBUTING.ko.md",
        getting_started => "../docs/getting-started.md",
        getting_started_ko => "../docs/getting-started.ko.md",
        common_api => "../docs/common-api.md",
        common_api_ko => "../docs/common-api.ko.md",
        providers => "../docs/providers.md",
        providers_ko => "../docs/providers.ko.md",
        upbit => "../docs/providers/upbit.md",
        upbit_ko => "../docs/providers/upbit.ko.md",
        bithumb => "../docs/providers/bithumb.md",
        bithumb_ko => "../docs/providers/bithumb.ko.md",
        binance => "../docs/providers/binance.md",
        binance_ko => "../docs/providers/binance.ko.md",
        hyperliquid => "../docs/providers/hyperliquid.md",
        hyperliquid_ko => "../docs/providers/hyperliquid.ko.md",
    }
}
