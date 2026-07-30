//! The data `maxt` speaks in.
//!
//! Every type here means the same thing on every exchange. Where an exchange
//! cannot express something, the field is `Option` and the adapter leaves it
//! `None`.

mod account;
mod data;
mod market;
mod stream;
mod time;

pub use account::{
    Balance, Cursor, FundingPayment, FundingRate, MarginMode, MarginSummary, Order, OrderStatus,
    OrderType, Page, Position, Size, TimeInForce,
};
pub use data::{Candle, Interval, Level, OrderBook, Side, Ticker, Trade};
pub use market::{Exchange, Market, MarketInfo, MarketKind, MarketStatus};
pub use stream::{AccountEvent, Feed, MarketEvent, Overflow, StreamConfig, Subscription};
pub use time::Timestamp;
