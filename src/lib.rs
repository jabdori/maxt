//! One Rust API for several cryptocurrency exchanges.
//!
//! `maxt` hides the differences between exchange APIs behind a single
//! [`Client`], so reading a ticker, subscribing to trades, or placing an order
//! looks the same on Upbit, Bithumb, Binance, and Hyperliquid.
//!
//! The exchanges are not identical, and the API does not pretend otherwise.
//! Anything that would lose meaning once generalized stays on the exchange's
//! own adapter as a typed method. Anything an exchange does not offer is
//! reported as [`Error::Unsupported`].
//!
//! # Getting started
//!
//! Pick an adapter, wrap it in a [`Client`], and call the common API.
//!
//! ```no_run
//! use maxt::{Client, Exchange, Market, adapters::UpbitAdapter};
//!
//! # async fn run() -> maxt::Result<()> {
//! let client = Client::new(UpbitAdapter::new());
//! let book = client
//!     .order_book(&Market::spot(Exchange::Upbit, "BTC", "KRW"), Some(5))
//!     .await?;
//!
//! if let Some(spread) = book.spread() {
//!     println!("spread: {spread}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Market data needs no credentials. Account and order calls do, and each
//! adapter takes them in whatever form its own exchange uses.
//!
//! # Where things live
//!
//! - [`Client`]: the common API for market data, account, orders, derivatives.
//! - [`adapters`]: one adapter per exchange, plus their exchange-specific
//!   methods.
//! - [`Feature`]: what an exchange can do, askable ahead of time with
//!   [`Client::supports`].
//! - [`Error`]: what can go wrong, split by what a caller can do about it.
//!
//! Runnable programs live in the repository's `examples/` directory.

#![doc(html_no_source)]

mod adapter;
mod client;
mod error;
mod feature;
mod request;
mod stream;
mod transport;
mod types;

pub mod adapters;

pub use adapter::{Adapter, BoxFuture};
pub use client::Client;
pub use error::{Error, ExchangeErrorKind, Result};
pub use feature::Feature;
pub use request::{CandleRequest, HistoryRequest, MarginRequest, OrderRequest};
pub use stream::{AccountStream, MarketStream};
pub use types::{
    AccountEvent, Balance, Candle, Cursor, Exchange, Feed, FundingPayment, FundingRate, Interval,
    Level, MarginMode, MarginSummary, Market, MarketEvent, MarketInfo, MarketKind, MarketStatus,
    Order, OrderBook, OrderStatus, OrderType, Overflow, Page, Position, Side, Size, StreamConfig,
    Subscription, Ticker, TimeInForce, Timestamp, Trade,
};

/// The exact decimal type used for every price, quantity, and amount.
///
/// Re-exported so callers need no direct dependency on `rust_decimal`, and
/// cannot end up on a different version of it.
pub use rust_decimal::Decimal;

/// Every Markdown file in the repository, compiled.
///
/// The `docs/` tree and both READMEs carry Rust the reader is meant to copy,
/// and nothing in the crate reached it: `cargo test --doc` collects only what
/// `src/` writes. Six rounds of review each rebuilt a scratch crate by hand to
/// find out whether those blocks still compiled, which is the shape of a check
/// that exists in nobody's habit.
///
/// `cfg(doctest)` keeps these out of the rendered documentation. They exist so
/// that a block a reader copies is a block CI compiled, in both editions, so
/// that the Korean translation cannot drift away from the code it carries.
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
