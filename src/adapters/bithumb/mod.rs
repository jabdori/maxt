//! Bithumb spot adapter.

mod parse;
mod private;
mod rest;
mod stream;
mod wallet;

use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use rust_decimal::Decimal;

pub use wallet::BithumbWithdrawalAddress;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{
    CancelOrdersRequest, CandleRequest, DepositAddressRequest, OrderHistoryRequest,
    OrderLookupRequest, OrderRequest, TransferHistoryRequest, TransferLookupRequest,
    WithdrawRequest,
};
use crate::stream::{AccountStream, MarketStream, TypedStream};
use crate::transport::HttpTransport;
use crate::types::{
    AssetNetwork, Balance, CancelOrdersResult, Candle, Cursor, Deposit, DepositAddress,
    DepositAddressEntry, Exchange, Market, MarketInfo, MarketKind, Network, Order, OrderBook,
    OrderRules, OrderType, Page, Side, StreamConfig, Subscription, Ticker, Timestamp, Trade,
    Withdrawal, WithdrawalFee, WithdrawalQuote,
};

/// Query parameters for Bithumb's KRW withdrawal list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BithumbKrwWithdrawalsRequest {
    /// Optional provider withdrawal state, under Bithumb's spelling.
    pub state: Option<String>,
    /// Optional provider withdrawal IDs.
    pub uuids: Vec<String>,
    /// Optional provider transaction IDs.
    pub txids: Vec<String>,
    /// Page number; Bithumb defaults to 1.
    pub page: Option<u32>,
    /// Number of rows; Bithumb defaults to 100 and caps this at 100.
    pub limit: Option<u32>,
    /// Result order; Bithumb defaults to newest first.
    pub order_by: Option<BithumbOrderDirection>,
}

impl BithumbKrwWithdrawalsRequest {
    /// Starts an unfiltered request using Bithumb's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by provider state.
    #[must_use]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Filters by provider withdrawal IDs.
    #[must_use]
    pub fn uuids(mut self, uuids: impl Into<Vec<String>>) -> Self {
        self.uuids = uuids.into();
        self
    }

    /// Filters by provider transaction IDs.
    #[must_use]
    pub fn txids(mut self, txids: impl Into<Vec<String>>) -> Self {
        self.txids = txids.into();
        self
    }

    /// Selects the page number.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Selects the page size.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Selects the result order.
    #[must_use]
    pub fn order_by(mut self, order_by: BithumbOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// Query parameters for Bithumb's KRW deposit list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BithumbKrwDepositsRequest {
    /// Optional provider deposit state, under Bithumb's spelling.
    pub state: Option<String>,
    /// Optional provider deposit IDs.
    pub uuids: Vec<String>,
    /// Optional provider transaction IDs.
    pub txids: Vec<String>,
    /// Page number; Bithumb defaults to 1.
    pub page: Option<u32>,
    /// Number of rows; Bithumb defaults to 100 and caps this at 100.
    pub limit: Option<u32>,
    /// Result order; Bithumb defaults to newest first.
    pub order_by: Option<BithumbOrderDirection>,
}

impl BithumbKrwDepositsRequest {
    /// Starts an unfiltered request using Bithumb's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by provider state.
    #[must_use]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Filters by provider deposit IDs.
    #[must_use]
    pub fn uuids(mut self, uuids: impl Into<Vec<String>>) -> Self {
        self.uuids = uuids.into();
        self
    }

    /// Filters by provider transaction IDs.
    #[must_use]
    pub fn txids(mut self, txids: impl Into<Vec<String>>) -> Self {
        self.txids = txids.into();
        self
    }

    /// Selects the page number.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Selects the page size.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Selects the result order.
    #[must_use]
    pub fn order_by(mut self, order_by: BithumbOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// Request body for a Bithumb KRW deposit or withdrawal.
///
/// Bithumb requires a registered bank account for KRW withdrawals and Kakao
/// second-factor authorization for both KRW transfer writes. The account is a
/// provider-side eligibility condition; Kakao is the only documented request
/// value and is supplied by the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbKrwTransferRequest {
    /// KRW amount submitted to Bithumb.
    pub amount: rust_decimal::Decimal,
}

impl BithumbKrwTransferRequest {
    /// Builds a request using Bithumb's only documented second-factor method.
    pub fn new(amount: rust_decimal::Decimal) -> Self {
        Self { amount }
    }
}

/// One KRW withdrawal returned by Bithumb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbKrwWithdrawal {
    /// Bithumb's transfer type (`withdraw`).
    pub transfer_type: String,
    /// Provider withdrawal identifier.
    pub uuid: String,
    /// Currency returned by Bithumb (normally `KRW`).
    pub currency: String,
    /// Provider network, when Bithumb returns one.
    pub net_type: Option<String>,
    /// Provider transaction identifier, when assigned.
    pub txid: Option<String>,
    /// Provider withdrawal state, under Bithumb's spelling.
    pub state: String,
    /// Creation time.
    pub created_at: Option<Timestamp>,
    /// Completion time, when complete.
    pub done_at: Option<Timestamp>,
    /// Withdrawal amount.
    pub amount: rust_decimal::Decimal,
    /// Withdrawal fee.
    pub fee: rust_decimal::Decimal,
    /// Provider transaction type, normally `default`.
    pub transaction_type: Option<String>,
}

/// One KRW deposit returned by Bithumb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbKrwDeposit {
    /// Bithumb's transfer type (`deposit`).
    pub transfer_type: String,
    /// Provider deposit identifier.
    pub uuid: String,
    /// Currency returned by Bithumb (normally `KRW`).
    pub currency: String,
    /// Provider network, when Bithumb returns one.
    pub net_type: Option<String>,
    /// Provider transaction identifier, when assigned.
    pub txid: Option<String>,
    /// Provider deposit state, under Bithumb's spelling.
    pub state: String,
    /// Creation time.
    pub created_at: Option<Timestamp>,
    /// Completion time, when complete.
    pub done_at: Option<Timestamp>,
    /// Deposit amount.
    pub amount: rust_decimal::Decimal,
    /// Deposit fee.
    pub fee: rust_decimal::Decimal,
    /// Provider transaction type, normally `default`.
    pub transaction_type: Option<String>,
}

pub(crate) const REST_BASE_URL: &str = "https://api.bithumb.com";
pub(crate) const WEBSOCKET_URL: &str = "wss://ws-api.bithumb.com/websocket/v1";
/// Private account events use a separate v2 WebSocket endpoint.
pub(crate) const PRIVATE_WEBSOCKET_URL: &str = "wss://ws-api.bithumb.com/websocket/v2/private";

/// A Bithumb order-book snapshot with provider-specific aggregate fields.
///
/// [`OrderBook`] carries the portable price levels. This type retains the
/// totals and aggregation level Bithumb includes around those levels.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderBookSnapshot {
    /// The portable order-book projection.
    pub common: OrderBook,
    /// Total resting ask size reported by Bithumb, when present.
    pub total_ask_size: Option<Decimal>,
    /// Total resting bid size reported by Bithumb, when present.
    pub total_bid_size: Option<Decimal>,
    /// Provider aggregation level, when Bithumb reports one.
    pub level: Option<Decimal>,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb order response with its portable projection and original body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderResponse {
    /// Portable order projection.
    pub common: Order,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb order-list response with its portable projections and original body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrdersResponse {
    /// Portable order projections.
    pub common: Vec<Order>,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb single-order cancellation response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbCancelOrderResponse {
    /// Bithumb order identifier confirmed by the cancellation response.
    pub order_id: String,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb batch-cancellation response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbCancelOrdersResponse {
    /// Portable cancellation projection.
    pub common: CancelOrdersResult,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb deposit lookup response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbDepositResponse {
    /// Portable deposit projection.
    pub common: Deposit,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb withdrawal lookup response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbWithdrawalResponse {
    /// Portable withdrawal projection.
    pub common: Withdrawal,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// A Bithumb withdrawal-cancellation response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbCancelWithdrawalResponse {
    /// Identifier supplied for the cancellation request.
    pub withdrawal_id: String,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// Bithumb's full-fidelity public market-data stream.
///
/// It keeps the same connection, reconnect, buffering, and close behavior as
/// [`MarketStream`], while yielding Bithumb's provider-specific event types.
/// Construct it with [`BithumbAdapter::subscribe_detailed`].
pub struct BithumbMarketStream {
    inner: TypedStream<BithumbMarketEvent>,
}

impl BithumbMarketStream {
    pub(crate) fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<BithumbMarketEvent>> + Send + 'static,
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

impl Stream for BithumbMarketStream {
    type Item = Result<BithumbMarketEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl fmt::Debug for BithumbMarketStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BithumbMarketStream")
            .finish_non_exhaustive()
    }
}

/// Bithumb's full-fidelity private account stream.
///
/// It keeps the same connection, reconnect, buffering, and close behavior as
/// [`AccountStream`], while yielding Bithumb's provider-specific event types.
/// Construct it with [`BithumbAdapter::subscribe_detailed_account`].
pub struct BithumbAccountStream {
    inner: TypedStream<BithumbAccountEvent>,
}

impl BithumbAccountStream {
    pub(crate) fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<BithumbAccountEvent>> + Send + 'static,
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

impl Stream for BithumbAccountStream {
    type Item = Result<BithumbAccountEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl fmt::Debug for BithumbAccountStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BithumbAccountStream")
            .finish_non_exhaustive()
    }
}

/// One Bithumb-specific public market event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BithumbMarketEvent {
    /// A trade and Bithumb's surrounding change metadata.
    Trade(BithumbTradeEvent),
    /// An order-book update and Bithumb's aggregate sizes.
    OrderBook(BithumbOrderBookEvent),
    /// A ticker update and Bithumb's market-state metadata.
    Ticker(BithumbTickerEvent),
    /// The underlying WebSocket reconnected.
    Reconnected,
}

/// One Bithumb-specific private account event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum BithumbAccountEvent {
    /// A balance update with the provider timestamps retained.
    Asset(BithumbAssetEvent),
    /// An order update with provider status and execution metadata retained.
    Order(BithumbOrderEvent),
    /// The underlying WebSocket reconnected.
    Reconnected,
}

/// A full Bithumb trade event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbTradeEvent {
    /// Portable trade projection.
    pub common: Trade,
    /// Previous close supplied by Bithumb.
    pub previous_closing_price: Option<Decimal>,
    /// Provider change direction such as `RISE` or `FALL`.
    pub change: Option<String>,
    /// Unsigned provider price change.
    pub change_price: Option<Decimal>,
    /// Time at which Bithumb published this frame.
    pub published_at: Option<Timestamp>,
    /// Provider stream phase such as `SNAPSHOT` or `REALTIME`.
    pub stream_type: Option<String>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Bithumb order-book event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderBookEvent {
    /// Portable order-book projection.
    pub common: OrderBook,
    /// Total resting ask size supplied by Bithumb.
    pub total_ask_size: Option<Decimal>,
    /// Total resting bid size supplied by Bithumb.
    pub total_bid_size: Option<Decimal>,
    /// Provider aggregation level, when present.
    pub level: Option<Decimal>,
    /// Provider stream phase such as `SNAPSHOT` or `REALTIME`.
    pub stream_type: Option<String>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Bithumb ticker event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbTickerEvent {
    /// Portable ticker projection.
    pub common: Ticker,
    /// Provider change direction such as `RISE` or `FALL`.
    pub change_direction: Option<String>,
    /// Provider market lifecycle state.
    pub market_state: Option<String>,
    /// Whether Bithumb reports trading as suspended.
    pub trading_suspended: Option<bool>,
    /// Provider market warning state.
    pub market_warning: Option<String>,
    /// Provider stream phase such as `SNAPSHOT` or `REALTIME`.
    pub stream_type: Option<String>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Bithumb private asset event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbAssetEvent {
    /// Portable balance projections.
    pub balances: Vec<Balance>,
    /// Time at which Bithumb assembled the asset snapshot.
    pub asset_timestamp: Option<Timestamp>,
    /// Time at which Bithumb published the frame.
    pub published_at: Option<Timestamp>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A full Bithumb private order event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderEvent {
    /// Portable order projection.
    pub common: Order,
    /// Client-provided order identifier, when present.
    pub client_order_id: Option<String>,
    /// Provider order type.
    pub order_type: Option<String>,
    /// Provider lifecycle state.
    pub state: Option<String>,
    /// Provider time-in-force value.
    pub time_in_force: Option<String>,
    /// Original order amount in quote currency, when supplied.
    pub order_amount: Option<Decimal>,
    /// Provider execution identifier, when the frame reports a fill.
    pub trade_id: Option<String>,
    /// Provider fill price, quantity, and amount.
    pub trade_price: Option<Decimal>,
    /// Provider fill quantity.
    pub trade_quantity: Option<Decimal>,
    /// Provider fill amount in quote currency.
    pub trade_amount: Option<Decimal>,
    /// Provider fill time.
    pub trade_timestamp: Option<Timestamp>,
    /// Provider cumulative executed amount in quote currency.
    pub executed_amount: Option<Decimal>,
    /// Provider paid and remaining fees.
    pub paid_fee: Option<Decimal>,
    /// Provider remaining fee.
    pub remaining_fee: Option<Decimal>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

pub(crate) fn network_from_provider(raw: &str) -> Network {
    match raw.trim().to_ascii_uppercase().as_str() {
        "BTC" | "BITCOIN" => Network::Bitcoin,
        "ETH" | "ETHEREUM" => Network::Ethereum,
        "ARB" | "ARBITRUM" | "ARBITRUM ONE" => Network::Arbitrum,
        "BSC" | "BEP20" | "BNB SMART CHAIN" => Network::BnbSmartChain,
        "TRX" | "TRON" => Network::Tron,
        "SOL" | "SOLANA" => Network::Solana,
        "MATIC" | "POLYGON" | "POLYGON POS" => Network::Polygon,
        "BASE" => Network::Base,
        "OP" | "OPTIMISM" => Network::Optimism,
        "AVAXC" | "AVAX-C" | "AVALANCHE C-CHAIN" => Network::AvalancheC,
        "XRP" | "XRP LEDGER" => Network::XrpLedger,
        "XLM" | "STELLAR" => Network::Stellar,
        "ATOM" | "COSMOS" | "COSMOS HUB" => Network::Cosmos,
        "APT" | "APTOS" => Network::Aptos,
        "SUI" => Network::Sui,
        "TON" => Network::Ton,
        "NEAR" => Network::Near,
        "DOT" | "POLKADOT" => Network::Polkadot,
        _ => Network::Other(raw.trim().to_owned()),
    }
}

/// Severity of a Bithumb market alert (경보제), ordered from least to most severe.
///
/// This is separate from the `CAUTION` investment-warning flag returned by
/// [`BithumbAdapter::market_warnings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BithumbAlertStep {
    /// Caution (주의), the lowest alert level.
    Caution,
    /// Warning (경고), the middle alert level.
    Warning,
    /// Danger (위험), the highest documented alert level.
    Danger,
    /// An unrecognized level, ordered above [`Self::Danger`] so thresholds surface it.
    Unknown,
}

/// An active Bithumb market alert for one market and criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbMarketAlert {
    /// Bithumb's criterion code, preserved verbatim for forward compatibility.
    pub kind: String,
    /// Alert severity.
    pub step: BithumbAlertStep,
    /// Alert expiry, converted from Bithumb's KST wall-clock value to UTC.
    pub ends_at: Timestamp,
}

/// One Bithumb exchange notice.
///
/// Bithumb publishes `published_at` and `modified_at` as Korea Standard Time
/// wall-clock values. They are converted to UTC timestamps at the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbNotice {
    /// Bithumb's notice categories, in the order supplied by the provider.
    pub categories: Vec<String>,
    /// Notice title.
    pub title: String,
    /// Provider-hosted notice URL.
    pub url: String,
    /// Publication time converted from KST to UTC.
    pub published_at: Timestamp,
    /// Most recent modification time converted from KST to UTC.
    pub modified_at: Timestamp,
}

/// One API key registered on a Bithumb account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbApiKey {
    /// Bithumb's API access-key identifier.
    pub access_key: String,
    /// Key expiration time, converted from Bithumb's offset timestamp.
    pub expires_at: Timestamp,
}

/// A state accepted by Bithumb's pending-order endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BithumbPendingOrderState {
    /// An order resting in the order book.
    Wait,
    /// A reserved order waiting for its trigger price.
    Watch,
}

/// A sort direction accepted by Bithumb's pending-order endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BithumbOrderDirection {
    /// Oldest orders first.
    Ascending,
    /// Newest orders first.
    Descending,
}

/// A final state accepted by Bithumb's closed-order endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BithumbClosedOrderState {
    /// An order whose execution completed.
    Done,
    /// An order that was cancelled.
    Cancel,
}

impl BithumbClosedOrderState {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Cancel => "cancel",
        }
    }
}

/// Filters for Bithumb's cursor-paginated closed-order endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BithumbClosedOrdersRequest {
    /// Optional Bithumb spot-market filter.
    pub market: Option<Market>,
    /// One final-state filter.
    pub state: Option<BithumbClosedOrderState>,
    /// Multiple final-state filters.
    pub states: Vec<BithumbClosedOrderState>,
    /// Optional beginning of the provider query window.
    pub start_time: Option<Timestamp>,
    /// Optional end of the provider query window.
    pub end_time: Option<Timestamp>,
    /// Page size from 1 through 1,000.
    pub limit: Option<u32>,
    /// Optional result order; Bithumb defaults to newest first.
    pub order_by: Option<BithumbOrderDirection>,
    /// Opaque cursor returned by the preceding page.
    pub cursor: Option<Cursor>,
}

impl BithumbClosedOrdersRequest {
    /// Starts an unfiltered request using Bithumb's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by one market.
    #[must_use]
    pub fn market(mut self, market: Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Filters by one final state.
    #[must_use]
    pub fn state(mut self, state: BithumbClosedOrderState) -> Self {
        self.state = Some(state);
        self
    }

    /// Filters by multiple final states.
    #[must_use]
    pub fn states(mut self, states: impl Into<Vec<BithumbClosedOrderState>>) -> Self {
        self.states = states.into();
        self
    }

    /// Sets the beginning of the provider query window.
    #[must_use]
    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the end of the provider query window.
    #[must_use]
    pub fn end_time(mut self, end_time: Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Selects the page size from 1 through 1,000.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Selects the result order.
    #[must_use]
    pub fn order_by(mut self, order_by: BithumbOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }

    /// Continues from a provider-issued opaque cursor.
    #[must_use]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

/// A state accepted by Bithumb's legacy order-list endpoint.
///
/// The `watch` state represents automatic orders. Bithumb does not allow it
/// in the same multi-state request as normal-order states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BithumbOrderListState {
    /// An order resting in the order book.
    Wait,
    /// An automatic order waiting for its trigger condition.
    Watch,
    /// An order whose execution completed.
    Done,
    /// An order that was cancelled.
    Cancel,
}

impl BithumbOrderListState {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Wait => "wait",
            Self::Watch => "watch",
            Self::Done => "done",
            Self::Cancel => "cancel",
        }
    }
}

/// Filters for Bithumb's legacy paginated order-list endpoint.
///
/// Set either `state` or `states`, never both. If both identifier lists are
/// non-empty, Bithumb resolves `uuids` and ignores `client_order_ids`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BithumbOrderListRequest {
    /// Optional Bithumb spot-market filter.
    pub market: Option<Market>,
    /// One lifecycle-state filter.
    pub state: Option<BithumbOrderListState>,
    /// Multiple lifecycle-state filters.
    pub states: Vec<BithumbOrderListState>,
    /// Exchange-assigned order identifiers, limited to 100 values.
    pub uuids: Vec<String>,
    /// Caller-assigned order identifiers, limited to 100 values.
    pub client_order_ids: Vec<String>,
    /// Page number; Bithumb defaults to 1.
    pub page: Option<u32>,
    /// Page size; Bithumb defaults to 100 and serves at most 100 rows.
    pub limit: Option<u32>,
    /// Result order; Bithumb defaults to newest first.
    pub order_by: Option<BithumbOrderDirection>,
}

impl BithumbOrderListRequest {
    /// Starts an unfiltered request using Bithumb's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by one market.
    #[must_use]
    pub fn market(mut self, market: Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Filters by one lifecycle state.
    #[must_use]
    pub fn state(mut self, state: BithumbOrderListState) -> Self {
        self.state = Some(state);
        self
    }

    /// Filters by multiple lifecycle states.
    #[must_use]
    pub fn states(mut self, states: impl Into<Vec<BithumbOrderListState>>) -> Self {
        self.states = states.into();
        self
    }

    /// Filters by Bithumb's exchange-assigned order identifiers.
    #[must_use]
    pub fn uuids(mut self, uuids: impl Into<Vec<String>>) -> Self {
        self.uuids = uuids.into();
        self
    }

    /// Filters by caller-assigned order identifiers.
    #[must_use]
    pub fn client_order_ids(mut self, client_order_ids: impl Into<Vec<String>>) -> Self {
        self.client_order_ids = client_order_ids.into();
        self
    }

    /// Selects a one-based Bithumb page number.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Selects the page size from 1 through 100.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Selects the result order.
    #[must_use]
    pub fn order_by(mut self, order_by: BithumbOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// Filters for Bithumb's paginated pending-order endpoint.
///
/// Leave `state` and `order_by` unset to use Bithumb's `wait` and `desc`
/// defaults. Leave `limit` unset to use Bithumb's default page size of 100.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BithumbPendingOrdersRequest {
    /// Optional market filter.
    pub market: Option<Market>,
    /// Resting (`wait`) or trigger-waiting (`watch`) orders.
    pub state: Option<BithumbPendingOrderState>,
    /// Page size from 1 through 100.
    pub limit: Option<u32>,
    /// Optional result order; Bithumb defaults to newest first.
    pub order_by: Option<BithumbOrderDirection>,
    /// Cursor returned by the preceding page.
    pub cursor: Option<Cursor>,
}

/// Identifies one Bithumb order while retaining its provider-specific detail.
///
/// [`BithumbOrderDetailRequest::market`] is not sent to Bithumb: `/v1/order`
/// accepts only identifiers. It makes the expected market explicit and lets
/// the adapter reject a response for a different market.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbOrderDetailRequest {
    /// Market the returned order must belong to.
    pub market: Market,
    /// Exchange-assigned order identifier.
    ///
    /// Bithumb gives this field precedence when both identifiers are present.
    pub uuid: Option<String>,
    /// Caller-assigned identifier, when the order was created with one.
    pub client_order_id: Option<String>,
}

impl BithumbOrderDetailRequest {
    /// Starts a lookup for one expected market.
    pub fn new(market: Market) -> Self {
        Self {
            market,
            uuid: None,
            client_order_id: None,
        }
    }

    /// Starts a lookup by Bithumb's exchange-assigned identifier.
    pub fn by_uuid(market: Market, uuid: impl Into<String>) -> Self {
        Self::new(market).uuid(uuid)
    }

    /// Starts a lookup by the caller-assigned identifier.
    pub fn by_client_order_id(market: Market, client_order_id: impl Into<String>) -> Self {
        Self::new(market).client_order_id(client_order_id)
    }

    /// Adds Bithumb's exchange-assigned identifier.
    ///
    /// When a client identifier is also present, Bithumb resolves the UUID.
    #[must_use]
    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.uuid = Some(uuid.into());
        self
    }

    /// Adds the caller-assigned identifier.
    #[must_use]
    pub fn client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }
}

/// One fill returned inside [`BithumbOrderDetail`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderDetailTrade {
    /// Market Bithumb reports for this fill.
    pub market: Market,
    /// Bithumb's fill identifier.
    pub uuid: String,
    /// Execution price.
    pub price: rust_decimal::Decimal,
    /// Executed base quantity.
    pub volume: rust_decimal::Decimal,
    /// Executed quote amount.
    pub funds: rust_decimal::Decimal,
    /// Bithumb's raw fill side (`bid` or `ask`).
    pub side: String,
    /// Execution time.
    pub created_at: Timestamp,
}

/// The full provider-specific response from Bithumb's single-order endpoint.
///
/// The common [`Order`] drops fee, fill, cancellation, and provider lifecycle
/// fields. This type retains them under Bithumb's documented semantics. Raw
/// provider enum strings remain strings so new Bithumb values do not make an
/// otherwise valid response undecodable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderDetail {
    /// Bithumb's exchange-assigned order identifier.
    pub uuid: String,
    /// Caller-assigned identifier, when the order was created with one.
    pub client_order_id: Option<String>,
    /// Bithumb's raw order side (`bid` or `ask`).
    pub side: String,
    /// Bithumb's raw order type, such as `limit`, `price`, `market`, or `best`.
    pub order_type: String,
    /// Order price.
    pub price: rust_decimal::Decimal,
    /// Bithumb's raw lifecycle state, such as `wait`, `watch`, `done`, or `cancel`.
    pub state: String,
    /// Bithumb spot market.
    pub market: Market,
    /// Order creation time.
    pub created_at: Timestamp,
    /// Submitted base quantity.
    pub volume: rust_decimal::Decimal,
    /// Base quantity remaining after fills.
    pub remaining_volume: rust_decimal::Decimal,
    /// Fee Bithumb reserved when the order was accepted.
    pub reserved_fee: rust_decimal::Decimal,
    /// Reserved fee not yet used.
    pub remaining_fee: rust_decimal::Decimal,
    /// Cumulative paid fee.
    pub paid_fee: rust_decimal::Decimal,
    /// Funds or quantity still locked in the order.
    pub locked: rust_decimal::Decimal,
    /// Cumulative executed base quantity.
    pub executed_volume: rust_decimal::Decimal,
    /// Cumulative executed quote amount.
    pub executed_funds: rust_decimal::Decimal,
    /// Number of fills Bithumb associates with this order.
    pub trades_count: u32,
    /// Individual fills in Bithumb's response order.
    pub trades: Vec<BithumbOrderDetailTrade>,
    /// Raw self-trade-prevention outcome, when Bithumb returns one.
    pub stp_type: Option<String>,
    /// Raw cancellation cause, when Bithumb returns one.
    pub cancel_type: Option<String>,
    /// Identifier of the order that caused an STP cancellation, when supplied.
    pub canceling_uuid: Option<String>,
    /// Raw time-in-force value, when the order used one.
    pub time_in_force: Option<String>,
}

/// One provider-specific order returned by Bithumb's legacy list endpoint.
///
/// Unlike [`BithumbOrderDetail`], the list endpoint does not return individual
/// fills or cancellation-cause fields. Every field it does return is retained
/// here instead of being reduced to the common [`Order`] model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbOrderListItem {
    /// Bithumb's exchange-assigned order identifier.
    pub uuid: String,
    /// Caller-assigned identifier, when the order was created with one.
    pub client_order_id: Option<String>,
    /// Bithumb's raw order side (`bid` or `ask`).
    pub side: String,
    /// Bithumb's raw order type, such as `limit`, `price`, `market`, or `best`.
    pub order_type: String,
    /// Order price.
    pub price: rust_decimal::Decimal,
    /// Bithumb's raw lifecycle state, such as `wait`, `watch`, `done`, or `cancel`.
    pub state: String,
    /// Bithumb spot market.
    pub market: Market,
    /// Order creation time.
    pub created_at: Timestamp,
    /// Submitted base quantity.
    pub volume: rust_decimal::Decimal,
    /// Base quantity remaining after fills.
    pub remaining_volume: rust_decimal::Decimal,
    /// Fee Bithumb reserved when the order was accepted.
    pub reserved_fee: rust_decimal::Decimal,
    /// Reserved fee not yet used.
    pub remaining_fee: rust_decimal::Decimal,
    /// Cumulative paid fee.
    pub paid_fee: rust_decimal::Decimal,
    /// Funds or quantity still locked in the order.
    pub locked: rust_decimal::Decimal,
    /// Cumulative executed base quantity.
    pub executed_volume: rust_decimal::Decimal,
    /// Cumulative executed quote amount.
    pub executed_funds: rust_decimal::Decimal,
    /// Number of fills Bithumb associates with this order.
    pub trades_count: u32,
    /// Raw self-trade-prevention outcome, when Bithumb returns one.
    pub stp_type: Option<String>,
    /// Raw time-in-force value, when the order used one.
    pub time_in_force: Option<String>,
    /// Complete provider order object encoded as JSON.
    pub raw_json: String,
}

/// One provider-specific order returned by Bithumb's closed-order endpoint.
///
/// Provider enum fields remain raw strings so newly introduced Bithumb values
/// do not make an otherwise valid response undecodable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbClosedOrder {
    /// Bithumb's exchange-assigned order identifier.
    pub order_id: String,
    /// Bithumb's raw order side.
    pub side: String,
    /// Bithumb's raw order type.
    pub order_type: String,
    /// Order price, when Bithumb returns it.
    pub price: Option<rust_decimal::Decimal>,
    /// Bithumb's raw final state.
    pub state: String,
    /// Bithumb spot market.
    pub market: Market,
    /// Order creation time, when Bithumb returns it.
    pub created_at: Option<Timestamp>,
    /// Submitted base quantity.
    pub volume: rust_decimal::Decimal,
    /// Base quantity remaining after fills.
    pub remaining_volume: rust_decimal::Decimal,
    /// Fee Bithumb reserved when the order was accepted.
    pub reserved_fee: rust_decimal::Decimal,
    /// Reserved fee not yet used.
    pub remaining_fee: rust_decimal::Decimal,
    /// Cumulative paid fee.
    pub paid_fee: rust_decimal::Decimal,
    /// Funds or quantity still locked in the order.
    pub locked: rust_decimal::Decimal,
    /// Cumulative executed base quantity.
    pub executed_volume: rust_decimal::Decimal,
    /// Cumulative executed quote amount.
    pub executed_funds: rust_decimal::Decimal,
    /// Number of fills Bithumb associates with this order.
    pub trades_count: u32,
    /// Caller-assigned identifier, when one exists.
    pub client_order_id: Option<String>,
    /// Raw self-trade-prevention outcome, when Bithumb returns one.
    pub stp_type: Option<String>,
    /// Raw time-in-force value, when the order used one.
    pub time_in_force: Option<String>,
    /// Raw cancellation cause, when Bithumb returns one.
    pub cancel_type: Option<String>,
    /// Order that caused an STP cancellation, when supplied.
    pub canceling_order_id: Option<String>,
}

/// One order in a Bithumb batch-order request.
///
/// The common [`OrderRequest`] is used directly so the validation and amount
/// semantics stay identical to single-order placement. Bithumb still exposes
/// this endpoint as a provider-specific operation because its response is
/// non-atomic: every item can be accepted or rejected independently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbBatchOrdersRequest {
    /// Orders to submit. Bithumb accepts between 1 and 20 items.
    pub orders: Vec<OrderRequest>,
}

impl BithumbBatchOrdersRequest {
    /// Creates a batch request from the supplied order list.
    pub fn new(orders: impl Into<Vec<OrderRequest>>) -> Self {
        Self {
            orders: orders.into(),
        }
    }
}

impl From<Vec<OrderRequest>> for BithumbBatchOrdersRequest {
    fn from(orders: Vec<OrderRequest>) -> Self {
        Self::new(orders)
    }
}

/// One accepted order returned by Bithumb's batch-order endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbBatchOrder {
    /// Exchange-assigned order identifier.
    pub order_id: String,
    /// Caller-assigned identifier, when supplied.
    pub client_order_id: Option<String>,
    /// Market returned by Bithumb.
    pub market: Market,
    /// Buy or sell.
    pub side: Side,
    /// Provider order type.
    pub order_type: OrderType,
    /// Provider time-in-force value, when the request specified one.
    ///
    /// This is retained as Bithumb's raw value so a newly introduced provider
    /// value does not make an otherwise successful batch response undecodable.
    pub time_in_force: Option<String>,
    /// Provider self-trade-prevention value, when returned.
    ///
    /// Bithumb may add values independently of the common order model, so the
    /// raw value is retained rather than narrowing it to a closed enum.
    pub stp_type: Option<String>,
    /// Acceptance time, when returned.
    pub created_at: Option<Timestamp>,
}

/// One rejected item returned by Bithumb's batch-order endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbBatchOrderFailure {
    /// Caller-assigned identifier, when supplied.
    pub client_order_id: Option<String>,
    /// Provider time-in-force value, when returned.
    pub time_in_force: Option<String>,
    /// Provider error code (`name`).
    pub code: String,
    /// Provider error message.
    pub message: String,
}

/// The non-atomic result of a Bithumb batch-order request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BithumbBatchOrderOutcome {
    /// This item was accepted for placement.
    Accepted(BithumbBatchOrder),
    /// This item was rejected while other items may have succeeded.
    Rejected(BithumbBatchOrderFailure),
}

/// All per-item outcomes returned by Bithumb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbBatchOrdersResult {
    /// Results retain Bithumb's response order.
    pub outcomes: Vec<BithumbBatchOrderOutcome>,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
}

/// State filter accepted by Bithumb's TWAP history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BithumbTwapState {
    /// The strategy is still submitting child orders.
    Progress,
    /// All child orders have completed.
    Done,
    /// The strategy was cancelled before completion.
    Cancel,
}

/// Sort direction accepted by Bithumb's TWAP history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BithumbTwapOrderDirection {
    /// Oldest TWAP orders first.
    Ascending,
    /// Newest TWAP orders first.
    Descending,
}

/// A page request for Bithumb TWAP orders.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BithumbTwapOrdersRequest {
    /// Optional market filter.
    pub market: Option<Market>,
    /// Optional TWAP identifiers. Bithumb accepts at most the server limit.
    pub uuids: Vec<String>,
    /// Optional lifecycle filter; Bithumb defaults to `progress`.
    pub state: Option<BithumbTwapState>,
    /// Cursor returned by the preceding page.
    pub cursor: Option<Cursor>,
    /// Page size from 1 through 100.
    pub limit: Option<u32>,
    /// Optional result order; Bithumb defaults to newest first.
    pub order_by: Option<BithumbTwapOrderDirection>,
}

impl BithumbTwapOrdersRequest {
    /// Starts an unfiltered request using Bithumb's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by market.
    #[must_use]
    pub fn market(mut self, market: Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Filters by one or more TWAP identifiers.
    #[must_use]
    pub fn uuids(mut self, uuids: impl Into<Vec<String>>) -> Self {
        self.uuids = uuids.into();
        self
    }

    /// Filters by lifecycle state.
    #[must_use]
    pub fn state(mut self, state: BithumbTwapState) -> Self {
        self.state = Some(state);
        self
    }

    /// Resumes from a cursor returned by the preceding page.
    #[must_use]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Sets the page size.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Selects the result order.
    #[must_use]
    pub fn order_by(mut self, order_by: BithumbTwapOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// Parameters for creating one Bithumb TWAP order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbTwapOrderRequest {
    /// Bithumb spot market.
    pub market: Market,
    /// Buy (`bid`) or sell (`ask`).
    pub side: Side,
    /// Total base quantity for a sell request.
    pub volume: Option<rust_decimal::Decimal>,
    /// Total quote amount for a buy request.
    pub price: Option<rust_decimal::Decimal>,
    /// Total execution duration in seconds, from 300 through 43,200.
    pub duration: u32,
    /// Child-order interval in seconds: 15, 20, 30, 60, or 120.
    pub frequency: u32,
}

/// One TWAP order returned by Bithumb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbTwapOrder {
    /// TWAP identifier (`uuid`, also called `algo_order_id`).
    pub id: String,
    /// Buy or sell side.
    pub side: Side,
    /// Price captured when the strategy was created.
    pub price: rust_decimal::Decimal,
    /// Provider lifecycle state.
    pub state: BithumbTwapState,
    /// Bithumb spot market.
    pub market: Market,
    /// Creation time.
    pub created_at: Timestamp,
    /// User-supplied total quantity or amount.
    pub volume: rust_decimal::Decimal,
    /// Completion time, when Bithumb reports one.
    pub finished_at: Option<Timestamp>,
    /// Number of child orders planned.
    pub total_order_count: u32,
    /// Number of child orders that traded.
    pub total_trades_count: u32,
    /// Number of child orders submitted so far.
    pub progress_count: u32,
    /// Total quote amount filled.
    pub total_executed_amount: rust_decimal::Decimal,
    /// Total base quantity filled.
    pub total_executed_volume: rust_decimal::Decimal,
    /// Average fill price.
    pub avg_trade_price: rust_decimal::Decimal,
    /// Bithumb wallet identifier, when Bithumb reports one.
    pub wallet_id: Option<String>,
    /// Cancellation time, when the order was cancelled.
    pub canceled_at: Option<Timestamp>,
    /// Cancellation reason, preserved when Bithumb reports one.
    pub cancel_type: Option<String>,
}

impl BithumbPendingOrdersRequest {
    /// Starts an unfiltered request using Bithumb's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by market.
    #[must_use]
    pub fn market(mut self, market: Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Selects the pending-order state.
    #[must_use]
    pub fn state(mut self, state: BithumbPendingOrderState) -> Self {
        self.state = Some(state);
        self
    }

    /// Sets the page size.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Selects the result order.
    #[must_use]
    pub fn order_by(mut self, order_by: BithumbOrderDirection) -> Self {
        self.order_by = Some(order_by);
        self
    }

    /// Resumes from a cursor returned by the preceding page.
    #[must_use]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

/// One Bithumb asset's public transfer-fee catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbAssetFee {
    /// Bithumb's display name for the asset.
    pub display_name: String,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Fee rules for every network Bithumb returned for this asset.
    pub networks: Vec<BithumbNetworkFee>,
}

/// One Bithumb network's public transfer-fee rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BithumbNetworkFee {
    /// Canonical network when maxt recognizes Bithumb's network name.
    pub network: Network,
    /// Bithumb's exact `net_name` value.
    pub provider_name: String,
    /// Deposit fee in the transferred asset.
    pub deposit_fee: rust_decimal::Decimal,
    /// Minimum deposit amount.
    pub minimum_deposit: rust_decimal::Decimal,
    /// Fixed or rate-based withdrawal fee rule.
    pub withdrawal_fee: WithdrawalFee,
    /// Minimum withdrawal amount.
    pub minimum_withdrawal: rust_decimal::Decimal,
}

/// Adapter for Bithumb spot markets.
///
/// Public REST supports markets, trades, order books, tickers, and candles.
/// Public WebSocket supports trades, order books, and tickers. Derivatives and
/// candle streams return [`Error::Unsupported`](crate::Error::Unsupported).
#[derive(Debug, Clone)]
pub struct BithumbAdapter {
    credentials: Option<BithumbCredentials>,
    // Stored as a result because `new` is infallible.
    http: Result<HttpTransport>,
}

#[derive(Debug, Clone)]
pub(crate) struct BithumbCredentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
}

impl BithumbAdapter {
    /// Creates an adapter for public market data.
    pub fn new() -> Self {
        Self {
            credentials: None,
            http: HttpTransport::new(REST_BASE_URL),
        }
    }

    /// Adds the access key and secret key required by private APIs.
    #[must_use]
    pub fn with_credentials(
        mut self,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(BithumbCredentials {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Returns every listed market with its raw investment-warning flag.
    ///
    /// The value is `NONE` or `CAUTION` (유의 종목). A warned market remains
    /// tradable and maps to [`MarketStatus::Unknown`](crate::MarketStatus::Unknown).
    /// This flag is separate from [`Self::market_alerts`].
    pub async fn market_warnings(&self) -> Result<Vec<(Market, String)>> {
        rest::market_warnings(self.http()?).await
    }

    /// Returns active alert-system rows, one per market and criterion.
    ///
    /// Markets without alerts are omitted. Alerts do not change the common
    /// [`MarketStatus`](crate::MarketStatus).
    pub async fn market_alerts(&self) -> Result<Vec<(Market, BithumbMarketAlert)>> {
        rest::market_alerts(self.http()?).await
    }

    /// Returns the newest Bithumb exchange notices first.
    ///
    /// `count` must be from 1 through 20. `None` uses Bithumb's documented
    /// five-notice default.
    pub async fn notices(&self, count: Option<u32>) -> Result<Vec<BithumbNotice>> {
        rest::notices(self.http()?, count).await
    }

    /// Returns Bithumb's public transfer-fee catalog for one asset or `ALL`.
    ///
    /// The returned values are fee rules, not account-specific withdrawal
    /// availability. Use [`Adapter::asset_networks`] for an authenticated
    /// account's current transfer status and limits.
    pub async fn transfer_fees(&self, currency: &str) -> Result<Vec<BithumbAssetFee>> {
        rest::transfer_fees(self.http()?, currency).await
    }

    /// Reads one Bithumb order-book snapshot without discarding its aggregate
    /// provider fields.
    pub async fn order_book_snapshot(
        &self,
        market: &Market,
        depth: Option<u32>,
    ) -> Result<BithumbOrderBookSnapshot> {
        rest::order_book_snapshot(self.http()?, market, depth).await
    }

    /// Subscribes to Bithumb market data without discarding provider fields.
    pub async fn subscribe_detailed(
        &self,
        subscription: &Subscription,
    ) -> Result<BithumbMarketStream> {
        self.subscribe_detailed_with(subscription, &crate::client::default_stream_config())
            .await
    }

    /// Subscribes to detailed Bithumb market data with explicit stream settings.
    pub async fn subscribe_detailed_with(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> Result<BithumbMarketStream> {
        stream::subscribe_detailed(subscription, config).await
    }

    /// Subscribes to Bithumb account updates without discarding provider fields.
    pub async fn subscribe_detailed_account(&self) -> Result<BithumbAccountStream> {
        self.subscribe_detailed_account_with(&crate::client::default_stream_config())
            .await
    }

    /// Subscribes to detailed Bithumb account updates with explicit stream settings.
    pub async fn subscribe_detailed_account_with(
        &self,
        config: &StreamConfig,
    ) -> Result<BithumbAccountStream> {
        stream::subscribe_detailed_account(self.credentials()?, config).await
    }

    /// Returns Bithumb's KRW withdrawal list.
    pub async fn krw_withdrawals(
        &self,
        request: &BithumbKrwWithdrawalsRequest,
    ) -> Result<Vec<BithumbKrwWithdrawal>> {
        wallet::krw_withdrawals(self.http()?, self.credentials()?, request).await
    }

    /// Requests a KRW withdrawal to the bank account registered with Bithumb.
    ///
    /// The account must be eligible on Bithumb and the API key must permit
    /// KRW withdrawals. Bithumb performs the Kakao second-factor step; this
    /// method does not accept or store local bank-account credentials.
    pub async fn withdraw_krw(
        &self,
        request: &BithumbKrwTransferRequest,
    ) -> Result<BithumbKrwWithdrawal> {
        wallet::withdraw_krw(self.http()?, self.credentials()?, request).await
    }

    /// Returns Bithumb's KRW deposit list.
    pub async fn krw_deposits(
        &self,
        request: &BithumbKrwDepositsRequest,
    ) -> Result<Vec<BithumbKrwDeposit>> {
        wallet::krw_deposits(self.http()?, self.credentials()?, request).await
    }

    /// Requests a KRW deposit using Bithumb's Kakao second-factor method.
    ///
    /// The source bank account and second-factor approval are provider-side
    /// eligibility conditions, not request fields accepted by this API.
    pub async fn deposit_krw(
        &self,
        request: &BithumbKrwTransferRequest,
    ) -> Result<BithumbKrwDeposit> {
        wallet::deposit_krw(self.http()?, self.credentials()?, request).await
    }

    /// Returns the API keys registered on this Bithumb account.
    pub async fn api_keys(&self) -> Result<Vec<BithumbApiKey>> {
        private::api_keys(self.http()?, self.credentials()?).await
    }

    /// Returns every registered Bithumb withdrawal-address entry for this account.
    ///
    /// This provider-specific list is distinct from [`Adapter::prepare_withdrawal`]:
    /// it returns Bithumb's registered address metadata and does not validate a
    /// prospective withdrawal or calculate a common withdrawal quote.
    pub async fn withdrawal_addresses(&self) -> Result<Vec<BithumbWithdrawalAddress>> {
        wallet::withdrawal_addresses(self.http()?, self.credentials()?).await
    }

    /// Returns one Bithumb order with its provider-specific fill, fee, and
    /// cancellation metadata.
    ///
    /// `/v1/order` resolves `uuid` before `client_order_id` when both are set.
    /// The request's market is validated locally against the returned order;
    /// Bithumb does not accept it as a query parameter.
    pub async fn order_detail(
        &self,
        request: &BithumbOrderDetailRequest,
    ) -> Result<BithumbOrderDetail> {
        private::order_detail(self.http()?, self.credentials()?, request).await
    }

    /// Looks up Bithumb orders by identifier without discarding the response body.
    pub async fn orders_by_ids_detail(
        &self,
        request: &OrderLookupRequest,
    ) -> Result<BithumbOrdersResponse> {
        private::orders_by_ids_detail(self.http()?, self.credentials()?, request).await
    }

    /// Places one Bithumb order without discarding the provider acknowledgement.
    pub async fn place_order_detail(&self, request: &OrderRequest) -> Result<BithumbOrderResponse> {
        private::place_order_detail(self.http()?, self.credentials()?, request).await
    }

    /// Cancels one Bithumb order by exchange identifier and preserves the response.
    pub async fn cancel_order_detail(
        &self,
        market: &Market,
        order_id: &str,
    ) -> Result<BithumbCancelOrderResponse> {
        private::cancel_order_detail(self.http()?, self.credentials()?, market, order_id).await
    }

    /// Cancels one Bithumb order by client identifier and preserves the response.
    pub async fn cancel_order_by_client_id_detail(
        &self,
        market: &Market,
        client_id: &str,
    ) -> Result<BithumbCancelOrderResponse> {
        private::cancel_order_by_client_id_detail(
            self.http()?,
            self.credentials()?,
            market,
            client_id,
        )
        .await
    }

    /// Cancels a Bithumb order batch without discarding provider failures or metadata.
    pub async fn cancel_orders_detail(
        &self,
        request: &CancelOrdersRequest,
    ) -> Result<BithumbCancelOrdersResponse> {
        private::cancel_orders_detail(self.http()?, self.credentials()?, request).await
    }

    /// Reads one Bithumb deposit without discarding provider fields.
    pub async fn deposit_detail(
        &self,
        request: &TransferLookupRequest,
    ) -> Result<BithumbDepositResponse> {
        wallet::deposit_detail(self.http()?, self.credentials()?, request).await
    }

    /// Reads one Bithumb withdrawal without discarding provider fields.
    pub async fn withdrawal_detail(
        &self,
        request: &TransferLookupRequest,
    ) -> Result<BithumbWithdrawalResponse> {
        wallet::withdrawal_detail(self.http()?, self.credentials()?, request).await
    }

    /// Cancels one Bithumb withdrawal and preserves the provider response body.
    pub async fn cancel_withdrawal_detail(
        &self,
        withdrawal_id: &str,
    ) -> Result<BithumbCancelWithdrawalResponse> {
        wallet::cancel_withdrawal_detail(self.http()?, self.credentials()?, withdrawal_id).await
    }

    /// Returns one legacy Bithumb order-list page without dropping provider fields.
    ///
    /// The endpoint accepts ordinary and automatic order states, but Bithumb
    /// forbids mixing `watch` with `wait`, `done`, or `cancel` in `states`.
    /// When both identifier lists are supplied, Bithumb gives `uuids`
    /// precedence over `client_order_ids`.
    pub async fn order_list(
        &self,
        request: &BithumbOrderListRequest,
    ) -> Result<Vec<BithumbOrderListItem>> {
        private::order_list(self.http()?, self.credentials()?, request).await
    }

    /// Returns one page of final Bithumb orders without dropping provider fields.
    pub async fn closed_orders(
        &self,
        request: &BithumbClosedOrdersRequest,
    ) -> Result<Page<BithumbClosedOrder>> {
        private::closed_orders(self.http()?, self.credentials()?, request).await
    }

    /// Returns one page of Bithumb `wait` or `watch` orders.
    pub async fn pending_orders(
        &self,
        request: &BithumbPendingOrdersRequest,
    ) -> Result<Page<Order>> {
        private::pending_orders(self.http()?, self.credentials()?, request).await
    }

    /// Returns one page of Bithumb TWAP orders.
    pub async fn twap_orders(
        &self,
        request: &BithumbTwapOrdersRequest,
    ) -> Result<Page<BithumbTwapOrder>> {
        private::twap_orders(self.http()?, self.credentials()?, request).await
    }

    /// Creates a TWAP order. This submits a financial request to Bithumb.
    pub async fn create_twap_order(&self, request: &BithumbTwapOrderRequest) -> Result<String> {
        private::create_twap_order(self.http()?, self.credentials()?, request).await
    }

    /// Cancels a TWAP order and returns the cancelled identifier.
    pub async fn cancel_twap_order(&self, algo_order_id: &str) -> Result<String> {
        private::cancel_twap_order(self.http()?, self.credentials()?, algo_order_id).await
    }

    /// Submits up to 20 orders and preserves each accepted/rejected outcome.
    ///
    /// This is a financial write. Bithumb processes items independently and
    /// may return HTTP 200 even when one or more items fail.
    pub async fn batch_orders(
        &self,
        request: &BithumbBatchOrdersRequest,
    ) -> Result<BithumbBatchOrdersResult> {
        private::batch_orders(self.http()?, self.credentials()?, request).await
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    /// Returns credentials or an authentication error before network I/O.
    fn credentials(&self) -> Result<&BithumbCredentials> {
        self.credentials
            .as_ref()
            .ok_or_else(|| Error::auth("bithumb needs both an access key and a secret key"))
    }

    pub(crate) fn http(&self) -> Result<&HttpTransport> {
        self.http.as_ref().map_err(Clone::clone)
    }
}

impl Default for BithumbAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for BithumbAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Bithumb
    }

    fn supports(&self, feature: Feature) -> bool {
        if matches!(feature, Feature::TravelRule) {
            return false;
        }
        if feature.is_derivatives_only() {
            return false;
        }
        // Bithumb does not publish a public candle stream.
        if matches!(feature, Feature::CandleStream) {
            return false;
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
        Box::pin(async move { rest::order_book(self.http()?, &market, depth).await })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move { rest::ticker(self.http()?, &market).await })
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
        let subscription = subscription.clone();
        let config = config.clone();
        Box::pin(async move { stream::subscribe(&subscription, &config).await })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let config = config.clone();
        Box::pin(async move { stream::subscribe_account(self.credentials()?, &config).await })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self.http()?, self.credentials()?).await })
    }

    fn order_rules(&self, market: &Market) -> BoxFuture<'_, Result<OrderRules>> {
        let market = market.clone();
        Box::pin(
            async move { private::order_rules(self.http()?, self.credentials()?, &market).await },
        )
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        let asset = asset.to_string();
        Box::pin(
            async move { wallet::asset_networks(self.http()?, self.credentials()?, &asset).await },
        )
    }

    fn deposit_addresses(&self) -> BoxFuture<'_, Result<Vec<DepositAddressEntry>>> {
        Box::pin(async move { wallet::deposit_addresses(self.http()?, self.credentials()?).await })
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::deposit_address(self.http()?, self.credentials()?, &request).await
        })
    }

    fn create_deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::create_deposit_address(self.http()?, self.credentials()?, &request).await
        })
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        let request = request.clone();
        Box::pin(async move {
            wallet::prepare_withdrawal(self.http()?, self.credentials()?, &request).await
        })
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(async move { wallet::withdraw(self.http()?, self.credentials()?, &request).await })
    }

    fn deposit(&self, request: &TransferLookupRequest) -> BoxFuture<'_, Result<Deposit>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposit(self.http()?, self.credentials()?, &request).await })
    }

    fn withdrawal(&self, request: &TransferLookupRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(
            async move { wallet::withdrawal(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn cancel_withdrawal(&self, withdrawal_id: &str) -> BoxFuture<'_, Result<()>> {
        let withdrawal_id = withdrawal_id.to_owned();
        Box::pin(async move {
            wallet::cancel_withdrawal(self.http()?, self.credentials()?, &withdrawal_id).await
        })
    }

    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposits(self.http()?, self.credentials()?, &request).await })
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        let request = request.clone();
        Box::pin(
            async move { wallet::withdrawals(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move {
            private::open_orders(self.http()?, self.credentials()?, market.as_ref()).await
        })
    }

    fn order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::order(self.http()?, self.credentials()?, &market, &order_id).await
        })
    }

    fn order_by_client_id(&self, market: &Market, client_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let client_id = client_id.to_string();
        Box::pin(async move {
            private::order_by_client_id(self.http()?, self.credentials()?, &market, &client_id)
                .await
        })
    }

    fn orders_by_ids(&self, request: &OrderLookupRequest) -> BoxFuture<'_, Result<Vec<Order>>> {
        let request = request.clone();
        Box::pin(async move {
            private::orders_by_ids(self.http()?, self.credentials()?, &request).await
        })
    }

    fn order_history(&self, request: &OrderHistoryRequest) -> BoxFuture<'_, Result<Page<Order>>> {
        let request = request.clone();
        Box::pin(async move {
            private::order_history(self.http()?, self.credentials()?, &request).await
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(
            async move { private::place_order(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::cancel_order(self.http()?, self.credentials()?, &market, &order_id).await
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
                self.http()?,
                self.credentials()?,
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
            private::cancel_orders(self.http()?, self.credentials()?, &request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candles_are_available_over_rest_but_not_as_a_stream() {
        let adapter = BithumbAdapter::new();

        assert!(adapter.supports(Feature::Candles));
        assert!(!adapter.supports(Feature::CandleStream));
    }

    #[test]
    fn every_other_public_stream_is_available() {
        let adapter = BithumbAdapter::new();

        for feature in [
            Feature::TradeStream,
            Feature::OrderBookStream,
            Feature::TickerStream,
        ] {
            assert!(adapter.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn a_spot_exchange_never_claims_derivatives_features() {
        let adapter = BithumbAdapter::new().with_credentials("access", "secret");

        for feature in [Feature::Positions, Feature::Margin, Feature::FundingRates] {
            assert!(!adapter.supports(feature), "{feature:?}");
        }
    }

    #[tokio::test]
    async fn subscribing_to_candles_is_refused_before_a_socket_is_opened() {
        use crate::types::{Feed, Interval, Market, StreamConfig, Subscription};

        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .feed(Feed::Candles(Interval::Min1));

        let error = BithumbAdapter::new()
            .subscribe(&subscription, &StreamConfig::default())
            .await
            .expect_err("bithumb publishes no candle stream");

        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::CandleStream,
                exchange: "bithumb",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn a_private_call_without_credentials_is_an_auth_failure_not_a_missing_feature() {
        let error = BithumbAdapter::new()
            .balances()
            .await
            .expect_err("no credentials were supplied");

        // The feature exists, but the request lacks credentials.
        assert!(
            matches!(error, Error::Auth { .. }),
            "expected an auth failure, got {error:?}"
        );
    }

    #[tokio::test]
    async fn pending_orders_without_credentials_are_rejected_before_network_io() {
        let error = BithumbAdapter::new()
            .pending_orders(&BithumbPendingOrdersRequest::new())
            .await
            .expect_err("no credentials were supplied");

        assert!(matches!(error, Error::Auth { .. }));
    }

    #[tokio::test]
    async fn withdrawal_addresses_without_credentials_are_rejected_before_network_io() {
        let error = BithumbAdapter::new()
            .withdrawal_addresses()
            .await
            .expect_err("no credentials were supplied");

        assert!(matches!(error, Error::Auth { .. }));
    }

    #[tokio::test]
    async fn order_detail_without_credentials_is_rejected_before_network_io() {
        let error = BithumbAdapter::new()
            .order_detail(&BithumbOrderDetailRequest::by_uuid(
                Market::spot(Exchange::Bithumb, "BTC", "KRW"),
                "C0101000000001818113",
            ))
            .await
            .expect_err("no credentials were supplied");

        assert!(matches!(error, Error::Auth { .. }));
    }

    #[tokio::test]
    async fn order_list_without_credentials_is_rejected_before_network_io() {
        let error = BithumbAdapter::new()
            .order_list(&BithumbOrderListRequest::new())
            .await
            .expect_err("no credentials were supplied");

        assert!(matches!(error, Error::Auth { .. }));
    }

    #[tokio::test]
    async fn closed_orders_without_credentials_are_rejected_before_network_io() {
        let error = BithumbAdapter::new()
            .closed_orders(&BithumbClosedOrdersRequest::new())
            .await
            .expect_err("no credentials were supplied");

        assert!(matches!(error, Error::Auth { .. }));
    }

    #[tokio::test]
    async fn batch_order_writes_without_credentials_are_rejected_before_network_io() {
        let request = BithumbBatchOrdersRequest::new(vec![OrderRequest::market(
            Market::spot(Exchange::Bithumb, "BTC", "KRW"),
            Side::Buy,
            crate::types::Size::Quote(crate::Decimal::ONE),
        )]);
        let error = BithumbAdapter::new()
            .batch_orders(&request)
            .await
            .expect_err("no credentials were supplied");

        assert!(matches!(error, Error::Auth { .. }));
    }

    #[test]
    fn credentials_are_what_unlock_the_private_half() {
        let public = BithumbAdapter::new();
        let private = BithumbAdapter::new().with_credentials("access", "secret");

        assert!(!private.supports(Feature::TravelRule));

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
}
