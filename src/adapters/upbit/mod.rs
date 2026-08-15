//! Upbit spot adapter for Korea, Singapore, Indonesia, and Thailand.

mod parse;
mod pockets;
mod private;
mod rest;
mod stream;
mod travel_rule;
mod wallet;

use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;
use std::sync::{Arc, Mutex, Weak};
use std::task::{Context, Poll};

use futures_core::Stream;
use futures_util::StreamExt;
use rust_decimal::Decimal;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{
    CancelOrdersRequest, CandleRequest, DepositAddressRequest, OrderHistoryRequest,
    OrderLookupRequest, OrderRequest, TransferHistoryRequest, TransferLookupRequest,
    WithdrawRequest,
};
use crate::stream::{AccountStream, MarketStream, TypedStream};
use crate::transport::{HttpTransport, WsCommand, WsConnect, WsSession, ws};
use crate::types::{
    AccountEvent, AssetNetwork, Balance, CancelOrdersResult, Candle, Deposit, DepositAddress,
    DepositAddressEntry, Exchange, Market, MarketEvent, MarketInfo, MarketKind, Network, Order,
    OrderBook, OrderRules, Page, Side, StreamConfig, Subscription, Ticker, TimeInForce, Timestamp,
    Trade, TransferDestination, Withdrawal, WithdrawalQuote,
};

pub use stream::{
    ListedSubscription as UpbitListedSubscription, SubscriptionList as UpbitSubscriptionList,
};
pub use travel_rule::{UpbitTravelRuleVasp, UpbitTravelRuleVerification};
pub use wallet::UpbitWithdrawalAddress;

#[derive(Clone, PartialEq, Eq, Hash)]
struct SubscriptionKey {
    markets: Vec<Market>,
    feeds: Vec<crate::types::Feed>,
}

impl From<&Subscription> for SubscriptionKey {
    fn from(subscription: &Subscription) -> Self {
        Self {
            markets: subscription.markets().to_vec(),
            feeds: subscription.feeds().to_vec(),
        }
    }
}

/// Active Upbit market connections indexed by their public subscription.
///
/// The exchange operation is connection-scoped. A selector is accepted only
/// when it identifies exactly one currently live connection.
#[derive(Default)]
struct ActiveSubscriptions {
    connections: Mutex<HashMap<SubscriptionKey, Vec<Weak<stream::SubscriptionControl>>>>,
}

impl std::fmt::Debug for ActiveSubscriptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveSubscriptions")
            .finish_non_exhaustive()
    }
}

impl ActiveSubscriptions {
    fn register(&self, subscription: SubscriptionKey, control: &Arc<stream::SubscriptionControl>) {
        self.connections
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(subscription)
            .or_default()
            .push(Arc::downgrade(control));
    }

    fn control(&self, subscription: &Subscription) -> Result<Arc<stream::SubscriptionControl>> {
        let key = SubscriptionKey::from(subscription);
        let live = {
            let mut connections = self
                .connections
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(controls) = connections.get_mut(&key) else {
                return Err(Error::invalid_request(
                    "subscription",
                    "no active Upbit connection matches this subscription",
                ));
            };
            controls.retain(|control| control.strong_count() > 0);
            let live = controls
                .iter()
                .filter_map(Weak::upgrade)
                .collect::<Vec<_>>();
            if controls.is_empty() {
                connections.remove(&key);
            }
            live
        };

        match live.as_slice() {
            [control] => Ok(Arc::clone(control)),
            [] => Err(Error::invalid_request(
                "subscription",
                "no active Upbit connection matches this subscription",
            )),
            _ => Err(Error::invalid_request(
                "subscription",
                "more than one active Upbit connection matches this subscription",
            )),
        }
    }
}

/// Selects an Upbit regional deployment.
///
/// Listings, order books, accounts, and credentials are isolated by region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum UpbitRegion {
    /// Upbit Korea. The default.
    #[default]
    Korea,
    /// Upbit Singapore.
    Singapore,
    /// Upbit Indonesia.
    Indonesia,
    /// Upbit Thailand.
    Thailand,
}

impl UpbitRegion {
    pub(crate) const fn rest_base_url(self) -> &'static str {
        match self {
            Self::Korea => "https://api.upbit.com",
            Self::Singapore => "https://sg-api.upbit.com",
            Self::Indonesia => "https://id-api.upbit.com",
            Self::Thailand => "https://th-api.upbit.com",
        }
    }

    pub(crate) const fn websocket_url(self) -> &'static str {
        match self {
            Self::Korea => "wss://api.upbit.com/websocket/v1",
            Self::Singapore => "wss://sg-api.upbit.com/websocket/v1",
            Self::Indonesia => "wss://id-api.upbit.com/websocket/v1",
            Self::Thailand => "wss://th-api.upbit.com/websocket/v1",
        }
    }
}

/// Warning and caution data for one listing.
///
/// Returned by [`UpbitAdapter::market_events`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct UpbitMarketEvent {
    /// Whether Upbit marks the listing with an investment warning (`유의 종목`).
    ///
    /// [`Client::markets`](crate::Client::markets) maps this to
    /// [`MarketStatus::Unknown`](crate::MarketStatus::Unknown). The value does
    /// not state whether new orders are currently accepted.
    pub warning: bool,
    /// Active investment-caution (`주의 종목`) criteria, sorted by Upbit's
    /// criterion name.
    ///
    /// These criteria do not change [`MarketStatus`](crate::MarketStatus).
    /// The list is empty outside [`UpbitRegion::Korea`], whose payload is the
    /// only regional payload that includes the criteria.
    pub cautions: Vec<String>,
}

/// Upbit's full-fidelity public market-data stream.
///
/// It preserves the connection and control-operation behavior of
/// [`MarketStream`] while yielding Upbit-specific event structures. Construct
/// it with [`UpbitAdapter::subscribe_detailed`].
pub struct UpbitMarketStream {
    inner: TypedStream<UpbitMarketStreamEvent>,
}

impl UpbitMarketStream {
    fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<UpbitMarketStreamEvent>> + Send + 'static,
        close: F,
    ) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            inner: TypedStream::new_with_close(inner, close),
        }
    }

    /// Stops this subscription and waits for the WebSocket to close.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl Stream for UpbitMarketStream {
    type Item = Result<UpbitMarketStreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl fmt::Debug for UpbitMarketStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpbitMarketStream").finish_non_exhaustive()
    }
}

/// Upbit's full-fidelity private account stream.
///
/// It preserves the connection behavior of [`AccountStream`] while yielding
/// Upbit-specific order and asset structures. Construct it with
/// [`UpbitAdapter::subscribe_detailed_account`].
pub struct UpbitAccountStream {
    inner: TypedStream<UpbitAccountStreamEvent>,
}

impl UpbitAccountStream {
    fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<UpbitAccountStreamEvent>> + Send + 'static,
        close: F,
    ) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            inner: TypedStream::new_with_close(inner, close),
        }
    }

    /// Stops this subscription and waits for the WebSocket to close.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl Stream for UpbitAccountStream {
    type Item = Result<UpbitAccountStreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl fmt::Debug for UpbitAccountStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpbitAccountStream").finish_non_exhaustive()
    }
}

/// One Upbit-specific public market event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UpbitMarketStreamEvent {
    /// A trade and Upbit's best-quote metadata.
    Trade(UpbitTradeStreamEvent),
    /// An order-book update and Upbit's aggregate sizes.
    OrderBook(UpbitOrderBookStreamEvent),
    /// A ticker update and Upbit's market-state metadata.
    Ticker(UpbitTickerStreamEvent),
    /// A candle update and Upbit's stream phase.
    Candle(UpbitCandleStreamEvent),
    /// The underlying WebSocket reconnected.
    Reconnected,
}

/// One Upbit-specific private account event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UpbitAccountStreamEvent {
    /// An account asset snapshot.
    Asset(UpbitAssetStreamEvent),
    /// An order lifecycle update.
    Order(UpbitOrderStreamEvent),
    /// The underlying WebSocket reconnected.
    Reconnected,
}

/// A full Upbit trade stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitTradeStreamEvent {
    /// Portable trade projection.
    pub common: Trade,
    /// Previous close supplied by Upbit.
    pub previous_closing_price: Option<Decimal>,
    /// Provider change direction such as `RISE` or `FALL`.
    pub change: Option<String>,
    /// Unsigned provider price change.
    pub change_price: Option<Decimal>,
    /// Best ask quote present when Upbit supplied it.
    pub best_ask_price: Option<Decimal>,
    /// Best ask size present when Upbit supplied it.
    pub best_ask_size: Option<Decimal>,
    /// Best bid quote present when Upbit supplied it.
    pub best_bid_price: Option<Decimal>,
    /// Best bid size present when Upbit supplied it.
    pub best_bid_size: Option<Decimal>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Upbit order-book stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitOrderBookStreamEvent {
    /// Portable order-book projection.
    pub common: OrderBook,
    /// Total resting ask size supplied by Upbit.
    pub total_ask_size: Option<Decimal>,
    /// Total resting bid size supplied by Upbit.
    pub total_bid_size: Option<Decimal>,
    /// Provider aggregation level, when present.
    pub level: Option<Decimal>,
    /// Provider stream phase such as `SNAPSHOT` or `REALTIME`.
    pub stream_type: Option<String>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Upbit ticker stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitTickerStreamEvent {
    /// Portable ticker projection.
    pub common: Ticker,
    /// Provider change direction such as `RISE` or `FALL`.
    pub change_direction: Option<String>,
    /// Provider market lifecycle state.
    pub market_state: Option<String>,
    /// Whether Upbit reports trading as suspended.
    pub trading_suspended: Option<bool>,
    /// Provider delisting date, when present.
    pub delisting_date: Option<String>,
    /// Provider market warning state.
    pub market_warning: Option<String>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Upbit candle stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitCandleStreamEvent {
    /// Portable candle projection.
    pub common: Candle,
    /// Provider stream phase such as `SNAPSHOT` or `REALTIME`.
    pub stream_type: Option<String>,
    /// Time at which Upbit published the frame.
    pub published_at: Option<Timestamp>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Upbit private asset stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitAssetStreamEvent {
    /// Portable balance projections.
    pub balances: Vec<Balance>,
    /// Upbit asset-snapshot identifier, when present.
    pub asset_uuid: Option<String>,
    /// Time at which Upbit assembled the asset snapshot.
    pub asset_timestamp: Option<Timestamp>,
    /// Time at which Upbit published the frame.
    pub published_at: Option<Timestamp>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Upbit private order stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitOrderStreamEvent {
    /// Portable order projection.
    pub common: Order,
    /// Provider order type.
    pub order_type: Option<String>,
    /// Provider execution identifier, when present.
    pub trade_uuid: Option<String>,
    /// Provider time-in-force value.
    pub time_in_force: Option<String>,
    /// Provider execution time.
    pub trade_timestamp: Option<Timestamp>,
    /// Provider fee for the current execution.
    pub trade_fee: Option<Decimal>,
    /// Whether Upbit reports this execution as maker-side.
    pub is_maker: Option<bool>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// One yearly candle returned by Upbit's quotation API.
///
/// This remains provider-specific because the common [`Interval`] type has no
/// yearly variant. `korea_open_time` is present only when Upbit includes its
/// Korea Standard Time wall-clock field in the response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitYearCandle {
    /// Market that produced the candle.
    pub market: Market,
    /// UTC opening time of the annual window.
    pub open_time: crate::types::Timestamp,
    /// Korea Standard Time opening time when the regional response includes it.
    pub korea_open_time: Option<crate::types::Timestamp>,
    /// Upbit's response timestamp.
    pub timestamp: crate::types::Timestamp,
    /// First trade price in the annual window.
    pub open: Decimal,
    /// Highest trade price in the annual window.
    pub high: Decimal,
    /// Lowest trade price in the annual window.
    pub low: Decimal,
    /// Last trade price in the annual window.
    pub close: Decimal,
    /// Cumulative base-asset volume in the annual window.
    pub volume: Decimal,
    /// Cumulative quote-asset value in the annual window.
    pub quote_volume: Decimal,
    /// First calendar day of Upbit's annual period, preserved as supplied.
    pub first_day_of_period: String,
}

/// Tick-size and supported order-book aggregation policy for one Upbit market.
///
/// Upbit omits `supported_levels` in some regional responses; that is exposed
/// as an empty list rather than inferred from another region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitOrderBookInstrument {
    /// Market governed by this policy.
    pub market: Market,
    /// Quote currency named by Upbit's response.
    pub quote_currency: String,
    /// Price increment currently applicable to the market's price band.
    ///
    /// Upbit can change this value when the order price moves into another
    /// band, so it is not a market-wide constant.
    pub tick_size: Decimal,
    /// Valid order-book aggregation levels currently published by Upbit.
    pub supported_levels: Vec<Decimal>,
}

/// Identifies one Upbit order while retaining its provider-specific detail.
///
/// [`UpbitOrderDetailRequest::market`] is not sent to `GET /v1/order` because
/// Upbit resolves orders globally by UUID or client identifier. It makes the
/// expected market explicit so the adapter can reject a response belonging to
/// another market.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitOrderDetailRequest {
    /// Market the returned order must belong to.
    pub market: Market,
    /// Upbit's exchange-assigned order UUID.
    ///
    /// Upbit resolves this field when both it and [`Self::identifier`] are
    /// present.
    pub uuid: Option<String>,
    /// Caller-assigned identifier supplied when the order was created.
    pub identifier: Option<String>,
}

impl UpbitOrderDetailRequest {
    /// Starts a lookup for one expected market.
    pub fn new(market: Market) -> Self {
        Self {
            market,
            uuid: None,
            identifier: None,
        }
    }

    /// Starts a lookup by Upbit's exchange-assigned order UUID.
    pub fn by_uuid(market: Market, uuid: impl Into<String>) -> Self {
        Self::new(market).uuid(uuid)
    }

    /// Starts a lookup by the caller-assigned order identifier.
    pub fn by_identifier(market: Market, identifier: impl Into<String>) -> Self {
        Self::new(market).identifier(identifier)
    }

    /// Adds Upbit's exchange-assigned order UUID.
    ///
    /// When an identifier is also present, Upbit resolves the UUID.
    #[must_use]
    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.uuid = Some(uuid.into());
        self
    }

    /// Adds the caller-assigned order identifier.
    #[must_use]
    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }
}

/// A final lifecycle state accepted by Upbit's closed-order endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitClosedOrderState {
    /// An order whose requested quantity was fully executed.
    Done,
    /// An order cancelled before its requested quantity was fully executed.
    Cancel,
}

impl UpbitClosedOrderState {
    const fn wire_name(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Cancel => "cancel",
        }
    }
}

/// Filters for Upbit's provider-specific closed-order list.
///
/// Set either [`Self::state`] or a non-empty [`Self::states`] list, never
/// both. Upbit defaults to both final states, the previous seven days, 100
/// results, and newest-first ordering when the corresponding fields are unset.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpbitClosedOrdersRequest {
    /// Optional Upbit spot-market filter.
    pub market: Option<Market>,
    /// One final lifecycle-state filter.
    pub state: Option<UpbitClosedOrderState>,
    /// Multiple final lifecycle-state filters.
    pub states: Vec<UpbitClosedOrderState>,
    /// Optional beginning of the order-creation window.
    pub start_time: Option<crate::types::Timestamp>,
    /// Optional end of the order-creation window.
    pub end_time: Option<crate::types::Timestamp>,
    /// Optional result count from 1 through 1,000.
    pub limit: Option<u32>,
    /// Optional creation-time ordering. Upbit defaults to newest first.
    pub order_by: Option<UpbitOrderDirection>,
}

impl UpbitClosedOrdersRequest {
    /// Starts an unfiltered request using Upbit's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by one market.
    #[must_use]
    pub fn market(mut self, market: Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Filters by one final lifecycle state.
    #[must_use]
    pub fn state(mut self, state: UpbitClosedOrderState) -> Self {
        self.state = Some(state);
        self
    }

    /// Filters by one or more final lifecycle states.
    #[must_use]
    pub fn states(mut self, states: impl Into<Vec<UpbitClosedOrderState>>) -> Self {
        self.states = states.into();
        self
    }

    /// Sets the beginning of the order-creation window.
    #[must_use]
    pub fn start_time(mut self, start_time: crate::types::Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the end of the order-creation window.
    #[must_use]
    pub fn end_time(mut self, end_time: crate::types::Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Limits the response to between 1 and 1,000 orders.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Chooses oldest-first or newest-first ordering.
    #[must_use]
    pub fn order_by(mut self, order_by: UpbitOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// One summary returned by Upbit's closed-order list endpoint.
///
/// This type intentionally does not reuse [`UpbitOrderDetail`]: closed-order
/// rows contain no `trades` array. Provider enum values remain strings so a
/// new Upbit value does not make an otherwise valid response undecodable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitClosedOrder {
    /// Upbit spot market.
    pub market: Market,
    /// Upbit's exchange-assigned order UUID.
    pub uuid: String,
    /// Upbit's raw order side, such as `ask` or `bid`.
    pub side: String,
    /// Upbit's raw order type, such as `limit`, `price`, `market`, or `best`.
    pub ord_type: String,
    /// Upbit's raw final lifecycle state, `done` or `cancel`.
    pub state: String,
    /// Order creation time.
    pub created_at: crate::types::Timestamp,
    /// Submitted base quantity, when the order type carries one.
    pub volume: Option<Decimal>,
    /// Order price, or total quote amount for a market buy.
    pub price: Option<Decimal>,
    /// Base quantity remaining after fills.
    pub remaining_volume: Decimal,
    /// Cumulative executed base quantity.
    pub executed_volume: Decimal,
    /// Cumulative executed quote amount when Upbit includes it.
    pub executed_funds: Option<Decimal>,
    /// Fee Upbit reserved when it accepted the order.
    pub reserved_fee: Decimal,
    /// Reserved fee not yet used.
    pub remaining_fee: Decimal,
    /// Cumulative paid fee.
    pub paid_fee: Decimal,
    /// Funds or quantity still locked in the order.
    pub locked: Decimal,
    /// Number of fills Upbit associates with this order.
    pub trades_count: u32,
    /// Quantity cancelled by self-match prevention.
    pub prevented_volume: Decimal,
    /// Assets unlocked by self-match prevention.
    pub prevented_locked: Decimal,
    /// Raw time-in-force value, when the order used one.
    pub time_in_force: Option<String>,
    /// Caller-assigned identifier, when the order was created with one.
    pub identifier: Option<String>,
    /// Raw self-match-prevention mode, when the order used one.
    pub smp_type: Option<String>,
}

/// One fill returned inside [`UpbitOrderDetail`].
///
/// Upbit's raw direction and trend strings remain strings so a newly added
/// provider value does not make an otherwise valid order unreadable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitOrderDetailTrade {
    /// Market reported for this fill.
    pub market: Market,
    /// Upbit's fill UUID.
    pub uuid: String,
    /// Execution price.
    pub price: Decimal,
    /// Executed base quantity.
    pub volume: Decimal,
    /// Executed quote amount.
    pub funds: Decimal,
    /// Upbit's raw price-trend value, such as `up` or `down`.
    pub trend: String,
    /// Execution time.
    pub created_at: crate::types::Timestamp,
    /// Upbit's raw order side for this fill, such as `ask` or `bid`.
    pub side: String,
}

/// The full provider-specific response from Upbit's single-order endpoint.
///
/// The common [`Order`] intentionally retains only cross-exchange fields.
/// This type preserves Upbit's fees, self-match-prevention values, and every
/// individual fill. Raw provider enum strings remain strings so future Upbit
/// values do not make an otherwise valid response undecodable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitOrderDetail {
    /// Upbit spot market.
    pub market: Market,
    /// Upbit's exchange-assigned order UUID.
    pub uuid: String,
    /// Upbit's raw order side, such as `ask` or `bid`.
    pub side: String,
    /// Upbit's raw order type, such as `limit`, `price`, `market`, or `best`.
    pub order_type: String,
    /// Order price, or total quote amount for a market buy.
    pub price: Option<Decimal>,
    /// Upbit's raw lifecycle state, such as `wait`, `watch`, `done`, or `cancel`.
    pub state: String,
    /// Order creation time.
    pub created_at: crate::types::Timestamp,
    /// Submitted base quantity, when the order type carries one.
    pub volume: Option<Decimal>,
    /// Base quantity remaining after fills.
    pub remaining_volume: Decimal,
    /// Cumulative executed base quantity.
    pub executed_volume: Decimal,
    /// Fee Upbit reserved when it accepted the order.
    pub reserved_fee: Decimal,
    /// Reserved fee not yet used.
    pub remaining_fee: Decimal,
    /// Cumulative paid fee.
    pub paid_fee: Decimal,
    /// Funds or quantity still locked in the order.
    pub locked: Decimal,
    /// Number of fills Upbit associates with this order.
    pub trades_count: u32,
    /// Quantity cancelled by self-match prevention.
    pub prevented_volume: Decimal,
    /// Assets unlocked by self-match prevention.
    pub prevented_locked: Decimal,
    /// Raw time-in-force value, when the order used one.
    pub time_in_force: Option<String>,
    /// Caller-assigned identifier, when the order was created with one.
    pub identifier: Option<String>,
    /// Raw self-match-prevention mode, when the order used one.
    pub smp_type: Option<String>,
    /// Individual fills in Upbit's response order.
    pub trades: Vec<UpbitOrderDetailTrade>,
}

/// An Upbit response from creating, testing, cancelling, or listing orders.
///
/// [`Order`] keeps the portable fields used by the common API. This type
/// retains provider-only order values when the endpoint includes them.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitOrderResponse {
    /// The portable order projection.
    pub common: Order,
    /// Upbit's raw order-type value.
    pub order_type: Option<String>,
    /// Submitted base quantity, when the order type carries one.
    pub volume: Option<Decimal>,
    /// Fee Upbit reserved at order creation.
    pub reserved_fee: Option<Decimal>,
    /// Reserved fee not yet used.
    pub remaining_fee: Option<Decimal>,
    /// Cumulative paid fee.
    pub paid_fee: Option<Decimal>,
    /// Funds or quantity still locked by Upbit.
    pub locked: Option<Decimal>,
    /// Number of fills Upbit associates with this response.
    pub trades_count: Option<u32>,
    /// Quantity affected by self-match prevention.
    pub prevented_volume: Option<Decimal>,
    /// Amount unlocked by self-match prevention.
    pub prevented_locked: Option<Decimal>,
    /// Provider time-in-force value.
    pub time_in_force: Option<String>,
    /// Caller-assigned identifier, when present.
    pub identifier: Option<String>,
    /// Provider self-match-prevention value.
    pub smp_type: Option<String>,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// An Upbit deposit lookup response with its portable projection and original body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitDepositResponse {
    /// Portable deposit projection.
    pub common: Deposit,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// An Upbit withdrawal lookup response with its portable projection and original body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitWithdrawalResponse {
    /// Portable withdrawal projection.
    pub common: Withdrawal,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// An Upbit withdrawal-cancellation response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitCancelWithdrawalResponse {
    /// Identifier supplied for the cancellation request.
    pub withdrawal_id: String,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// An Upbit batch-cancellation response with its portable projection and
/// complete provider body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitCancelOrdersResponse {
    /// Portable cancellation projection.
    pub common: CancelOrdersResult,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// Upbit가 반환한 한 자산·네트워크의 입금 가능 정보입니다.
///
/// `network`과 `provider_network`은 응답의 `net_type`을 그대로 보존합니다.
/// Upbit는 이 필드를 null로 반환할 수 있으므로, 요청에 사용한 네트워크로
/// 임의 보정하지 않습니다. 이 정보는 실시간 상태가 아니며 몇 분 지연될 수 있습니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitDepositInfo {
    /// 대문자로 정규화한 자산 코드입니다.
    pub asset: String,
    /// Upbit가 응답에 포함한 정규화 네트워크입니다.
    pub network: Option<Network>,
    /// Upbit가 응답에 포함한 원본 네트워크 식별자입니다.
    pub provider_network: Option<String>,
    /// 현재 입금 가능 여부입니다.
    pub is_deposit_possible: bool,
    /// 입금이 불가능할 때 Upbit가 제공한 사유입니다.
    pub deposit_impossible_reason: Option<String>,
    /// Upbit가 처리하는 최소 입금 수량입니다.
    pub minimum_deposit_amount: Decimal,
    /// 입금 반영에 필요한 최소 블록 확인 수입니다.
    pub minimum_deposit_confirmations: u64,
    /// 입금 수량에 적용하는 소수 자릿수입니다.
    pub decimal_precision: u64,
}

/// Required second-factor method for an Upbit Korea KRW transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitKrwTwoFactorType {
    /// Kakao authentication.
    Kakao,
    /// Naver certificate authentication.
    Naver,
    /// Hana certificate authentication.
    Hana,
}

impl UpbitKrwTwoFactorType {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Kakao => "kakao",
            Self::Naver => "naver",
            Self::Hana => "hana",
        }
    }
}

/// Request for an Upbit Korea KRW deposit or withdrawal.
///
/// Upbit verifies the registered transfer account and the selected
/// second-factor method itself. Those account credentials are never accepted
/// or stored by this API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitKrwTransferRequest {
    /// KRW amount to transfer. It must be greater than zero.
    pub amount: Decimal,
    /// Required provider-side second-factor method.
    pub two_factor_type: UpbitKrwTwoFactorType,
}

impl UpbitKrwTransferRequest {
    /// Starts a KRW transfer request with its required second-factor method.
    pub fn new(amount: Decimal, two_factor_type: UpbitKrwTwoFactorType) -> Self {
        Self {
            amount,
            two_factor_type,
        }
    }
}

/// One Upbit Korea KRW deposit returned after a deposit request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitKrwDeposit {
    /// Upbit's transfer type, normally `deposit`.
    pub transfer_type: String,
    /// Upbit-issued deposit UUID.
    pub uuid: String,
    /// Currency returned by Upbit, normally `KRW`.
    pub currency: String,
    /// Provider network. KRW deposits return `None`.
    pub net_type: Option<String>,
    /// Provider transaction identifier.
    pub txid: String,
    /// Provider lifecycle state under Upbit's spelling.
    pub state: String,
    /// Request creation time.
    pub created_at: crate::types::Timestamp,
    /// Completion time, when Upbit has completed the deposit.
    pub done_at: Option<crate::types::Timestamp>,
    /// Deposited KRW amount.
    pub amount: Decimal,
    /// Provider fee.
    pub fee: Decimal,
    /// Upbit transfer type, such as `default` or `internal`.
    pub transaction_type: String,
}

/// One Upbit Korea KRW withdrawal returned after a withdrawal request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitKrwWithdrawal {
    /// Upbit's transfer type, normally `withdraw`.
    pub transfer_type: String,
    /// Upbit-issued withdrawal UUID.
    pub uuid: String,
    /// Currency returned by Upbit, normally `KRW`.
    pub currency: String,
    /// Provider network. KRW withdrawals return `None`.
    pub net_type: Option<String>,
    /// Provider transaction identifier, when one has been assigned.
    pub txid: Option<String>,
    /// Provider lifecycle state under Upbit's spelling.
    pub state: String,
    /// Request creation time.
    pub created_at: crate::types::Timestamp,
    /// Completion time, when Upbit has completed the withdrawal.
    pub done_at: Option<crate::types::Timestamp>,
    /// Withdrawn KRW amount.
    pub amount: Decimal,
    /// Provider fee.
    pub fee: Decimal,
    /// Upbit transfer type, such as `default` or `internal`.
    pub transaction_type: String,
    /// Whether Upbit currently permits cancellation, when supplied.
    pub is_cancelable: Option<bool>,
}

/// One API key registered on an Upbit Korea account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitApiKey {
    /// Upbit access-key identifier.
    pub access_key: String,
    /// Provider-reported key expiry time.
    pub expires_at: crate::types::Timestamp,
}

/// One Upbit Korea account pocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocket {
    /// Upbit's pocket UUID.
    pub uuid: String,
    /// User-visible pocket name.
    pub name: String,
    /// Provider pocket type, such as `main` or `user_spot_trading`.
    ///
    /// This stays a provider string so a newly introduced pocket type does not
    /// make a successful response undecodable.
    pub kind: String,
}

/// One API key issued for a pocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocketApiKey {
    /// Upbit access-key identifier. Secret-key material is never returned.
    pub access_key: String,
    /// Provider permission names assigned to this key.
    pub permissions: Vec<String>,
    /// IP addresses that Upbit permits this key to use.
    pub allowed_ips: Vec<String>,
    /// Provider-reported creation time.
    pub created_at: crate::types::Timestamp,
    /// Provider-reported expiry time.
    pub expired_at: crate::types::Timestamp,
}

/// API keys grouped by their owning Upbit pocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocketApiKeyGroup {
    /// Owning pocket UUID.
    pub uuid: String,
    /// Keys issued for this pocket.
    pub keys: Vec<UpbitPocketApiKey>,
}

/// Request filters for Upbit Korea's pocket API-key list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpbitPocketApiKeysRequest {
    /// Optional pocket UUID filter. An empty list returns every visible pocket.
    pub uuids: Vec<String>,
    /// Whether expired keys are included. `false` is Upbit's documented default.
    pub include_expired: bool,
}

impl UpbitPocketApiKeysRequest {
    /// Starts an unfiltered API-key list request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Limits the response to these pocket UUIDs.
    #[must_use]
    pub fn uuids(mut self, uuids: impl Into<Vec<String>>) -> Self {
        self.uuids = uuids.into();
        self
    }

    /// Includes keys that have expired.
    #[must_use]
    pub fn include_expired(mut self) -> Self {
        self.include_expired = true;
        self
    }
}

/// One balance row belonging to an Upbit pocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocketBalance {
    /// Asset code returned by Upbit.
    pub currency: String,
    /// Amount currently available to trade or transfer.
    pub balance: Decimal,
    /// Amount reserved by orders or transfers.
    pub locked: Decimal,
    /// Provider-reported average acquisition price.
    pub avg_buy_price: Decimal,
    /// Whether Upbit has adjusted the average acquisition price.
    pub avg_buy_price_modified: bool,
    /// Currency in which `avg_buy_price` is expressed.
    pub unit_currency: String,
}

/// A state accepted by Upbit's pocket-transfer list endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitPocketTransferState {
    /// The transfer request was accepted.
    Submitted,
    /// Upbit is processing the transfer.
    Processing,
    /// The transfer completed.
    Done,
    /// The transfer failed.
    Failed,
}

impl UpbitPocketTransferState {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Processing => "processing",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// Direction accepted by the current sub-pocket transfer-history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitPocketTransferDirection {
    /// Transfers received by the current sub-pocket.
    Incoming,
    /// Transfers sent by the current sub-pocket.
    Outgoing,
    /// Both incoming and outgoing transfers.
    All,
}

impl UpbitPocketTransferDirection {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Incoming => "in",
            Self::Outgoing => "out",
            Self::All => "all",
        }
    }
}

/// Result ordering accepted by Upbit's pocket-transfer list endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitPocketTransferOrder {
    /// Oldest transfer first.
    Ascending,
    /// Newest transfer first.
    Descending,
}

impl UpbitPocketTransferOrder {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

/// Filters for one Upbit pocket-transfer history endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpbitPocketTransferQuery {
    /// Optional source-pocket UUID filter for a main-pocket history query.
    pub from: Option<String>,
    /// Optional destination-pocket UUID filter for a main-pocket history query.
    pub to: Option<String>,
    /// Optional direction filter for a sub-pocket history query.
    pub direction: Option<UpbitPocketTransferDirection>,
    /// Optional lifecycle-state filters.
    pub states: Vec<UpbitPocketTransferState>,
    /// Optional Upbit transfer UUID filters, limited to 20 values.
    pub uuids: Vec<String>,
    /// Optional caller identifier filters, limited to 20 values.
    pub identifiers: Vec<String>,
    /// Optional inclusive beginning of the query window.
    pub start_time: Option<crate::types::Timestamp>,
    /// Optional inclusive end of the query window.
    pub end_time: Option<crate::types::Timestamp>,
    /// Optional asset-code filter.
    pub currency: Option<String>,
    /// Optional page size from 1 through 100.
    pub limit: Option<u32>,
    /// Optional result order. Upbit defaults to newest first.
    pub order_by: Option<UpbitPocketTransferOrder>,
}

impl UpbitPocketTransferQuery {
    /// Starts an unfiltered transfer-history query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters a main-pocket history query by source pocket UUID.
    #[must_use]
    pub fn from(mut self, value: impl Into<String>) -> Self {
        self.from = Some(value.into());
        self
    }

    /// Filters a main-pocket history query by destination pocket UUID.
    #[must_use]
    pub fn to(mut self, value: impl Into<String>) -> Self {
        self.to = Some(value.into());
        self
    }

    /// Filters a sub-pocket history query by direction.
    #[must_use]
    pub fn direction(mut self, value: UpbitPocketTransferDirection) -> Self {
        self.direction = Some(value);
        self
    }

    /// Filters by one or more transfer states.
    #[must_use]
    pub fn states(mut self, values: impl Into<Vec<UpbitPocketTransferState>>) -> Self {
        self.states = values.into();
        self
    }

    /// Filters by Upbit transfer UUIDs.
    #[must_use]
    pub fn uuids(mut self, values: impl Into<Vec<String>>) -> Self {
        self.uuids = values.into();
        self
    }

    /// Filters by caller-assigned transfer identifiers.
    #[must_use]
    pub fn identifiers(mut self, values: impl Into<Vec<String>>) -> Self {
        self.identifiers = values.into();
        self
    }

    /// Sets the inclusive start of the query window.
    #[must_use]
    pub fn start_time(mut self, value: crate::types::Timestamp) -> Self {
        self.start_time = Some(value);
        self
    }

    /// Sets the inclusive end of the query window.
    #[must_use]
    pub fn end_time(mut self, value: crate::types::Timestamp) -> Self {
        self.end_time = Some(value);
        self
    }

    /// Filters by asset code.
    #[must_use]
    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    /// Sets the result count from 1 through 100.
    #[must_use]
    pub fn limit(mut self, value: u32) -> Self {
        self.limit = Some(value);
        self
    }

    /// Sets result ordering.
    #[must_use]
    pub fn order_by(mut self, value: UpbitPocketTransferOrder) -> Self {
        self.order_by = Some(value);
        self
    }
}

/// Main-pocket request to move assets between Upbit pockets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocketUniversalTransferRequest {
    /// Optional source-pocket UUID. Omit it to use the API key's pocket.
    pub from: Option<String>,
    /// Destination-pocket UUID. This is required by Upbit's current OpenAPI contract.
    pub to: String,
    /// Asset code to move.
    pub currency: String,
    /// Positive asset amount to move.
    pub amount: Decimal,
    /// Optional one-time caller identifier.
    pub identifier: Option<String>,
}

impl UpbitPocketUniversalTransferRequest {
    /// Creates a main-pocket transfer request with Upbit's required destination.
    pub fn new(to: impl Into<String>, currency: impl Into<String>, amount: Decimal) -> Self {
        Self {
            from: None,
            to: to.into(),
            currency: currency.into(),
            amount,
            identifier: None,
        }
    }

    /// Uses an explicit source pocket instead of the API key's pocket.
    #[must_use]
    pub fn from(mut self, value: impl Into<String>) -> Self {
        self.from = Some(value.into());
        self
    }

    /// Supplies Upbit's one-time transfer identifier.
    #[must_use]
    pub fn identifier(mut self, value: impl Into<String>) -> Self {
        self.identifier = Some(value.into());
        self
    }
}

/// Sub-pocket request to move assets to another Upbit pocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocketTransferRequest {
    /// Destination-pocket UUID. This is required by Upbit's current OpenAPI contract.
    pub to: String,
    /// Asset code to move.
    pub currency: String,
    /// Positive asset amount to move.
    pub amount: Decimal,
    /// Optional one-time caller identifier.
    pub identifier: Option<String>,
}

impl UpbitPocketTransferRequest {
    /// Creates a sub-pocket transfer request with Upbit's required destination.
    pub fn new(to: impl Into<String>, currency: impl Into<String>, amount: Decimal) -> Self {
        Self {
            to: to.into(),
            currency: currency.into(),
            amount,
            identifier: None,
        }
    }

    /// Supplies Upbit's one-time transfer identifier.
    #[must_use]
    pub fn identifier(mut self, value: impl Into<String>) -> Self {
        self.identifier = Some(value.into());
        self
    }
}

/// One transfer between Upbit pockets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitPocketTransfer {
    /// Upbit's transfer UUID.
    pub uuid: String,
    /// Caller-supplied transfer identifier, when one was used.
    pub identifier: Option<String>,
    /// Source-pocket UUID.
    pub from: String,
    /// Destination-pocket UUID.
    pub to: String,
    /// Provider lifecycle state, retained under Upbit's spelling.
    pub state: String,
    /// Asset code moved by this transfer.
    pub currency: String,
    /// Asset amount moved by this transfer.
    pub amount: Decimal,
    /// Provider-reported request creation time.
    pub created_at: crate::types::Timestamp,
}

/// Upbit's ordering when choosing open orders to cancel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitOrderDirection {
    /// Cancel the oldest matching orders first.
    Ascending,
    /// Cancel the newest matching orders first.
    Descending,
}

/// The explicit set of Upbit open orders considered for one batch cancellation.
///
/// [`Self::All`] is deliberately a named variant: it selects every eligible
/// market, while Upbit still applies the request count (default 20, maximum
/// 300) to matching open orders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpbitBatchCancelScope {
    /// Every eligible Upbit market; the request count still limits cancellations.
    All,
    /// Eligible orders in markets with one of these quote currencies.
    QuoteCurrencies {
        /// Quote currencies used to select eligible markets.
        values: Vec<String>,
    },
    /// Eligible orders in these explicit Upbit spot markets.
    Pairs {
        /// Upbit spot markets used to select eligible orders.
        values: Vec<Market>,
    },
}

/// Filters for Upbit's conditional batch-cancellation endpoint.
///
/// The endpoint can cancel at most 300 `wait` orders per request. It never
/// cancels `watch` orders. A successful response can still contain failures
/// because matching orders may fill or change state while Upbit processes the
/// request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitBatchCancelRequest {
    /// The explicit base set of orders to consider.
    pub scope: UpbitBatchCancelScope,
    /// Optional Upbit spot markets to leave untouched.
    pub excluded_pairs: Option<Vec<Market>>,
    /// Optional buy/sell filter. `None` leaves Upbit's `all` default.
    pub side: Option<Side>,
    /// Optional cancellation count. `None` leaves Upbit's default of 20.
    pub count: Option<u32>,
    /// Optional creation-time ordering. `None` leaves Upbit's `desc` default.
    pub order_by: Option<UpbitOrderDirection>,
}

impl UpbitBatchCancelRequest {
    /// Starts a batch cancellation with an explicit scope.
    pub fn new(scope: UpbitBatchCancelScope) -> Self {
        Self {
            scope,
            excluded_pairs: None,
            side: None,
            count: None,
            order_by: None,
        }
    }

    /// Leaves these Upbit spot markets untouched.
    #[must_use]
    pub fn excluded_pairs(mut self, pairs: impl Into<Vec<Market>>) -> Self {
        self.excluded_pairs = Some(pairs.into());
        self
    }

    /// Limits cancellation to one order side.
    #[must_use]
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    /// Limits how many matching orders Upbit cancels.
    #[must_use]
    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Chooses whether Upbit considers oldest or newest orders first.
    #[must_use]
    pub fn order_by(mut self, order_by: UpbitOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// Identifies the existing order for Upbit's cancel-and-new operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpbitOrderReference {
    /// Upbit-issued order UUID.
    Uuid(String),
    /// Caller-assigned order identifier.
    Identifier(String),
}

impl UpbitOrderReference {
    /// Uses an Upbit-issued order UUID.
    pub fn uuid(value: impl Into<String>) -> Self {
        Self::Uuid(value.into())
    }

    /// Uses a caller-assigned order identifier.
    pub fn identifier(value: impl Into<String>) -> Self {
        Self::Identifier(value.into())
    }
}

/// New-order volume for Upbit cancel-and-new.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpbitOrderVolume {
    /// Explicit base-asset volume.
    Amount(Decimal),
    /// Reuse the previous order's remaining volume.
    RemainOnly,
}

/// Self-match prevention mode for the replacement order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpbitSmpType {
    /// Cancel the maker order when a self-match would occur.
    CancelMaker,
    /// Cancel the taker order when a self-match would occur.
    CancelTaker,
    /// Reduce both orders by the self-matched amount.
    Reduce,
}

/// The replacement order shape accepted by Upbit's cancel-and-new endpoint.
///
/// The endpoint inherits the previous order's market and side. The buy/sell
/// variants make Upbit's direction-dependent `price`/`volume` fields explicit
/// without pretending that the endpoint accepts a new market or side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpbitCancelAndNewOrder {
    /// Limit order. `time_in_force` may be omitted for Upbit's default GTC.
    Limit {
        /// New base-asset volume, or `RemainOnly`.
        volume: UpbitOrderVolume,
        /// New quote price.
        price: Decimal,
        /// Optional IOC, FOK, or post-only policy.
        time_in_force: Option<TimeInForce>,
    },
    /// Market buy (`new_ord_type = "price"`).
    MarketBuy {
        /// Total quote amount to spend.
        price: Decimal,
    },
    /// Market sell (`new_ord_type = "market"`).
    MarketSell {
        /// New base-asset volume, or `RemainOnly`.
        volume: UpbitOrderVolume,
    },
    /// Best-price buy. Upbit requires IOC or FOK.
    BestBuy {
        /// Total quote amount to spend.
        price: Decimal,
        /// Required IOC or FOK policy.
        time_in_force: TimeInForce,
    },
    /// Best-price sell. Upbit requires IOC or FOK.
    BestSell {
        /// New base-asset volume, or `RemainOnly`.
        volume: UpbitOrderVolume,
        /// Required IOC or FOK policy.
        time_in_force: TimeInForce,
    },
}

/// Request for Upbit's single-request cancel-then-create order operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitCancelAndNewOrderRequest {
    /// Existing order to cancel.
    pub previous_order: UpbitOrderReference,
    /// Replacement order to create after cancellation.
    pub new_order: UpbitCancelAndNewOrder,
    /// Optional new caller-assigned identifier.
    pub new_identifier: Option<String>,
    /// Optional self-match prevention mode for the replacement order.
    pub new_smp_type: Option<UpbitSmpType>,
}

impl UpbitCancelAndNewOrderRequest {
    /// Starts a cancel-and-new request.
    pub fn new(previous_order: UpbitOrderReference, new_order: UpbitCancelAndNewOrder) -> Self {
        Self {
            previous_order,
            new_order,
            new_identifier: None,
            new_smp_type: None,
        }
    }

    /// Assigns the replacement order's client identifier.
    #[must_use]
    pub fn new_identifier(mut self, value: impl Into<String>) -> Self {
        self.new_identifier = Some(value.into());
        self
    }

    /// Selects the replacement order's self-match prevention mode.
    #[must_use]
    pub fn new_smp_type(mut self, value: UpbitSmpType) -> Self {
        self.new_smp_type = Some(value);
        self
    }
}

/// Result returned by Upbit's cancel-and-new endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitCancelAndNewOrderResult {
    /// The previous order after Upbit processed the request.
    ///
    /// It is usually cancelled, but it can be filled when it completed before
    /// cancellation. Inspect its status instead of assuming cancellation.
    pub previous_order: Order,
    /// UUID of the replacement order, when one was created.
    ///
    /// This is `None` when the old order filled before cancellation completed;
    /// a successful HTTP response alone does not imply replacement creation.
    pub new_order_uuid: Option<String>,
    /// Identifier of the replacement order, when one was requested and created.
    pub new_order_identifier: Option<String>,
}

/// The cancel-and-new result with the provider details of the previous order.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpbitCancelAndNewOrderDetailResult {
    /// The portable result retained for existing callers.
    pub common: UpbitCancelAndNewOrderResult,
    /// Provider fields returned for the previous order.
    pub previous_order: UpbitOrderResponse,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

impl UpbitCancelAndNewOrderResult {
    /// Whether Upbit reported that a replacement order was created.
    pub fn replacement_created(&self) -> bool {
        self.new_order_uuid.is_some()
    }
}

/// Adapter for Upbit spot markets.
///
/// Derivative features return [`Error::Unsupported`](crate::Error::Unsupported).
#[derive(Debug, Clone)]
pub struct UpbitAdapter {
    region: UpbitRegion,
    credentials: Option<UpbitCredentials>,
    /// Cached transport initialization result, reported on first use.
    http: std::result::Result<HttpTransport, Error>,
    active_subscriptions: Arc<ActiveSubscriptions>,
}

#[derive(Debug, Clone)]
pub(crate) struct UpbitCredentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
}

impl UpbitCredentials {
    fn validate(&self) -> Result<()> {
        if self.access_key.trim().is_empty() || self.secret_key.trim().is_empty() {
            return Err(Error::auth(
                "upbit needs both an access key and a secret key",
            ));
        }
        Ok(())
    }
}

impl UpbitAdapter {
    /// Creates an unauthenticated adapter for Upbit Korea.
    pub fn new() -> Self {
        Self::with_region(UpbitRegion::Korea)
    }

    /// Creates an unauthenticated adapter for `region`.
    pub fn with_region(region: UpbitRegion) -> Self {
        Self {
            region,
            credentials: None,
            http: HttpTransport::new(region.rest_base_url()),
            active_subscriptions: Arc::default(),
        }
    }

    /// Adds credentials for account, order, and private-stream calls.
    ///
    /// The key pair must be issued by the adapter's selected region.
    #[must_use]
    pub fn with_credentials(
        mut self,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(UpbitCredentials {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Returns the selected region.
    pub fn region(&self) -> UpbitRegion {
        self.region
    }

    /// Fetches order books for one or more markets in one REST request.
    ///
    /// `depth` is the number of levels per side and must be from 1 through 30.
    /// `None` uses Upbit's 30-level default.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidRequest`](crate::Error::InvalidRequest) for an
    /// empty market list, an invalid depth, a different exchange, or an invalid
    /// asset code. Non-spot markets return
    /// [`Error::Unsupported`](crate::Error::Unsupported). Transport, exchange,
    /// and decoding errors are propagated.
    pub async fn order_books(
        &self,
        markets: &[Market],
        depth: Option<u32>,
    ) -> Result<Vec<OrderBook>> {
        rest::order_books(self.http()?, markets, depth).await
    }

    /// Fetches Upbit Korea order books aggregated at one provider level.
    ///
    /// `level` must be zero or positive. Read [`Self::orderbook_instruments`]
    /// immediately before this call to select a current non-zero value: Upbit
    /// changes supported levels when a market moves between price bands. Global
    /// regional deployments do not accept this parameter.
    pub async fn order_books_at_level(
        &self,
        markets: &[Market],
        level: Decimal,
        depth: Option<u32>,
    ) -> Result<Vec<OrderBook>> {
        if self.region != UpbitRegion::Korea {
            return Err(Error::unsupported(
                Feature::OrderBook,
                Exchange::Upbit.id(),
                "order-book aggregation levels are available only in the Upbit Korea region",
            ));
        }
        rest::order_books_at_level(self.http()?, markets, level, depth).await
    }

    /// Fetches tickers for one or more markets in one REST request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidRequest`](crate::Error::InvalidRequest) for an
    /// empty market list, a different exchange, or an invalid asset code.
    /// Non-spot markets return [`Error::Unsupported`](crate::Error::Unsupported).
    /// Transport, exchange, and decoding errors are propagated.
    pub async fn tickers(&self, markets: &[Market]) -> Result<Vec<Ticker>> {
        rest::tickers(self.http()?, markets).await
    }

    /// Fetches every ticker in one or more quote-currency markets.
    ///
    /// The input is normalized to uppercase ASCII currency codes. It must name
    /// at least one non-empty code. This is distinct from [`Self::tickers`],
    /// which queries explicit trading pairs.
    pub async fn tickers_by_quote(&self, quote_currencies: &[String]) -> Result<Vec<Ticker>> {
        rest::tickers_by_quote(self.http()?, quote_currencies).await
    }

    /// Fetches Upbit's yearly candles for one market.
    ///
    /// `to` is an optional exclusive ISO-8601 boundary and `count`, when set,
    /// must be from 1 through 200. Results are oldest first. The endpoint does
    /// not use the common [`crate::types::Candle`] model because its annual
    /// interval is unique to Upbit's current public surface.
    pub async fn year_candles(
        &self,
        market: &Market,
        to: Option<crate::types::Timestamp>,
        count: Option<u32>,
    ) -> Result<Vec<UpbitYearCandle>> {
        rest::year_candles(self.http()?, market, to, count).await
    }

    /// Fetches tick-size and order-book aggregation policy for one or more markets.
    ///
    /// The returned tick size and `supported_levels` list are live provider
    /// metadata. A caller must not assume that either remains valid after the
    /// intended price moves into another band.
    pub async fn orderbook_instruments(
        &self,
        markets: &[Market],
    ) -> Result<Vec<UpbitOrderBookInstrument>> {
        rest::orderbook_instruments(self.http()?, markets).await
    }

    /// Fetches warning and caution data for every listed market.
    ///
    /// Caution criteria are empty outside [`UpbitRegion::Korea`].
    ///
    /// # Errors
    ///
    /// Propagates transport, exchange, and decoding errors.
    pub async fn market_events(&self) -> Result<Vec<(Market, UpbitMarketEvent)>> {
        rest::market_events(self.http()?).await
    }

    /// Opens an Upbit market stream that retains provider-specific fields.
    pub async fn subscribe_detailed(
        &self,
        subscription: &Subscription,
    ) -> Result<UpbitMarketStream> {
        self.subscribe_detailed_with(subscription, &crate::client::default_stream_config())
            .await
    }

    /// Opens a detailed Upbit market stream with explicit stream settings.
    ///
    /// The returned stream is also eligible for [`Self::list_subscriptions`],
    /// because that operation is scoped to this exact socket.
    pub async fn subscribe_detailed_with(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> Result<UpbitMarketStream> {
        let frame = stream::subscribe_frame(subscription, &ticket())?;
        let session = ws::connect(
            WsConnect {
                url: self.region.websocket_url().to_string(),
                headers: None,
                subscribe: WsConnect::fixed(vec![frame]),
                heartbeat: Some(stream::HEARTBEAT),
            },
            config,
        )
        .await?;
        let close = session.close_handle();
        let control = Arc::new(stream::SubscriptionControl::new(session.send_handle()));
        self.active_subscriptions
            .register(SubscriptionKey::from(subscription), &control);

        Ok(UpbitMarketStream::new_with_close(
            controlled_detailed_market_events(session, control, stream::DetailedDecoder::default()),
            move || async move { close.close().await },
        ))
    }

    /// Opens an Upbit account stream that retains provider-specific fields.
    pub async fn subscribe_detailed_account(&self) -> Result<UpbitAccountStream> {
        self.subscribe_detailed_account_with(&crate::client::default_stream_config())
            .await
    }

    /// Opens a detailed Upbit account stream with explicit stream settings.
    pub async fn subscribe_detailed_account_with(
        &self,
        config: &StreamConfig,
    ) -> Result<UpbitAccountStream> {
        let credentials = self.credentials()?.clone();
        let session = ws::connect(
            WsConnect {
                url: format!("{}/private", self.region.websocket_url()),
                headers: Some(Box::new(move || {
                    Ok(vec![(
                        private::AUTHORIZATION.to_string(),
                        private::authorization(&credentials, "")?,
                    )])
                })),
                subscribe: WsConnect::fixed(vec![private::subscribe_frame(&ticket())?]),
                heartbeat: Some(stream::HEARTBEAT),
            },
            config,
        )
        .await?;
        let close = session.close_handle();

        Ok(UpbitAccountStream::new_with_close(
            events(
                session,
                private::detailed_account_events,
                UpbitAccountStreamEvent::Reconnected,
            ),
            move || async move { close.close().await },
        ))
    }

    /// Reads subscriptions on the one active public connection matching `subscription`.
    ///
    /// Upbit scopes `LIST_SUBSCRIPTIONS` to an existing WebSocket connection.
    /// Call [`Adapter::subscribe`] first and pass that exact subscription. The
    /// adapter rejects no match or multiple matching live connections rather
    /// than returning a result for the wrong socket. Keep the returned
    /// [`MarketStream`](crate::MarketStream) running so it can dispatch the
    /// operation reply.
    pub async fn list_subscriptions(
        &self,
        subscription: &Subscription,
    ) -> Result<UpbitSubscriptionList> {
        self.active_subscriptions
            .control(subscription)?
            .list_subscriptions()
            .await
    }

    /// Validates an order without creating it on Upbit.
    ///
    /// Upbit returns a dry-run order object. Its identifier and status do not
    /// represent a live order, so it cannot be queried or cancelled.
    pub async fn test_order(&self, request: &OrderRequest) -> Result<Order> {
        private::test_order(self.credentials()?, self.http()?, request).await
    }

    /// Validates an order and returns Upbit's provider-specific response
    /// fields alongside the common order projection.
    pub async fn test_order_detail(&self, request: &OrderRequest) -> Result<UpbitOrderResponse> {
        private::test_order_detail(self.credentials()?, self.http()?, request).await
    }

    /// Creates an order and returns Upbit's provider-specific response fields
    /// alongside the common order projection.
    pub async fn place_order_detail(&self, request: &OrderRequest) -> Result<UpbitOrderResponse> {
        private::place_order_detail(self.credentials()?, self.http()?, request).await
    }

    /// Cancels an order by exchange identifier and returns Upbit's full
    /// response object.
    pub async fn cancel_order_detail(
        &self,
        market: &Market,
        order_id: &str,
    ) -> Result<UpbitOrderResponse> {
        private::cancel_order_detail(self.credentials()?, self.http()?, market, order_id).await
    }

    /// Cancels an order by caller-assigned identifier and returns Upbit's full
    /// response object.
    pub async fn cancel_order_by_client_id_detail(
        &self,
        market: &Market,
        client_id: &str,
    ) -> Result<UpbitOrderResponse> {
        private::cancel_order_by_client_id_detail(
            self.credentials()?,
            self.http()?,
            market,
            client_id,
        )
        .await
    }

    /// Reads orders by identifier without collapsing Upbit's provider-only
    /// fields into the common order model.
    pub async fn orders_by_ids_detail(
        &self,
        request: &OrderLookupRequest,
    ) -> Result<Vec<UpbitOrderResponse>> {
        private::orders_by_ids_detail(self.credentials()?, self.http()?, request).await
    }

    /// Cancels orders by identifier without discarding provider failures or
    /// metadata.
    pub async fn cancel_orders_detail(
        &self,
        request: &CancelOrdersRequest,
    ) -> Result<UpbitCancelOrdersResponse> {
        private::cancel_orders_detail(self.credentials()?, self.http()?, request).await
    }

    /// Retrieves one Upbit order with its fees, SMP outcome, and fills.
    ///
    /// The common [`Adapter::order`] result intentionally carries only fields
    /// shared across exchanges. Use this provider-specific read when the
    /// complete `GET /v1/order` response is required. Upbit uses `uuid` when
    /// both request identifiers are set.
    pub async fn order_detail(
        &self,
        request: &UpbitOrderDetailRequest,
    ) -> Result<UpbitOrderDetail> {
        private::order_detail(self.credentials()?, self.http()?, request).await
    }

    /// Retrieves closed Upbit orders without discarding provider-only fields.
    ///
    /// This read uses `GET /v1/orders/closed`, requires an API key with View
    /// Orders permission, and returns summary rows without individual trades.
    pub async fn closed_orders(
        &self,
        request: &UpbitClosedOrdersRequest,
    ) -> Result<Vec<UpbitClosedOrder>> {
        private::closed_orders(self.credentials()?, self.http()?, request).await
    }

    /// 한 자산·네트워크의 Upbit 입금 가능 정보를 조회합니다.
    ///
    /// Upbit의 응답은 실시간 서비스 상태를 보장하지 않으며 몇 분 지연될 수 있습니다.
    pub async fn deposit_info(&self, asset: &str, network: &Network) -> Result<UpbitDepositInfo> {
        wallet::deposit_info(self.credentials()?, self.http()?, asset, network).await
    }

    /// Returns every withdrawal address registered on this Upbit account.
    ///
    /// This provider-specific list preserves Upbit's network, recipient, and
    /// wallet metadata. It does not validate a prospective withdrawal or
    /// calculate a common withdrawal quote; use
    /// [`Adapter::prepare_withdrawal`] for that purpose.
    pub async fn withdrawal_addresses(&self) -> Result<Vec<UpbitWithdrawalAddress>> {
        wallet::withdrawal_addresses(self.credentials()?, self.http()?).await
    }

    /// Reads one Upbit deposit without discarding provider fields.
    pub async fn deposit_detail(
        &self,
        request: &TransferLookupRequest,
    ) -> Result<UpbitDepositResponse> {
        wallet::deposit_detail(self.credentials()?, self.http()?, request).await
    }

    /// Reads one Upbit withdrawal without discarding provider fields.
    pub async fn withdrawal_detail(
        &self,
        request: &TransferLookupRequest,
    ) -> Result<UpbitWithdrawalResponse> {
        wallet::withdrawal_detail(self.credentials()?, self.http()?, request).await
    }

    /// Cancels one Upbit withdrawal and preserves the provider response body.
    pub async fn cancel_withdrawal_detail(
        &self,
        withdrawal_id: &str,
    ) -> Result<UpbitCancelWithdrawalResponse> {
        wallet::cancel_withdrawal_detail(self.credentials()?, self.http()?, withdrawal_id).await
    }

    /// Deposits KRW from the registered account using Upbit Korea's required
    /// second-factor confirmation.
    ///
    /// This is a financial write and is available only for
    /// [`UpbitRegion::Korea`]. Upbit confirms the account and second factor on
    /// its side; no bank credentials are accepted here.
    pub async fn deposit_krw(&self, request: &UpbitKrwTransferRequest) -> Result<UpbitKrwDeposit> {
        self.ensure_korea_wallet_region()?;
        wallet::deposit_krw(self.credentials()?, self.http()?, request).await
    }

    /// Requests a KRW withdrawal from Upbit Korea with second-factor
    /// confirmation.
    ///
    /// This is a financial write. A withdrawal-enabled API key can still be
    /// rejected when Upbit's withdrawal safety lock is enabled.
    pub async fn withdraw_krw(
        &self,
        request: &UpbitKrwTransferRequest,
    ) -> Result<UpbitKrwWithdrawal> {
        self.ensure_korea_wallet_region()?;
        wallet::withdraw_krw(self.credentials()?, self.http()?, request).await
    }

    /// Lists access keys registered on this Upbit Korea account.
    ///
    /// This provider-specific read returns key identifiers and expiry times,
    /// never secret-key material.
    pub async fn api_keys(&self) -> Result<Vec<UpbitApiKey>> {
        self.ensure_korea_wallet_region()?;
        wallet::api_keys(self.credentials()?, self.http()?).await
    }

    /// Lists pockets visible to this Upbit Korea API key.
    ///
    /// Pocket management is a Korea-only provider API and is kept separate
    /// from the common account contract because it exposes provider-specific
    /// account topology.
    pub async fn list_pockets(&self) -> Result<Vec<UpbitPocket>> {
        self.ensure_korea_pockets_region()?;
        pockets::list(self.credentials()?, self.http()?).await
    }

    /// Lists API keys grouped by their Upbit Korea pocket.
    pub async fn list_pocket_api_keys(
        &self,
        request: &UpbitPocketApiKeysRequest,
    ) -> Result<Vec<UpbitPocketApiKeyGroup>> {
        self.ensure_korea_pockets_region()?;
        pockets::list_api_keys(self.credentials()?, self.http()?, request).await
    }

    /// Lists balances held by one Upbit Korea sub-pocket.
    pub async fn sub_pocket_balances(&self, pocket_uuid: &str) -> Result<Vec<UpbitPocketBalance>> {
        self.ensure_korea_pockets_region()?;
        pockets::balances(self.credentials()?, self.http()?, pocket_uuid).await
    }

    /// Moves assets between pockets through Upbit Korea's main-pocket endpoint.
    ///
    /// This is a financial write. The destination (`to`) is required by the
    /// current official OpenAPI contract.
    pub async fn universal_transfer(
        &self,
        request: &UpbitPocketUniversalTransferRequest,
    ) -> Result<UpbitPocketTransfer> {
        self.ensure_korea_pockets_region()?;
        pockets::universal_transfer(self.credentials()?, self.http()?, request).await
    }

    /// Lists transfers recorded by Upbit Korea's main-pocket endpoint.
    pub async fn universal_transfers(
        &self,
        request: &UpbitPocketTransferQuery,
    ) -> Result<Vec<UpbitPocketTransfer>> {
        self.ensure_korea_pockets_region()?;
        pockets::universal_transfers(self.credentials()?, self.http()?, request).await
    }

    /// Moves assets from the current Upbit Korea sub-pocket.
    ///
    /// This is a financial write. The destination (`to`) is required by the
    /// current official OpenAPI contract.
    pub async fn sub_pocket_transfer(
        &self,
        request: &UpbitPocketTransferRequest,
    ) -> Result<UpbitPocketTransfer> {
        self.ensure_korea_pockets_region()?;
        pockets::sub_pocket_transfer(self.credentials()?, self.http()?, request).await
    }

    /// Lists transfers recorded by the current Upbit Korea sub-pocket.
    pub async fn sub_pocket_transfers(
        &self,
        request: &UpbitPocketTransferQuery,
    ) -> Result<Vec<UpbitPocketTransfer>> {
        self.ensure_korea_pockets_region()?;
        pockets::sub_pocket_transfers(self.credentials()?, self.http()?, request).await
    }

    /// Lists VASPs supported by Upbit's Travel Rule service.
    ///
    /// This read requires the API key's `View Deposits` permission and is
    /// available only when this adapter targets [`UpbitRegion::Korea`] or
    /// [`UpbitRegion::Singapore`].
    pub async fn travel_rule_vasps(&self) -> Result<Vec<UpbitTravelRuleVasp>> {
        travel_rule::ensure_supported_region(self.region)?;
        travel_rule::vasps(self.region, self.credentials()?, self.http()?).await
    }

    /// Requests Travel Rule account-owner verification by deposit UUID.
    ///
    /// This is a financial write requiring the API key's `Deposit` permission.
    /// Upbit permits at most one request for the same deposit every 10 minutes;
    /// that repeat restriction is enforced by Upbit, not tracked client-side.
    /// The endpoint is available only in [`UpbitRegion::Korea`] or
    /// [`UpbitRegion::Singapore`].
    pub async fn verify_travel_rule_by_uuid(
        &self,
        deposit_uuid: &str,
        vasp_uuid: &str,
    ) -> Result<UpbitTravelRuleVerification> {
        travel_rule::ensure_supported_region(self.region)?;
        travel_rule::verify_by_uuid(
            self.region,
            self.credentials()?,
            self.http()?,
            deposit_uuid,
            vasp_uuid,
        )
        .await
    }

    /// Requests Travel Rule account-owner verification by deposit transaction ID.
    ///
    /// This is a financial write requiring the API key's `Deposit` permission.
    /// Upbit permits at most one request for the same deposit every 10 minutes;
    /// that repeat restriction is enforced by Upbit, not tracked client-side.
    /// The endpoint is available only in [`UpbitRegion::Korea`] or
    /// [`UpbitRegion::Singapore`].
    pub async fn verify_travel_rule_by_txid(
        &self,
        txid: &str,
        vasp_uuid: &str,
        currency: &str,
        net_type: &str,
    ) -> Result<UpbitTravelRuleVerification> {
        travel_rule::ensure_supported_region(self.region)?;
        travel_rule::verify_by_txid(
            self.region,
            self.credentials()?,
            self.http()?,
            txid,
            vasp_uuid,
            currency,
            net_type,
        )
        .await
    }

    /// Cancels matching Upbit `wait` orders in one conditional request.
    ///
    /// This is a financial write. The returned value separates orders Upbit
    /// cancelled from orders that changed state before cancellation completed.
    pub async fn batch_cancel_open_orders(
        &self,
        request: &UpbitBatchCancelRequest,
    ) -> Result<CancelOrdersResult> {
        private::batch_cancel_open_orders(self.credentials()?, self.http()?, request).await
    }

    /// Cancels one existing order and creates its replacement in one request.
    ///
    /// Upbit keeps the previous order's market and side. A `201` response can
    /// still report no replacement UUID when the previous order filled before
    /// cancellation completed; inspect [`UpbitCancelAndNewOrderResult::replacement_created`]
    /// instead of treating HTTP success as an atomic replacement guarantee.
    pub async fn cancel_and_new_order(
        &self,
        request: &UpbitCancelAndNewOrderRequest,
    ) -> Result<UpbitCancelAndNewOrderResult> {
        private::cancel_and_new_order(self.credentials()?, self.http()?, request).await
    }

    /// Cancels and replaces an order while retaining the provider fields for
    /// the previous order in Upbit's response.
    pub async fn cancel_and_new_order_detail(
        &self,
        request: &UpbitCancelAndNewOrderRequest,
    ) -> Result<UpbitCancelAndNewOrderDetailResult> {
        private::cancel_and_new_order_detail(self.credentials()?, self.http()?, request).await
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials
            .as_ref()
            .is_some_and(|credentials| credentials.validate().is_ok())
    }

    fn http(&self) -> Result<&HttpTransport> {
        self.http.as_ref().map_err(Clone::clone)
    }

    fn credentials(&self) -> Result<&UpbitCredentials> {
        let credentials = self.credentials.as_ref().ok_or_else(|| {
            Error::auth(
                "this Upbit adapter has no credentials; add them with \
                 `UpbitAdapter::with_credentials`",
            )
        })?;
        credentials.validate()?;
        Ok(credentials)
    }

    fn ensure_korea_wallet_region(&self) -> Result<()> {
        if self.region == UpbitRegion::Korea {
            Ok(())
        } else {
            Err(Error::invalid_request(
                "region",
                "Upbit KRW transfers and API-key listing are available only in the Korea region",
            ))
        }
    }

    fn ensure_korea_pockets_region(&self) -> Result<()> {
        if self.region == UpbitRegion::Korea {
            Ok(())
        } else {
            Err(Error::invalid_request(
                "region",
                "Upbit pocket APIs are available only in the Korea region",
            ))
        }
    }

    fn validate_withdrawal_destination(&self, request: &WithdrawRequest) -> Result<()> {
        if self.region == UpbitRegion::Indonesia
            && !matches!(
                &request.destination,
                TransferDestination::Exchange(destination)
                    if destination.exchange == Exchange::Upbit
            )
        {
            return Err(Error::unsupported(
                Feature::Withdrawals,
                "upbit",
                "Upbit Indonesia external withdrawals require beneficiary fields that are not yet represented by the common withdrawal request",
            ));
        }
        Ok(())
    }
}

impl Default for UpbitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for UpbitAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Upbit
    }

    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() {
            return false;
        }
        if feature == Feature::TravelRule {
            return matches!(self.region, UpbitRegion::Korea | UpbitRegion::Singapore)
                && self.is_authenticated();
        }
        if feature.needs_credentials() {
            return self.is_authenticated();
        }
        true
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move { rest::markets(self.http()?, kind).await })
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let market = market.clone();
        Box::pin(async move { rest::trades(self.http()?, &market, limit).await })
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let market = market.clone();
        Box::pin(async move {
            let books = self
                .order_books(std::slice::from_ref(&market), depth)
                .await?;
            rest::only(books, &market)
        })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move {
            let tickers = self.tickers(std::slice::from_ref(&market)).await?;
            rest::only(tickers, &market)
        })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let request = request.clone();
        Box::pin(async move { rest::candles(self.http()?, &request).await })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        let frame = stream::subscribe_frame(subscription, &ticket());
        let url = self.region.websocket_url().to_string();
        let config = config.clone();
        let active_subscriptions = Arc::clone(&self.active_subscriptions);
        let subscription = SubscriptionKey::from(subscription);

        Box::pin(async move {
            let session = ws::connect(
                WsConnect {
                    url,
                    headers: None,
                    subscribe: WsConnect::fixed(vec![frame?]),
                    heartbeat: Some(stream::HEARTBEAT),
                },
                &config,
            )
            .await?;
            let close = session.close_handle();
            let control = Arc::new(stream::SubscriptionControl::new(session.send_handle()));
            active_subscriptions.register(subscription, &control);

            // Candle completion state belongs to one WebSocket connection.
            let decoder = stream::Decoder::default();

            Ok(MarketStream::new_with_close(
                controlled_market_events(session, control, decoder),
                move || async move { close.close().await },
            ))
        })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self.credentials()?, self.http()?).await })
    }

    fn order_rules(&self, market: &Market) -> BoxFuture<'_, Result<OrderRules>> {
        let market = market.clone();
        Box::pin(
            async move { private::order_rules(self.credentials()?, self.http()?, &market).await },
        )
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        let asset = asset.to_string();
        Box::pin(
            async move { wallet::asset_networks(self.credentials()?, self.http()?, &asset).await },
        )
    }

    fn deposit_addresses(&self) -> BoxFuture<'_, Result<Vec<DepositAddressEntry>>> {
        Box::pin(async move { wallet::deposit_addresses(self.credentials()?, self.http()?).await })
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::deposit_address(self.credentials()?, self.http()?, &request).await
        })
    }

    fn create_deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::create_deposit_address(self.credentials()?, self.http()?, &request).await
        })
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        let request = request.clone();
        Box::pin(async move {
            self.validate_withdrawal_destination(&request)?;
            wallet::prepare_withdrawal(self.credentials()?, self.http()?, &request).await
        })
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(async move {
            self.validate_withdrawal_destination(&request)?;
            wallet::withdraw(self.credentials()?, self.http()?, &request).await
        })
    }

    fn deposit(&self, request: &TransferLookupRequest) -> BoxFuture<'_, Result<Deposit>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposit(self.credentials()?, self.http()?, &request).await })
    }

    fn withdrawal(&self, request: &TransferLookupRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(
            async move { wallet::withdrawal(self.credentials()?, self.http()?, &request).await },
        )
    }

    fn cancel_withdrawal(&self, withdrawal_id: &str) -> BoxFuture<'_, Result<()>> {
        let withdrawal_id = withdrawal_id.to_owned();
        Box::pin(async move {
            wallet::cancel_withdrawal(self.credentials()?, self.http()?, &withdrawal_id).await
        })
    }

    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposits(self.credentials()?, self.http()?, &request).await })
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        let request = request.clone();
        Box::pin(
            async move { wallet::withdrawals(self.credentials()?, self.http()?, &request).await },
        )
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move {
            private::open_orders(self.credentials()?, self.http()?, market.as_ref()).await
        })
    }

    fn order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::order(self.credentials()?, self.http()?, &market, &order_id).await
        })
    }

    fn order_by_client_id(&self, market: &Market, client_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let client_id = client_id.to_string();
        Box::pin(async move {
            private::order_by_client_id(self.credentials()?, self.http()?, &market, &client_id)
                .await
        })
    }

    fn orders_by_ids(&self, request: &OrderLookupRequest) -> BoxFuture<'_, Result<Vec<Order>>> {
        let request = request.clone();
        Box::pin(async move {
            private::orders_by_ids(self.credentials()?, self.http()?, &request).await
        })
    }

    fn order_history(&self, request: &OrderHistoryRequest) -> BoxFuture<'_, Result<Page<Order>>> {
        let request = request.clone();
        Box::pin(async move {
            private::order_history(self.credentials()?, self.http()?, &request).await
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(
            async move { private::place_order(self.credentials()?, self.http()?, &request).await },
        )
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::cancel_order(self.credentials()?, self.http()?, &market, &order_id).await
        })
    }

    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let client_id = client_id.to_string();
        Box::pin(async move {
            private::cancel_order_by_client_id(
                self.credentials()?,
                self.http()?,
                &market,
                &client_id,
            )
            .await
        })
    }

    fn cancel_orders(
        &self,
        request: &CancelOrdersRequest,
    ) -> BoxFuture<'_, Result<CancelOrdersResult>> {
        let request = request.clone();
        Box::pin(async move {
            private::cancel_orders(self.credentials()?, self.http()?, &request).await
        })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let url = format!("{}/private", self.region.websocket_url());
        let config = config.clone();

        Box::pin(async move {
            // The reconnect callback owns the credentials it signs with.
            let credentials = self.credentials()?.clone();
            let session = ws::connect(
                WsConnect {
                    url,
                    // Mint a fresh authorization value for every handshake.
                    headers: Some(Box::new(move || {
                        Ok(vec![(
                            private::AUTHORIZATION.to_string(),
                            private::authorization(&credentials, "")?,
                        )])
                    })),
                    subscribe: WsConnect::fixed(vec![private::subscribe_frame(&ticket())?]),
                    heartbeat: Some(stream::HEARTBEAT),
                },
                &config,
            )
            .await?;
            let close = session.close_handle();

            Ok(AccountStream::new_with_close(
                events(session, private::account_events, AccountEvent::Reconnected),
                move || async move { close.close().await },
            ))
        })
    }
}

/// Generates a unique subscription ticket.
fn ticket() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Decodes one public Upbit connection while reserving operation replies for
/// the caller that issued them on that same socket.
fn controlled_market_events(
    session: WsSession,
    control: Arc<stream::SubscriptionControl>,
    mut decoder: stream::Decoder,
) -> impl Stream<Item = Result<MarketEvent>> + Send {
    session.flat_map(move |item| {
        let items = match item {
            Ok(WsCommand::Text(text)) => {
                if control.handle_frame(&text) {
                    Vec::new()
                } else {
                    split(decoder.decode(&text))
                }
            }
            Ok(WsCommand::Binary(bytes)) => match String::from_utf8(bytes) {
                Ok(text) if control.handle_frame(&text) => Vec::new(),
                Ok(text) => split(decoder.decode(&text)),
                Err(err) => vec![Err(Error::decode(format!(
                    "upbit sent a frame that is not UTF-8: {err}"
                )))],
            },
            Ok(WsCommand::Reconnected) => {
                control.fail_pending();
                vec![Ok(MarketEvent::Reconnected)]
            }
            Err(err) => {
                control.fail_pending();
                vec![Err(err)]
            }
        };

        futures_util::stream::iter(items)
    })
}

/// Decodes a detailed public Upbit connection while reserving operation replies
/// for the caller that issued them on that same socket.
fn controlled_detailed_market_events(
    session: WsSession,
    control: Arc<stream::SubscriptionControl>,
    mut decoder: stream::DetailedDecoder,
) -> impl Stream<Item = Result<UpbitMarketStreamEvent>> + Send {
    session.flat_map(move |item| {
        let items = match item {
            Ok(WsCommand::Text(text)) => {
                if control.handle_frame(&text) {
                    Vec::new()
                } else {
                    split(decoder.decode(&text))
                }
            }
            Ok(WsCommand::Binary(bytes)) => match String::from_utf8(bytes) {
                Ok(text) if control.handle_frame(&text) => Vec::new(),
                Ok(text) => split(decoder.decode(&text)),
                Err(err) => vec![Err(Error::decode(format!(
                    "upbit sent a frame that is not UTF-8: {err}"
                )))],
            },
            Ok(WsCommand::Reconnected) => {
                control.fail_pending();
                vec![Ok(UpbitMarketStreamEvent::Reconnected)]
            }
            Err(err) => {
                control.fail_pending();
                vec![Err(err)]
            }
        };

        futures_util::stream::iter(items)
    })
}

/// Decodes frames in arrival order and flattens each into zero or more events.
fn events<T: Clone + Send + 'static>(
    session: WsSession,
    mut decode: impl FnMut(&str) -> Result<Vec<T>> + Send + 'static,
    reconnected: T,
) -> impl Stream<Item = Result<T>> + Send {
    session.flat_map(move |item| {
        let items = match item {
            Ok(WsCommand::Text(text)) => split(decode(&text)),
            Ok(WsCommand::Binary(bytes)) => match String::from_utf8(bytes) {
                Ok(text) => split(decode(&text)),
                Err(err) => vec![Err(Error::decode(format!(
                    "upbit sent a frame that is not UTF-8: {err}"
                )))],
            },
            Ok(WsCommand::Reconnected) => vec![Ok(reconnected.clone())],
            Err(err) => vec![Err(err)],
        };

        futures_util::stream::iter(items)
    })
}

/// Converts one decoded frame into stream items.
fn split<T>(decoded: Result<Vec<T>>) -> Vec<Result<T>> {
    match decoded {
        Ok(items) => items.into_iter().map(Ok).collect(),
        Err(err) => vec![Err(err)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spot_exchange_never_claims_derivatives_features() {
        let adapter = UpbitAdapter::new().with_credentials("access", "secret");

        for feature in [
            Feature::Positions,
            Feature::Margin,
            Feature::FundingRates,
            Feature::FundingPayments,
            Feature::MarginConfig,
            Feature::ReduceOnlyOrders,
        ] {
            assert!(!adapter.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn credentials_are_what_unlock_the_private_half() {
        let public = UpbitAdapter::new();
        let private = UpbitAdapter::new().with_credentials("access", "secret");

        for feature in [
            Feature::Balances,
            Feature::AssetNetworks,
            Feature::DepositAddresses,
            Feature::DepositHistory,
            Feature::DepositLookup,
            Feature::WithdrawalQuotes,
            Feature::Withdrawals,
            Feature::WithdrawalHistory,
            Feature::WithdrawalLookup,
            Feature::WithdrawalCancellation,
            Feature::Trading,
            Feature::AccountStream,
        ] {
            assert!(!public.supports(feature), "{feature:?}");
            assert!(private.supports(feature), "{feature:?}");
        }
    }

    #[tokio::test]
    async fn credentials_reject_blank_keys_and_accept_a_nonblank_pair() {
        for (access_key, secret_key) in [
            ("", "secret"),
            (" \t", "secret"),
            ("access", ""),
            ("access", " \n"),
        ] {
            let adapter = UpbitAdapter::new().with_credentials(access_key, secret_key);
            assert!(matches!(adapter.credentials(), Err(Error::Auth { .. })));
            assert!(matches!(adapter.balances().await, Err(Error::Auth { .. })));
            assert!(!adapter.is_authenticated());
        }

        let adapter = UpbitAdapter::new().with_credentials("access", "secret");
        assert!(adapter.credentials().is_ok());
        assert!(adapter.is_authenticated());
    }

    #[test]
    fn public_market_data_works_without_credentials() {
        let public = UpbitAdapter::new();

        for feature in [
            Feature::Markets,
            Feature::Trades,
            Feature::OrderBook,
            Feature::Ticker,
            Feature::Candles,
            Feature::CandleStream,
        ] {
            assert!(public.supports(feature), "{feature:?}");
        }
    }

    #[tokio::test]
    async fn list_subscriptions_never_opens_a_temporary_connection() {
        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Upbit, "BTC", "KRW"))
            .feed(crate::types::Feed::Ticker);

        let error = UpbitAdapter::new()
            .list_subscriptions(&subscription)
            .await
            .expect_err("a connection-scoped operation needs an active connection");
        assert!(matches!(
            error,
            Error::InvalidRequest { field, .. } if field == "subscription"
        ));
    }

    #[test]
    fn travel_rule_requires_a_supported_region_and_credentials() {
        assert!(!UpbitAdapter::new().supports(Feature::TravelRule));
        assert!(!UpbitAdapter::with_region(UpbitRegion::Singapore).supports(Feature::TravelRule));
        assert!(
            UpbitAdapter::new()
                .with_credentials("access", "secret")
                .supports(Feature::TravelRule)
        );
        assert!(
            UpbitAdapter::with_region(UpbitRegion::Singapore)
                .with_credentials("access", "secret")
                .supports(Feature::TravelRule)
        );
        assert!(
            !UpbitAdapter::with_region(UpbitRegion::Indonesia)
                .with_credentials("access", "secret")
                .supports(Feature::TravelRule)
        );
    }

    #[tokio::test]
    async fn korea_only_wallet_methods_reject_other_regions_before_credentials() {
        let adapter = UpbitAdapter::with_region(UpbitRegion::Singapore);
        let request = UpbitKrwTransferRequest::new(Decimal::ONE, UpbitKrwTwoFactorType::Kakao);

        for result in [
            adapter.deposit_krw(&request).await.map(|_| ()),
            adapter.withdraw_krw(&request).await.map(|_| ()),
            adapter.api_keys().await.map(|_| ()),
        ] {
            assert!(
                matches!(result, Err(Error::InvalidRequest { field, .. }) if field == "region")
            );
        }
    }

    #[tokio::test]
    async fn korea_only_pocket_methods_reject_other_regions_before_credentials() {
        let adapter = UpbitAdapter::with_region(UpbitRegion::Singapore);
        let api_keys = UpbitPocketApiKeysRequest::new();
        let universal = UpbitPocketUniversalTransferRequest::new("pocket-2", "XRP", Decimal::ONE);
        let sub_pocket = UpbitPocketTransferRequest::new("pocket-2", "XRP", Decimal::ONE);
        let history = UpbitPocketTransferQuery::new();

        for result in [
            adapter.list_pockets().await.map(|_| ()),
            adapter.list_pocket_api_keys(&api_keys).await.map(|_| ()),
            adapter.sub_pocket_balances("pocket-1").await.map(|_| ()),
            adapter.universal_transfer(&universal).await.map(|_| ()),
            adapter.universal_transfers(&history).await.map(|_| ()),
            adapter.sub_pocket_transfer(&sub_pocket).await.map(|_| ()),
            adapter.sub_pocket_transfers(&history).await.map(|_| ()),
        ] {
            assert!(
                matches!(result, Err(Error::InvalidRequest { field, .. }) if field == "region")
            );
        }
    }

    #[tokio::test]
    async fn travel_rule_region_precedes_credential_validation() {
        let error = UpbitAdapter::with_region(UpbitRegion::Indonesia)
            .travel_rule_vasps()
            .await
            .expect_err("unsupported region must fail before credentials");
        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::TravelRule,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn aggregated_order_books_fail_before_network_outside_korea() {
        let singapore = UpbitAdapter::with_region(UpbitRegion::Singapore);
        let market = Market::spot(Exchange::Upbit, "BTC", "SGD");

        assert!(matches!(
            singapore
                .order_books_at_level(&[market], Decimal::ONE, Some(1))
                .await,
            Err(Error::Unsupported {
                feature: Feature::OrderBook,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn an_account_call_without_credentials_fails_before_the_network() {
        let public = UpbitAdapter::new();
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let order = crate::request::OrderRequest::market(
            market.clone(),
            crate::types::Side::Sell,
            crate::types::Size::Base(rust_decimal::Decimal::ONE),
        );

        assert!(matches!(public.balances().await, Err(Error::Auth { .. })));
        assert!(matches!(
            public.open_orders(None).await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.place_order(&order).await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.test_order(&order).await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public
                .order_detail(&UpbitOrderDetailRequest::by_uuid(market.clone(), "order-1"))
                .await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public
                .closed_orders(&UpbitClosedOrdersRequest::new().market(market.clone()))
                .await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public
                .cancel_and_new_order(&UpbitCancelAndNewOrderRequest::new(
                    UpbitOrderReference::uuid("order-1"),
                    UpbitCancelAndNewOrder::MarketSell {
                        volume: UpbitOrderVolume::RemainOnly,
                    },
                ))
                .await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public
                .deposit_info("BTC", &crate::types::Network::Bitcoin)
                .await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.withdrawal_addresses().await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public
                .batch_cancel_open_orders(
                    &UpbitBatchCancelRequest::new(UpbitBatchCancelScope::All,)
                )
                .await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.cancel_order(&market, "an-order").await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.subscribe_account(&StreamConfig::default()).await,
            Err(Error::Auth { .. })
        ));
    }

    #[tokio::test]
    async fn the_derivatives_half_stays_at_the_trait_default() {
        let adapter = UpbitAdapter::new().with_credentials("access", "secret");
        let request =
            crate::request::HistoryRequest::new(Market::perpetual(Exchange::Upbit, "BTC", "KRW"));

        assert!(matches!(
            adapter.positions(None).await,
            Err(Error::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.margin_summary().await,
            Err(Error::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.funding_rates(&request).await,
            Err(Error::Unsupported { .. })
        ));
    }

    #[tokio::test]
    async fn upbit_lists_no_derivatives_and_says_so_with_an_empty_answer() {
        let markets = UpbitAdapter::new()
            .markets(MarketKind::Perpetual)
            .await
            .expect("a listable kind");

        assert!(markets.is_empty());
    }

    #[test]
    fn a_frame_that_carries_no_events_yields_none_and_a_bad_one_yields_one_error() {
        let mut decoder = stream::Decoder::default();

        assert!(
            decoder
                .decode(r#"{"status":"UP"}"#)
                .expect("a control frame")
                .is_empty()
        );
        assert_eq!(split(decoder.decode("not json")).len(), 1);
        assert_eq!(split(decoder.decode(r#"{"status":"UP"}"#)).len(), 0);
    }

    #[test]
    fn each_region_is_a_separate_deployment() {
        assert_eq!(UpbitAdapter::new().region(), UpbitRegion::Korea);
        assert_ne!(
            UpbitRegion::Korea.rest_base_url(),
            UpbitRegion::Singapore.rest_base_url()
        );
        assert!(UpbitRegion::Thailand.websocket_url().starts_with("wss://"));
    }

    #[tokio::test]
    async fn indonesia_external_withdrawal_fails_during_preparation_and_submission() {
        use crate::types::{ChainDestination, Network};
        use rust_decimal::Decimal;

        let request = WithdrawRequest::new(
            "BTC",
            Network::Bitcoin,
            Decimal::ONE,
            TransferDestination::Chain(ChainDestination {
                asset: "BTC".to_string(),
                network: Network::Bitcoin,
                address: "bc1destination".to_string(),
                memo: None,
            }),
        );

        let adapter = UpbitAdapter::with_region(UpbitRegion::Indonesia);
        for result in [
            adapter.prepare_withdrawal(&request).await.map(|_| ()),
            adapter.withdraw(&request).await.map(|_| ()),
        ] {
            assert!(matches!(
                result,
                Err(Error::Unsupported {
                    feature: Feature::Withdrawals,
                    ..
                })
            ));
        }
    }
}
