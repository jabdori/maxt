//! Common exchange data types.
//!
//! Provider-specific missing values are represented as `None` on optional
//! fields rather than as invented defaults.

mod account;
mod data;
mod market;
mod stream;
mod time;
mod wallet;

pub use account::{
    Balance, CancelOrdersResult, CancelledOrder, Cursor, FundingPayment, FundingRate, MarginMode,
    MarginSummary, Order, OrderAccount, OrderCancelFailure, OrderOption, OrderRules, OrderStatus,
    OrderType, Page, Position, Size, TimeInForce,
};
pub use data::{Candle, Interval, Level, OrderBook, Side, Ticker, Trade};
pub use market::{Exchange, Market, MarketInfo, MarketKind, MarketStatus};
pub use stream::{AccountEvent, Feed, MarketEvent, Overflow, StreamConfig, Subscription};
pub use time::Timestamp;
pub use wallet::{
    AssetNetwork, ChainDestination, ChainTransferRequest, Deposit, DepositAddress, DepositStatus,
    ExchangeDestination, ExchangeTransferRequest, Network, TransferDestination,
    TravelRuleRequirement, Withdrawal, WithdrawalFee, WithdrawalQuote, WithdrawalStatus,
};

#[cfg(test)]
pub(crate) use time::clock;
