//! Exchange adapter implementations.
//!
//! Adapters support public data without credentials and expose
//! provider-specific methods through [`Client::adapter`](crate::Client::adapter).
//! See `docs/providers.md` for capabilities and credential types.

use crate::Timestamp;

mod binance;
mod bithumb;
mod hyperliquid;
mod upbit;

pub(crate) mod candles;
pub(crate) mod decimal;

/// Last whole millisecond strictly before an exclusive timestamp.
pub(crate) fn inclusive_millis_before(end: Timestamp) -> i64 {
    let nanos = end.as_nanos();
    nanos.div_euclid(1_000_000) - i64::from(nanos.rem_euclid(1_000_000) == 0)
}

/// Smallest millisecond satisfying `millis * 1_000_000 >= timestamp`.
pub(crate) fn inclusive_millis_at_or_after(start: Timestamp) -> i64 {
    let nanos = start.as_nanos();
    nanos.div_euclid(1_000_000) + i64::from(nanos.rem_euclid(1_000_000) != 0)
}

pub use binance::{
    BinanceAdapter, BinanceListenKey, BinanceMarket, BinanceSpotOrderDetail, BinanceSymbolFilters,
};
pub use bithumb::{BithumbAdapter, BithumbAlertStep, BithumbMarketAlert};
pub use hyperliquid::{
    HyperliquidAdapter, HyperliquidAssetContext, HyperliquidLedgerEntry, HyperliquidLedgerKind,
};
pub use upbit::{UpbitAdapter, UpbitMarketEvent, UpbitRegion};
