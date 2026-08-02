//! Request builders for the calls that take more than one argument.

use rust_decimal::Decimal;

use crate::types::{
    Cursor, Interval, MarginMode, Market, OrderType, Side, Size, TimeInForce, Timestamp,
};

/// Which candles to read.
///
/// Results are sorted oldest first.
///
/// - `from`: inclusive lower bound. Without `limit`, reads through `to` or now.
/// - `to`: exclusive upper bound.
/// - `limit`: selects the oldest matches when `from` is set, otherwise the
///   newest.
/// - Paging: at most 100 exchange calls per request.
///
/// With all optional fields unset, returns the exchange's most recent page.
/// Requests estimated to exceed the paging limit return
/// [`Error::InvalidRequest`](crate::Error::InvalidRequest) before the first
/// exchange call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleRequest {
    /// The market to read candles for.
    pub market: Market,
    /// The candle interval.
    pub interval: Interval,
    /// Oldest candle to return, by open time, inclusive.
    pub from: Option<Timestamp>,
    /// Newest candle to return, by open time, exclusive.
    pub to: Option<Timestamp>,
    /// Target number of candles. Must be at least one when set.
    ///
    /// This may span multiple provider responses, subject to the request's
    /// paging limit.
    pub limit: Option<u32>,
}

impl CandleRequest {
    /// The most recent candles for a market at one interval.
    pub fn new(market: Market, interval: Interval) -> Self {
        Self {
            market,
            interval,
            from: None,
            to: None,
            limit: None,
        }
    }

    /// Sets the inclusive bound `open_time >= from`.
    #[must_use]
    pub fn from(mut self, from: Timestamp) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the exclusive bound `open_time < to`.
    #[must_use]
    pub fn to(mut self, to: Timestamp) -> Self {
        self.to = Some(to);
        self
    }

    /// Returns at most this many candles.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// An order to place.
///
/// Build it with [`OrderRequest::market`] or [`OrderRequest::limit`], which
/// keep price and size in step. A market order has no price, and a limit order
/// always has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderRequest {
    /// The market to trade.
    pub market: Market,
    /// Buy or sell.
    pub side: Side,
    /// Market or limit.
    pub order_type: OrderType,
    /// How much, in base or quote terms.
    pub size: Size,
    /// The limit price. Always `None` for market orders.
    pub price: Option<Decimal>,
    /// How long the order stays live. `None` leaves it to the exchange default.
    pub time_in_force: Option<TimeInForce>,
    /// Whether the order may only reduce an existing position.
    ///
    /// Derivatives only. Set it with [`OrderRequest::reduce_only`].
    pub reduce_only: bool,
}

impl OrderRequest {
    /// An order that takes whatever the book offers, immediately.
    pub fn market(market: Market, side: Side, size: Size) -> Self {
        Self {
            market,
            side,
            order_type: OrderType::Market,
            size,
            price: None,
            time_in_force: None,
            reduce_only: false,
        }
    }

    /// An order that rests on the book at a stated price.
    pub fn limit(market: Market, side: Side, size: Size, price: Decimal) -> Self {
        Self {
            market,
            side,
            order_type: OrderType::Limit,
            size,
            price: Some(price),
            time_in_force: None,
            reduce_only: false,
        }
    }

    /// Sets how long the order stays live.
    #[must_use]
    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = Some(time_in_force);
        self
    }

    /// Restricts the order to reducing an existing position.
    ///
    /// Rejected as [`Error::Unsupported`](crate::Error::Unsupported) on spot
    /// markets, which have no positions to reduce.
    #[must_use]
    pub fn reduce_only(mut self) -> Self {
        self.reduce_only = true;
        self
    }
}

/// A window of history to read, one page at a time.
///
/// Leave [`HistoryRequest::cursor`] unset for the first page, then pass the
/// [`Page::next`](crate::Page::next) cursor back for each page after that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRequest {
    /// The market to read history for.
    pub market: Market,
    /// Oldest entry to return, inclusive.
    pub from: Option<Timestamp>,
    /// Newest entry to return, exclusive.
    pub to: Option<Timestamp>,
    /// Where to resume from. `None` starts at the beginning of the window.
    pub cursor: Option<Cursor>,
    /// Target page size, not a hard maximum.
    ///
    /// A provider may return more entries to avoid splitting entries that share
    /// one timestamp.
    pub limit: Option<u32>,
}

impl HistoryRequest {
    /// The first page of history for a market.
    pub fn new(market: Market) -> Self {
        Self {
            market,
            from: None,
            to: None,
            cursor: None,
            limit: None,
        }
    }

    /// Sets the inclusive bound `item.timestamp >= from`.
    #[must_use]
    pub fn from(mut self, from: Timestamp) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the exclusive bound `item.timestamp < to`.
    #[must_use]
    pub fn to(mut self, to: Timestamp) -> Self {
        self.to = Some(to);
        self
    }

    /// Resumes from a cursor returned by a previous page.
    #[must_use]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Sets the target page size, not a hard maximum.
    ///
    /// A page may be longer when trimming it would split entries sharing one
    /// timestamp and make the cursor skip entries. Do not size a fixed buffer
    /// from this value. Zero is invalid; provider maximums differ.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// A change to how margin backs one market.
///
/// At least one of leverage or margin mode must be set. A request that changes
/// nothing is rejected as
/// [`Error::InvalidRequest`](crate::Error::InvalidRequest).
///
/// Provider constraints differ. Some accept either field; others require both.
/// Applying both is not guaranteed to be atomic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarginRequest {
    /// The market to configure.
    pub market: Market,
    /// The leverage to set.
    pub leverage: Option<Decimal>,
    /// The margin mode to set.
    pub margin_mode: Option<MarginMode>,
}

impl MarginRequest {
    /// A margin change for one market, with nothing set yet.
    pub fn new(market: Market) -> Self {
        Self {
            market,
            leverage: None,
            margin_mode: None,
        }
    }

    /// Sets the leverage.
    #[must_use]
    pub fn leverage(mut self, leverage: Decimal) -> Self {
        self.leverage = Some(leverage);
        self
    }

    /// Sets the margin mode.
    #[must_use]
    pub fn margin_mode(mut self, margin_mode: MarginMode) -> Self {
        self.margin_mode = Some(margin_mode);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Exchange;

    fn btc_krw() -> Market {
        Market::spot(Exchange::Upbit, "BTC", "KRW")
    }

    #[test]
    fn market_orders_carry_no_price_and_limit_orders_always_do() {
        let market_order =
            OrderRequest::market(btc_krw(), Side::Buy, Size::Quote(Decimal::from(10_000)));
        let limit_order = OrderRequest::limit(
            btc_krw(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000_000),
        );

        assert_eq!(market_order.order_type, OrderType::Market);
        assert_eq!(market_order.price, None);
        assert_eq!(limit_order.order_type, OrderType::Limit);
        assert_eq!(limit_order.price, Some(Decimal::from(100_000_000)));
    }

    #[test]
    fn orders_are_not_reduce_only_unless_asked() {
        let plain = OrderRequest::market(btc_krw(), Side::Buy, Size::Base(Decimal::ONE));
        let reducing = plain.clone().reduce_only();

        assert!(!plain.reduce_only);
        assert!(reducing.reduce_only);
    }

    #[test]
    fn builders_leave_unset_fields_alone() {
        let request = CandleRequest::new(btc_krw(), Interval::Min1).limit(200);

        assert_eq!(request.limit, Some(200));
        assert_eq!(request.from, None);
        assert_eq!(request.to, None);
    }

    #[test]
    fn a_history_page_resumes_from_the_previous_cursor() {
        let first = HistoryRequest::new(btc_krw());
        assert!(first.cursor.is_none());

        let second = HistoryRequest::new(btc_krw()).cursor(Cursor("page-2".to_string()));
        assert_eq!(second.cursor.unwrap().as_str(), "page-2");
    }

    #[test]
    fn a_margin_request_can_set_either_field_independently() {
        let leverage_only = MarginRequest::new(btc_krw()).leverage(Decimal::from(10));
        let mode_only = MarginRequest::new(btc_krw()).margin_mode(MarginMode::Isolated);

        assert_eq!(leverage_only.leverage, Some(Decimal::from(10)));
        assert_eq!(leverage_only.margin_mode, None);
        assert_eq!(mode_only.leverage, None);
        assert_eq!(mode_only.margin_mode, Some(MarginMode::Isolated));
    }
}
