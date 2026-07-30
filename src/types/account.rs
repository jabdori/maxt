//! Account state: balances, orders, positions, margin, funding.

use rust_decimal::Decimal;

use crate::types::{Market, Side, Timestamp};

/// How much of one asset an account holds.
///
/// ```
/// use maxt::Balance;
/// use rust_decimal::Decimal;
///
/// let krw = Balance {
///     asset: "KRW".to_string(),
///     available: Decimal::from(750_000),
///     locked: Decimal::from(250_000),
/// };
///
/// // Size an order off `available`. The locked part is promised to resting
/// // orders, and spending it is what the exchange rejects.
/// assert_eq!(krw.available, Decimal::from(750_000));
/// // Report `total`, which is what the account is actually worth.
/// assert_eq!(krw.total(), Decimal::from(1_000_000));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    /// The asset, uppercase. For example `KRW` or `BTC`.
    pub asset: String,
    /// Free to trade or withdraw.
    pub available: Decimal,
    /// Reserved against open orders or positions.
    pub locked: Decimal,
}

impl Balance {
    /// Available plus locked.
    pub fn total(&self) -> Decimal {
        self.available + self.locked
    }
}

/// How an order is priced and matched.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OrderType {
    /// Take whatever the book offers, immediately.
    Market,
    /// Rest on the book at a stated price.
    Limit,
}

/// How long an order stays live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeInForce {
    /// Rest until filled or cancelled.
    GoodTilCancelled,
    /// Fill what is available immediately, cancel the rest.
    ImmediateOrCancel,
    /// Fill entirely and immediately, or not at all.
    FillOrKill,
    /// Rest only if it does not take liquidity; otherwise reject.
    PostOnly,
}

/// How an order's size is expressed.
///
/// A market buy is usually sized in the quote asset ("spend 10,000 KRW") while
/// a market sell is sized in the base asset ("sell 0.01 BTC"). Making that
/// explicit at the type level keeps the two from being confused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Size {
    /// A quantity of the base asset. For example `0.01` BTC.
    Base(Decimal),
    /// An amount of the quote asset. For example `10000` KRW.
    Quote(Decimal),
}

/// Where an order stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OrderStatus {
    /// Accepted, not yet known to be on the book.
    Accepted,
    /// Resting on the book, unfilled.
    Open,
    /// Resting on the book, partly filled.
    PartiallyFilled,
    /// Completely filled.
    Filled,
    /// Cancelled, whether by the caller or by the exchange.
    Cancelled,
    /// Rejected by the exchange.
    Rejected,
    /// The exchange reported a status `maxt` does not map.
    Unknown,
}

impl OrderStatus {
    /// Whether the order can still fill.
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Accepted | Self::Open | Self::PartiallyFilled)
    }
}

/// An order as the exchange currently sees it.
///
/// ```
/// use maxt::{Order, OrderStatus};
/// use rust_decimal::Decimal;
///
/// /// Whether it is worth asking about this order again.
/// fn still_working(order: &Order) -> bool {
///     // Partially filled counts: the rest can still fill, and dropping it
///     // from a tracking table would leak the remainder.
///     order.status.is_live() && !order.remaining_quantity.is_zero()
/// }
///
/// // The same order part-filled, then finished.
/// # use maxt::{Exchange, Market, Side};
/// # let partial = Order {
/// #     id: "9c8f".to_string(),
/// #     market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
/// #     side: Side::Buy,
/// #     status: OrderStatus::PartiallyFilled,
/// #     filled_quantity: Decimal::new(4, 1),
/// #     remaining_quantity: Decimal::new(6, 1),
/// #     price: Some(Decimal::from(100_000_000)),
/// #     created_at: None,
/// # };
/// # let done = Order { status: OrderStatus::Filled, remaining_quantity: Decimal::ZERO, ..partial.clone() };
/// assert!(still_working(&partial));
/// assert!(!still_working(&done));
///
/// // `id` is the exchange's own, and is what cancels the order.
/// assert_eq!(partial.id, "9c8f");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    /// The exchange's own order identifier. Pass this to cancel.
    pub id: String,
    /// The market it was placed on.
    pub market: Market,
    /// Buy or sell.
    pub side: Side,
    /// Where it stands.
    pub status: OrderStatus,
    /// Filled so far, in the base asset.
    pub filled_quantity: Decimal,
    /// Still working, in the base asset.
    pub remaining_quantity: Decimal,
    /// The limit price, for orders that have one.
    pub price: Option<Decimal>,
    /// When the exchange accepted it, when it says.
    pub created_at: Option<Timestamp>,
}

/// An open derivatives position.
///
/// Fields an exchange does not publish are `None`. Reading `None` as zero would
/// misreport an unleveraged position as a missing one.
///
/// ```
/// use maxt::{Exchange, Market, Position, Side};
/// use rust_decimal::Decimal;
///
/// let long = Position {
///     market: Market::perpetual(Exchange::Binance, "BTC", "USDT"),
///     side: Some(Side::Buy),
///     // Always unsigned: the direction lives in `side`, so a short does not
///     // arrive here as a negative quantity.
///     quantity: Decimal::new(5, 1),
///     entry_price: Some(Decimal::from(60_000)),
///     mark_price: Some(Decimal::from(61_000)),
///     notional: Some(Decimal::from(30_500)),
///     unrealized_pnl: Some(Decimal::from(500)),
///     leverage: None,
///     margin_mode: None,
/// };
///
/// assert!(!long.is_flat());
/// // Unset means the exchange did not say. Defaulting it to 1 would understate
/// // the risk of a position opened at 20x.
/// assert_eq!(long.leverage, None);
///
/// let closed = Position { quantity: Decimal::ZERO, side: None, ..long };
/// assert!(closed.is_flat());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    /// The market the position is in.
    pub market: Market,
    /// Long or short. `None` when the position is flat.
    pub side: Option<Side>,
    /// Position size, in the base asset, unsigned.
    pub quantity: Decimal,
    /// Average entry price.
    pub entry_price: Option<Decimal>,
    /// The exchange's current mark price.
    pub mark_price: Option<Decimal>,
    /// Position value in the quote asset.
    pub notional: Option<Decimal>,
    /// Unrealized profit and loss, in the quote asset.
    pub unrealized_pnl: Option<Decimal>,
    /// Configured leverage.
    pub leverage: Option<Decimal>,
    /// Configured margin mode.
    pub margin_mode: Option<MarginMode>,
}

impl Position {
    /// Whether the position carries no size.
    pub fn is_flat(&self) -> bool {
        self.quantity.is_zero()
    }
}

/// How margin backs a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MarginMode {
    /// The whole account balance backs every position.
    Cross,
    /// Margin is ring-fenced per position.
    Isolated,
}

/// Account-wide margin state.
///
/// ```
/// use maxt::MarginSummary;
/// use rust_decimal::Decimal;
///
/// /// The largest notional this account may open at `leverage`.
/// ///
/// /// `None` when the exchange did not publish free margin. A zero default
/// /// would silently disable trading, and a guess would over-commit the
/// /// account.
/// fn budget(margin: &MarginSummary, leverage: Decimal) -> Option<Decimal> {
///     Some(margin.available_balance? * leverage)
/// }
///
/// let summary = MarginSummary {
///     asset: "USDT".to_string(),
///     equity: Some(Decimal::from(12_000)),
///     margin_balance: Some(Decimal::from(10_000)),
///     available_balance: Some(Decimal::from(4_000)),
/// };
///
/// assert_eq!(budget(&summary, Decimal::from(3)), Some(Decimal::from(12_000)));
/// assert_eq!(budget(&MarginSummary { available_balance: None, ..summary }, Decimal::from(3)), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarginSummary {
    /// The asset the figures are denominated in.
    pub asset: String,
    /// Balance plus unrealized profit and loss.
    pub equity: Option<Decimal>,
    /// Total balance posted as margin.
    pub margin_balance: Option<Decimal>,
    /// Balance free to open new positions with.
    pub available_balance: Option<Decimal>,
}

/// One funding rate observation for a perpetual market.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FundingRate {
    /// The market it applies to.
    pub market: Market,
    /// When it applied.
    pub timestamp: Timestamp,
    /// The rate, as a signed ratio. Positive means longs pay shorts.
    pub rate: Decimal,
    /// The mark price at the time, when the exchange publishes one.
    pub mark_price: Option<Decimal>,
}

/// One funding payment actually charged to or credited to an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FundingPayment {
    /// The market it was charged against.
    pub market: Market,
    /// When it was charged.
    pub timestamp: Timestamp,
    /// The amount, signed. Negative means the account paid.
    pub amount: Decimal,
    /// The rate it was charged at, when the exchange publishes one.
    pub rate: Option<Decimal>,
    /// The exchange's own identifier for the entry, when it publishes one.
    pub id: Option<String>,
}

/// An opaque position in a paginated history.
///
/// Produced by the exchange, and only meaningful to the exchange that produced
/// it. Pass it back unchanged to fetch the next page; do not parse it.
///
/// ```
/// use maxt::{Cursor, Exchange, HistoryRequest, Market};
///
/// // What a previous `Page::next` handed back.
/// let cursor = Cursor::new("1700000000000");
///
/// // Store the string, not what it looks like. One exchange's cursor is a
/// // timestamp, another's is an order id, and neither is a promise.
/// let saved: String = cursor.as_str().to_string();
///
/// let resumed = HistoryRequest::new(Market::perpetual(Exchange::Binance, "BTC", "USDT"))
///     .cursor(Cursor::new(saved));
///
/// assert_eq!(resumed.cursor.as_ref().map(Cursor::as_str), Some("1700000000000"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor(pub(crate) String);

impl Cursor {
    /// Wraps a resume point produced by an exchange.
    ///
    /// For adapters, and for restoring a cursor that was persisted between
    /// runs. Callers reading a history get theirs from
    /// [`Page::next`] and pass it back unchanged.
    pub fn new(cursor: impl Into<String>) -> Self {
        Self(cursor.into())
    }

    /// The cursor's opaque contents, for persisting between runs.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One page of a paginated history.
///
/// A history is walked by feeding [`Page::next`] back into the request until it
/// comes back `None`. The loop below stands in for
/// [`Client::funding_rates`](crate::Client::funding_rates) or
/// [`Client::funding_payments`](crate::Client::funding_payments), whose pages
/// behave the same way:
///
/// ```
/// use maxt::{Cursor, Page};
///
/// fn read_page(cursor: Option<&Cursor>) -> Page<u32> {
///     match cursor.map(Cursor::as_str) {
///         None => Page { items: vec![1, 2], next: Some(Cursor::new("after-2")) },
///         // The middle page is empty here on purpose: a window that filters
///         // out everything on one page is not the end of the history.
///         Some("after-2") => Page { items: vec![], next: Some(Cursor::new("after-3")) },
///         _ => Page { items: vec![3], next: None },
///     }
/// }
///
/// let mut cursor: Option<Cursor> = None;
/// let mut history = Vec::new();
///
/// loop {
///     let page = read_page(cursor.as_ref());
///     history.extend(page.items);
///
///     // Only an absent cursor ends the walk. Stopping on an empty or short
///     // page truncates the history, since a page is only as full as the
///     // exchange made it.
///     let Some(next) = page.next else { break };
///     cursor = Some(next);
/// }
///
/// assert_eq!(history, [1, 2, 3]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    /// The entries on this page.
    pub items: Vec<T>,
    /// The cursor for the next page, or `None` at the end of the history.
    pub next: Option<Cursor>,
}

impl<T> Page<T> {
    /// Whether another page follows.
    pub fn has_more(&self) -> bool {
        self.next.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Exchange, Market};

    #[test]
    fn balance_total_counts_locked_funds() {
        let balance = Balance {
            asset: "KRW".to_string(),
            available: Decimal::from(1_000),
            locked: Decimal::from(500),
        };

        assert_eq!(balance.total(), Decimal::from(1_500));
    }

    #[test]
    fn only_unfinished_orders_are_live() {
        for status in [
            OrderStatus::Accepted,
            OrderStatus::Open,
            OrderStatus::PartiallyFilled,
        ] {
            assert!(status.is_live(), "{status:?}");
        }
        for status in [
            OrderStatus::Filled,
            OrderStatus::Cancelled,
            OrderStatus::Rejected,
            OrderStatus::Unknown,
        ] {
            assert!(!status.is_live(), "{status:?}");
        }
    }

    #[test]
    fn base_and_quote_sizing_are_not_interchangeable() {
        let ten_thousand_krw = Size::Quote(Decimal::from(10_000));
        let ten_thousand_btc = Size::Base(Decimal::from(10_000));

        assert_ne!(ten_thousand_krw, ten_thousand_btc);
    }

    #[test]
    fn a_zero_size_position_is_flat() {
        let mut position = Position {
            market: Market::perpetual(Exchange::Binance, "BTC", "USDT"),
            side: None,
            quantity: Decimal::ZERO,
            entry_price: None,
            mark_price: None,
            notional: None,
            unrealized_pnl: None,
            leverage: None,
            margin_mode: None,
        };

        assert!(position.is_flat());
        position.quantity = Decimal::ONE;
        assert!(!position.is_flat());
    }

    #[test]
    fn the_last_page_reports_no_more() {
        let last = Page::<u8> {
            items: vec![],
            next: None,
        };
        let more = Page::<u8> {
            items: vec![],
            next: Some(Cursor("next".to_string())),
        };

        assert!(!last.has_more());
        assert!(more.has_more());
        assert_eq!(more.next.unwrap().as_str(), "next");
    }
}
