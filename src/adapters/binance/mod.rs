//! Binance Spot and USD-margined perpetual futures.

mod parse;
mod private;
mod rest;
mod stream;
mod wallet;

#[cfg(test)]
mod execution_checklist;

use std::fmt;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};

use futures_core::Stream;
use rust_decimal::Decimal;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{
    CandleRequest, DepositAddressRequest, HistoryRequest, MarginRequest, OrderRequest,
    TransferHistoryRequest, WithdrawRequest,
};
use crate::stream::{AccountStream, MarketStream, TypedStream};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    AssetNetwork, Balance, Candle, Cursor, Deposit, DepositAddress, Exchange, FundingPayment,
    FundingRate, Interval, MarginSummary, Market, MarketInfo, MarketKind, MarketStatus, Order,
    OrderBook, Page, Position, Side, StreamConfig, Subscription, Ticker, Timestamp, Trade,
    Withdrawal, WithdrawalQuote,
};

#[allow(unused_imports)]
pub use private::{
    BinanceAccountTrade, BinanceC2cTrade, BinanceC2cTradeHistoryPage, BinanceListenKey,
    BinanceOrderResponse, BinanceSpotAccountBalance, BinanceSpotAccountInformation,
    BinanceSpotCancelAllOpenOrders, BinanceSpotCancelledOrder, BinanceSpotCommissionRates,
    BinanceSpotOrderDetail, BinanceTestOrder, BinanceUsdMAccountAsset,
    BinanceUsdMAccountInformation, BinanceUsdMAccountPosition, BinanceUsdMPositionInformation,
};
#[allow(unused_imports)]
pub use rest::{BinanceExchangeInfo, BinanceExchangeSymbol, BinanceSymbolFilters};
#[allow(unused_imports)]
pub use wallet::{
    BinanceApiKeyPermissions, BinanceCoinInformation, BinanceCoinNetworkInformation,
    BinanceDepositHistory, BinanceDepositHistoryEntry, BinanceQuestionnaireRequirements,
    BinanceWithdrawHistory, BinanceWithdrawHistoryEntry, BinanceWithdrawalAddress,
};

/// The C2C side filter Binance accepts.
///
/// Binance's C2C history endpoint accepts these two uppercase values only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinanceC2cTradeType {
    /// C2C orders that bought the crypto asset.
    Buy,
    /// C2C orders that sold the crypto asset.
    Sell,
}

impl BinanceC2cTradeType {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

/// Filters one numbered page of Binance C2C order history.
///
/// Leaving both timestamp bounds unset preserves Binance's documented default:
/// the most recent 30 days. When both are set, their inclusive span may be at
/// most 30 days. C2C uses one-based page numbers rather than a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceC2cTradeHistoryRequest {
    /// Required C2C order side filter.
    pub trade_type: BinanceC2cTradeType,
    /// Inclusive lower timestamp bound.
    pub start_timestamp: Option<Timestamp>,
    /// Inclusive upper timestamp bound.
    pub end_timestamp: Option<Timestamp>,
    /// One-based result page; `None` sends Binance's documented default of 1.
    pub page: Option<u32>,
    /// Result rows; `None` sends Binance's documented default of 100.
    pub rows: Option<u32>,
    /// Signed-request receive window in milliseconds, up to 60,000.
    pub recv_window: Option<u64>,
}

impl BinanceC2cTradeHistoryRequest {
    /// Starts a C2C-history request for the required trade side.
    pub fn new(trade_type: BinanceC2cTradeType) -> Self {
        Self {
            trade_type,
            start_timestamp: None,
            end_timestamp: None,
            page: None,
            rows: None,
            recv_window: None,
        }
    }

    /// Sets the inclusive lower timestamp bound.
    #[must_use]
    pub fn start_timestamp(mut self, start_timestamp: Timestamp) -> Self {
        self.start_timestamp = Some(start_timestamp);
        self
    }

    /// Sets the inclusive upper timestamp bound.
    #[must_use]
    pub fn end_timestamp(mut self, end_timestamp: Timestamp) -> Self {
        self.end_timestamp = Some(end_timestamp);
        self
    }

    /// Selects Binance's one-based result page.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Selects the number of C2C orders returned in the page.
    #[must_use]
    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = Some(rows);
        self
    }

    /// Sets the signed-request receive window in milliseconds.
    #[must_use]
    pub fn recv_window(mut self, recv_window: u64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }
}

/// A Binance test-order request.
///
/// This keeps Spot's optional commission-rate calculation out of the common
/// [`OrderRequest`] contract. Binance documents the option only for Spot;
/// requesting it from a USD-M adapter is rejected before a request is sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceTestOrderRequest {
    /// The order to validate.
    pub order: OrderRequest,
    /// Ask Spot to include the commission-rate response object.
    pub compute_commission_rates: bool,
}

impl BinanceTestOrderRequest {
    /// Starts a test-order request without commission-rate calculation.
    pub fn new(order: OrderRequest) -> Self {
        Self {
            order,
            compute_commission_rates: false,
        }
    }

    /// Requests Spot's commission-rate calculation response.
    #[must_use]
    pub fn compute_commission_rates(mut self) -> Self {
        self.compute_commission_rates = true;
        self
    }
}

/// A request for Binance compressed aggregate trades.
///
/// `from_id` selects an inclusive aggregate-trade cursor. `start_time` and
/// `end_time` select inclusive millisecond time bounds. Spot accepts each time
/// bound independently and has no fixed time-window cap. USD-M permits a
/// paired window shorter than one hour and does not allow it with `from_id`.
/// The endpoint returns one page; use the last aggregate ID plus one as the
/// next `from_id` when walking by ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceAggregateTradesRequest {
    /// The Spot or USD-M perpetual market to query.
    pub market: Market,
    /// Inclusive Binance aggregate-trade ID to start at.
    pub from_id: Option<u64>,
    /// Inclusive lower time bound.
    pub start_time: Option<Timestamp>,
    /// Inclusive upper time bound.
    pub end_time: Option<Timestamp>,
    /// Number of aggregate trades, from 1 through 1,000. `None` uses Binance's
    /// documented default of 500.
    pub limit: Option<u32>,
}

impl BinanceAggregateTradesRequest {
    /// Starts a request for one Spot or USD-M perpetual market.
    pub fn new(market: Market) -> Self {
        Self {
            market,
            from_id: None,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }

    /// Starts at an inclusive Binance aggregate-trade ID.
    #[must_use]
    pub fn with_from_id(mut self, from_id: u64) -> Self {
        self.from_id = Some(from_id);
        self
    }

    /// Sets the inclusive lower time bound.
    #[must_use]
    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the inclusive upper time bound.
    #[must_use]
    pub fn end_time(mut self, end_time: Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Sets the page size from 1 through 1,000.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Filters Binance Wallet SAPI deposit-history records.
///
/// All timestamps are sent as Binance millisecond values. Leaving a field
/// unset leaves Binance's documented default in effect; this provider request
/// deliberately does not turn the Wallet endpoint into the common cursor API.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BinanceDepositHistoryRequest {
    /// Restricts records to one asset code.
    pub coin: Option<String>,
    /// Binance's provider-specific deposit status code.
    pub status: Option<u32>,
    /// Inclusive lower creation-time bound.
    pub start_time: Option<Timestamp>,
    /// Inclusive upper creation-time bound.
    pub end_time: Option<Timestamp>,
    /// Zero-based result offset.
    pub offset: Option<u64>,
    /// Result count from 1 through 1,000.
    pub limit: Option<u32>,
    /// Restricts records to one transaction hash.
    pub tx_id: Option<String>,
    /// Requests Binance's source-address field when it is available.
    pub include_source: bool,
}

impl BinanceDepositHistoryRequest {
    /// Starts a request with Binance's default filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restricts the history to one asset.
    #[must_use]
    pub fn coin(mut self, coin: impl Into<String>) -> Self {
        self.coin = Some(coin.into());
        self
    }

    /// Restricts the history to a Binance deposit status code.
    #[must_use]
    pub fn status(mut self, status: u32) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the inclusive lower creation-time bound.
    #[must_use]
    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the inclusive upper creation-time bound.
    #[must_use]
    pub fn end_time(mut self, end_time: Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Sets Binance's zero-based result offset.
    #[must_use]
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the result count from 1 through 1,000.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Restricts the history to one transaction hash.
    #[must_use]
    pub fn tx_id(mut self, tx_id: impl Into<String>) -> Self {
        self.tx_id = Some(tx_id.into());
        self
    }

    /// Requests source-address data when Binance provides it.
    #[must_use]
    pub fn include_source(mut self) -> Self {
        self.include_source = true;
        self
    }
}

/// Filters Binance Wallet SAPI withdrawal-history records.
///
/// All timestamps are sent as Binance millisecond values. The Wallet endpoint
/// has provider-specific status and ID-list semantics, so they remain explicit
/// rather than being collapsed into [`TransferHistoryRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BinanceWithdrawHistoryRequest {
    /// Restricts records to one asset code.
    pub coin: Option<String>,
    /// Restricts records to the caller's withdrawal order identifier.
    pub withdraw_order_id: Option<String>,
    /// Binance's provider-specific withdrawal status code.
    pub status: Option<u32>,
    /// Zero-based result offset.
    pub offset: Option<u64>,
    /// Result count from 1 through 1,000.
    pub limit: Option<u32>,
    /// Binance withdrawal identifiers, sent as the endpoint's comma-separated `idList`.
    pub id_list: Vec<String>,
    /// Inclusive lower apply-time bound.
    pub start_time: Option<Timestamp>,
    /// Inclusive upper apply-time bound.
    pub end_time: Option<Timestamp>,
}

impl BinanceWithdrawHistoryRequest {
    /// Starts a request with Binance's default filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restricts the history to one asset.
    #[must_use]
    pub fn coin(mut self, coin: impl Into<String>) -> Self {
        self.coin = Some(coin.into());
        self
    }

    /// Restricts the history to the caller's withdrawal order identifier.
    #[must_use]
    pub fn withdraw_order_id(mut self, withdraw_order_id: impl Into<String>) -> Self {
        self.withdraw_order_id = Some(withdraw_order_id.into());
        self
    }

    /// Restricts the history to a Binance withdrawal status code.
    #[must_use]
    pub fn status(mut self, status: u32) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets Binance's zero-based result offset.
    #[must_use]
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the result count from 1 through 1,000.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Supplies Binance withdrawal identifiers for its `idList` filter.
    #[must_use]
    pub fn id_list(mut self, id_list: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.id_list = id_list.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the inclusive lower apply-time bound.
    #[must_use]
    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the inclusive upper apply-time bound.
    #[must_use]
    pub fn end_time(mut self, end_time: Timestamp) -> Self {
        self.end_time = Some(end_time);
        self
    }
}

/// One Binance compressed aggregate trade.
///
/// Binance combines market fills that occur within 100 ms at the same price
/// and taking side. `first_trade_id` and `last_trade_id` preserve the covered
/// individual-fill range; this is not interchangeable with [`Trade`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceAggregateTrade {
    /// The Spot or USD-M perpetual market.
    pub market: Market,
    /// Binance's aggregate-trade identifier.
    pub aggregate_id: u64,
    /// The first individual trade ID covered by this aggregate.
    pub first_trade_id: u64,
    /// The last individual trade ID covered by this aggregate.
    pub last_trade_id: u64,
    /// Execution time of the aggregate, in milliseconds at the provider.
    pub timestamp: Timestamp,
    /// Aggregate execution price, in the quote asset.
    pub price: Decimal,
    /// Aggregate quantity, in the base asset. RPI fills may be included on
    /// venues that provide them.
    pub quantity: Decimal,
    /// Quantity excluding RPI fills when Binance provides the optional `nq`
    /// field.
    pub normal_quantity: Option<Decimal>,
    /// Spot's optional best-price-match marker (`M`). USD-M does not publish
    /// this field.
    pub best_price_match: Option<bool>,
    /// The side that took liquidity. Binance's `m` field identifies the maker
    /// as the buyer, so it is inverted here.
    pub taker_side: Side,
    /// Complete provider payload for this aggregate trade.
    pub raw_json: String,
}

/// A full-fidelity Binance public market subscription.
///
/// It keeps the same transport, reconnection, buffering, and close semantics
/// as [`MarketStream`] while yielding Binance's native event details.
pub struct BinanceMarketStream {
    inner: TypedStream<BinanceMarketEvent>,
}

impl BinanceMarketStream {
    pub(crate) fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<BinanceMarketEvent>> + Send + 'static,
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

    /// Stops this subscription and waits for its sockets to close.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl Stream for BinanceMarketStream {
    type Item = Result<BinanceMarketEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl fmt::Debug for BinanceMarketStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinanceMarketStream")
            .finish_non_exhaustive()
    }
}

/// A full-fidelity Binance private account subscription.
///
/// It keeps the same authentication, listen-key refresh, reconnection,
/// buffering, and close behavior as [`AccountStream`] while retaining events
/// outside the portable account model. Construct it with
/// [`BinanceAdapter::subscribe_detailed_account`].
pub struct BinanceAccountStream {
    inner: TypedStream<BinanceAccountStreamEvent>,
}

impl BinanceAccountStream {
    pub(crate) fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<BinanceAccountStreamEvent>> + Send + 'static,
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

    /// Stops this subscription and waits for its socket and key refresher to close.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl Stream for BinanceAccountStream {
    type Item = Result<BinanceAccountStreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl fmt::Debug for BinanceAccountStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinanceAccountStream")
            .finish_non_exhaustive()
    }
}

/// One Binance-specific account-stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BinanceAccountStreamEvent {
    /// One portable balance projection and its provider envelope.
    Balance(BinanceBalanceStreamEvent),
    /// One portable order projection and its provider envelope.
    Order(BinanceOrderStreamEvent),
    /// A Binance account event with no portable equivalent.
    Other(BinanceRawAccountEvent),
    /// The underlying WebSocket reconnected.
    Reconnected,
}

/// A Binance account balance event with provider metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceBalanceStreamEvent {
    /// Portable balance projection.
    pub common: Balance,
    /// Binance event name, such as `outboundAccountPosition`.
    pub event_type: String,
    /// Provider event time, when present.
    pub event_time: Option<Timestamp>,
    /// Provider transaction time, when present.
    pub transaction_time: Option<Timestamp>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A Binance account order event with provider metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceOrderStreamEvent {
    /// Portable order projection.
    pub common: Order,
    /// Binance event name, such as `executionReport`.
    pub event_type: String,
    /// Provider event time, when present.
    pub event_time: Option<Timestamp>,
    /// Provider transaction time, when present.
    pub transaction_time: Option<Timestamp>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A Binance account event that has no portable account-event projection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceRawAccountEvent {
    /// Binance event name.
    pub event_type: String,
    /// Provider event time, when present.
    pub event_time: Option<Timestamp>,
    /// Provider transaction time, when present.
    pub transaction_time: Option<Timestamp>,
    /// Complete provider frame encoded as JSON.
    pub raw_json: String,
}

/// A native Binance public market event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BinanceMarketEvent {
    /// One individual Spot trade.
    Trade(BinanceTradeEvent),
    /// One partial-depth update or snapshot.
    OrderBook(BinanceOrderBookEvent),
    /// One 24-hour ticker update.
    Ticker(BinanceTickerEvent),
    /// One candlestick update.
    Candle(BinanceCandleEvent),
    /// One underlying socket reconnected.
    Reconnected,
}

/// A Binance individual Spot trade together with native stream fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceTradeEvent {
    /// Portable trade projection used by the common stream API.
    pub common: Trade,
    /// Binance's event timestamp.
    pub event_time: Option<Timestamp>,
    /// Binance's execution timestamp.
    pub trade_time: Option<Timestamp>,
    /// Whether Binance identifies the buyer as the maker.
    pub buyer_is_maker: Option<bool>,
    /// Spot's best-price-match marker.
    pub best_price_match: Option<bool>,
    /// Complete provider payload for this event.
    pub raw_json: String,
}

/// A Binance depth event together with native update sequencing fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceOrderBookEvent {
    /// Portable order-book projection used by the common stream API.
    pub common: OrderBook,
    /// Binance's event timestamp, when emitted by the venue.
    pub event_time: Option<Timestamp>,
    /// Binance's transaction timestamp, when emitted by the venue.
    pub transaction_time: Option<Timestamp>,
    /// First update identifier in a diff-depth range.
    pub first_update_id: Option<u64>,
    /// Final update identifier in a diff-depth range.
    pub final_update_id: Option<u64>,
    /// Previous final update identifier in a diff-depth range.
    pub previous_final_update_id: Option<u64>,
    /// Spot partial-depth snapshot identifier.
    pub last_update_id: Option<u64>,
    /// Complete provider payload for this event.
    pub raw_json: String,
}

/// A Binance 24-hour ticker update with native metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceTickerEvent {
    /// Portable ticker projection used by the common stream API.
    pub common: Ticker,
    /// Binance's event timestamp.
    pub event_time: Option<Timestamp>,
    /// Close timestamp of the rolling window.
    pub close_time: Option<Timestamp>,
    /// First trade identifier in the rolling window.
    pub first_trade_id: Option<u64>,
    /// Last trade identifier in the rolling window.
    pub last_trade_id: Option<u64>,
    /// Number of trades in the rolling window.
    pub trade_count: Option<u64>,
    /// Complete provider payload for this event.
    pub raw_json: String,
}

/// A Binance candlestick update with native aggregation fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceCandleEvent {
    /// Portable candle projection used by the common stream API.
    pub common: Candle,
    /// Inclusive close timestamp of the candle window.
    pub close_time: Option<Timestamp>,
    /// First trade identifier in the candle.
    pub first_trade_id: Option<u64>,
    /// Last trade identifier in the candle.
    pub last_trade_id: Option<u64>,
    /// Number of trades in the candle.
    pub trade_count: Option<u64>,
    /// Taker-buy base volume, when Binance publishes it.
    pub taker_buy_base_volume: Option<Decimal>,
    /// Taker-buy quote volume, when Binance publishes it.
    pub taker_buy_quote_volume: Option<Decimal>,
    /// Complete provider payload for this event.
    pub raw_json: String,
}

/// Binance Spot's current average price for one market.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotAveragePrice {
    /// The Spot market this snapshot describes.
    pub market: Market,
    /// Binance's averaging interval, in minutes.
    pub minutes: u32,
    /// The average price, in the quote asset.
    pub price: Decimal,
    /// The last trade time included in the average.
    pub close_time: Timestamp,
}

/// Binance USD-M's current mark-price snapshot for one perpetual market.
///
/// Binance returns the funding and index context alongside the mark price from
/// `GET /fapi/v1/premiumIndex`. The values are provider-specific and therefore
/// remain on [`BinanceAdapter`] until the common API has a matching contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceMarkPrice {
    /// The perpetual market this snapshot describes.
    pub market: Market,
    /// Binance's liquidation-reference mark price, in the quote asset.
    pub mark_price: Decimal,
    /// The underlying index price, in the quote asset.
    pub index_price: Decimal,
    /// Estimated settlement price, when Binance publishes a meaningful value.
    pub estimated_settle_price: Option<Decimal>,
    /// The latest funding rate as a signed ratio.
    pub last_funding_rate: Decimal,
    /// Binance's fixed interest rate component as a signed ratio.
    pub interest_rate: Decimal,
    /// When the next funding payment is scheduled.
    pub next_funding_time: Timestamp,
    /// When Binance produced this snapshot.
    pub time: Timestamp,
}

/// Binance USD-M's current open interest for one perpetual market.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinanceOpenInterest {
    /// The perpetual market this snapshot describes.
    pub market: Market,
    /// Open contracts, denominated in the base asset.
    pub open_interest: Decimal,
    /// When Binance recorded the open-interest value.
    pub time: Timestamp,
}

pub(crate) const SPOT_REST_BASE_URL: &str = "https://api.binance.com";
/// Wallet SAPI is account-wide and always lives on the Spot API host.
pub(crate) const WALLET_REST_BASE_URL: &str = SPOT_REST_BASE_URL;
pub(crate) const SPOT_WEBSOCKET_URL: &str = "wss://stream.binance.com:9443/stream";
/// The Spot WebSocket API used for signed account subscriptions.
pub(crate) const SPOT_WEBSOCKET_API_URL: &str = "wss://ws-api.binance.com:443/ws-api/v3";
pub(crate) const USD_M_REST_BASE_URL: &str = "https://fapi.binance.com";
/// The USD-M entry point for order books.
pub(crate) const USD_M_PUBLIC_WEBSOCKET_URL: &str = "wss://fstream.binance.com/public/stream";
/// The USD-M entry point for regular market feeds.
pub(crate) const USD_M_MARKET_WEBSOCKET_URL: &str = "wss://fstream.binance.com/market/stream";

/// The header every authenticated Binance request carries its API key in.
pub(crate) const API_KEY_HEADER: &str = "X-MBX-APIKEY";

/// The name `maxt` reports Binance under in errors and market identities.
pub(crate) const EXCHANGE: &str = Exchange::Binance.id();

/// Which Binance venue an adapter talks to.
///
/// Spot and USD-M futures are separate APIs with separate hosts, separate
/// balances, and separate listings. One adapter talks to one of them, chosen at
/// construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum BinanceMarket {
    /// Binance Spot. The default.
    #[default]
    Spot,
    /// Binance USD-margined perpetual futures.
    UsdMFutures,
}

impl BinanceMarket {
    /// The kind of instrument this venue lists.
    ///
    /// Every market handed to the adapter is checked against it, so a spot
    /// market never reaches the futures host by accident. Binance lists
    /// `BTCUSDT` on both, at different prices.
    pub(crate) const fn market_kind(self) -> MarketKind {
        match self {
            Self::Spot => MarketKind::Spot,
            Self::UsdMFutures => MarketKind::Perpetual,
        }
    }

    pub(crate) const fn rest_base_url(self) -> &'static str {
        match self {
            Self::Spot => SPOT_REST_BASE_URL,
            Self::UsdMFutures => USD_M_REST_BASE_URL,
        }
    }

    /// Binance's own name for a candle interval on this venue.
    ///
    /// One-second candles exist on spot only; USD-M futures starts at one
    /// minute, which is the one interval the two venues disagree about.
    pub(crate) fn interval_code(self, interval: Interval) -> Result<&'static str> {
        Ok(match (self, interval) {
            (Self::UsdMFutures, Interval::Sec1) => {
                return Err(Error::unsupported(
                    Feature::Candles,
                    EXCHANGE,
                    "USD-M futures publishes no one-second candles; one minute is the shortest",
                ));
            }
            (_, Interval::Sec1) => "1s",
            (_, Interval::Min1) => "1m",
            (_, Interval::Min3) => "3m",
            (_, Interval::Min5) => "5m",
            (_, Interval::Min10) => {
                return Err(Error::unsupported(
                    Feature::Candles,
                    EXCHANGE,
                    "Binance publishes no ten-minute candles",
                ));
            }
            (_, Interval::Min15) => "15m",
            (_, Interval::Min30) => "30m",
            (_, Interval::Hour1) => "1h",
            (_, Interval::Hour2) => "2h",
            (_, Interval::Hour4) => "4h",
            (_, Interval::Hour6) => "6h",
            (_, Interval::Hour8) => "8h",
            (_, Interval::Hour12) => "12h",
            (_, Interval::Day1) => "1d",
            (_, Interval::Day3) => "3d",
            (_, Interval::Week1) => "1w",
            (_, Interval::Month1) => "1M",
        })
    }
}

/// Binance Spot or USD-M perpetual futures.
///
/// Select one venue with [`Self::spot`] or [`Self::usd_m_futures`]. One adapter
/// uses that venue's hosts and market kind for its lifetime. Spot margin is a
/// separate Binance product and is not exposed here.
#[derive(Debug, Clone)]
pub struct BinanceAdapter {
    venue: BinanceMarket,
    credentials: Option<BinanceCredentials>,
    /// Built on first use so the constructors stay infallible, and shared from
    /// then on so connections are reused across calls.
    http: OnceLock<HttpTransport>,
    /// Wallet SAPI never follows the selected trading venue to `fapi.binance.com`.
    wallet_http: OnceLock<HttpTransport>,
}

#[derive(Debug, Clone)]
pub(crate) struct BinanceCredentials {
    pub(crate) api_key: String,
    pub(crate) secret_key: String,
}

impl BinanceAdapter {
    /// An adapter for public Binance Spot market data.
    pub fn spot() -> Self {
        Self::for_venue(BinanceMarket::Spot)
    }

    /// An adapter for public Binance USD-M futures market data.
    pub fn usd_m_futures() -> Self {
        Self::for_venue(BinanceMarket::UsdMFutures)
    }

    fn for_venue(venue: BinanceMarket) -> Self {
        Self {
            venue,
            credentials: None,
            http: OnceLock::new(),
            wallet_http: OnceLock::new(),
        }
    }

    /// Adds the HMAC-SHA-256 API key and secret used by private calls.
    ///
    /// RSA and Ed25519 credentials are not supported. Binance also enforces the
    /// key's venue and permissions.
    #[must_use]
    pub fn with_credentials(
        mut self,
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(BinanceCredentials {
            api_key: api_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Which venue this adapter talks to.
    pub fn venue(&self) -> BinanceMarket {
        self.venue
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    pub(crate) fn credentials(&self) -> Result<&BinanceCredentials> {
        self.credentials
            .as_ref()
            .ok_or_else(|| Error::auth("binance needs both an API key and a secret key"))
    }

    /// The transport for this venue's REST host.
    pub(crate) fn http(&self) -> Result<&HttpTransport> {
        if let Some(transport) = self.http.get() {
            return Ok(transport);
        }
        let transport = HttpTransport::new(self.venue.rest_base_url())?;
        Ok(self.http.get_or_init(|| transport))
    }

    /// The account-wide Wallet SAPI transport.
    pub(crate) fn wallet_http(&self) -> Result<&HttpTransport> {
        if let Some(transport) = self.wallet_http.get() {
            return Ok(transport);
        }
        let transport = HttpTransport::new(WALLET_REST_BASE_URL)?;
        Ok(self.wallet_http.get_or_init(|| transport))
    }

    /// Sends a request and returns the body, or Binance's own verdict.
    pub(crate) async fn send(&self, request: HttpRequest) -> Result<String> {
        let response = self.http()?.send(&request).await?;
        if response.is_success() {
            return Ok(response.body);
        }
        Err(exchange_error(response.status, &response.body))
    }

    /// Sends one Wallet SAPI request without automatic retry.
    pub(crate) async fn send_wallet(&self, request: HttpRequest) -> Result<String> {
        let response = self.wallet_http()?.send(&request).await?;
        if response.is_success() {
            return Ok(response.body);
        }
        Err(exchange_error(response.status, &response.body))
    }

    /// The Binance symbol for a market, after checking it belongs here.
    ///
    /// `BTCUSDT` alone does not say which venue it came from, so the market's
    /// kind is checked against the adapter's venue before the symbol is built.
    pub(crate) fn symbol(&self, market: &Market) -> Result<String> {
        if market.exchange != Exchange::Binance {
            return Err(Error::invalid_request(
                "market",
                format!("{market} is not a Binance market"),
            ));
        }
        if market.kind != self.venue.market_kind() {
            return Err(Error::invalid_request(
                "market",
                format!(
                    "this adapter trades {:?} markets; {market} is {:?}",
                    self.venue.market_kind(),
                    market.kind
                ),
            ));
        }
        check_asset("base", &market.base)?;
        check_asset("quote", &market.quote)?;
        Ok(format!("{}{}", market.base, market.quote))
    }

    /// Resolves a symbol-only REST or account payload on this adapter's venue.
    ///
    /// Public market streams instead use the markets supplied at subscription
    /// time, because a concatenated symbol does not uniquely identify its
    /// base and quote assets.
    pub(crate) fn market(&self, symbol: &str) -> Result<Market> {
        let (base, quote) = split_symbol(symbol).ok_or_else(|| {
            Error::decode(format!(
                "`{symbol}` does not end in an asset Binance quotes in"
            ))
        })?;
        Ok(Market::new(
            Exchange::Binance,
            self.venue.market_kind(),
            base,
            quote,
        ))
    }

    /// Reads Binance's price, quantity, and notional filters for one Spot market.
    ///
    /// Returns [`Error::Unsupported`] on a USD-M adapter.
    pub async fn spot_symbol_filters(&self, market: &Market) -> Result<BinanceSymbolFilters> {
        rest::spot_symbol_filters(self, market).await
    }

    /// Looks up a Spot order by Binance's numeric order id.
    ///
    /// Includes filled and cancelled orders. Returns [`Error::Unsupported`] on
    /// a USD-M adapter.
    pub async fn spot_order(
        &self,
        market: &Market,
        order_id: &str,
    ) -> Result<BinanceSpotOrderDetail> {
        private::spot_order(self, market, order_id).await
    }

    /// Creates one Binance order and returns its provider-specific response.
    ///
    /// [`Adapter::place_order`] keeps the portable order projection. Use this
    /// method when the Binance-only execution, position, or order-list fields
    /// from the create response are needed.
    pub async fn place_order_detail(&self, request: &OrderRequest) -> Result<BinanceOrderResponse> {
        private::place_order_detail(self, request).await
    }

    /// Cancels one Binance order by exchange identifier and returns the full
    /// provider response.
    pub async fn cancel_order_detail(
        &self,
        market: &Market,
        order_id: &str,
    ) -> Result<BinanceOrderResponse> {
        private::cancel_order_detail(self, market, order_id).await
    }

    /// Cancels one Binance order by client identifier and returns the full
    /// provider response.
    pub async fn cancel_order_by_client_id_detail(
        &self,
        market: &Market,
        client_id: &str,
    ) -> Result<BinanceOrderResponse> {
        private::cancel_order_by_client_id_detail(self, market, client_id).await
    }

    /// Reads one page of signed account trades for Spot or USD-M.
    ///
    /// Binance supplies an ID-based continuation, but this provider method
    /// does not manufacture a cursor until it can carry that ID safely. A
    /// full page therefore has no `next` cursor; use the returned trade ID to
    /// make a provider-specific follow-up when that API is added.
    pub async fn account_trades(
        &self,
        request: &HistoryRequest,
    ) -> Result<Page<BinanceAccountTrade>> {
        private::account_trades(self, request).await
    }

    /// Reads one numbered page of the configured account's C2C order history.
    ///
    /// This is a Spot/Funding Wallet SAPI operation. Binance returns its own
    /// `code`, `message`, `data`, `total`, and `success` envelope, so this
    /// provider method preserves that envelope instead of inventing a common
    /// cursor. Increment [`BinanceC2cTradeHistoryRequest::page`] while the
    /// requested page and row count have not covered `total`.
    pub async fn c2c_trade_history(
        &self,
        request: &BinanceC2cTradeHistoryRequest,
    ) -> Result<BinanceC2cTradeHistoryPage> {
        private::c2c_trade_history(self, request).await
    }

    /// Validates an order without sending it to Binance's matching engine.
    ///
    /// Spot ordinarily returns `{}`; requesting its commission-rate calculation
    /// instead returns a commission object. USD-M returns an order-shaped object.
    /// The canonical JSON response preserves every successful object shape.
    pub async fn test_order(&self, request: &BinanceTestOrderRequest) -> Result<BinanceTestOrder> {
        private::test_order(self, request).await
    }

    /// Cancels every open order for one market in one provider call.
    ///
    /// Spot returns cancellation reports while USD-M returns an acknowledgement,
    /// so the common result is deliberately a successful unit result rather
    /// than an incomplete order list.
    pub async fn cancel_all_open_orders(&self, market: &Market) -> Result<()> {
        private::cancel_all_open_orders(self, market).await
    }

    /// Reads Spot's signed account-information response without discarding
    /// commissions, permissions, or provider-only account flags.
    pub async fn spot_account_information(&self) -> Result<BinanceSpotAccountInformation> {
        private::spot_account_information(self).await
    }

    /// Cancels every Spot open order for one market and preserves Binance's
    /// cancellation reports.
    pub async fn spot_cancel_all_open_orders(
        &self,
        market: &Market,
    ) -> Result<BinanceSpotCancelAllOpenOrders> {
        private::spot_cancel_all_open_orders(self, market).await
    }

    /// Reads Spot's complete exchange-information listing, including symbols
    /// and provider-specific raw filter data.
    pub async fn spot_exchange_info(&self) -> Result<BinanceExchangeInfo> {
        rest::exchange_info(self, BinanceMarket::Spot).await
    }

    /// Reads USD-M account information, including the asset and position
    /// figures that do not fit the common balance or margin contracts.
    pub async fn usd_m_account_information(&self) -> Result<BinanceUsdMAccountInformation> {
        private::usd_m_account_information(self).await
    }

    /// Reads USD-M's complete exchange-information listing, without dropping
    /// dated contracts from Binance's provider response.
    pub async fn usd_m_exchange_info(&self) -> Result<BinanceExchangeInfo> {
        rest::exchange_info(self, BinanceMarket::UsdMFutures).await
    }

    /// Reads USD-M position-risk information with Binance's margin and risk
    /// fields preserved for every returned position.
    pub async fn usd_m_position_information(
        &self,
        market: Option<&Market>,
    ) -> Result<Vec<BinanceUsdMPositionInformation>> {
        private::usd_m_position_information(self, market).await
    }

    /// Reads all Wallet SAPI coin and network configuration entries.
    pub async fn all_coins_information(&self) -> Result<Vec<BinanceCoinInformation>> {
        wallet::all_coins_information(self).await
    }

    /// Reads the configured API key's Wallet SAPI permission flags.
    pub async fn api_key_permissions(&self) -> Result<BinanceApiKeyPermissions> {
        wallet::api_key_permissions(self).await
    }

    /// Reads Wallet SAPI deposit history without reducing provider status,
    /// pagination, or time fields to the common transfer-history model.
    pub async fn deposit_history(
        &self,
        request: &BinanceDepositHistoryRequest,
    ) -> Result<BinanceDepositHistory> {
        wallet::deposit_history(self, request).await
    }

    /// Reads the Wallet SAPI Travel Rule questionnaire requirement.
    pub async fn questionnaire_requirements(&self) -> Result<BinanceQuestionnaireRequirements> {
        wallet::questionnaire_requirements(self).await
    }

    /// Reads registered Wallet SAPI withdrawal addresses and their whitelist state.
    pub async fn withdraw_address_list(&self) -> Result<Vec<BinanceWithdrawalAddress>> {
        wallet::withdraw_address_list(self).await
    }

    /// Reads Wallet SAPI withdrawal history without reducing provider status,
    /// pagination, or time fields to the common transfer-history model.
    pub async fn withdraw_history(
        &self,
        request: &BinanceWithdrawHistoryRequest,
    ) -> Result<BinanceWithdrawHistory> {
        wallet::withdraw_history(self, request).await
    }

    /// Reads Binance Spot's current average price for one market.
    ///
    /// This public endpoint needs no credentials. A USD-M adapter or a
    /// non-Spot market is rejected before a request is sent.
    pub async fn spot_average_price(&self, market: &Market) -> Result<BinanceSpotAveragePrice> {
        rest::spot_average_price(self, market).await
    }

    /// Reads the current USD-M mark price and funding context for one market.
    pub async fn mark_price(&self, market: &Market) -> Result<BinanceMarkPrice> {
        rest::mark_price(self, market).await
    }

    /// Reads current USD-M mark prices for every listed perpetual market.
    ///
    /// This keeps Binance's symbol-omitted `/fapi/v1/premiumIndex` response
    /// visible instead of silently discarding the array form.
    pub async fn mark_prices(&self) -> Result<Vec<BinanceMarkPrice>> {
        rest::mark_prices(self).await
    }

    /// Reads the current USD-M open interest for one market.
    pub async fn open_interest(&self, market: &Market) -> Result<BinanceOpenInterest> {
        rest::open_interest(self, market).await
    }

    /// Reads one page of Spot or USD-M compressed aggregate trades.
    ///
    /// This is Binance's 100 ms/same-price/same-taking-side aggregation, not a
    /// list of individual fills. The endpoint is public and needs no key.
    pub async fn aggregate_trades(
        &self,
        request: &BinanceAggregateTradesRequest,
    ) -> Result<Vec<BinanceAggregateTrade>> {
        rest::aggregate_trades(self, request).await
    }

    /// Subscribes to Binance market data without discarding native stream
    /// fields such as update ranges, trade counts, and event timestamps.
    pub async fn subscribe_detailed(
        &self,
        subscription: &Subscription,
    ) -> Result<BinanceMarketStream> {
        self.subscribe_detailed_with(subscription, &crate::client::default_stream_config())
            .await
    }

    /// Subscribes to detailed Binance market data with explicit reconnect and
    /// buffering policy.
    pub async fn subscribe_detailed_with(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> Result<BinanceMarketStream> {
        stream::subscribe_detailed(self, subscription, config).await
    }

    /// Subscribes to Binance account updates without discarding native events.
    pub async fn subscribe_detailed_account(&self) -> Result<BinanceAccountStream> {
        self.subscribe_detailed_account_with(&crate::client::default_stream_config())
            .await
    }

    /// Subscribes to detailed Binance account updates with explicit stream settings.
    pub async fn subscribe_detailed_account_with(
        &self,
        config: &StreamConfig,
    ) -> Result<BinanceAccountStream> {
        stream::subscribe_detailed_account(self, config).await
    }

    /// Creates or extends the account's USD-M user-data listen key.
    ///
    /// [`Client::subscribe_account`](crate::Client::subscribe_account) manages
    /// this lifecycle when it owns the socket.
    pub async fn usd_m_create_listen_key(&self) -> Result<BinanceListenKey> {
        self.check_usd_m("listen keys")?;
        private::create_listen_key(self).await
    }

    /// Extends the USD-M listen key owned by the configured API key.
    ///
    /// Binance's endpoint extends the active key owned by the configured API
    /// key and accepts no listen-key parameter.
    pub async fn usd_m_keepalive_listen_key(&self) -> Result<()> {
        self.check_usd_m("listen keys")?;
        private::keepalive_listen_key(self).await
    }

    /// Closes the active USD-M listen key owned by the configured API key.
    pub async fn usd_m_close_listen_key(&self) -> Result<()> {
        self.check_usd_m("listen keys")?;
        private::close_listen_key(self).await
    }

    fn check_usd_m(&self, what: &str) -> Result<()> {
        if self.venue == BinanceMarket::UsdMFutures {
            return Ok(());
        }
        Err(Error::unsupported(
            Feature::AccountStream,
            EXCHANGE,
            format!("these {what} are the USD-M ones; build the adapter with `usd_m_futures`"),
        ))
    }
}

impl Default for BinanceAdapter {
    fn default() -> Self {
        Self::spot()
    }
}

/// Rejects an asset code that could disguise query or stream syntax.
///
/// Binance uses uppercase ASCII letters and digits as well as UTF-8 letter and
/// number names. ASCII punctuation remains invalid; REST percent-encodes the
/// accepted UTF-8 bytes and WebSocket subscriptions serialize them as JSON.
fn check_asset(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::invalid_request(field, "must not be empty"));
    }
    if !value.chars().all(|character| {
        character.is_ascii_uppercase()
            || character.is_ascii_digit()
            || (!character.is_ascii() && character.is_alphanumeric())
    }) {
        return Err(Error::invalid_request(
            field,
            format!(
                "`{value}` is not a Binance asset code: expected uppercase ASCII or UTF-8 letters and digits"
            ),
        ));
    }
    Ok(())
}

/// The assets Binance prices markets in.
///
/// Membership is what matters, not order: [`split_symbol`] takes the longest
/// matching suffix, and two different codes of the same length cannot both end
/// the same symbol.
const QUOTE_ASSETS: &[&str] = &[
    "FDUSD", "USDT", "USDC", "USD1", "BUSD", "TUSD", "USDP", "AEUR", "BIDR", "IDRT", "DOGE", "BTC",
    "ETH", "BNB", "XRP", "SOL", "TRX", "DAI", "TRY", "EUR", "GBP", "BRL", "ARS", "JPY", "MXN",
    "ZAR", "COP", "CZK", "PLN", "RON", "UAH", "NGN", "RUB", "AUD", "VAI", "PAX",
];

/// Splits a symbol-only private payload on the longest known quote suffix.
///
/// Listings use Binance's explicit `baseAsset` and `quoteAsset`, and public
/// streams use their subscription mapping. `None` means no known suffix fits.
fn split_symbol(symbol: &str) -> Option<(&str, &str)> {
    QUOTE_ASSETS
        .iter()
        .filter_map(|quote| {
            let base = symbol.strip_suffix(*quote)?;
            (!base.is_empty()).then_some((base, *quote))
        })
        .max_by_key(|(_, quote)| quote.len())
}

/// Turns a non-2xx response into Binance's own verdict.
///
/// Binance answers every failure with `{"code": -1121, "msg": "Invalid
/// symbol."}`, rate limits included. The numeric code is the part worth
/// branching on and is kept verbatim. The HTTP status carries the retry
/// classification, which lands 429 and Binance's 418 IP ban on
/// [`ExchangeErrorKind::RateLimited`](crate::ExchangeErrorKind::RateLimited).
pub(crate) fn exchange_error(status: u16, body: &str) -> Error {
    #[derive(serde::Deserialize)]
    struct RawError {
        code: i64,
        msg: String,
    }

    match serde_json::from_str::<RawError>(body) {
        Ok(raw) => Error::exchange_http(EXCHANGE, status, raw.code.to_string(), raw.msg),
        // A body that is not the error envelope is still a failure; report the
        // status and whatever arrived rather than inventing a code.
        Err(_) => Error::exchange_http(EXCHANGE, status, "unknown", body.trim()),
    }
}

/// Reads a market status out of an `exchangeInfo` listing.
pub(crate) fn market_status(raw: &str) -> MarketStatus {
    match raw {
        "TRADING" => MarketStatus::Active,
        // Halted but still listed: orders are rejected and the pair comes back.
        "BREAK" | "HALT" | "PENDING_TRADING" | "PRE_TRADING" | "POST_TRADING" | "AUCTION_MATCH" => {
            MarketStatus::Paused
        }
        "DELISTED" | "CLOSE" | "SETTLING" => MarketStatus::Delisted,
        _ => MarketStatus::Unknown,
    }
}

/// The current time in Binance's unit, for signing and for candle closing.
pub(crate) fn now_millis() -> i64 {
    Timestamp::now().as_millis()
}

/// Encodes one position in a paginated history.
///
/// Both of Binance's paginated histories are windowed by `startTime`, so a
/// cursor is one millisecond timestamp. It is tagged so a cursor from another
/// exchange, or from a later change of scheme, is refused instead of misread.
pub(crate) fn encode_cursor(resume_from_millis: i64) -> Cursor {
    Cursor(format!("t{resume_from_millis}"))
}

/// Reads back a cursor from [`encode_cursor`].
pub(crate) fn decode_cursor(cursor: &Cursor) -> Result<i64> {
    cursor
        .as_str()
        .strip_prefix('t')
        .and_then(|millis| millis.parse().ok())
        .ok_or_else(|| Error::invalid_request("cursor", "not a cursor this exchange produced"))
}

impl Adapter for BinanceAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Binance
    }

    fn supports(&self, feature: Feature) -> bool {
        if matches!(
            feature,
            Feature::OrderHistory
                | Feature::DepositLookup
                | Feature::TravelRule
                | Feature::WithdrawalLookup
                | Feature::WithdrawalCancellation
        ) {
            return false;
        }
        if wallet::is_wallet_feature(feature) && self.venue != BinanceMarket::Spot {
            return false;
        }
        if feature.is_derivatives_only() && self.venue == BinanceMarket::Spot {
            return false;
        }
        if feature.needs_credentials() {
            return self.is_authenticated();
        }
        true
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move { rest::markets(self, kind).await })
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let market = market.clone();
        Box::pin(async move { rest::trades(self, &market, limit).await })
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let market = market.clone();
        Box::pin(async move { rest::order_book(self, &market, depth).await })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move { rest::ticker(self, &market).await })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let request = request.clone();
        Box::pin(async move { rest::candles(self, &request).await })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        let subscription = subscription.clone();
        let config = config.clone();
        Box::pin(async move { stream::subscribe(self, &subscription, &config).await })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let config = config.clone();
        Box::pin(async move { stream::subscribe_account(self, &config).await })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self).await })
    }

    fn asset_networks(&self, asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
        let asset = asset.to_string();
        Box::pin(async move { wallet::asset_networks(self, &asset).await })
    }

    fn deposit_address(
        &self,
        request: &DepositAddressRequest,
    ) -> BoxFuture<'_, Result<DepositAddress>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposit_address(self, &request).await })
    }

    fn prepare_withdrawal(
        &self,
        request: &WithdrawRequest,
    ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
        let request = request.clone();
        Box::pin(async move { wallet::prepare_withdrawal(self, &request).await })
    }

    fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
        let request = request.clone();
        Box::pin(async move { wallet::withdraw(self, &request).await })
    }

    fn deposits(&self, request: &TransferHistoryRequest) -> BoxFuture<'_, Result<Page<Deposit>>> {
        let request = request.clone();
        Box::pin(async move { wallet::deposits(self, &request).await })
    }

    fn withdrawals(
        &self,
        request: &TransferHistoryRequest,
    ) -> BoxFuture<'_, Result<Page<Withdrawal>>> {
        let request = request.clone();
        Box::pin(async move { wallet::withdrawals(self, &request).await })
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move { private::open_orders(self, market.as_ref()).await })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(async move { private::place_order(self, &request).await })
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move { private::cancel_order(self, &market, &order_id).await })
    }

    fn cancel_order_by_client_id(
        &self,
        market: &Market,
        client_id: &str,
    ) -> BoxFuture<'_, Result<()>> {
        let market = market.clone();
        let client_id = client_id.to_string();
        Box::pin(async move { private::cancel_order_by_client_id(self, &market, &client_id).await })
    }

    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        let market = market.cloned();
        Box::pin(async move { private::positions(self, market.as_ref()).await })
    }

    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        Box::pin(async move { private::margin_summary(self).await })
    }

    fn funding_rates(&self, request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        let request = request.clone();
        Box::pin(async move { private::funding_rates(self, &request).await })
    }

    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        let request = request.clone();
        Box::pin(async move { private::funding_payments(self, &request).await })
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        let request = request.clone();
        Box::pin(async move { private::set_margin(self, &request).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExchangeErrorKind;

    #[test]
    fn spot_has_no_derivatives_features_and_futures_has_all_of_them() {
        let spot = BinanceAdapter::spot().with_credentials("key", "secret");
        let futures = BinanceAdapter::usd_m_futures().with_credentials("key", "secret");

        for feature in [
            Feature::Positions,
            Feature::Margin,
            Feature::FundingRates,
            Feature::FundingPayments,
            Feature::MarginConfig,
            Feature::ReduceOnlyOrders,
        ] {
            assert!(!spot.supports(feature), "spot {feature:?}");
            assert!(futures.supports(feature), "futures {feature:?}");
        }
    }

    #[test]
    fn both_venues_serve_public_market_data_without_credentials() {
        for adapter in [BinanceAdapter::spot(), BinanceAdapter::usd_m_futures()] {
            for feature in [
                Feature::Markets,
                Feature::Trades,
                Feature::OrderBook,
                Feature::Ticker,
                Feature::Candles,
                Feature::CandleStream,
            ] {
                assert!(
                    adapter.supports(feature),
                    "{:?} {feature:?}",
                    adapter.venue()
                );
            }
        }
    }

    #[test]
    fn both_venues_read_balances_and_open_orders_once_credentials_are_set() {
        // Spot answers these on `/api/v3/account` and `/api/v3/openOrders`,
        // USD-M on `/fapi/v3/account` and `/fapi/v1/openOrders`. Both venues
        // answer both questions, so `supports` claims both on both.
        for adapter in [
            BinanceAdapter::spot().with_credentials("key", "secret"),
            BinanceAdapter::usd_m_futures().with_credentials("key", "secret"),
        ] {
            assert!(adapter.supports(Feature::Balances), "{:?}", adapter.venue());
            assert!(
                adapter.supports(Feature::OpenOrders),
                "{:?}",
                adapter.venue()
            );
        }
    }

    #[test]
    fn funding_rates_are_public_but_funding_payments_are_not() {
        let public = BinanceAdapter::usd_m_futures();

        assert!(public.supports(Feature::FundingRates));
        assert!(!public.supports(Feature::FundingPayments));
    }

    #[test]
    fn the_two_venues_are_separate_hosts() {
        assert_ne!(SPOT_REST_BASE_URL, USD_M_REST_BASE_URL);
        assert_ne!(SPOT_WEBSOCKET_URL, USD_M_PUBLIC_WEBSOCKET_URL);
        assert_ne!(SPOT_WEBSOCKET_URL, USD_M_MARKET_WEBSOCKET_URL);
        assert_eq!(BinanceAdapter::default().venue(), BinanceMarket::Spot);
    }

    #[test]
    fn a_symbol_splits_on_the_longest_quote_asset_that_ends_it() {
        assert_eq!(split_symbol("BTCUSDT"), Some(("BTC", "USDT")));
        assert_eq!(split_symbol("ETHBTC"), Some(("ETH", "BTC")));
        assert_eq!(split_symbol("BTCUSDC"), Some(("BTC", "USDC")));
        // The one a shortest-match or fixed-offset split gets wrong.
        assert_eq!(split_symbol("USDCUSDT"), Some(("USDC", "USDT")));
        assert_eq!(split_symbol("BTCFDUSD"), Some(("BTC", "FDUSD")));
        assert_eq!(split_symbol("USDTTRY"), Some(("USDT", "TRY")));
        assert_eq!(split_symbol("BNBETH"), Some(("BNB", "ETH")));
        assert_eq!(split_symbol("BTCTUSD"), Some(("BTC", "TUSD")));
    }

    #[test]
    fn a_symbol_with_no_known_quote_asset_is_refused_rather_than_guessed() {
        assert_eq!(split_symbol("BTCXYZ"), None);
        // A quote asset on its own has no base.
        assert_eq!(split_symbol("USDT"), None);
        assert_eq!(split_symbol(""), None);
    }

    #[test]
    fn a_symbol_round_trips_through_the_venues_market_kind() {
        let spot = BinanceAdapter::spot();
        let perp = BinanceAdapter::usd_m_futures();

        let spot_market = Market::spot(Exchange::Binance, "BTC", "USDT");
        let perp_market = Market::perpetual(Exchange::Binance, "BTC", "USDT");

        assert_eq!(spot.symbol(&spot_market).expect("a spot market"), "BTCUSDT");
        assert_eq!(perp.symbol(&perp_market).expect("a perp market"), "BTCUSDT");
        assert_eq!(
            spot.market("BTCUSDT").expect("a listed symbol"),
            spot_market
        );
        assert_eq!(
            perp.market("BTCUSDT").expect("a listed symbol"),
            perp_market
        );
    }

    #[test]
    fn a_market_from_the_wrong_venue_or_exchange_never_reaches_the_network() {
        let spot = BinanceAdapter::spot();

        assert!(matches!(
            spot.symbol(&Market::perpetual(Exchange::Binance, "BTC", "USDT")),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
        assert!(matches!(
            spot.symbol(&Market::spot(Exchange::Upbit, "BTC", "KRW")),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
    }

    #[test]
    fn a_symbol_that_could_smuggle_a_query_parameter_is_rejected() {
        let injected = Market::spot(Exchange::Binance, "BTC&limit=5000", "USDT");

        assert!(matches!(
            BinanceAdapter::spot().symbol(&injected),
            Err(Error::InvalidRequest { field, .. }) if field == "base"
        ));
    }

    #[test]
    fn one_second_candles_are_spot_only() {
        assert_eq!(
            BinanceMarket::Spot
                .interval_code(Interval::Sec1)
                .expect("spot serves one-second candles"),
            "1s"
        );
        assert_eq!(
            BinanceMarket::UsdMFutures
                .interval_code(Interval::Min1)
                .expect("one minute is the futures floor"),
            "1m"
        );
        assert!(matches!(
            BinanceMarket::UsdMFutures.interval_code(Interval::Sec1),
            Err(Error::Unsupported {
                feature: Feature::Candles,
                ..
            })
        ));
    }

    #[test]
    fn month_intervals_keep_binances_capital_m() {
        // `1m` and `1M` are a minute and a month; lowercasing the code silently
        // asks for the wrong candles.
        assert_eq!(
            BinanceMarket::Spot
                .interval_code(Interval::Month1)
                .expect("a served interval"),
            "1M"
        );
        assert_eq!(
            BinanceMarket::Spot
                .interval_code(Interval::Min1)
                .expect("a served interval"),
            "1m"
        );
    }

    #[test]
    fn an_error_body_keeps_binances_numeric_code() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-api-information
        let error = exchange_error(400, r#"{"code":-1121,"msg":"Invalid symbol."}"#);

        let Error::Exchange {
            code,
            message,
            status,
            kind,
            ..
        } = &error
        else {
            panic!("expected an exchange error");
        };
        assert_eq!(code, "-1121");
        assert_eq!(message, "Invalid symbol.");
        assert_eq!(*status, Some(400));
        assert_eq!(*kind, ExchangeErrorKind::Rejected);
        assert!(!error.is_retryable());
    }

    /// A credential rejected by Binance remains an exchange error with its code.
    #[test]
    fn a_credential_binance_refused_keeps_binances_own_code() {
        let cases = [
            (
                400,
                r#"{"code":-1022,"msg":"Signature for this request is not valid."}"#,
                "-1022",
            ),
            (
                401,
                r#"{"code":-2015,"msg":"Invalid API-key, IP, or permissions for action."}"#,
                "-2015",
            ),
            (
                401,
                r#"{"code":-2014,"msg":"API-key format invalid."}"#,
                "-2014",
            ),
        ];

        for (status, body, expected) in cases {
            let error = exchange_error(status, body);

            let Error::Exchange { code, .. } = &error else {
                panic!("expected the exchange's own verdict for {expected}, got {error:?}");
            };
            assert_eq!(code, expected, "{error:?}");
            // Retrying the identical signed request cannot make a wrong secret
            // right, so the classification the status gives is the right one.
            assert!(!error.is_retryable(), "{error:?}");
        }
    }

    /// A timestamp outside Binance's receive window is rejected, not retried.
    #[test]
    fn a_timestamp_outside_the_receive_window_is_rejected_rather_than_retried() {
        for body in [
            r#"{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."}"#,
            r#"{"code":-1021,"msg":"Timestamp for this request was 1000ms ahead of the server's time."}"#,
        ] {
            let error = exchange_error(400, body);

            let Error::Exchange { code, kind, .. } = &error else {
                panic!("expected an exchange error, got {error:?}");
            };
            assert_eq!(code, "-1021", "{error:?}");
            assert_eq!(*kind, ExchangeErrorKind::Rejected, "{error:?}");
            assert!(!error.is_retryable(), "{error:?}");
            assert!(!error.is_rate_limited(), "{error:?}");
        }
    }

    #[test]
    fn rate_limits_and_ip_bans_classify_as_rate_limited() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits
        let too_many = exchange_error(429, r#"{"code":-1003,"msg":"Too many requests."}"#);
        // 418 is what Binance answers with once a 429 has been ignored.
        let banned = exchange_error(
            418,
            r#"{"code":-1003,"msg":"Way too many requests; IP banned until 1499865549590."}"#,
        );

        assert!(too_many.is_rate_limited());
        assert!(banned.is_rate_limited());
        assert!(banned.is_retryable());
    }

    #[test]
    fn an_error_body_that_is_not_json_still_reports_the_status() {
        let error = exchange_error(502, "<html>Bad Gateway</html>");

        let Error::Exchange {
            kind,
            status,
            message,
            ..
        } = error
        else {
            panic!("expected an exchange error");
        };
        assert_eq!(status, Some(502));
        assert_eq!(kind, ExchangeErrorKind::Unavailable);
        assert_eq!(message, "<html>Bad Gateway</html>");
    }

    #[test]
    fn a_cursor_round_trips_without_the_caller_reading_it() {
        let cursor = encode_cursor(1_499_865_549_590);

        assert_eq!(
            decode_cursor(&cursor).expect("its own cursor reads back"),
            1_499_865_549_590
        );
        assert!(decode_cursor(&Cursor("page-2".to_string())).is_err());
        assert!(decode_cursor(&Cursor("t".to_string())).is_err());
    }

    #[test]
    fn listing_statuses_separate_a_halt_from_a_delisting() {
        assert_eq!(market_status("TRADING"), MarketStatus::Active);
        assert_eq!(market_status("BREAK"), MarketStatus::Paused);
        assert_eq!(market_status("SETTLING"), MarketStatus::Delisted);
        assert_eq!(market_status("SOMETHING_NEW"), MarketStatus::Unknown);
    }

    #[test]
    fn private_calls_refuse_to_run_without_credentials() {
        assert!(matches!(
            BinanceAdapter::spot().credentials(),
            Err(Error::Auth { .. })
        ));
        assert!(
            BinanceAdapter::spot()
                .with_credentials("key", "secret")
                .credentials()
                .is_ok()
        );
    }
}
