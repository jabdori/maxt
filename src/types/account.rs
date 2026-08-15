//! Account state: balances, orders, positions, margin, funding.

use rust_decimal::Decimal;

use crate::types::{Market, MarketStatus, Side, Timestamp};

/// How much of one asset an account reports as available and locked.
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
    /// The reported available amount plus the reported locked amount.
    ///
    /// This is an asset quantity, not a price-valued account total. On margin
    /// venues it may also differ from a wallet-balance field the exchange
    /// exposes outside this common shape.
    pub fn total(&self) -> Decimal {
        self.available + self.locked
    }
}

/// One order type and time-in-force combination accepted by a market.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderOption {
    /// Exchange value, preserved for diagnostics and future variants.
    pub provider_id: String,
    /// Normalized pricing behavior, or `None` for a newly added provider value.
    pub order_type: Option<OrderType>,
    /// Explicit lifetime, or `None` for the exchange's default.
    pub time_in_force: Option<TimeInForce>,
}

/// Account values returned with dynamic order rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderAccount {
    /// Available and locked amounts for the account asset.
    pub balance: Balance,
    /// Average buy price reported by the exchange.
    pub average_buy_price: Decimal,
    /// Whether the exchange says the average buy price was modified.
    pub average_buy_price_modified: bool,
    /// Currency used for the average buy price, when published.
    pub average_buy_price_unit: Option<String>,
}

/// Dynamic fees, limits, supported orders, and balances for one market.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderRules {
    /// Market these rules apply to.
    pub market: Market,
    /// Exchange-provided market name.
    pub market_name: String,
    /// Current market operation status.
    pub status: MarketStatus,
    /// Fee rate applied to ordinary buy orders.
    pub buy_fee_rate: Decimal,
    /// Fee rate applied to ordinary sell orders.
    pub sell_fee_rate: Decimal,
    /// Fee rate applied to maker buy orders.
    pub maker_buy_fee_rate: Decimal,
    /// Fee rate applied to maker sell orders.
    pub maker_sell_fee_rate: Decimal,
    /// Sides currently accepted by the market.
    pub sides: Vec<Side>,
    /// Supported buy order combinations.
    pub buy_options: Vec<OrderOption>,
    /// Supported sell order combinations.
    pub sell_options: Vec<OrderOption>,
    /// Price unit published for buy orders, when available.
    pub buy_price_unit: Option<Decimal>,
    /// Price unit published for sell orders, when available.
    pub sell_price_unit: Option<Decimal>,
    /// Minimum buy value in the quote asset.
    pub minimum_buy_total: Decimal,
    /// Minimum sell value in the quote asset.
    pub minimum_sell_total: Decimal,
    /// Maximum order value in the quote asset.
    pub maximum_total: Decimal,
    /// Quote-asset account used to buy.
    pub quote_account: OrderAccount,
    /// Base-asset account used to sell.
    pub base_account: OrderAccount,
}

/// How an order is priced and matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OrderType {
    /// Take whatever the book offers, immediately.
    Market,
    /// Rest on the book at a stated price.
    Limit,
    /// Use the best opposing price available when the exchange accepts the order.
    Best,
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

/// An order size expressed in either the base or quote asset.
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

/// An order as the exchange currently reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    /// The exchange's own order identifier. Pass it to cancel only for a live order.
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

/// One order an exchange accepted for cancellation in a batch request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelledOrder {
    /// Exchange-assigned order identifier.
    pub order_id: String,
    /// Caller-assigned identifier, when the exchange returns it.
    pub client_id: Option<String>,
    /// Order market, when the exchange returns it.
    pub market: Option<Market>,
    /// When the exchange accepted the cancellation, when published.
    pub cancelled_at: Option<Timestamp>,
}

/// One order an exchange did not cancel in a batch request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderCancelFailure {
    /// Exchange-assigned order identifier, when known.
    pub order_id: Option<String>,
    /// Caller-assigned identifier, when known.
    pub client_id: Option<String>,
    /// Order market, when the exchange returns it.
    pub market: Option<Market>,
    /// Provider error code, when the batch response includes one.
    pub code: Option<String>,
    /// Provider error message, when the batch response includes one.
    pub message: Option<String>,
}

/// Per-order outcome of one non-atomic batch cancellation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelOrdersResult {
    /// Orders accepted for cancellation.
    pub cancelled: Vec<CancelledOrder>,
    /// Orders the exchange did not cancel.
    pub failed: Vec<OrderCancelFailure>,
}

/// An open derivatives position.
///
/// Quantity is unsigned; direction is carried by [`Position::side`]. Fields an
/// exchange does not publish are `None` and must not be interpreted as zero.
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
    ///
    /// [`Client::positions`](crate::Client::positions) and
    /// [`Client::positions_on`](crate::Client::positions_on) remove flat rows.
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
/// Values an exchange does not publish are `None`.
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
/// Produced by an adapter from a native or synthesized resume point. Pass it
/// back unchanged to the same adapter; do not parse it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor(pub(crate) String);

impl Cursor {
    /// Wraps an adapter resume point.
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
/// comes back `None`. An empty or short page is not an end marker when `next`
/// is present.
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
