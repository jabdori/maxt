//! Exchange adapter implementations.
//!
//! Adapters support public data without credentials and expose
//! provider-specific methods through [`Client::adapter`](crate::Client::adapter).
//! See `docs/providers.md` for capabilities and credential types.

use crate::{Error, OrderHistoryRequest, OrderStatus, Result, Timestamp};

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

/// Provider state code for a final-order filter, or `None` for both states.
pub(crate) fn final_order_state(statuses: &[OrderStatus]) -> Result<Option<&'static str>> {
    let mut filled = false;
    let mut cancelled = false;

    for status in statuses {
        match status {
            OrderStatus::Filled => filled = true,
            OrderStatus::Cancelled => cancelled = true,
            other => {
                return Err(Error::invalid_request(
                    "statuses",
                    format!("order history accepts only Filled or Cancelled, not {other:?}"),
                ));
            }
        }
    }

    Ok(match (filled, cancelled) {
        (true, false) => Some("done"),
        (false, true) => Some("cancel"),
        _ => None,
    })
}

/// Validates the shared seven-day final-order history window.
pub(crate) fn order_history_window(
    request: &OrderHistoryRequest,
) -> Result<(u32, Option<i64>, Option<i64>)> {
    const DEFAULT_LIMIT: u32 = 100;
    const MAX_LIMIT: u32 = 1_000;
    const SEVEN_DAYS_NANOS: i64 = 7 * 24 * 60 * 60 * 1_000_000_000;

    let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!("order history serves 1 to {MAX_LIMIT} entries per page, not {limit}"),
        ));
    }

    if let (Some(from), Some(to)) = (request.from, request.to) {
        let width = to
            .as_nanos()
            .checked_sub(from.as_nanos())
            .ok_or_else(|| Error::invalid_request("to", "must be later than `from`"))?;
        if width <= 0 {
            return Err(Error::invalid_request("to", "must be later than `from`"));
        }
        if width > SEVEN_DAYS_NANOS {
            return Err(Error::invalid_request(
                "to",
                "order history windows cannot exceed seven days",
            ));
        }
    }

    Ok((
        limit,
        request.from.map(inclusive_millis_at_or_after),
        request.to.map(inclusive_millis_before),
    ))
}

pub use binance::{
    BinanceAdapter, BinanceListenKey, BinanceMarket, BinanceSpotOrderDetail, BinanceSymbolFilters,
};
pub use bithumb::{BithumbAdapter, BithumbAlertStep, BithumbMarketAlert};
pub use hyperliquid::{
    HyperliquidAdapter, HyperliquidAssetContext, HyperliquidLedgerEntry, HyperliquidLedgerKind,
};
pub use upbit::{UpbitAdapter, UpbitMarketEvent, UpbitRegion};
