use std::fmt;

pub mod generated_models;
mod generated_shape_guard;

pub use generated_models::*;

use maxt::adapters::{
    BinanceSpotOrderDetail, BinanceSymbolFilters, BithumbAlertStep, BithumbMarketAlert,
    HyperliquidAssetContext, HyperliquidLedgerEntry, HyperliquidLedgerKind, UpbitMarketEvent,
};
use maxt::{
    Balance, Candle, CandleRequest, Cursor, Decimal, Error, Exchange, ExchangeErrorKind, Feature,
    FundingPayment, FundingRate, HistoryRequest, Interval, Level, MarginMode, MarginRequest,
    MarginSummary, Market, MarketInfo, MarketKind, MarketStatus, Order, OrderBook, OrderRequest,
    OrderStatus, OrderType, Page, Position, Side, Size, Ticker, TimeInForce, Timestamp, Trade,
    TransferErrorKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireExchange {
    Upbit,
    Bithumb,
    Binance,
    Hyperliquid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireMarketKind {
    Spot,
    Perpetual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireMarketStatus {
    Active,
    Paused,
    Delisted,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireInterval {
    Sec1,
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour2,
    Hour4,
    Hour8,
    Hour12,
    Day1,
    Day3,
    Week1,
    Month1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireFeature {
    Markets,
    Trades,
    OrderBook,
    Ticker,
    Candles,
    TradeStream,
    OrderBookStream,
    TickerStream,
    CandleStream,
    Balances,
    AssetNetworks,
    DepositAddresses,
    DepositHistory,
    WithdrawalQuotes,
    Withdrawals,
    WithdrawalHistory,
    OpenOrders,
    AccountStream,
    Trading,
    Positions,
    Margin,
    FundingRates,
    FundingPayments,
    MarginConfig,
    ReduceOnlyOrders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireOrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireTimeInForce {
    GoodTilCancelled,
    ImmediateOrCancel,
    FillOrKill,
    PostOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireOrderStatus {
    Accepted,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireMarginMode {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireSizeKind {
    Base,
    Quote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSize {
    pub kind: WireSizeKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireMarket {
    pub exchange: WireExchange,
    pub kind: WireMarketKind,
    pub base: String,
    pub quote: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireMarketInfo {
    pub market: WireMarket,
    pub native_symbol: String,
    pub status: WireMarketStatus,
    pub korean_name: Option<String>,
    pub english_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireTrade {
    pub market: WireMarket,
    pub timestamp_ns: i64,
    pub price: String,
    pub quantity: String,
    pub taker_side: WireSide,
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireLevel {
    pub price: String,
    pub quantity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireOrderBook {
    pub market: WireMarket,
    pub timestamp_ns: i64,
    pub bids: Vec<WireLevel>,
    pub asks: Vec<WireLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireTicker {
    pub market: WireMarket,
    pub timestamp_ns: i64,
    pub last_trade_time_ns: Option<i64>,
    pub last_price: String,
    pub change: Option<String>,
    pub change_rate: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub volume: Option<String>,
    pub quote_volume: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCandle {
    pub market: WireMarket,
    pub interval: WireInterval,
    pub open_time_ns: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub quote_volume: Option<String>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBalance {
    pub asset: String,
    pub available: String,
    pub locked: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireOrder {
    pub id: String,
    pub market: WireMarket,
    pub side: WireSide,
    pub status: WireOrderStatus,
    pub filled_quantity: String,
    pub remaining_quantity: String,
    pub price: Option<String>,
    pub created_at_ns: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirePosition {
    pub market: WireMarket,
    pub side: Option<WireSide>,
    pub quantity: String,
    pub entry_price: Option<String>,
    pub mark_price: Option<String>,
    pub notional: Option<String>,
    pub unrealized_pnl: Option<String>,
    pub leverage: Option<String>,
    pub margin_mode: Option<WireMarginMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireMarginSummary {
    pub asset: String,
    pub equity: Option<String>,
    pub margin_balance: Option<String>,
    pub available_balance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFundingRate {
    pub market: WireMarket,
    pub timestamp_ns: i64,
    pub rate: String,
    pub mark_price: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFundingPayment {
    pub market: WireMarket,
    pub timestamp_ns: i64,
    pub amount: String,
    pub rate: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFundingRatePage {
    pub items: Vec<WireFundingRate>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFundingPaymentPage {
    pub items: Vec<WireFundingPayment>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCandleRequest {
    pub market: WireMarket,
    pub interval: WireInterval,
    pub from_ns: Option<i64>,
    pub to_ns: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireOrderRequest {
    pub market: WireMarket,
    pub side: WireSide,
    pub order_type: WireOrderType,
    pub size: WireSize,
    pub price: Option<String>,
    pub time_in_force: Option<WireTimeInForce>,
    pub reduce_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireHistoryRequest {
    pub market: WireMarket,
    pub from_ns: Option<i64>,
    pub to_ns: Option<i64>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireMarginRequest {
    pub market: WireMarket,
    pub leverage: Option<String>,
    pub margin_mode: Option<WireMarginMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireUpbitMarketEvent {
    pub market: WireMarket,
    pub warning: bool,
    pub cautions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBithumbMarketWarning {
    pub market: WireMarket,
    pub warning: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireBithumbAlertStep {
    Caution,
    Warning,
    Danger,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBithumbMarketAlert {
    pub market: WireMarket,
    pub kind: String,
    pub step: WireBithumbAlertStep,
    pub ends_at_ns: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBinanceSymbolFilters {
    pub symbol: String,
    pub tick_size: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
    pub step_size: Option<String>,
    pub min_quantity: Option<String>,
    pub max_quantity: Option<String>,
    pub min_notional: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBinanceSpotOrderDetail {
    pub order: WireOrder,
    pub client_order_id: String,
    pub order_type: String,
    pub time_in_force: String,
    pub filled_quote_quantity: String,
    pub updated_at_ns: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireHyperliquidLedgerKind {
    Deposit,
    Withdraw,
    InternalTransfer,
    SubAccountTransfer,
    SpotTransfer,
    AccountClassTransfer,
    VaultDeposit,
    VaultWithdraw,
    VaultDistribution,
    Liquidation,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireHyperliquidLedgerEntry {
    pub kind: WireHyperliquidLedgerKind,
    pub provider_kind: Option<String>,
    pub time_ns: i64,
    pub hash: String,
    pub asset: Option<String>,
    pub amount: Option<String>,
    pub fee: Option<String>,
    pub counterparty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireHyperliquidLedgerPage {
    pub items: Vec<WireHyperliquidLedgerEntry>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireHyperliquidAssetContext {
    pub mid_price: Option<String>,
    pub mark_price: Option<String>,
    pub oracle_price: Option<String>,
    pub funding_rate: Option<String>,
    pub open_interest: Option<String>,
    pub size_decimals: u32,
    pub price_decimals: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeErrorKind {
    InvalidRequest,
    Transfer,
    Unsupported,
    Adapter,
    Auth,
    Exchange,
    Transport,
    Decode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireTransferErrorKind {
    AssetMismatch,
    NetworkMismatch,
    AmbiguousNetwork,
    NetworkUnavailable,
    MemoRequired,
    DestinationUnavailable,
    AddressNotAllowed,
    TravelRuleRequired,
    AmountOutOfRange,
    PlanExpired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireExchangeErrorKind {
    Rejected,
    RateLimited,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeError {
    pub kind: NativeErrorKind,
    pub message: String,
    pub detail: Option<String>,
    pub field: Option<String>,
    pub transfer_kind: Option<WireTransferErrorKind>,
    pub feature: Option<WireFeature>,
    pub exchange: Option<String>,
    pub code: Option<String>,
    pub status: Option<u16>,
    pub exchange_kind: Option<WireExchangeErrorKind>,
    pub retryable: bool,
    pub rate_limited: bool,
}

impl NativeError {
    pub(crate) fn invalid_request(field: &str, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self {
            kind: NativeErrorKind::InvalidRequest,
            message: format!("invalid request: `{field}`: {detail}"),
            detail: Some(detail),
            field: Some(field.to_owned()),
            transfer_kind: None,
            feature: None,
            exchange: None,
            code: None,
            status: None,
            exchange_kind: None,
            retryable: false,
            rate_limited: false,
        }
    }
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NativeError {}

pub(crate) fn decimal_to_wire(value: Decimal) -> String {
    value.to_string()
}

fn decimal_option_to_wire(value: Option<Decimal>) -> Option<String> {
    value.map(decimal_to_wire)
}

fn decimal_option_from_wire(value: Option<String>, field: &str) -> Result<Option<Decimal>, Error> {
    value
        .as_deref()
        .map(|value| decimal_from_wire(value, field))
        .transpose()
}

pub(crate) fn decimal_from_wire(value: &str, field: &str) -> Result<Decimal, Error> {
    maxt::parse_decimal_exact(value).map_err(|error| Error::InvalidRequest {
        field: field.to_owned(),
        detail: format!("`{value}` is not an exact decimal: {error}"),
    })
}

pub(crate) fn timestamp_to_wire(value: Timestamp) -> i64 {
    value.as_nanos()
}

fn timestamp_option_to_wire(value: Option<Timestamp>) -> Option<i64> {
    value.map(timestamp_to_wire)
}

impl From<Exchange> for WireExchange {
    fn from(value: Exchange) -> Self {
        match value {
            Exchange::Upbit => Self::Upbit,
            Exchange::Bithumb => Self::Bithumb,
            Exchange::Binance => Self::Binance,
            Exchange::Hyperliquid => Self::Hyperliquid,
            _ => unreachable!("새 maxt exchange에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<WireExchange> for Exchange {
    fn from(value: WireExchange) -> Self {
        match value {
            WireExchange::Upbit => Self::Upbit,
            WireExchange::Bithumb => Self::Bithumb,
            WireExchange::Binance => Self::Binance,
            WireExchange::Hyperliquid => Self::Hyperliquid,
        }
    }
}

impl From<MarketKind> for WireMarketKind {
    fn from(value: MarketKind) -> Self {
        match value {
            MarketKind::Spot => Self::Spot,
            MarketKind::Perpetual => Self::Perpetual,
            _ => unreachable!("새 maxt market kind에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<WireMarketKind> for MarketKind {
    fn from(value: WireMarketKind) -> Self {
        match value {
            WireMarketKind::Spot => Self::Spot,
            WireMarketKind::Perpetual => Self::Perpetual,
        }
    }
}

impl From<MarketStatus> for WireMarketStatus {
    fn from(value: MarketStatus) -> Self {
        match value {
            MarketStatus::Active => Self::Active,
            MarketStatus::Paused => Self::Paused,
            MarketStatus::Delisted => Self::Delisted,
            MarketStatus::Unknown => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

impl From<Side> for WireSide {
    fn from(value: Side) -> Self {
        match value {
            Side::Buy => Self::Buy,
            Side::Sell => Self::Sell,
        }
    }
}

impl From<WireSide> for Side {
    fn from(value: WireSide) -> Self {
        match value {
            WireSide::Buy => Self::Buy,
            WireSide::Sell => Self::Sell,
        }
    }
}

impl From<Interval> for WireInterval {
    fn from(value: Interval) -> Self {
        match value {
            Interval::Sec1 => Self::Sec1,
            Interval::Min1 => Self::Min1,
            Interval::Min3 => Self::Min3,
            Interval::Min5 => Self::Min5,
            Interval::Min15 => Self::Min15,
            Interval::Min30 => Self::Min30,
            Interval::Hour1 => Self::Hour1,
            Interval::Hour2 => Self::Hour2,
            Interval::Hour4 => Self::Hour4,
            Interval::Hour8 => Self::Hour8,
            Interval::Hour12 => Self::Hour12,
            Interval::Day1 => Self::Day1,
            Interval::Day3 => Self::Day3,
            Interval::Week1 => Self::Week1,
            Interval::Month1 => Self::Month1,
            _ => unreachable!("새 maxt interval에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<WireInterval> for Interval {
    fn from(value: WireInterval) -> Self {
        match value {
            WireInterval::Sec1 => Self::Sec1,
            WireInterval::Min1 => Self::Min1,
            WireInterval::Min3 => Self::Min3,
            WireInterval::Min5 => Self::Min5,
            WireInterval::Min15 => Self::Min15,
            WireInterval::Min30 => Self::Min30,
            WireInterval::Hour1 => Self::Hour1,
            WireInterval::Hour2 => Self::Hour2,
            WireInterval::Hour4 => Self::Hour4,
            WireInterval::Hour8 => Self::Hour8,
            WireInterval::Hour12 => Self::Hour12,
            WireInterval::Day1 => Self::Day1,
            WireInterval::Day3 => Self::Day3,
            WireInterval::Week1 => Self::Week1,
            WireInterval::Month1 => Self::Month1,
        }
    }
}

impl From<Feature> for WireFeature {
    fn from(value: Feature) -> Self {
        match value {
            Feature::Markets => Self::Markets,
            Feature::Trades => Self::Trades,
            Feature::OrderBook => Self::OrderBook,
            Feature::Ticker => Self::Ticker,
            Feature::Candles => Self::Candles,
            Feature::TradeStream => Self::TradeStream,
            Feature::OrderBookStream => Self::OrderBookStream,
            Feature::TickerStream => Self::TickerStream,
            Feature::CandleStream => Self::CandleStream,
            Feature::Balances => Self::Balances,
            Feature::AssetNetworks => Self::AssetNetworks,
            Feature::DepositAddresses => Self::DepositAddresses,
            Feature::DepositHistory => Self::DepositHistory,
            Feature::WithdrawalQuotes => Self::WithdrawalQuotes,
            Feature::Withdrawals => Self::Withdrawals,
            Feature::WithdrawalHistory => Self::WithdrawalHistory,
            Feature::OpenOrders => Self::OpenOrders,
            Feature::AccountStream => Self::AccountStream,
            Feature::Trading => Self::Trading,
            Feature::Positions => Self::Positions,
            Feature::Margin => Self::Margin,
            Feature::FundingRates => Self::FundingRates,
            Feature::FundingPayments => Self::FundingPayments,
            Feature::MarginConfig => Self::MarginConfig,
            Feature::ReduceOnlyOrders => Self::ReduceOnlyOrders,
            _ => unreachable!("새 maxt feature에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<WireFeature> for Feature {
    fn from(value: WireFeature) -> Self {
        match value {
            WireFeature::Markets => Self::Markets,
            WireFeature::Trades => Self::Trades,
            WireFeature::OrderBook => Self::OrderBook,
            WireFeature::Ticker => Self::Ticker,
            WireFeature::Candles => Self::Candles,
            WireFeature::TradeStream => Self::TradeStream,
            WireFeature::OrderBookStream => Self::OrderBookStream,
            WireFeature::TickerStream => Self::TickerStream,
            WireFeature::CandleStream => Self::CandleStream,
            WireFeature::Balances => Self::Balances,
            WireFeature::AssetNetworks => Self::AssetNetworks,
            WireFeature::DepositAddresses => Self::DepositAddresses,
            WireFeature::DepositHistory => Self::DepositHistory,
            WireFeature::WithdrawalQuotes => Self::WithdrawalQuotes,
            WireFeature::Withdrawals => Self::Withdrawals,
            WireFeature::WithdrawalHistory => Self::WithdrawalHistory,
            WireFeature::OpenOrders => Self::OpenOrders,
            WireFeature::AccountStream => Self::AccountStream,
            WireFeature::Trading => Self::Trading,
            WireFeature::Positions => Self::Positions,
            WireFeature::Margin => Self::Margin,
            WireFeature::FundingRates => Self::FundingRates,
            WireFeature::FundingPayments => Self::FundingPayments,
            WireFeature::MarginConfig => Self::MarginConfig,
            WireFeature::ReduceOnlyOrders => Self::ReduceOnlyOrders,
        }
    }
}

impl From<WireTimeInForce> for TimeInForce {
    fn from(value: WireTimeInForce) -> Self {
        match value {
            WireTimeInForce::GoodTilCancelled => Self::GoodTilCancelled,
            WireTimeInForce::ImmediateOrCancel => Self::ImmediateOrCancel,
            WireTimeInForce::FillOrKill => Self::FillOrKill,
            WireTimeInForce::PostOnly => Self::PostOnly,
        }
    }
}

impl From<WireOrderType> for OrderType {
    fn from(value: WireOrderType) -> Self {
        match value {
            WireOrderType::Market => Self::Market,
            WireOrderType::Limit => Self::Limit,
        }
    }
}

impl From<OrderStatus> for WireOrderStatus {
    fn from(value: OrderStatus) -> Self {
        match value {
            OrderStatus::Accepted => Self::Accepted,
            OrderStatus::Open => Self::Open,
            OrderStatus::PartiallyFilled => Self::PartiallyFilled,
            OrderStatus::Filled => Self::Filled,
            OrderStatus::Cancelled => Self::Cancelled,
            OrderStatus::Rejected => Self::Rejected,
            OrderStatus::Unknown => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

impl From<WireOrderStatus> for OrderStatus {
    fn from(value: WireOrderStatus) -> Self {
        match value {
            WireOrderStatus::Accepted => Self::Accepted,
            WireOrderStatus::Open => Self::Open,
            WireOrderStatus::PartiallyFilled => Self::PartiallyFilled,
            WireOrderStatus::Filled => Self::Filled,
            WireOrderStatus::Cancelled => Self::Cancelled,
            WireOrderStatus::Rejected => Self::Rejected,
            WireOrderStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<WireMarketStatus> for MarketStatus {
    fn from(value: WireMarketStatus) -> Self {
        match value {
            WireMarketStatus::Active => Self::Active,
            WireMarketStatus::Paused => Self::Paused,
            WireMarketStatus::Delisted => Self::Delisted,
            WireMarketStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<MarginMode> for WireMarginMode {
    fn from(value: MarginMode) -> Self {
        match value {
            MarginMode::Cross => Self::Cross,
            MarginMode::Isolated => Self::Isolated,
            _ => unreachable!("새 maxt margin mode에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<WireMarginMode> for MarginMode {
    fn from(value: WireMarginMode) -> Self {
        match value {
            WireMarginMode::Cross => Self::Cross,
            WireMarginMode::Isolated => Self::Isolated,
        }
    }
}

impl From<WireMarket> for Market {
    fn from(value: WireMarket) -> Self {
        Market::new(
            value.exchange.into(),
            value.kind.into(),
            value.base,
            value.quote,
        )
    }
}

impl From<Market> for WireMarket {
    fn from(value: Market) -> Self {
        Self {
            exchange: value.exchange.into(),
            kind: value.kind.into(),
            base: value.base,
            quote: value.quote,
        }
    }
}

impl TryFrom<WireSize> for Size {
    type Error = NativeError;

    fn try_from(value: WireSize) -> Result<Self, Self::Error> {
        let amount = decimal_from_wire(&value.value, "size")?;
        Ok(match value.kind {
            WireSizeKind::Base => Self::Base(amount),
            WireSizeKind::Quote => Self::Quote(amount),
        })
    }
}

impl From<Size> for WireSize {
    fn from(value: Size) -> Self {
        match value {
            Size::Base(value) => Self {
                kind: WireSizeKind::Base,
                value: decimal_to_wire(value),
            },
            Size::Quote(value) => Self {
                kind: WireSizeKind::Quote,
                value: decimal_to_wire(value),
            },
            _ => unreachable!("새 maxt size kind에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<OrderType> for WireOrderType {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Market => Self::Market,
            OrderType::Limit => Self::Limit,
            _ => unreachable!("새 maxt order type에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<TimeInForce> for WireTimeInForce {
    fn from(value: TimeInForce) -> Self {
        match value {
            TimeInForce::GoodTilCancelled => Self::GoodTilCancelled,
            TimeInForce::ImmediateOrCancel => Self::ImmediateOrCancel,
            TimeInForce::FillOrKill => Self::FillOrKill,
            TimeInForce::PostOnly => Self::PostOnly,
            _ => unreachable!("새 maxt time in force에는 새 Dart wire variant가 필요합니다"),
        }
    }
}

impl From<MarketInfo> for WireMarketInfo {
    fn from(value: MarketInfo) -> Self {
        Self {
            market: value.market.into(),
            native_symbol: value.native_symbol,
            status: value.status.into(),
            korean_name: value.korean_name,
            english_name: value.english_name,
        }
    }
}

impl From<Trade> for WireTrade {
    fn from(value: Trade) -> Self {
        Self {
            market: value.market.into(),
            timestamp_ns: timestamp_to_wire(value.timestamp),
            price: decimal_to_wire(value.price),
            quantity: decimal_to_wire(value.quantity),
            taker_side: value.taker_side.into(),
            id: value.id,
        }
    }
}

impl From<Level> for WireLevel {
    fn from(value: Level) -> Self {
        Self {
            price: decimal_to_wire(value.price),
            quantity: decimal_to_wire(value.quantity),
        }
    }
}

impl From<OrderBook> for WireOrderBook {
    fn from(value: OrderBook) -> Self {
        Self {
            market: value.market.into(),
            timestamp_ns: timestamp_to_wire(value.timestamp),
            bids: value.bids.into_iter().map(Into::into).collect(),
            asks: value.asks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Ticker> for WireTicker {
    fn from(value: Ticker) -> Self {
        Self {
            market: value.market.into(),
            timestamp_ns: timestamp_to_wire(value.timestamp),
            last_trade_time_ns: timestamp_option_to_wire(value.last_trade_time),
            last_price: decimal_to_wire(value.last_price),
            change: decimal_option_to_wire(value.change),
            change_rate: decimal_option_to_wire(value.change_rate),
            high: decimal_option_to_wire(value.high),
            low: decimal_option_to_wire(value.low),
            volume: decimal_option_to_wire(value.volume),
            quote_volume: decimal_option_to_wire(value.quote_volume),
        }
    }
}

impl From<Candle> for WireCandle {
    fn from(value: Candle) -> Self {
        Self {
            market: value.market.into(),
            interval: value.interval.into(),
            open_time_ns: timestamp_to_wire(value.open_time),
            open: decimal_to_wire(value.open),
            high: decimal_to_wire(value.high),
            low: decimal_to_wire(value.low),
            close: decimal_to_wire(value.close),
            volume: decimal_to_wire(value.volume),
            quote_volume: decimal_option_to_wire(value.quote_volume),
            closed: value.closed,
        }
    }
}

impl From<Balance> for WireBalance {
    fn from(value: Balance) -> Self {
        Self {
            asset: value.asset,
            available: decimal_to_wire(value.available),
            locked: decimal_to_wire(value.locked),
        }
    }
}

impl From<Order> for WireOrder {
    fn from(value: Order) -> Self {
        Self {
            id: value.id,
            market: value.market.into(),
            side: value.side.into(),
            status: value.status.into(),
            filled_quantity: decimal_to_wire(value.filled_quantity),
            remaining_quantity: decimal_to_wire(value.remaining_quantity),
            price: decimal_option_to_wire(value.price),
            created_at_ns: timestamp_option_to_wire(value.created_at),
        }
    }
}

impl From<Position> for WirePosition {
    fn from(value: Position) -> Self {
        Self {
            market: value.market.into(),
            side: value.side.map(Into::into),
            quantity: decimal_to_wire(value.quantity),
            entry_price: decimal_option_to_wire(value.entry_price),
            mark_price: decimal_option_to_wire(value.mark_price),
            notional: decimal_option_to_wire(value.notional),
            unrealized_pnl: decimal_option_to_wire(value.unrealized_pnl),
            leverage: decimal_option_to_wire(value.leverage),
            margin_mode: value.margin_mode.map(Into::into),
        }
    }
}

impl From<MarginSummary> for WireMarginSummary {
    fn from(value: MarginSummary) -> Self {
        Self {
            asset: value.asset,
            equity: decimal_option_to_wire(value.equity),
            margin_balance: decimal_option_to_wire(value.margin_balance),
            available_balance: decimal_option_to_wire(value.available_balance),
        }
    }
}

impl From<FundingRate> for WireFundingRate {
    fn from(value: FundingRate) -> Self {
        Self {
            market: value.market.into(),
            timestamp_ns: timestamp_to_wire(value.timestamp),
            rate: decimal_to_wire(value.rate),
            mark_price: decimal_option_to_wire(value.mark_price),
        }
    }
}

impl From<FundingPayment> for WireFundingPayment {
    fn from(value: FundingPayment) -> Self {
        Self {
            market: value.market.into(),
            timestamp_ns: timestamp_to_wire(value.timestamp),
            amount: decimal_to_wire(value.amount),
            rate: decimal_option_to_wire(value.rate),
            id: value.id,
        }
    }
}

impl From<Page<FundingRate>> for WireFundingRatePage {
    fn from(value: Page<FundingRate>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            next: value.next.map(|cursor| cursor.as_str().to_owned()),
        }
    }
}

impl From<Page<FundingPayment>> for WireFundingPaymentPage {
    fn from(value: Page<FundingPayment>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            next: value.next.map(|cursor| cursor.as_str().to_owned()),
        }
    }
}

impl From<WireCandleRequest> for CandleRequest {
    fn from(value: WireCandleRequest) -> Self {
        Self {
            market: value.market.into(),
            interval: value.interval.into(),
            from: value.from_ns.map(Timestamp::from_nanos),
            to: value.to_ns.map(Timestamp::from_nanos),
            limit: value.limit,
        }
    }
}

impl From<CandleRequest> for WireCandleRequest {
    fn from(value: CandleRequest) -> Self {
        Self {
            market: value.market.into(),
            interval: value.interval.into(),
            from_ns: timestamp_option_to_wire(value.from),
            to_ns: timestamp_option_to_wire(value.to),
            limit: value.limit,
        }
    }
}

impl TryFrom<WireOrderRequest> for OrderRequest {
    type Error = NativeError;

    fn try_from(value: WireOrderRequest) -> Result<Self, Self::Error> {
        let market = value.market.into();
        let side = value.side.into();
        let size = value.size.try_into()?;
        let mut request = match (value.order_type, value.price) {
            (WireOrderType::Market, None) => Self::market(market, side, size),
            (WireOrderType::Market, Some(_)) => {
                return Err(NativeError::invalid_request(
                    "price",
                    "a market order must not have a price",
                ));
            }
            (WireOrderType::Limit, Some(price)) => {
                Self::limit(market, side, size, decimal_from_wire(&price, "price")?)
            }
            (WireOrderType::Limit, None) => {
                return Err(NativeError::invalid_request(
                    "price",
                    "a limit order requires a price",
                ));
            }
        };
        if let Some(time_in_force) = value.time_in_force {
            request = request.time_in_force(time_in_force.into());
        }
        if value.reduce_only {
            request = request.reduce_only();
        }
        Ok(request)
    }
}

impl From<OrderRequest> for WireOrderRequest {
    fn from(value: OrderRequest) -> Self {
        Self {
            market: value.market.into(),
            side: value.side.into(),
            order_type: value.order_type.into(),
            size: value.size.into(),
            price: decimal_option_to_wire(value.price),
            time_in_force: value.time_in_force.map(Into::into),
            reduce_only: value.reduce_only,
        }
    }
}

impl From<WireHistoryRequest> for HistoryRequest {
    fn from(value: WireHistoryRequest) -> Self {
        Self {
            market: value.market.into(),
            from: value.from_ns.map(Timestamp::from_nanos),
            to: value.to_ns.map(Timestamp::from_nanos),
            cursor: value.cursor.map(Cursor::new),
            limit: value.limit,
        }
    }
}

impl From<HistoryRequest> for WireHistoryRequest {
    fn from(value: HistoryRequest) -> Self {
        Self {
            market: value.market.into(),
            from_ns: timestamp_option_to_wire(value.from),
            to_ns: timestamp_option_to_wire(value.to),
            cursor: value.cursor.map(|cursor| cursor.as_str().to_owned()),
            limit: value.limit,
        }
    }
}

impl TryFrom<WireMarginRequest> for MarginRequest {
    type Error = NativeError;

    fn try_from(value: WireMarginRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            leverage: value
                .leverage
                .as_deref()
                .map(|leverage| decimal_from_wire(leverage, "leverage"))
                .transpose()?,
            margin_mode: value.margin_mode.map(Into::into),
        })
    }
}

impl From<MarginRequest> for WireMarginRequest {
    fn from(value: MarginRequest) -> Self {
        Self {
            market: value.market.into(),
            leverage: decimal_option_to_wire(value.leverage),
            margin_mode: value.margin_mode.map(Into::into),
        }
    }
}

impl From<(Market, UpbitMarketEvent)> for WireUpbitMarketEvent {
    fn from((market, event): (Market, UpbitMarketEvent)) -> Self {
        Self {
            market: market.into(),
            warning: event.warning,
            cautions: event.cautions,
        }
    }
}

impl From<(Market, String)> for WireBithumbMarketWarning {
    fn from((market, warning): (Market, String)) -> Self {
        Self {
            market: market.into(),
            warning,
        }
    }
}

impl From<BithumbAlertStep> for WireBithumbAlertStep {
    fn from(value: BithumbAlertStep) -> Self {
        match value {
            BithumbAlertStep::Caution => Self::Caution,
            BithumbAlertStep::Warning => Self::Warning,
            BithumbAlertStep::Danger => Self::Danger,
            BithumbAlertStep::Unknown => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

impl From<(Market, BithumbMarketAlert)> for WireBithumbMarketAlert {
    fn from((market, alert): (Market, BithumbMarketAlert)) -> Self {
        Self {
            market: market.into(),
            kind: alert.kind,
            step: alert.step.into(),
            ends_at_ns: timestamp_to_wire(alert.ends_at),
        }
    }
}

impl From<BinanceSymbolFilters> for WireBinanceSymbolFilters {
    fn from(value: BinanceSymbolFilters) -> Self {
        Self {
            symbol: value.symbol,
            tick_size: decimal_option_to_wire(value.tick_size),
            min_price: decimal_option_to_wire(value.min_price),
            max_price: decimal_option_to_wire(value.max_price),
            step_size: decimal_option_to_wire(value.step_size),
            min_quantity: decimal_option_to_wire(value.min_quantity),
            max_quantity: decimal_option_to_wire(value.max_quantity),
            min_notional: decimal_option_to_wire(value.min_notional),
        }
    }
}

impl From<BinanceSpotOrderDetail> for WireBinanceSpotOrderDetail {
    fn from(value: BinanceSpotOrderDetail) -> Self {
        Self {
            order: value.order.into(),
            client_order_id: value.client_order_id,
            order_type: value.order_type,
            time_in_force: value.time_in_force,
            filled_quote_quantity: decimal_to_wire(value.filled_quote_quantity),
            updated_at_ns: timestamp_option_to_wire(value.updated_at),
        }
    }
}

impl From<HyperliquidLedgerEntry> for WireHyperliquidLedgerEntry {
    fn from(value: HyperliquidLedgerEntry) -> Self {
        let (kind, provider_kind) = match value.kind {
            HyperliquidLedgerKind::Deposit => (WireHyperliquidLedgerKind::Deposit, None),
            HyperliquidLedgerKind::Withdraw => (WireHyperliquidLedgerKind::Withdraw, None),
            HyperliquidLedgerKind::InternalTransfer => {
                (WireHyperliquidLedgerKind::InternalTransfer, None)
            }
            HyperliquidLedgerKind::SubAccountTransfer => {
                (WireHyperliquidLedgerKind::SubAccountTransfer, None)
            }
            HyperliquidLedgerKind::SpotTransfer => (WireHyperliquidLedgerKind::SpotTransfer, None),
            HyperliquidLedgerKind::AccountClassTransfer => {
                (WireHyperliquidLedgerKind::AccountClassTransfer, None)
            }
            HyperliquidLedgerKind::VaultDeposit => (WireHyperliquidLedgerKind::VaultDeposit, None),
            HyperliquidLedgerKind::VaultWithdraw => {
                (WireHyperliquidLedgerKind::VaultWithdraw, None)
            }
            HyperliquidLedgerKind::VaultDistribution => {
                (WireHyperliquidLedgerKind::VaultDistribution, None)
            }
            HyperliquidLedgerKind::Liquidation => (WireHyperliquidLedgerKind::Liquidation, None),
            HyperliquidLedgerKind::Other(name) => (WireHyperliquidLedgerKind::Other, Some(name)),
            _ => (WireHyperliquidLedgerKind::Other, None),
        };
        Self {
            kind,
            provider_kind,
            time_ns: timestamp_to_wire(value.time),
            hash: value.hash,
            asset: value.asset,
            amount: decimal_option_to_wire(value.amount),
            fee: decimal_option_to_wire(value.fee),
            counterparty: value.counterparty,
        }
    }
}

impl From<Page<HyperliquidLedgerEntry>> for WireHyperliquidLedgerPage {
    fn from(value: Page<HyperliquidLedgerEntry>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            next: value.next.map(|cursor| cursor.as_str().to_owned()),
        }
    }
}

impl From<HyperliquidAssetContext> for WireHyperliquidAssetContext {
    fn from(value: HyperliquidAssetContext) -> Self {
        Self {
            mid_price: decimal_option_to_wire(value.mid_price),
            mark_price: decimal_option_to_wire(value.mark_price),
            oracle_price: decimal_option_to_wire(value.oracle_price),
            funding_rate: decimal_option_to_wire(value.funding_rate),
            open_interest: decimal_option_to_wire(value.open_interest),
            size_decimals: value.size_decimals,
            price_decimals: value.price_decimals,
        }
    }
}

impl TryFrom<WireMarketInfo> for MarketInfo {
    type Error = NativeError;

    fn try_from(value: WireMarketInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            native_symbol: value.native_symbol,
            status: value.status.into(),
            korean_name: value.korean_name,
            english_name: value.english_name,
        })
    }
}

impl TryFrom<WireTrade> for Trade {
    type Error = NativeError;

    fn try_from(value: WireTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            timestamp: Timestamp::from_nanos(value.timestamp_ns),
            price: decimal_from_wire(&value.price, "price")?,
            quantity: decimal_from_wire(&value.quantity, "quantity")?,
            taker_side: value.taker_side.into(),
            id: value.id,
        })
    }
}

impl TryFrom<WireLevel> for Level {
    type Error = NativeError;

    fn try_from(value: WireLevel) -> Result<Self, Self::Error> {
        Ok(Self {
            price: decimal_from_wire(&value.price, "price")?,
            quantity: decimal_from_wire(&value.quantity, "quantity")?,
        })
    }
}

impl TryFrom<WireOrderBook> for OrderBook {
    type Error = NativeError;

    fn try_from(value: WireOrderBook) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            timestamp: Timestamp::from_nanos(value.timestamp_ns),
            bids: value
                .bids
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            asks: value
                .asks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<WireTicker> for Ticker {
    type Error = NativeError;

    fn try_from(value: WireTicker) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            timestamp: Timestamp::from_nanos(value.timestamp_ns),
            last_trade_time: value.last_trade_time_ns.map(Timestamp::from_nanos),
            last_price: decimal_from_wire(&value.last_price, "last_price")?,
            change: decimal_option_from_wire(value.change, "change")?,
            change_rate: decimal_option_from_wire(value.change_rate, "change_rate")?,
            high: decimal_option_from_wire(value.high, "high")?,
            low: decimal_option_from_wire(value.low, "low")?,
            volume: decimal_option_from_wire(value.volume, "volume")?,
            quote_volume: decimal_option_from_wire(value.quote_volume, "quote_volume")?,
        })
    }
}

impl TryFrom<WireCandle> for Candle {
    type Error = NativeError;

    fn try_from(value: WireCandle) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            interval: value.interval.into(),
            open_time: Timestamp::from_nanos(value.open_time_ns),
            open: decimal_from_wire(&value.open, "open")?,
            high: decimal_from_wire(&value.high, "high")?,
            low: decimal_from_wire(&value.low, "low")?,
            close: decimal_from_wire(&value.close, "close")?,
            volume: decimal_from_wire(&value.volume, "volume")?,
            quote_volume: decimal_option_from_wire(value.quote_volume, "quote_volume")?,
            closed: value.closed,
        })
    }
}

impl TryFrom<WireBalance> for Balance {
    type Error = NativeError;

    fn try_from(value: WireBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            available: decimal_from_wire(&value.available, "available")?,
            locked: decimal_from_wire(&value.locked, "locked")?,
        })
    }
}

impl TryFrom<WireOrder> for Order {
    type Error = NativeError;

    fn try_from(value: WireOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            market: value.market.into(),
            side: value.side.into(),
            status: value.status.into(),
            filled_quantity: decimal_from_wire(&value.filled_quantity, "filled_quantity")?,
            remaining_quantity: decimal_from_wire(&value.remaining_quantity, "remaining_quantity")?,
            price: decimal_option_from_wire(value.price, "price")?,
            created_at: value.created_at_ns.map(Timestamp::from_nanos),
        })
    }
}

impl TryFrom<WirePosition> for Position {
    type Error = NativeError;

    fn try_from(value: WirePosition) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            side: value.side.map(Into::into),
            quantity: decimal_from_wire(&value.quantity, "quantity")?,
            entry_price: decimal_option_from_wire(value.entry_price, "entry_price")?,
            mark_price: decimal_option_from_wire(value.mark_price, "mark_price")?,
            notional: decimal_option_from_wire(value.notional, "notional")?,
            unrealized_pnl: decimal_option_from_wire(value.unrealized_pnl, "unrealized_pnl")?,
            leverage: decimal_option_from_wire(value.leverage, "leverage")?,
            margin_mode: value.margin_mode.map(Into::into),
        })
    }
}

impl TryFrom<WireMarginSummary> for MarginSummary {
    type Error = NativeError;

    fn try_from(value: WireMarginSummary) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            equity: decimal_option_from_wire(value.equity, "equity")?,
            margin_balance: decimal_option_from_wire(value.margin_balance, "margin_balance")?,
            available_balance: decimal_option_from_wire(
                value.available_balance,
                "available_balance",
            )?,
        })
    }
}

impl TryFrom<WireFundingRate> for FundingRate {
    type Error = NativeError;

    fn try_from(value: WireFundingRate) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            timestamp: Timestamp::from_nanos(value.timestamp_ns),
            rate: decimal_from_wire(&value.rate, "rate")?,
            mark_price: decimal_option_from_wire(value.mark_price, "mark_price")?,
        })
    }
}

impl TryFrom<WireFundingPayment> for FundingPayment {
    type Error = NativeError;

    fn try_from(value: WireFundingPayment) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.into(),
            timestamp: Timestamp::from_nanos(value.timestamp_ns),
            amount: decimal_from_wire(&value.amount, "amount")?,
            rate: decimal_option_from_wire(value.rate, "rate")?,
            id: value.id,
        })
    }
}

impl TryFrom<WireFundingRatePage> for Page<FundingRate> {
    type Error = NativeError;

    fn try_from(value: WireFundingRatePage) -> Result<Self, Self::Error> {
        Ok(Self {
            items: value
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            next: value.next.map(Cursor::new),
        })
    }
}

impl TryFrom<WireFundingPaymentPage> for Page<FundingPayment> {
    type Error = NativeError;

    fn try_from(value: WireFundingPaymentPage) -> Result<Self, Self::Error> {
        Ok(Self {
            items: value
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            next: value.next.map(Cursor::new),
        })
    }
}

impl From<ExchangeErrorKind> for WireExchangeErrorKind {
    fn from(value: ExchangeErrorKind) -> Self {
        match value {
            ExchangeErrorKind::Rejected => Self::Rejected,
            ExchangeErrorKind::RateLimited => Self::RateLimited,
            ExchangeErrorKind::Unavailable => Self::Unavailable,
            ExchangeErrorKind::Unknown => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

impl From<WireExchangeErrorKind> for ExchangeErrorKind {
    fn from(value: WireExchangeErrorKind) -> Self {
        match value {
            WireExchangeErrorKind::Rejected => Self::Rejected,
            WireExchangeErrorKind::RateLimited => Self::RateLimited,
            WireExchangeErrorKind::Unavailable => Self::Unavailable,
            WireExchangeErrorKind::Unknown => Self::Unknown,
        }
    }
}

impl From<TransferErrorKind> for WireTransferErrorKind {
    fn from(value: TransferErrorKind) -> Self {
        match value {
            TransferErrorKind::AssetMismatch => Self::AssetMismatch,
            TransferErrorKind::NetworkMismatch => Self::NetworkMismatch,
            TransferErrorKind::AmbiguousNetwork => Self::AmbiguousNetwork,
            TransferErrorKind::NetworkUnavailable => Self::NetworkUnavailable,
            TransferErrorKind::MemoRequired => Self::MemoRequired,
            TransferErrorKind::DestinationUnavailable => Self::DestinationUnavailable,
            TransferErrorKind::AddressNotAllowed => Self::AddressNotAllowed,
            TransferErrorKind::TravelRuleRequired => Self::TravelRuleRequired,
            TransferErrorKind::AmountOutOfRange => Self::AmountOutOfRange,
            TransferErrorKind::PlanExpired => Self::PlanExpired,
            _ => unreachable!("new transfer errors require a Dart wire variant"),
        }
    }
}

impl From<WireTransferErrorKind> for TransferErrorKind {
    fn from(value: WireTransferErrorKind) -> Self {
        match value {
            WireTransferErrorKind::AssetMismatch => Self::AssetMismatch,
            WireTransferErrorKind::NetworkMismatch => Self::NetworkMismatch,
            WireTransferErrorKind::AmbiguousNetwork => Self::AmbiguousNetwork,
            WireTransferErrorKind::NetworkUnavailable => Self::NetworkUnavailable,
            WireTransferErrorKind::MemoRequired => Self::MemoRequired,
            WireTransferErrorKind::DestinationUnavailable => Self::DestinationUnavailable,
            WireTransferErrorKind::AddressNotAllowed => Self::AddressNotAllowed,
            WireTransferErrorKind::TravelRuleRequired => Self::TravelRuleRequired,
            WireTransferErrorKind::AmountOutOfRange => Self::AmountOutOfRange,
            WireTransferErrorKind::PlanExpired => Self::PlanExpired,
        }
    }
}

impl From<Error> for NativeError {
    fn from(value: Error) -> Self {
        let message = value.to_string();
        let retryable = value.is_retryable();
        let rate_limited = value.is_rate_limited();
        let mut error = Self {
            kind: NativeErrorKind::Decode,
            message,
            detail: None,
            field: None,
            transfer_kind: None,
            feature: None,
            exchange: None,
            code: None,
            status: None,
            exchange_kind: None,
            retryable,
            rate_limited,
        };
        match value {
            Error::InvalidRequest { field, detail } => {
                error.kind = NativeErrorKind::InvalidRequest;
                error.field = Some(field.to_owned());
                error.detail = Some(detail);
            }
            Error::Transfer { kind, detail } => {
                error.kind = NativeErrorKind::Transfer;
                error.transfer_kind = Some(kind.into());
                error.detail = Some(detail);
            }
            Error::Unsupported {
                feature,
                exchange,
                detail,
            } => {
                error.kind = NativeErrorKind::Unsupported;
                error.feature = Some(feature.into());
                error.exchange = Some(exchange.to_owned());
                error.detail = Some(detail);
            }
            Error::Adapter { detail } => {
                error.kind = NativeErrorKind::Adapter;
                error.detail = Some(detail);
            }
            Error::Auth { detail } => {
                error.kind = NativeErrorKind::Auth;
                error.detail = Some(detail);
            }
            Error::Exchange {
                exchange,
                code,
                message,
                status,
                kind,
                ..
            } => {
                error.kind = NativeErrorKind::Exchange;
                error.exchange = Some(exchange.to_owned());
                error.code = Some(code);
                error.detail = Some(message);
                error.status = status;
                error.exchange_kind = Some(kind.into());
            }
            Error::Transport { detail } => {
                error.kind = NativeErrorKind::Transport;
                error.detail = Some(detail);
            }
            Error::Decode { detail } => {
                error.kind = NativeErrorKind::Decode;
                error.detail = Some(detail);
            }
            _ => {
                error.kind = NativeErrorKind::Adapter;
                error.detail = Some("unrecognized maxt error variant".to_owned());
            }
        }
        error
    }
}

fn known_error_exchange(value: &str) -> Option<&'static str> {
    Some(match value {
        "upbit" => "upbit",
        "bithumb" => "bithumb",
        "binance" => "binance",
        "hyperliquid" => "hyperliquid",
        _ => return None,
    })
}

impl TryFrom<NativeError> for Error {
    type Error = Error;

    fn try_from(value: NativeError) -> Result<Self, Self::Error> {
        let NativeError {
            kind,
            message,
            detail,
            field,
            transfer_kind,
            feature,
            exchange,
            code,
            status,
            exchange_kind,
            retryable: _,
            rate_limited: _,
        } = value;
        let detail = detail.unwrap_or(message);
        match kind {
            NativeErrorKind::InvalidRequest => {
                let field = field
                    .ok_or_else(|| Error::adapter("foreign invalid-request error has no field"))?;
                Ok(Error::InvalidRequest { field, detail })
            }
            NativeErrorKind::Transfer => {
                let kind = transfer_kind.ok_or_else(|| {
                    Error::adapter("foreign transfer error has no transfer category")
                })?;
                Ok(Error::Transfer {
                    kind: kind.into(),
                    detail,
                })
            }
            NativeErrorKind::Unsupported => {
                let feature = feature
                    .ok_or_else(|| Error::adapter("foreign unsupported error has no feature"))?;
                let exchange = exchange
                    .as_deref()
                    .and_then(known_error_exchange)
                    .ok_or_else(|| {
                        Error::adapter("foreign unsupported error has no known exchange")
                    })?;
                Ok(Error::Unsupported {
                    feature: feature.into(),
                    exchange,
                    detail,
                })
            }
            NativeErrorKind::Adapter => Ok(Error::adapter(detail)),
            NativeErrorKind::Auth => Ok(Error::Auth { detail }),
            NativeErrorKind::Exchange => {
                let exchange = exchange
                    .as_deref()
                    .and_then(known_error_exchange)
                    .ok_or_else(|| {
                        Error::adapter("foreign exchange error has no known exchange")
                    })?;
                let code = code
                    .ok_or_else(|| Error::adapter("foreign exchange error has no provider code"))?;
                let kind = exchange_kind.ok_or_else(|| {
                    Error::adapter("foreign exchange error has no retry classification")
                })?;
                Ok(Error::Exchange {
                    exchange,
                    code,
                    message: detail,
                    status,
                    kind: kind.into(),
                })
            }
            NativeErrorKind::Transport => Ok(Error::Transport { detail }),
            NativeErrorKind::Decode => Ok(Error::Decode { detail }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_input은_표현할_수_없는_값을_반올림하지_않는다() {
        let rejected = [
            "2.5e-28",
            "0.00000000000000000000000000001",
            "79228162514264337593543950335.4",
        ]
        .map(|value| decimal_from_wire(value, "price").is_err());

        assert_eq!(rejected, [true, true, true]);
        assert_eq!(
            decimal_from_wire("8.428e-05", "price").unwrap().to_string(),
            "0.00008428",
        );
    }

    #[test]
    fn decimal과_timestamp는_정밀도를_잃지_않는다() {
        assert_eq!(decimal_to_wire(Decimal::new(12_300, 4)), "1.2300");
        assert_eq!(
            timestamp_to_wire(Timestamp::from_nanos(1_700_000_000_000_000_123)),
            1_700_000_000_000_000_123,
        );
    }

    #[test]
    fn public_wire_trade도_decimal과_timestamp를_그대로_보존한다() {
        let wire = WireTrade::from(Trade {
            market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
            timestamp: Timestamp::from_nanos(1_700_000_000_000_000_123),
            price: Decimal::new(12_300, 4),
            quantity: Decimal::new(10, 8),
            taker_side: Side::Buy,
            id: Some("trade-1".to_owned()),
        });

        assert_eq!(wire.price, "1.2300");
        assert_eq!(wire.quantity, "0.00000010");
        assert_eq!(wire.timestamp_ns, 1_700_000_000_000_000_123);
    }

    #[test]
    fn malformed_order_shape는_native_error로_거절된다() {
        let request = WireOrderRequest {
            market: WireMarket {
                exchange: WireExchange::Upbit,
                kind: WireMarketKind::Spot,
                base: "BTC".to_owned(),
                quote: "KRW".to_owned(),
            },
            side: WireSide::Buy,
            order_type: WireOrderType::Limit,
            size: WireSize {
                kind: WireSizeKind::Base,
                value: "0.1".to_owned(),
            },
            price: None,
            time_in_force: None,
            reduce_only: false,
        };

        let error = OrderRequest::try_from(request).unwrap_err();
        assert_eq!(error.kind, NativeErrorKind::InvalidRequest);
        assert_eq!(error.field.as_deref(), Some("price"));
    }

    #[test]
    fn structured_error는_variant와_raw_detail을_왕복한다() {
        let original = Error::Adapter {
            detail: "dispatcher stopped".to_owned(),
        };
        let wire = NativeError::from(original.clone());

        assert_eq!(wire.kind, NativeErrorKind::Adapter);
        assert_eq!(wire.detail.as_deref(), Some("dispatcher stopped"));
        assert_eq!(Error::try_from(wire).unwrap(), original);
    }

    #[test]
    fn invalid_request의_동적_field를_손실_없이_왕복한다() {
        let original = Error::InvalidRequest {
            field: "provider_specific_parameter".to_owned(),
            detail: "must be positive".to_owned(),
        };
        let wire = NativeError::from(original.clone());

        assert_eq!(wire.field.as_deref(), Some("provider_specific_parameter"));
        assert_eq!(Error::try_from(wire).unwrap(), original);
    }
}
