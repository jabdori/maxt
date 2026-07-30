//! Request builders for the calls that take more than one argument.

use rust_decimal::Decimal;

use crate::types::{
    Cursor, Interval, MarginMode, Market, OrderType, Side, Size, TimeInForce, Timestamp,
};

/// Which candles to read.
///
/// The market and interval are required; everything else narrows the range.
/// With no range the exchange returns its most recent candles.
///
/// ```
/// use maxt::{CandleRequest, Exchange, Interval, Market, Timestamp};
///
/// let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
///
/// // The newest 200 one-minute candles.
/// let latest = CandleRequest::new(market.clone(), Interval::Min1).limit(200);
///
/// // The same 200, counted forward from a start time. That is what a backfill
/// // walking towards the present asks for. `limit` alone would have given the
/// // newest 200 and skipped everything between.
/// let backfill = latest
///     .clone()
///     .from(Timestamp::from_millis(1_700_000_000_000));
///
/// // A bounded window: both ends set, and `limit` left to the range.
/// let window = CandleRequest::new(market, Interval::Hour1)
///     .from(Timestamp::from_millis(1_700_000_000_000))
///     .to(Timestamp::from_millis(1_700_086_400_000));
///
/// assert_eq!(latest.from, None);
/// assert_eq!(backfill.limit, Some(200));
/// assert_eq!(window.limit, None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleRequest {
    /// The market to read candles for.
    pub market: Market,
    /// The candle interval.
    pub interval: Interval,
    /// Oldest candle to return, by open time, inclusive.
    ///
    /// With no `limit` this is the only thing bounding the read, and `maxt`
    /// walks backwards one page at a time to reach it. A window needing more
    /// than a hundred pages is refused as
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) before the first
    /// call, rather than begun and abandoned by a rate limit: at Upbit's two
    /// hundred candles a call that is twenty thousand candles. Set `limit` to
    /// take a wide history in batches.
    pub from: Option<Timestamp>,
    /// Newest candle to return, by open time, exclusive.
    pub to: Option<Timestamp>,
    /// How many candles to return.
    ///
    /// Which end it counts from depends on whether the range is anchored. With
    /// [`CandleRequest::from`] set it is the *oldest* `limit` candles at or
    /// after that time. Without it, the newest `limit`.
    ///
    /// Exchanges cap how many candles one response may carry. `maxt` pages
    /// behind the scenes to reach a larger `limit`, so the cap does not bound
    /// this field. Paging is itself bounded at a hundred calls, so a `limit`
    /// above a hundred times the exchange's per-response cap is refused, and
    /// the refusal says what the ceiling is there. That ceiling is the only thing
    /// that refuses a count, at every interval including [`Interval::Month1`],
    /// whose pages are walked in calendar months.
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

    /// Returns candles opening at or after this time.
    #[must_use]
    pub fn from(mut self, from: Timestamp) -> Self {
        self.from = Some(from);
        self
    }

    /// Returns candles opening strictly before this time.
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
///
/// ```
/// use maxt::{Exchange, Market, OrderRequest, Side, Size};
/// use rust_decimal::Decimal;
///
/// let btc_krw = Market::spot(Exchange::Upbit, "BTC", "KRW");
///
/// // Spend 10,000 KRW at whatever the book offers.
/// let buy = OrderRequest::market(btc_krw.clone(), Side::Buy, Size::Quote(Decimal::from(10_000)));
///
/// // Offer 0.01 BTC at 100,000,000 KRW.
/// let sell = OrderRequest::limit(
///     btc_krw,
///     Side::Sell,
///     Size::Base(Decimal::new(1, 2)),
///     Decimal::from(100_000_000),
/// );
///
/// assert!(buy.price.is_none());
/// assert!(sell.price.is_some());
/// ```
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
    ///
    /// ```
    /// use maxt::{Exchange, Market, OrderRequest, Side, Size};
    /// use rust_decimal::Decimal;
    ///
    /// // Closing half a long. Reduce-only stops a late fill from opening a
    /// // short if the position shrank in the meantime.
    /// let close = OrderRequest::market(
    ///     Market::perpetual(Exchange::Binance, "BTC", "USDT"),
    ///     Side::Sell,
    ///     Size::Base(Decimal::new(25, 2)),
    /// )
    /// .reduce_only();
    ///
    /// assert!(close.reduce_only);
    /// ```
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
///
/// ```
/// use maxt::{Cursor, Exchange, HistoryRequest, Market, Timestamp};
///
/// let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
///
/// let first = HistoryRequest::new(market)
///     .from(Timestamp::from_millis(1_700_000_000_000))
///     .to(Timestamp::from_millis(1_700_086_400_000))
///     // Per page, not in total: the window is what bounds the walk.
///     .limit(100);
///
/// // Each later page keeps the window and changes only the cursor, so the
/// // range is read the same way throughout the walk.
/// let second = first.clone().cursor(Cursor::new("after-page-1"));
///
/// assert_eq!(first.cursor, None);
/// assert_eq!(second.from, first.from);
/// assert_eq!(second.cursor.as_ref().map(Cursor::as_str), Some("after-page-1"));
/// ```
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
    /// Roughly how many entries to return per page. See
    /// [`HistoryRequest::limit`] for why a page may be a little longer.
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

    /// Returns entries at or after this time.
    #[must_use]
    pub fn from(mut self, from: Timestamp) -> Self {
        self.from = Some(from);
        self
    }

    /// Returns entries strictly before this time.
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

    /// Sets the page size. A total is what the window and the cursor decide.
    ///
    /// Read it as "about this many per page", not as a hard ceiling. A page may
    /// come back slightly longer: Hyperliquid reads 500 entries at a time and
    /// trims to this figure, and because the next cursor resumes one
    /// millisecond past the last entry kept, a cut landing inside a run of
    /// entries that share one millisecond would strand the rest of that run.
    /// The trim backs up to the start of the run, and when the run reaches the
    /// front of the page there is nothing to back up to, so the whole run is
    /// kept. Extra entries a caller can drop beat entries the cursor has
    /// already moved past.
    ///
    /// So do not size a fixed buffer off this, and do not assert
    /// `page.items.len() <= limit`. A busy millisecond breaks both.
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
/// ```
/// use maxt::{Exchange, MarginMode, MarginRequest, Market};
/// use rust_decimal::Decimal;
///
/// let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
///
/// // Both at once. Two requests would briefly leave the account isolated at
/// // whatever leverage it already had.
/// let both = MarginRequest::new(market.clone())
///     .leverage(Decimal::from(5))
///     .margin_mode(MarginMode::Isolated);
///
/// // Either alone is fine. A field left unset is left alone on the exchange.
/// let leverage_only = MarginRequest::new(market).leverage(Decimal::from(3));
///
/// assert_eq!(both.margin_mode, Some(MarginMode::Isolated));
/// assert_eq!(leverage_only.margin_mode, None);
/// ```
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
