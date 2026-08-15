use maxt::adapters::{
    BinanceAccountTrade, BinanceAggregateTrade, BinanceAggregateTradesRequest,
    BinanceApiKeyPermissions, BinanceC2cTrade, BinanceC2cTradeHistoryPage,
    BinanceC2cTradeHistoryRequest, BinanceC2cTradeType, BinanceCoinInformation,
    BinanceCoinNetworkInformation, BinanceDepositHistory, BinanceDepositHistoryEntry,
    BinanceDepositHistoryRequest, BinanceExchangeInfo, BinanceExchangeSymbol, BinanceMarkPrice,
    BinanceMarket, BinanceOpenInterest, BinanceQuestionnaireRequirements,
    BinanceSpotAccountBalance, BinanceSpotAccountInformation, BinanceSpotCancelAllOpenOrders,
    BinanceSpotCancelledOrder, BinanceSpotCommissionRates, BinanceSpotOrderDetail,
    BinanceSymbolFilters, BinanceTestOrder, BinanceTestOrderRequest, BinanceUsdMAccountAsset,
    BinanceUsdMAccountInformation, BinanceUsdMAccountPosition, BinanceUsdMPositionInformation,
    BinanceWithdrawHistory, BinanceWithdrawHistoryEntry, BinanceWithdrawHistoryRequest,
    BinanceWithdrawalAddress, BithumbAlertStep, BithumbApiKey, BithumbAssetFee, BithumbBatchOrder,
    BithumbBatchOrderFailure, BithumbBatchOrderOutcome, BithumbBatchOrdersRequest,
    BithumbBatchOrdersResult, BithumbClosedOrder, BithumbClosedOrderState,
    BithumbClosedOrdersRequest, BithumbKrwDeposit, BithumbKrwDepositsRequest,
    BithumbKrwTransferRequest, BithumbKrwWithdrawal, BithumbKrwWithdrawalsRequest,
    BithumbMarketAlert, BithumbNetworkFee, BithumbNotice, BithumbOrderDetail,
    BithumbOrderDetailRequest, BithumbOrderDetailTrade, BithumbOrderDirection,
    BithumbOrderListItem, BithumbOrderListRequest, BithumbOrderListState, BithumbPendingOrderState,
    BithumbPendingOrdersRequest, BithumbTwapOrder, BithumbTwapOrderDirection,
    BithumbTwapOrderRequest, BithumbTwapOrdersRequest, BithumbTwapState, BithumbWithdrawalAddress,
    HyperliquidAssetContext, HyperliquidBookLevel, HyperliquidCandleSnapshot,
    HyperliquidDailyVolume, HyperliquidEvmContract, HyperliquidFundingHistoryEntry,
    HyperliquidL2Book, HyperliquidLedgerEntry, HyperliquidLedgerKind, HyperliquidMidPrice,
    HyperliquidOpenOrder, HyperliquidOrderDetail, HyperliquidOrderInfo, HyperliquidOrderReference,
    HyperliquidOrderStatusResponse, HyperliquidPortfolioPeriod, HyperliquidPortfolioPoint,
    HyperliquidRecentTrade, HyperliquidReferral, HyperliquidReferrer, HyperliquidSpotAssetContext,
    HyperliquidSpotBalance, HyperliquidSpotClearinghouseState, HyperliquidSpotMeta,
    HyperliquidSpotMetaAndAssetContexts, HyperliquidSpotPair, HyperliquidSpotToken,
    HyperliquidSubAccount, HyperliquidUserFees, HyperliquidUserFill, HyperliquidUserFunding,
    HyperliquidUserRateLimit, HyperliquidUserRole, HyperliquidVaultEquity, UpbitApiKey,
    UpbitBatchCancelRequest, UpbitBatchCancelScope, UpbitCancelAndNewOrder,
    UpbitCancelAndNewOrderRequest, UpbitCancelAndNewOrderResult, UpbitClosedOrder,
    UpbitClosedOrderState, UpbitClosedOrdersRequest, UpbitDepositInfo, UpbitKrwDeposit,
    UpbitKrwTransferRequest, UpbitKrwTwoFactorType, UpbitKrwWithdrawal, UpbitMarketEvent,
    UpbitOrderBookInstrument, UpbitOrderDetail, UpbitOrderDetailRequest, UpbitOrderDetailTrade,
    UpbitOrderDirection, UpbitOrderReference, UpbitOrderVolume, UpbitPocket, UpbitPocketApiKey,
    UpbitPocketApiKeyGroup, UpbitPocketApiKeysRequest, UpbitPocketBalance, UpbitPocketTransfer,
    UpbitPocketTransferDirection, UpbitPocketTransferOrder, UpbitPocketTransferQuery,
    UpbitPocketTransferRequest, UpbitPocketTransferState, UpbitPocketUniversalTransferRequest,
    UpbitSmpType, UpbitTravelRuleVasp, UpbitTravelRuleVerification, UpbitWithdrawalAddress,
    UpbitYearCandle,
};
use maxt::{
    AccountEvent, AssetNetwork, Balance, CancelOrdersRequest, CancelOrdersResult, CancelledOrder,
    Candle, CandleRequest, ChainDestination, ChainTransferRequest, Cursor, Decimal, Deposit,
    DepositAddress, DepositAddressEntry, DepositAddressRequest, DepositStatus, Error, Exchange,
    ExchangeDestination, ExchangeErrorKind, ExchangeTransferRequest, Feature, Feed, FundingPayment,
    FundingRate, HistoryRequest, Interval, Level, MarginMode, MarginRequest, MarginSummary, Market,
    MarketEvent, MarketInfo, MarketKind, MarketStatus, Network, Order, OrderAccount, OrderBook,
    OrderCancelFailure, OrderHistoryRequest, OrderIdKind, OrderLookupRequest, OrderOption,
    OrderRequest, OrderRules, OrderStatus, OrderType, Overflow, Page, Position, Side, Size,
    StreamConfig, Subscription, Ticker, TimeInForce, Timestamp, Trade, TransferDestination,
    TransferErrorKind, TransferHistoryRequest, TransferLookupRequest, TransferPlan,
    TravelRuleRequirement, WithdrawRequest, Withdrawal, WithdrawalFee, WithdrawalQuote,
    WithdrawalStatus,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_WIRE_JSON_DEPTH: usize = 64;
const MAX_JSON_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireMarket {
    pub(crate) exchange: String,
    pub(crate) kind: String,
    pub(crate) base: String,
    pub(crate) quote: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireSize {
    Base { value: String },
    Quote { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireCandleRequest {
    pub(crate) market: WireMarket,
    pub(crate) interval: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) from: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) to: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderRequest {
    pub(crate) market: WireMarket,
    pub(crate) side: String,
    pub(crate) order_type: String,
    pub(crate) size: WireSize,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    pub(crate) reduce_only: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderLookupRequest {
    pub(crate) kind: String,
    pub(crate) ids: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireCancelOrdersRequest {
    pub(crate) kind: String,
    pub(crate) ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderHistoryRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    pub(crate) statuses: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) from: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) to: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cursor: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireDepositAddressRequest {
    pub(crate) asset: String,
    pub(crate) network: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireWithdrawRequest {
    pub(crate) asset: String,
    pub(crate) network: String,
    pub(crate) amount: String,
    pub(crate) destination: WireTransferDestination,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireTransferLookupRequest {
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireTransferHistoryRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) asset: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cursor: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireFeed {
    Trades,
    OrderBook,
    Ticker,
    Candles { interval: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireMarketEvent {
    Trade { trade: WireTrade },
    OrderBook { order_book: WireOrderBook },
    Ticker { ticker: WireTicker },
    Candle { candle: WireCandle },
    Reconnected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireAccountEvent {
    Balance { balance: WireBalance },
    Order { order: WireOrder },
    Reconnected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireMarketStreamItem {
    Event { event: Box<WireMarketEvent> },
    Error { error: WireError },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireAccountStreamItem {
    Event { event: WireAccountEvent },
    Error { error: WireError },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireSubscription {
    pub(crate) markets: Vec<WireMarket>,
    pub(crate) feeds: Vec<WireFeed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitListedSubscription {
    pub(crate) feed_type: String,
    pub(crate) markets: Vec<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitSubscriptionList {
    pub(crate) ticket: String,
    pub(crate) subscriptions: Vec<WireUpbitListedSubscription>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireStreamConfig {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) max_reconnect_attempts: Option<u32>,
    pub(crate) initial_reconnect_delay_ms: String,
    pub(crate) max_reconnect_delay_ms: String,
    pub(crate) idle_timeout_ms: String,
    pub(crate) buffer_size: String,
    pub(crate) overflow: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHistoryRequest {
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) from: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) to: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cursor: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireMarginRequest {
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) leverage: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) margin_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
pub(crate) struct WireUpbitMarketEvent {
    pub(crate) warning: bool,
    pub(crate) cautions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitYearCandle {
    pub(crate) market: WireMarket,
    pub(crate) open_time: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) korea_open_time: Option<String>,
    pub(crate) timestamp: String,
    pub(crate) open: String,
    pub(crate) high: String,
    pub(crate) low: String,
    pub(crate) close: String,
    pub(crate) volume: String,
    pub(crate) quote_volume: String,
    pub(crate) first_day_of_period: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitOrderBookInstrument {
    pub(crate) market: WireMarket,
    pub(crate) quote_currency: String,
    pub(crate) tick_size: String,
    pub(crate) supported_levels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitOrderDetailRequest {
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) uuid: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitOrderDetailTrade {
    pub(crate) market: WireMarket,
    pub(crate) uuid: String,
    pub(crate) price: String,
    pub(crate) volume: String,
    pub(crate) funds: String,
    pub(crate) trend: String,
    pub(crate) created_at: String,
    pub(crate) side: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitOrderDetail {
    pub(crate) market: WireMarket,
    pub(crate) uuid: String,
    pub(crate) side: String,
    pub(crate) order_type: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    pub(crate) state: String,
    pub(crate) created_at: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) volume: Option<String>,
    pub(crate) remaining_volume: String,
    pub(crate) executed_volume: String,
    pub(crate) reserved_fee: String,
    pub(crate) remaining_fee: String,
    pub(crate) paid_fee: String,
    pub(crate) locked: String,
    pub(crate) trades_count: u32,
    pub(crate) prevented_volume: String,
    pub(crate) prevented_locked: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) identifier: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) smp_type: Option<String>,
    pub(crate) trades: Vec<WireUpbitOrderDetailTrade>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitClosedOrdersRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    pub(crate) states: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitClosedOrder {
    pub(crate) market: WireMarket,
    pub(crate) uuid: String,
    pub(crate) side: String,
    pub(crate) ord_type: String,
    pub(crate) state: String,
    pub(crate) created_at: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) volume: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    pub(crate) remaining_volume: String,
    pub(crate) executed_volume: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) executed_funds: Option<String>,
    pub(crate) reserved_fee: String,
    pub(crate) remaining_fee: String,
    pub(crate) paid_fee: String,
    pub(crate) locked: String,
    pub(crate) trades_count: u32,
    pub(crate) prevented_volume: String,
    pub(crate) prevented_locked: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) identifier: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) smp_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitDepositInfo {
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) provider_network: Option<String>,
    pub(crate) is_deposit_possible: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) deposit_impossible_reason: Option<String>,
    pub(crate) minimum_deposit_amount: String,
    pub(crate) minimum_deposit_confirmations: String,
    pub(crate) decimal_precision: String,
}

#[allow(dead_code, reason = "provider bridge output DTO")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitWithdrawalAddress {
    pub(crate) currency: String,
    pub(crate) net_type: String,
    pub(crate) network_name: String,
    pub(crate) withdraw_address: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) secondary_address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) beneficiary_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) beneficiary_company_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) beneficiary_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) exchange_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) wallet_type: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitTravelRuleVasp {
    pub(crate) vasp_name: String,
    pub(crate) vasp_uuid: String,
    pub(crate) depositable: bool,
    pub(crate) withdrawable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitTravelRuleVerification {
    pub(crate) deposit_uuid: String,
    pub(crate) deposit_state: String,
    pub(crate) verification_result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireUpbitBatchCancelScope {
    All,
    QuoteCurrencies { values: Vec<String> },
    Pairs { values: Vec<WireMarket> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitBatchCancelRequest {
    pub(crate) scope: WireUpbitBatchCancelScope,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) excluded_pairs: Option<Vec<WireMarket>>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) side: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) count: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireUpbitOrderReference {
    Uuid { value: String },
    Identifier { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireUpbitOrderVolume {
    Amount { value: String },
    RemainOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireUpbitCancelAndNewOrder {
    Limit {
        volume: WireUpbitOrderVolume,
        price: String,
        #[serde(deserialize_with = "explicit_option")]
        time_in_force: Option<String>,
    },
    MarketBuy {
        price: String,
    },
    MarketSell {
        volume: WireUpbitOrderVolume,
    },
    BestBuy {
        price: String,
        time_in_force: String,
    },
    BestSell {
        volume: WireUpbitOrderVolume,
        time_in_force: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitCancelAndNewOrderRequest {
    pub(crate) previous_order: WireUpbitOrderReference,
    pub(crate) new_order: WireUpbitCancelAndNewOrder,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) new_identifier: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) new_smp_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitCancelAndNewOrderResult {
    pub(crate) previous_order: WireOrder,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) new_order_uuid: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) new_order_identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitKrwTransferRequest {
    pub(crate) amount: String,
    pub(crate) two_factor_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitKrwDeposit {
    pub(crate) transfer_type: String,
    pub(crate) uuid: String,
    pub(crate) currency: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) net_type: Option<String>,
    pub(crate) txid: String,
    pub(crate) state: String,
    pub(crate) created_at: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) done_at: Option<String>,
    pub(crate) amount: String,
    pub(crate) fee: String,
    pub(crate) transaction_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitKrwWithdrawal {
    pub(crate) transfer_type: String,
    pub(crate) uuid: String,
    pub(crate) currency: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) net_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) txid: Option<String>,
    pub(crate) state: String,
    pub(crate) created_at: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) done_at: Option<String>,
    pub(crate) amount: String,
    pub(crate) fee: String,
    pub(crate) transaction_type: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) is_cancelable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitApiKey {
    pub(crate) access_key: String,
    pub(crate) expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocket {
    pub(crate) uuid: String,
    pub(crate) name: String,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketApiKey {
    pub(crate) access_key: String,
    pub(crate) permissions: Vec<String>,
    pub(crate) allowed_ips: Vec<String>,
    pub(crate) created_at: String,
    pub(crate) expired_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketApiKeyGroup {
    pub(crate) uuid: String,
    pub(crate) keys: Vec<WireUpbitPocketApiKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketApiKeysRequest {
    pub(crate) uuids: Vec<String>,
    pub(crate) include_expired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketBalance {
    pub(crate) currency: String,
    pub(crate) balance: String,
    pub(crate) locked: String,
    pub(crate) avg_buy_price: String,
    pub(crate) avg_buy_price_modified: bool,
    pub(crate) unit_currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketTransferQuery {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) from: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) to: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) direction: Option<String>,
    pub(crate) states: Vec<String>,
    pub(crate) uuids: Vec<String>,
    pub(crate) identifiers: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) currency: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketUniversalTransferRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) from: Option<String>,
    pub(crate) to: String,
    pub(crate) currency: String,
    pub(crate) amount: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketTransferRequest {
    pub(crate) to: String,
    pub(crate) currency: String,
    pub(crate) amount: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireUpbitPocketTransfer {
    pub(crate) uuid: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) identifier: Option<String>,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) state: String,
    pub(crate) currency: String,
    pub(crate) amount: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
pub(crate) struct WireBithumbMarketAlert {
    pub(crate) kind: String,
    pub(crate) step: String,
    pub(crate) ends_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbNotice {
    pub(crate) categories: Vec<String>,
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) published_at: String,
    pub(crate) modified_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbApiKey {
    pub(crate) access_key: String,
    pub(crate) expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbKrwWithdrawalsRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    pub(crate) uuids: Vec<String>,
    pub(crate) txids: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) page: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbKrwDepositsRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    pub(crate) uuids: Vec<String>,
    pub(crate) txids: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) page: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbKrwTransferRequest {
    pub(crate) amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbKrwWithdrawal {
    pub(crate) transfer_type: String,
    pub(crate) uuid: String,
    pub(crate) currency: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) net_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) txid: Option<String>,
    pub(crate) state: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) done_at: Option<String>,
    pub(crate) amount: String,
    pub(crate) fee: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) transaction_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbKrwDeposit {
    pub(crate) transfer_type: String,
    pub(crate) uuid: String,
    pub(crate) currency: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) net_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) txid: Option<String>,
    pub(crate) state: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) done_at: Option<String>,
    pub(crate) amount: String,
    pub(crate) fee: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) transaction_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbPendingOrdersRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbClosedOrdersRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    pub(crate) states: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbClosedOrder {
    pub(crate) order_id: String,
    pub(crate) side: String,
    pub(crate) order_type: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    pub(crate) state: String,
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
    pub(crate) volume: String,
    pub(crate) remaining_volume: String,
    pub(crate) reserved_fee: String,
    pub(crate) remaining_fee: String,
    pub(crate) paid_fee: String,
    pub(crate) locked: String,
    pub(crate) executed_volume: String,
    pub(crate) executed_funds: String,
    pub(crate) trades_count: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) stp_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cancel_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) canceling_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbBatchOrdersRequest {
    pub(crate) orders: Vec<WireOrderRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbBatchOrder {
    pub(crate) order_id: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    pub(crate) market: WireMarket,
    pub(crate) side: String,
    pub(crate) order_type: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) stp_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbBatchOrderFailure {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    pub(crate) code: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireBithumbBatchOrderOutcome {
    Accepted { value: WireBithumbBatchOrder },
    Rejected { value: WireBithumbBatchOrderFailure },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbBatchOrdersResult {
    pub(crate) outcomes: Vec<WireBithumbBatchOrderOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbTwapOrdersRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    pub(crate) uuids: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cursor: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbTwapOrderRequest {
    pub(crate) market: WireMarket,
    pub(crate) side: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) volume: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    pub(crate) duration: u32,
    pub(crate) frequency: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbTwapOrder {
    pub(crate) id: String,
    pub(crate) side: String,
    pub(crate) price: String,
    pub(crate) state: String,
    pub(crate) market: WireMarket,
    pub(crate) created_at: String,
    pub(crate) volume: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) finished_at: Option<String>,
    pub(crate) total_order_count: u32,
    pub(crate) total_trades_count: u32,
    pub(crate) progress_count: u32,
    pub(crate) total_executed_amount: String,
    pub(crate) total_executed_volume: String,
    pub(crate) avg_trade_price: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) wallet_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) canceled_at: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cancel_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbAssetFee {
    pub(crate) display_name: String,
    pub(crate) asset: String,
    pub(crate) networks: Vec<WireBithumbNetworkFee>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbNetworkFee {
    pub(crate) network: String,
    pub(crate) provider_name: String,
    pub(crate) deposit_fee: String,
    pub(crate) minimum_deposit: String,
    pub(crate) withdrawal_fee: WireWithdrawalFee,
    pub(crate) minimum_withdrawal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbWithdrawalAddress {
    pub(crate) currency: String,
    pub(crate) net_type: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network_name: Option<String>,
    pub(crate) withdraw_address: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) secondary_address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) exchange_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) owner_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) owner_ko_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) owner_en_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) owner_corp_ko_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) owner_corp_en_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbOrderDetailRequest {
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) uuid: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbOrderDetailTrade {
    pub(crate) market: WireMarket,
    pub(crate) uuid: String,
    pub(crate) price: String,
    pub(crate) volume: String,
    pub(crate) funds: String,
    pub(crate) side: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbOrderDetail {
    pub(crate) uuid: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    pub(crate) side: String,
    pub(crate) order_type: String,
    pub(crate) price: String,
    pub(crate) state: String,
    pub(crate) market: WireMarket,
    pub(crate) created_at: String,
    pub(crate) volume: String,
    pub(crate) remaining_volume: String,
    pub(crate) reserved_fee: String,
    pub(crate) remaining_fee: String,
    pub(crate) paid_fee: String,
    pub(crate) locked: String,
    pub(crate) executed_volume: String,
    pub(crate) executed_funds: String,
    pub(crate) trades_count: u32,
    pub(crate) trades: Vec<WireBithumbOrderDetailTrade>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) stp_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cancel_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) canceling_uuid: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbOrderListRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) state: Option<String>,
    pub(crate) states: Vec<String>,
    pub(crate) uuids: Vec<String>,
    pub(crate) client_order_ids: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) page: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBithumbOrderListItem {
    pub(crate) uuid: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    pub(crate) side: String,
    pub(crate) order_type: String,
    pub(crate) price: String,
    pub(crate) state: String,
    pub(crate) market: WireMarket,
    pub(crate) created_at: String,
    pub(crate) volume: String,
    pub(crate) remaining_volume: String,
    pub(crate) reserved_fee: String,
    pub(crate) remaining_fee: String,
    pub(crate) paid_fee: String,
    pub(crate) locked: String,
    pub(crate) executed_volume: String,
    pub(crate) executed_funds: String,
    pub(crate) trades_count: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) stp_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
pub(crate) struct WireBinanceSymbolFilters {
    pub(crate) symbol: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tick_size: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) min_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) max_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) step_size: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) min_quantity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) max_quantity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) min_notional: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
pub(crate) struct WireBinanceSpotOrderDetail {
    pub(crate) order: WireOrder,
    pub(crate) client_order_id: String,
    pub(crate) order_type: String,
    pub(crate) time_in_force: String,
    pub(crate) filled_quote_quantity: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceSpotAveragePrice {
    pub(crate) market: WireMarket,
    pub(crate) minutes: u32,
    pub(crate) price: String,
    pub(crate) close_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceDepositHistoryRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) coin: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) status: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) offset: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_id: Option<String>,
    pub(crate) include_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceWithdrawHistoryRequest {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) coin: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdraw_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) status: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) offset: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
    pub(crate) id_list: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceSpotCommissionRates {
    pub(crate) maker: String,
    pub(crate) taker: String,
    pub(crate) buyer: String,
    pub(crate) seller: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceSpotAccountBalance {
    pub(crate) asset: String,
    pub(crate) free: String,
    pub(crate) locked: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceSpotAccountInformation {
    pub(crate) maker_commission: String,
    pub(crate) taker_commission: String,
    pub(crate) buyer_commission: String,
    pub(crate) seller_commission: String,
    pub(crate) commission_rates: WireBinanceSpotCommissionRates,
    pub(crate) can_trade: bool,
    pub(crate) can_withdraw: bool,
    pub(crate) can_deposit: bool,
    pub(crate) update_time: String,
    pub(crate) account_type: String,
    pub(crate) balances: Vec<WireBinanceSpotAccountBalance>,
    pub(crate) permissions: Vec<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) uid: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceSpotCancelledOrder {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) symbol: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) original_client_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) status: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) original_quantity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) executed_quantity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cumulative_quote_quantity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) transact_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_list_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) contingency_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) list_status_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) list_order_status: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) list_client_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) transaction_time: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceSpotCancelAllOpenOrders {
    pub(crate) reports: Vec<WireBinanceSpotCancelledOrder>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceExchangeSymbol {
    pub(crate) symbol: String,
    pub(crate) status: String,
    pub(crate) base_asset: String,
    pub(crate) quote_asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) contract_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) margin_asset: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceExchangeInfo {
    pub(crate) venue: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) timezone: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) server_time: Option<String>,
    pub(crate) symbols: Vec<WireBinanceExchangeSymbol>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceUsdMAccountAsset {
    pub(crate) asset: String,
    pub(crate) wallet_balance: String,
    pub(crate) unrealized_profit: String,
    pub(crate) margin_balance: String,
    pub(crate) maintenance_margin: String,
    pub(crate) initial_margin: String,
    pub(crate) position_initial_margin: String,
    pub(crate) open_order_initial_margin: String,
    pub(crate) cross_wallet_balance: String,
    pub(crate) cross_unrealized_profit: String,
    pub(crate) available_balance: String,
    pub(crate) max_withdraw_amount: String,
    pub(crate) update_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceUsdMAccountPosition {
    pub(crate) symbol: String,
    pub(crate) position_side: String,
    pub(crate) position_amount: String,
    pub(crate) unrealized_profit: String,
    pub(crate) isolated_margin: String,
    pub(crate) notional: String,
    pub(crate) isolated_wallet: String,
    pub(crate) initial_margin: String,
    pub(crate) maintenance_margin: String,
    pub(crate) update_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceUsdMAccountInformation {
    pub(crate) total_initial_margin: String,
    pub(crate) total_maintenance_margin: String,
    pub(crate) total_wallet_balance: String,
    pub(crate) total_unrealized_profit: String,
    pub(crate) total_margin_balance: String,
    pub(crate) total_position_initial_margin: String,
    pub(crate) total_open_order_initial_margin: String,
    pub(crate) total_cross_wallet_balance: String,
    pub(crate) total_cross_unrealized_profit: String,
    pub(crate) available_balance: String,
    pub(crate) max_withdraw_amount: String,
    pub(crate) assets: Vec<WireBinanceUsdMAccountAsset>,
    pub(crate) positions: Vec<WireBinanceUsdMAccountPosition>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceUsdMPositionInformation {
    pub(crate) symbol: String,
    pub(crate) position_side: String,
    pub(crate) position_amount: String,
    pub(crate) entry_price: String,
    pub(crate) break_even_price: String,
    pub(crate) mark_price: String,
    pub(crate) unrealized_profit: String,
    pub(crate) liquidation_price: String,
    pub(crate) isolated_margin: String,
    pub(crate) notional: String,
    pub(crate) margin_asset: String,
    pub(crate) isolated_wallet: String,
    pub(crate) initial_margin: String,
    pub(crate) maintenance_margin: String,
    pub(crate) position_initial_margin: String,
    pub(crate) open_order_initial_margin: String,
    pub(crate) adl: String,
    pub(crate) bid_notional: String,
    pub(crate) ask_notional: String,
    pub(crate) update_time: String,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceCoinNetworkInformation {
    pub(crate) network: String,
    pub(crate) deposit_enabled: bool,
    pub(crate) withdraw_enabled: bool,
    pub(crate) busy: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdrawal_integer_multiple: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdrawal_fee: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) minimum_withdrawal: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) maximum_withdrawal: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdrawal_tag: Option<bool>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) is_default: Option<bool>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) minimum_confirmations: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) unlock_confirmations: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) contract_address: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceCoinInformation {
    pub(crate) coin: String,
    pub(crate) deposit_all_enabled: bool,
    pub(crate) withdraw_all_enabled: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) free: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) locked: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) freeze: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdrawing: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) is_legal_money: Option<bool>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) trading: Option<bool>,
    pub(crate) networks: Vec<WireBinanceCoinNetworkInformation>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceApiKeyPermissions {
    pub(crate) ip_restrict: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) create_time: Option<String>,
    pub(crate) enable_reading: bool,
    pub(crate) enable_withdrawals: bool,
    pub(crate) enable_internal_transfer: bool,
    pub(crate) enable_margin: bool,
    pub(crate) enable_spot_and_margin_trading: bool,
    pub(crate) enable_futures: bool,
    pub(crate) permits_universal_transfer: bool,
    pub(crate) enable_vanilla_options: bool,
    pub(crate) enable_fix_api_trade: bool,
    pub(crate) enable_fix_read_only: bool,
    pub(crate) enable_portfolio_margin_trading: bool,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceDepositHistoryEntry {
    pub(crate) id: String,
    pub(crate) amount: String,
    pub(crate) coin: String,
    pub(crate) network: String,
    pub(crate) status: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address_tag: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_id: Option<String>,
    pub(crate) insert_time: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) complete_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) transfer_type: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) source_address: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceDepositHistory {
    pub(crate) entries: Vec<WireBinanceDepositHistoryEntry>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceQuestionnaireRequirements {
    pub(crate) questionnaire_country_code: String,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceWithdrawalAddress {
    pub(crate) address: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address_tag: Option<String>,
    pub(crate) coin: String,
    pub(crate) network: String,
    pub(crate) white_status: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) origin: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) origin_type: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceWithdrawHistoryEntry {
    pub(crate) id: String,
    pub(crate) amount: String,
    pub(crate) transaction_fee: String,
    pub(crate) coin: String,
    pub(crate) status: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) apply_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdraw_order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) info: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) transfer_type: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) confirm_no: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) wallet_type: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_key: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) complete_time: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceWithdrawHistory {
    pub(crate) entries: Vec<WireBinanceWithdrawHistoryEntry>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceMarkPrice {
    pub(crate) market: WireMarket,
    pub(crate) mark_price: String,
    pub(crate) index_price: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) estimated_settle_price: Option<String>,
    pub(crate) last_funding_rate: String,
    pub(crate) interest_rate: String,
    pub(crate) next_funding_time: String,
    pub(crate) time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceOpenInterest {
    pub(crate) market: WireMarket,
    pub(crate) open_interest: String,
    pub(crate) time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceAggregateTradesRequest {
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) from_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_time: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceAggregateTrade {
    pub(crate) market: WireMarket,
    pub(crate) aggregate_id: String,
    pub(crate) first_trade_id: String,
    pub(crate) last_trade_id: String,
    pub(crate) timestamp: String,
    pub(crate) price: String,
    pub(crate) quantity: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) normal_quantity: Option<String>,
    pub(crate) taker_side: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceAccountTrade {
    pub(crate) market: WireMarket,
    pub(crate) id: String,
    pub(crate) order_id: String,
    pub(crate) timestamp: String,
    pub(crate) side: String,
    pub(crate) maker: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) best_match: Option<bool>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_list_id: Option<String>,
    pub(crate) price: String,
    pub(crate) quantity: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) quote_quantity: Option<String>,
    pub(crate) commission: String,
    pub(crate) commission_asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) realized_pnl: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) position_side: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) pair: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) base_quantity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) margin_asset: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceTestOrderRequest {
    pub(crate) order: WireOrderRequest,
    pub(crate) compute_commission_rates: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceTestOrder {
    pub(crate) response_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceC2cTradeHistoryRequest {
    pub(crate) trade_type: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) start_timestamp: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) end_timestamp: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) page: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) rows: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) recv_window: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceC2cTrade {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_number: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) adv_no: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) trade_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) asset: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) fiat: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) fiat_symbol: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) amount: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) total_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) unit_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_status: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) commission: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) counterparty_nickname: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) pay_method_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) additional_kyc_verify: Option<u32>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) taker_commission_rate: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) taker_commission: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) taker_amount: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) advertisement_role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBinanceC2cTradeHistoryPage {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) code: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) message: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) data: Option<Vec<WireBinanceC2cTrade>>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) total: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) success: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
pub(crate) struct WireHyperliquidLedgerEntry {
    pub(crate) kind: String,
    pub(crate) time: String,
    pub(crate) hash: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) asset: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) amount: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) fee: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) counterparty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
pub(crate) struct WireHyperliquidAssetContext {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) mid_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) mark_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) oracle_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) funding_rate: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) open_interest: Option<String>,
    pub(crate) size_decimals: u32,
    pub(crate) price_decimals: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidCandleSnapshot {
    pub(crate) coin: String,
    pub(crate) market: WireMarket,
    pub(crate) interval: String,
    pub(crate) open_time: String,
    pub(crate) close_time: String,
    pub(crate) open: String,
    pub(crate) high: String,
    pub(crate) low: String,
    pub(crate) close: String,
    pub(crate) volume: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) trade_count: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidBookLevel {
    pub(crate) price: String,
    pub(crate) size: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_count: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidL2Book {
    pub(crate) coin: String,
    pub(crate) market: WireMarket,
    pub(crate) time: String,
    pub(crate) bids: Vec<WireHyperliquidBookLevel>,
    pub(crate) asks: Vec<WireHyperliquidBookLevel>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidRecentTrade {
    pub(crate) coin: String,
    pub(crate) market: WireMarket,
    pub(crate) side: String,
    pub(crate) price: String,
    pub(crate) size: String,
    pub(crate) time: String,
    pub(crate) trade_id: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) hash: Option<String>,
    pub(crate) users: Vec<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidFundingHistoryEntry {
    pub(crate) coin: String,
    pub(crate) market: WireMarket,
    pub(crate) funding_rate: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) premium: Option<String>,
    pub(crate) time: String,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidUserFunding {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) kind: Option<String>,
    pub(crate) coin: String,
    pub(crate) market: WireMarket,
    pub(crate) usdc: String,
    pub(crate) funding_rate: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) position_size: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) sample_count: Option<String>,
    pub(crate) hash: String,
    pub(crate) time: String,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotBalance {
    pub(crate) coin: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) token: Option<u32>,
    pub(crate) total: String,
    pub(crate) hold: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) entry_notional: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotClearinghouseState {
    pub(crate) balances: Vec<WireHyperliquidSpotBalance>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidEvmContract {
    pub(crate) address: String,
    pub(crate) extra_wei_decimals: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotToken {
    pub(crate) name: String,
    pub(crate) size_decimals: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) wei_decimals: Option<u32>,
    pub(crate) index: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) token_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) is_canonical: Option<bool>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) evm_contract: Option<WireHyperliquidEvmContract>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) full_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) deployer_trading_fee_share: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotPair {
    pub(crate) name: String,
    pub(crate) tokens: Vec<u32>,
    pub(crate) index: u32,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) is_canonical: Option<bool>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotMeta {
    pub(crate) tokens: Vec<WireHyperliquidSpotToken>,
    pub(crate) universe: Vec<WireHyperliquidSpotPair>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotAssetContext {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) coin: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) mid_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) mark_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) previous_day_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) day_base_volume: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) day_notional_volume: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) circulating_supply: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) total_supply: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSpotMetaAndAssetContexts {
    pub(crate) meta: WireHyperliquidSpotMeta,
    pub(crate) contexts: Vec<WireHyperliquidSpotAssetContext>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidMidPrice {
    pub(crate) market: WireMarket,
    pub(crate) price: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidUserRateLimit {
    pub(crate) cumulative_volume: String,
    pub(crate) requests_used: String,
    pub(crate) requests_cap: String,
    pub(crate) requests_surplus: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireHyperliquidUserRole {
    User,
    Agent {
        #[serde(deserialize_with = "explicit_option")]
        user: Option<String>,
    },
    Vault,
    SubAccount {
        #[serde(deserialize_with = "explicit_option")]
        master: Option<String>,
    },
    Missing,
    Other {
        role: String,
        #[serde(deserialize_with = "explicit_option")]
        data_json: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidReferrer {
    pub(crate) address: String,
    pub(crate) code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidReferral {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) referred_by: Option<WireHyperliquidReferrer>,
    pub(crate) cumulative_volume: String,
    pub(crate) unclaimed_rewards: String,
    pub(crate) claimed_rewards: String,
    pub(crate) builder_rewards: String,
    pub(crate) referrer_state_json: String,
    pub(crate) reward_history_json: String,
    pub(crate) token_to_state_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidDailyVolume {
    pub(crate) date: String,
    pub(crate) user_cross: String,
    pub(crate) user_add: String,
    pub(crate) exchange: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidUserFees {
    pub(crate) daily_volumes: Vec<WireHyperliquidDailyVolume>,
    pub(crate) fee_schedule_json: String,
    pub(crate) user_cross_rate: String,
    pub(crate) user_add_rate: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) user_spot_cross_rate: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) user_spot_add_rate: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) active_referral_discount: Option<String>,
    pub(crate) details_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidUserFill {
    pub(crate) coin: String,
    pub(crate) price: String,
    pub(crate) size: String,
    pub(crate) side: String,
    pub(crate) time: String,
    pub(crate) start_position: String,
    pub(crate) direction: String,
    pub(crate) closed_pnl: String,
    pub(crate) hash: String,
    pub(crate) order_id: String,
    pub(crate) crossed: bool,
    pub(crate) fee: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) builder_fee: Option<String>,
    pub(crate) trade_id: String,
    pub(crate) fee_token: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) twap_id: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireHyperliquidOrderReference {
    OrderId { value: String },
    ClientOrderId { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidOpenOrder {
    pub(crate) coin: String,
    pub(crate) limit_price: String,
    pub(crate) order_id: String,
    pub(crate) side: String,
    pub(crate) size: String,
    pub(crate) timestamp: String,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidOrderDetail {
    pub(crate) coin: String,
    pub(crate) side: String,
    pub(crate) limit_price: String,
    pub(crate) size: String,
    pub(crate) order_id: String,
    pub(crate) timestamp: String,
    pub(crate) trigger_condition: String,
    pub(crate) is_trigger: bool,
    pub(crate) trigger_price: String,
    pub(crate) children_json: String,
    pub(crate) is_position_tpsl: bool,
    pub(crate) reduce_only: bool,
    pub(crate) order_type: String,
    pub(crate) original_size: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_order_id: Option<String>,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidOrderInfo {
    pub(crate) order: WireHyperliquidOrderDetail,
    pub(crate) status: String,
    pub(crate) status_timestamp: String,
    pub(crate) raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireHyperliquidOrderStatusResponse {
    Order { value: WireHyperliquidOrderInfo },
    UnknownOrder,
    Other { status: String, raw_json: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidPortfolioPoint {
    pub(crate) time: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidPortfolioPeriod {
    pub(crate) period: String,
    pub(crate) account_value_history: Vec<WireHyperliquidPortfolioPoint>,
    pub(crate) pnl_history: Vec<WireHyperliquidPortfolioPoint>,
    pub(crate) volume: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidSubAccount {
    pub(crate) name: String,
    pub(crate) user: String,
    pub(crate) master: String,
    pub(crate) perpetual_state_json: String,
    pub(crate) spot_state_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireHyperliquidVaultEquity {
    pub(crate) vault_address: String,
    pub(crate) equity: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) locked_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireMarketInfo {
    pub(crate) market: WireMarket,
    pub(crate) native_symbol: String,
    pub(crate) status: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) korean_name: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) english_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireTrade {
    pub(crate) market: WireMarket,
    pub(crate) timestamp: String,
    pub(crate) price: String,
    pub(crate) quantity: String,
    pub(crate) taker_side: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireLevel {
    pub(crate) price: String,
    pub(crate) quantity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderBook {
    pub(crate) market: WireMarket,
    pub(crate) timestamp: String,
    pub(crate) bids: Vec<WireLevel>,
    pub(crate) asks: Vec<WireLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireTicker {
    pub(crate) market: WireMarket,
    pub(crate) timestamp: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) last_trade_time: Option<String>,
    pub(crate) last_price: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) change: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) change_rate: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) high: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) low: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) volume: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) quote_volume: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireCandle {
    pub(crate) market: WireMarket,
    pub(crate) interval: String,
    pub(crate) open_time: String,
    pub(crate) open: String,
    pub(crate) high: String,
    pub(crate) low: String,
    pub(crate) close: String,
    pub(crate) volume: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) quote_volume: Option<String>,
    pub(crate) closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireBalance {
    pub(crate) asset: String,
    pub(crate) available: String,
    pub(crate) locked: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderAccount {
    pub(crate) balance: WireBalance,
    pub(crate) average_buy_price: String,
    pub(crate) average_buy_price_modified: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) average_buy_price_unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderOption {
    pub(crate) provider_id: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_type: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) time_in_force: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderRules {
    pub(crate) market: WireMarket,
    pub(crate) market_name: String,
    pub(crate) status: String,
    pub(crate) buy_fee_rate: String,
    pub(crate) sell_fee_rate: String,
    pub(crate) maker_buy_fee_rate: String,
    pub(crate) maker_sell_fee_rate: String,
    pub(crate) sides: Vec<String>,
    pub(crate) buy_options: Vec<WireOrderOption>,
    pub(crate) sell_options: Vec<WireOrderOption>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) buy_price_unit: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) sell_price_unit: Option<String>,
    pub(crate) minimum_buy_total: String,
    pub(crate) minimum_sell_total: String,
    pub(crate) maximum_total: String,
    pub(crate) quote_account: WireOrderAccount,
    pub(crate) base_account: WireOrderAccount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireWithdrawalFee {
    Fixed {
        value: String,
    },
    Rate {
        rate: String,
        #[serde(deserialize_with = "explicit_option")]
        minimum: Option<String>,
        #[serde(deserialize_with = "explicit_option")]
        maximum: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireAssetNetwork {
    pub(crate) exchange: String,
    pub(crate) asset: String,
    pub(crate) network: String,
    pub(crate) provider_id: String,
    pub(crate) deposit_enabled: bool,
    pub(crate) withdrawal_enabled: bool,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) withdrawal_fee: Option<WireWithdrawalFee>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) minimum_withdrawal: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) maximum_withdrawal: Option<String>,
    pub(crate) memo_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireDepositAddress {
    pub(crate) exchange: String,
    pub(crate) asset: String,
    pub(crate) network: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireDepositAddressEntry {
    pub(crate) exchange: String,
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) provider_network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireExchangeDestination {
    pub(crate) exchange: String,
    pub(crate) asset: String,
    pub(crate) network: String,
    pub(crate) address: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireChainDestination {
    pub(crate) asset: String,
    pub(crate) network: String,
    pub(crate) address: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireExchangeTransferRequest {
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) source_network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) destination_network: Option<String>,
    pub(crate) amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireChainTransferRequest {
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) source_network: Option<String>,
    pub(crate) destination: WireChainDestination,
    pub(crate) amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireTransferDestination {
    Exchange { value: WireExchangeDestination },
    Chain { value: WireChainDestination },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireTravelRuleRequirement {
    NotRequired,
    Required {
        #[serde(deserialize_with = "explicit_option")]
        consent_url: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireWithdrawalQuote {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) fee: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) expected_receive: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) minimum_amount: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) maximum_amount: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address_allowed: Option<bool>,
    pub(crate) travel_rule: WireTravelRuleRequirement,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireTransferPlan {
    pub(crate) source: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) destination: Option<String>,
    pub(crate) request: WireWithdrawRequest,
    pub(crate) quote: WireWithdrawalQuote,
    pub(crate) created_at: String,
    pub(crate) expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireWithdrawal {
    pub(crate) id: String,
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) provider_network: Option<String>,
    pub(crate) amount: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) fee: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) destination: Option<WireTransferDestination>,
    pub(crate) status: String,
    pub(crate) provider_status: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireDeposit {
    pub(crate) id: String,
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) network: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) provider_network: Option<String>,
    pub(crate) amount: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) memo: Option<String>,
    pub(crate) status: String,
    pub(crate) provider_status: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) tx_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrder {
    pub(crate) id: String,
    pub(crate) market: WireMarket,
    pub(crate) side: String,
    pub(crate) status: String,
    pub(crate) filled_quantity: String,
    pub(crate) remaining_quantity: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireCancelledOrder {
    pub(crate) order_id: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) cancelled_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireOrderCancelFailure {
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) order_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) client_id: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) market: Option<WireMarket>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) code: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireCancelOrdersResult {
    pub(crate) cancelled: Vec<WireCancelledOrder>,
    pub(crate) failed: Vec<WireOrderCancelFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WirePosition {
    pub(crate) market: WireMarket,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) side: Option<String>,
    pub(crate) quantity: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) entry_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) mark_price: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) notional: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) unrealized_pnl: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) leverage: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) margin_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireMarginSummary {
    pub(crate) asset: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) equity: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) margin_balance: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) available_balance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireFundingRate {
    pub(crate) market: WireMarket,
    pub(crate) timestamp: String,
    pub(crate) rate: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) mark_price: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireFundingPayment {
    pub(crate) market: WireMarket,
    pub(crate) timestamp: String,
    pub(crate) amount: String,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) rate: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WirePage<T> {
    pub(crate) items: Vec<T>,
    #[serde(deserialize_with = "explicit_option")]
    pub(crate) next: Option<String>,
}

fn explicit_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

pub(crate) fn from_wire_value<T: DeserializeOwned>(value: Value, field: &str) -> maxt::Result<T> {
    serde_json::from_value(value).map_err(|error| Error::InvalidRequest {
        field: field.to_owned(),
        detail: error.to_string(),
    })
}

pub(crate) fn from_wire_text<T: DeserializeOwned>(text: &str, field: &str) -> maxt::Result<T> {
    let value = serde_json::from_str(text).map_err(|error| Error::InvalidRequest {
        field: field.to_owned(),
        detail: format!("invalid JSON text: {error}"),
    })?;
    validate_wire_json_depth(&value, field)?;
    from_wire_value(value, field)
}

fn validate_wire_json_depth(value: &Value, field: &str) -> maxt::Result<()> {
    let mut pending = vec![(value, 0_usize)];
    while let Some((value, parent_depth)) = pending.pop() {
        let children: Box<dyn Iterator<Item = &Value> + '_> = match value {
            Value::Array(values) => Box::new(values.iter()),
            Value::Object(values) => Box::new(values.values()),
            _ => continue,
        };
        let depth = parent_depth + 1;
        if depth > MAX_WIRE_JSON_DEPTH {
            return Err(Error::InvalidRequest {
                field: field.to_owned(),
                detail: format!("maximum JSON nesting depth is {MAX_WIRE_JSON_DEPTH}"),
            });
        }
        pending.extend(children.map(|child| (child, depth)));
    }
    Ok(())
}

fn safe_u64_from_wire(value: &str, field: &str) -> maxt::Result<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_safe_integer(field, value));
    }
    let value = value
        .parse::<u64>()
        .map_err(|_| invalid_safe_integer(field, value))?;
    if value > MAX_JSON_SAFE_INTEGER {
        return Err(invalid_safe_integer(field, &value.to_string()));
    }
    Ok(value)
}

fn u64_from_wire(value: &str, field: &str) -> maxt::Result<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::InvalidRequest {
            field: field.to_owned(),
            detail: "must be an unsigned 64-bit integer".to_owned(),
        });
    }
    value.parse::<u64>().map_err(|_| Error::InvalidRequest {
        field: field.to_owned(),
        detail: "must be an unsigned 64-bit integer".to_owned(),
    })
}

fn safe_usize_from_wire(value: &str, field: &str) -> maxt::Result<usize> {
    usize::try_from(safe_u64_from_wire(value, field)?)
        .map_err(|_| invalid_safe_integer(field, value))
}

fn safe_u64_to_wire(value: u64, field: &str) -> maxt::Result<String> {
    if value > MAX_JSON_SAFE_INTEGER {
        return Err(invalid_safe_integer(field, &value.to_string()));
    }
    Ok(value.to_string())
}

fn safe_usize_to_wire(value: usize, field: &str) -> maxt::Result<String> {
    let value = u64::try_from(value).map_err(|_| invalid_safe_integer(field, "usize overflow"))?;
    safe_u64_to_wire(value, field)
}

fn invalid_safe_integer(field: &str, value: &str) -> Error {
    Error::InvalidRequest {
        field: field.to_owned(),
        detail: format!(
            "`{value}` must be an unsigned decimal integer no greater than {MAX_JSON_SAFE_INTEGER}"
        ),
    }
}

pub(crate) fn outcome<T: Serialize>(result: maxt::Result<T>) -> Value {
    let mut envelope = serde_json::Map::new();
    match result {
        Ok(value) => match serde_json::to_value(value) {
            Ok(value) => {
                envelope.insert("ok".to_owned(), Value::Bool(true));
                envelope.insert("value".to_owned(), value);
            }
            Err(error) => {
                envelope.insert("ok".to_owned(), Value::Bool(false));
                envelope.insert(
                    "error".to_owned(),
                    adapter_wire_value(format!("could not serialize native result: {error}")),
                );
            }
        },
        Err(error) => {
            let error = serde_json::to_value(WireError::from(error)).unwrap_or_else(|error| {
                adapter_wire_value(format!("could not serialize native error: {error}"))
            });
            envelope.insert("ok".to_owned(), Value::Bool(false));
            envelope.insert("error".to_owned(), error);
        }
    }
    Value::Object(envelope)
}

fn adapter_wire_value(detail: String) -> Value {
    Value::Object(serde_json::Map::from_iter([
        ("kind".to_owned(), Value::String("adapter".to_owned())),
        ("detail".to_owned(), Value::String(detail)),
    ]))
}

impl TryFrom<WireMarket> for Market {
    type Error = Error;

    fn try_from(value: WireMarket) -> Result<Self, Self::Error> {
        Ok(Self::new(
            exchange_from_wire(&value.exchange, "market.exchange")?,
            market_kind_from_wire(&value.kind, "market.kind")?,
            value.base,
            value.quote,
        ))
    }
}

impl TryFrom<WireSize> for Size {
    type Error = Error;

    fn try_from(value: WireSize) -> Result<Self, Self::Error> {
        match value {
            WireSize::Base { value } => Ok(Self::Base(decimal_from_wire(&value, "size.value")?)),
            WireSize::Quote { value } => Ok(Self::Quote(decimal_from_wire(&value, "size.value")?)),
        }
    }
}

impl TryFrom<WireCandleRequest> for CandleRequest {
    type Error = Error;

    fn try_from(value: WireCandleRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            interval: interval_from_wire(&value.interval, "interval")?,
            from: value
                .from
                .as_deref()
                .map(|value| timestamp_from_wire(value, "from"))
                .transpose()?,
            to: value
                .to
                .as_deref()
                .map(|value| timestamp_from_wire(value, "to"))
                .transpose()?,
            limit: value.limit,
        })
    }
}

impl TryFrom<WireOrderRequest> for OrderRequest {
    type Error = Error;

    fn try_from(value: WireOrderRequest) -> Result<Self, Self::Error> {
        let market = value.market.try_into()?;
        let side = side_from_wire(&value.side, "side")?;
        let size = value.size.try_into()?;
        let order_type = order_type_from_wire(&value.order_type, "order_type")?;
        let price = value
            .price
            .as_deref()
            .map(|value| decimal_from_wire(value, "price"))
            .transpose()?;
        let time_in_force = value
            .time_in_force
            .as_deref()
            .map(|value| time_in_force_from_wire(value, "time_in_force"))
            .transpose()?;
        let best = matches!(&order_type, OrderType::Best);
        let mut request = match (order_type, price, time_in_force) {
            (OrderType::Market, None, _) => Self::market(market, side, size),
            (OrderType::Limit, Some(price), _) => Self::limit(market, side, size, price),
            (OrderType::Best, None, Some(policy)) => Self::best(market, side, size, policy),
            (OrderType::Market, Some(_), _) => {
                return Err(Error::InvalidRequest {
                    field: "price".to_owned(),
                    detail: "a market order must not have a price".to_owned(),
                });
            }
            (OrderType::Limit, None, _) => {
                return Err(Error::InvalidRequest {
                    field: "price".to_owned(),
                    detail: "a limit order requires a price".to_owned(),
                });
            }
            (OrderType::Best, Some(_), _) => {
                return Err(Error::InvalidRequest {
                    field: "price".to_owned(),
                    detail: "a best order must not have a price".to_owned(),
                });
            }
            (OrderType::Best, None, None) => {
                return Err(Error::InvalidRequest {
                    field: "time_in_force".to_owned(),
                    detail: "a best order requires a time-in-force".to_owned(),
                });
            }
            _ => return Err(binding_contract("OrderType")),
        };
        if !best && let Some(time_in_force) = time_in_force {
            request = request.time_in_force(time_in_force);
        }
        if value.reduce_only {
            request = request.reduce_only();
        }
        if let Some(client_id) = value.client_id {
            request = request.client_id(client_id);
        }
        Ok(request)
    }
}

impl TryFrom<WireOrderHistoryRequest> for OrderHistoryRequest {
    type Error = Error;

    fn try_from(value: WireOrderHistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            statuses: value
                .statuses
                .iter()
                .map(|status| order_status_from_wire(status, "statuses"))
                .collect::<Result<_, _>>()?,
            from: timestamp_option_from_wire(value.from, "from")?,
            to: timestamp_option_from_wire(value.to, "to")?,
            cursor: value.cursor.map(Cursor::new),
            limit: value.limit,
        })
    }
}

impl TryFrom<WireBithumbPendingOrdersRequest> for BithumbPendingOrdersRequest {
    type Error = Error;

    fn try_from(value: WireBithumbPendingOrdersRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            state: value
                .state
                .as_deref()
                .map(|value| match value {
                    "wait" => Ok(BithumbPendingOrderState::Wait),
                    "watch" => Ok(BithumbPendingOrderState::Watch),
                    _ => Err(invalid_enum("state", value)),
                })
                .transpose()?,
            limit: value.limit,
            order_by: value
                .order_by
                .as_deref()
                .map(|value| match value {
                    "asc" => Ok(BithumbOrderDirection::Ascending),
                    "desc" => Ok(BithumbOrderDirection::Descending),
                    _ => Err(invalid_enum("order_by", value)),
                })
                .transpose()?,
            cursor: value.cursor.map(Cursor::new),
        })
    }
}

impl TryFrom<WireBithumbClosedOrdersRequest> for BithumbClosedOrdersRequest {
    type Error = Error;

    fn try_from(value: WireBithumbClosedOrdersRequest) -> Result<Self, Self::Error> {
        let closed_order_state = |value: &str, field| match value {
            "done" => Ok(BithumbClosedOrderState::Done),
            "cancel" => Ok(BithumbClosedOrderState::Cancel),
            _ => Err(invalid_enum(field, value)),
        };
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            state: value
                .state
                .as_deref()
                .map(|value| closed_order_state(value, "state"))
                .transpose()?,
            states: value
                .states
                .iter()
                .map(|value| closed_order_state(value, "states"))
                .collect::<Result<_, _>>()?,
            start_time: timestamp_option_from_wire(value.start_time, "start_time")?,
            end_time: timestamp_option_from_wire(value.end_time, "end_time")?,
            limit: value.limit,
            order_by: bithumb_order_direction_from_wire(value.order_by)?,
            cursor: value.cursor.map(Cursor::new),
        })
    }
}

impl TryFrom<BithumbClosedOrder> for WireBithumbClosedOrder {
    type Error = Error;

    fn try_from(value: BithumbClosedOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: value.order_id,
            side: value.side,
            order_type: value.order_type,
            price: decimal_option_to_wire(value.price),
            state: value.state,
            market: value.market.try_into()?,
            created_at: timestamp_option_to_wire(value.created_at),
            volume: decimal_to_wire(value.volume),
            remaining_volume: decimal_to_wire(value.remaining_volume),
            reserved_fee: decimal_to_wire(value.reserved_fee),
            remaining_fee: decimal_to_wire(value.remaining_fee),
            paid_fee: decimal_to_wire(value.paid_fee),
            locked: decimal_to_wire(value.locked),
            executed_volume: decimal_to_wire(value.executed_volume),
            executed_funds: decimal_to_wire(value.executed_funds),
            trades_count: value.trades_count,
            client_order_id: value.client_order_id,
            stp_type: value.stp_type,
            time_in_force: value.time_in_force,
            cancel_type: value.cancel_type,
            canceling_order_id: value.canceling_order_id,
        })
    }
}

fn bithumb_order_direction_from_wire(
    value: Option<String>,
) -> Result<Option<BithumbOrderDirection>, Error> {
    value
        .as_deref()
        .map(|value| match value {
            "asc" => Ok(BithumbOrderDirection::Ascending),
            "desc" => Ok(BithumbOrderDirection::Descending),
            _ => Err(invalid_enum("order_by", value)),
        })
        .transpose()
}

fn bithumb_order_list_state_from_wire(
    value: &str,
    field: &str,
) -> Result<BithumbOrderListState, Error> {
    match value {
        "wait" => Ok(BithumbOrderListState::Wait),
        "watch" => Ok(BithumbOrderListState::Watch),
        "done" => Ok(BithumbOrderListState::Done),
        "cancel" => Ok(BithumbOrderListState::Cancel),
        _ => Err(invalid_enum(field, value)),
    }
}

impl TryFrom<WireBithumbKrwWithdrawalsRequest> for BithumbKrwWithdrawalsRequest {
    type Error = Error;

    fn try_from(value: WireBithumbKrwWithdrawalsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            state: value.state,
            uuids: value.uuids,
            txids: value.txids,
            page: value.page,
            limit: value.limit,
            order_by: bithumb_order_direction_from_wire(value.order_by)?,
        })
    }
}

impl TryFrom<WireBithumbKrwDepositsRequest> for BithumbKrwDepositsRequest {
    type Error = Error;

    fn try_from(value: WireBithumbKrwDepositsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            state: value.state,
            uuids: value.uuids,
            txids: value.txids,
            page: value.page,
            limit: value.limit,
            order_by: bithumb_order_direction_from_wire(value.order_by)?,
        })
    }
}

impl TryFrom<WireBithumbKrwTransferRequest> for BithumbKrwTransferRequest {
    type Error = Error;

    fn try_from(value: WireBithumbKrwTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self::new(decimal_from_wire(&value.amount, "amount")?))
    }
}

impl TryFrom<WireBithumbTwapOrdersRequest> for BithumbTwapOrdersRequest {
    type Error = Error;

    fn try_from(value: WireBithumbTwapOrdersRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            uuids: value.uuids,
            state: value
                .state
                .as_deref()
                .map(|value| match value {
                    "progress" => Ok(BithumbTwapState::Progress),
                    "done" => Ok(BithumbTwapState::Done),
                    "cancel" => Ok(BithumbTwapState::Cancel),
                    _ => Err(invalid_enum("state", value)),
                })
                .transpose()?,
            cursor: value.cursor.map(Cursor::new),
            limit: value.limit,
            order_by: value
                .order_by
                .as_deref()
                .map(|value| match value {
                    "asc" => Ok(BithumbTwapOrderDirection::Ascending),
                    "desc" => Ok(BithumbTwapOrderDirection::Descending),
                    _ => Err(invalid_enum("order_by", value)),
                })
                .transpose()?,
        })
    }
}

impl TryFrom<WireBithumbTwapOrderRequest> for BithumbTwapOrderRequest {
    type Error = Error;

    fn try_from(value: WireBithumbTwapOrderRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            side: side_from_wire(&value.side, "side")?,
            volume: decimal_option_from_wire(value.volume, "volume")?,
            price: decimal_option_from_wire(value.price, "price")?,
            duration: value.duration,
            frequency: value.frequency,
        })
    }
}

impl TryFrom<WireUpbitBatchCancelRequest> for UpbitBatchCancelRequest {
    type Error = Error;

    fn try_from(value: WireUpbitBatchCancelRequest) -> Result<Self, Self::Error> {
        let scope = match value.scope {
            WireUpbitBatchCancelScope::All => UpbitBatchCancelScope::All,
            WireUpbitBatchCancelScope::QuoteCurrencies { values } => {
                UpbitBatchCancelScope::QuoteCurrencies { values }
            }
            WireUpbitBatchCancelScope::Pairs { values } => UpbitBatchCancelScope::Pairs {
                values: values
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            },
        };
        Ok(UpbitBatchCancelRequest {
            scope,
            excluded_pairs: value
                .excluded_pairs
                .map(|values| values.into_iter().map(TryInto::try_into).collect())
                .transpose()?,
            side: value
                .side
                .as_deref()
                .map(|value| side_from_wire(value, "side"))
                .transpose()?,
            count: value.count,
            order_by: value
                .order_by
                .as_deref()
                .map(|value| match value {
                    "asc" => Ok(UpbitOrderDirection::Ascending),
                    "desc" => Ok(UpbitOrderDirection::Descending),
                    _ => Err(invalid_enum("order_by", value)),
                })
                .transpose()?,
        })
    }
}

impl TryFrom<WireUpbitOrderReference> for UpbitOrderReference {
    type Error = Error;

    fn try_from(value: WireUpbitOrderReference) -> Result<Self, Self::Error> {
        Ok(match value {
            WireUpbitOrderReference::Uuid { value } => Self::Uuid(value),
            WireUpbitOrderReference::Identifier { value } => Self::Identifier(value),
        })
    }
}

impl TryFrom<WireUpbitOrderVolume> for UpbitOrderVolume {
    type Error = Error;

    fn try_from(value: WireUpbitOrderVolume) -> Result<Self, Self::Error> {
        Ok(match value {
            WireUpbitOrderVolume::Amount { value } => {
                Self::Amount(decimal_from_wire(&value, "volume")?)
            }
            WireUpbitOrderVolume::RemainOnly => Self::RemainOnly,
        })
    }
}

impl TryFrom<WireUpbitCancelAndNewOrder> for UpbitCancelAndNewOrder {
    type Error = Error;

    fn try_from(value: WireUpbitCancelAndNewOrder) -> Result<Self, Self::Error> {
        Ok(match value {
            WireUpbitCancelAndNewOrder::Limit {
                volume,
                price,
                time_in_force,
            } => Self::Limit {
                volume: volume.try_into()?,
                price: decimal_from_wire(&price, "price")?,
                time_in_force: time_in_force
                    .as_deref()
                    .map(|value| time_in_force_from_wire(value, "time_in_force"))
                    .transpose()?,
            },
            WireUpbitCancelAndNewOrder::MarketBuy { price } => Self::MarketBuy {
                price: decimal_from_wire(&price, "price")?,
            },
            WireUpbitCancelAndNewOrder::MarketSell { volume } => Self::MarketSell {
                volume: volume.try_into()?,
            },
            WireUpbitCancelAndNewOrder::BestBuy {
                price,
                time_in_force,
            } => Self::BestBuy {
                price: decimal_from_wire(&price, "price")?,
                time_in_force: time_in_force_from_wire(&time_in_force, "time_in_force")?,
            },
            WireUpbitCancelAndNewOrder::BestSell {
                volume,
                time_in_force,
            } => Self::BestSell {
                volume: volume.try_into()?,
                time_in_force: time_in_force_from_wire(&time_in_force, "time_in_force")?,
            },
        })
    }
}

impl TryFrom<WireUpbitCancelAndNewOrderRequest> for UpbitCancelAndNewOrderRequest {
    type Error = Error;

    fn try_from(value: WireUpbitCancelAndNewOrderRequest) -> Result<Self, Self::Error> {
        let new_smp_type = value
            .new_smp_type
            .as_deref()
            .map(|value| match value {
                "cancel_maker" => Ok(UpbitSmpType::CancelMaker),
                "cancel_taker" => Ok(UpbitSmpType::CancelTaker),
                "reduce" => Ok(UpbitSmpType::Reduce),
                _ => Err(invalid_enum("new_smp_type", value)),
            })
            .transpose()?;
        Ok(Self {
            previous_order: value.previous_order.try_into()?,
            new_order: value.new_order.try_into()?,
            new_identifier: value.new_identifier,
            new_smp_type,
        })
    }
}

impl TryFrom<UpbitCancelAndNewOrderResult> for WireUpbitCancelAndNewOrderResult {
    type Error = Error;

    fn try_from(value: UpbitCancelAndNewOrderResult) -> Result<Self, Self::Error> {
        Ok(Self {
            previous_order: value.previous_order.try_into()?,
            new_order_uuid: value.new_order_uuid,
            new_order_identifier: value.new_order_identifier,
        })
    }
}

impl TryFrom<WireBithumbBatchOrdersRequest> for BithumbBatchOrdersRequest {
    type Error = Error;

    fn try_from(value: WireBithumbBatchOrdersRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            orders: value
                .orders
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<BithumbBatchOrder> for WireBithumbBatchOrder {
    type Error = Error;

    fn try_from(value: BithumbBatchOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: value.order_id,
            client_order_id: value.client_order_id,
            market: value.market.try_into()?,
            side: side_to_wire(value.side).to_owned(),
            order_type: order_type_to_wire(value.order_type)?.to_owned(),
            time_in_force: value.time_in_force,
            stp_type: value.stp_type,
            created_at: timestamp_option_to_wire(value.created_at),
        })
    }
}

impl TryFrom<BithumbBatchOrderFailure> for WireBithumbBatchOrderFailure {
    type Error = Error;

    fn try_from(value: BithumbBatchOrderFailure) -> Result<Self, Self::Error> {
        Ok(Self {
            client_order_id: value.client_order_id,
            time_in_force: value.time_in_force,
            code: value.code,
            message: value.message,
        })
    }
}

impl TryFrom<BithumbBatchOrderOutcome> for WireBithumbBatchOrderOutcome {
    type Error = Error;

    fn try_from(value: BithumbBatchOrderOutcome) -> Result<Self, Self::Error> {
        match value {
            BithumbBatchOrderOutcome::Accepted(value) => Ok(Self::Accepted {
                value: value.try_into()?,
            }),
            BithumbBatchOrderOutcome::Rejected(value) => Ok(Self::Rejected {
                value: value.try_into()?,
            }),
        }
    }
}

impl TryFrom<BithumbBatchOrdersResult> for WireBithumbBatchOrdersResult {
    type Error = Error;

    fn try_from(value: BithumbBatchOrdersResult) -> Result<Self, Self::Error> {
        Ok(Self {
            outcomes: value
                .outcomes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<WireBinanceAggregateTradesRequest> for BinanceAggregateTradesRequest {
    type Error = Error;

    fn try_from(value: WireBinanceAggregateTradesRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            from_id: value
                .from_id
                .as_deref()
                .map(|value| u64_from_wire(value, "from_id"))
                .transpose()?,
            start_time: timestamp_option_from_wire(value.start_time, "start_time")?,
            end_time: timestamp_option_from_wire(value.end_time, "end_time")?,
            limit: value.limit,
        })
    }
}

impl TryFrom<BinanceAggregateTrade> for WireBinanceAggregateTrade {
    type Error = Error;

    fn try_from(value: BinanceAggregateTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            aggregate_id: value.aggregate_id.to_string(),
            first_trade_id: value.first_trade_id.to_string(),
            last_trade_id: value.last_trade_id.to_string(),
            timestamp: timestamp_to_wire(value.timestamp),
            price: decimal_to_wire(value.price),
            quantity: decimal_to_wire(value.quantity),
            normal_quantity: decimal_option_to_wire(value.normal_quantity),
            taker_side: side_to_wire(value.taker_side).to_owned(),
        })
    }
}

impl TryFrom<BinanceAccountTrade> for WireBinanceAccountTrade {
    type Error = Error;

    fn try_from(value: BinanceAccountTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            id: value.id,
            order_id: value.order_id,
            timestamp: timestamp_to_wire(value.timestamp),
            side: side_to_wire(value.side).to_owned(),
            maker: value.maker,
            best_match: value.best_match,
            order_list_id: value.order_list_id,
            price: decimal_to_wire(value.price),
            quantity: decimal_to_wire(value.quantity),
            quote_quantity: decimal_option_to_wire(value.quote_quantity),
            commission: decimal_to_wire(value.commission),
            commission_asset: value.commission_asset,
            realized_pnl: decimal_option_to_wire(value.realized_pnl),
            position_side: value.position_side,
            pair: value.pair,
            base_quantity: decimal_option_to_wire(value.base_quantity),
            margin_asset: value.margin_asset,
        })
    }
}

impl TryFrom<WireBinanceTestOrderRequest> for BinanceTestOrderRequest {
    type Error = Error;

    fn try_from(value: WireBinanceTestOrderRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            order: value.order.try_into()?,
            compute_commission_rates: value.compute_commission_rates,
        })
    }
}

impl TryFrom<BinanceTestOrder> for WireBinanceTestOrder {
    type Error = Error;

    fn try_from(value: BinanceTestOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            response_json: value.response_json,
        })
    }
}

impl TryFrom<WireBinanceC2cTradeHistoryRequest> for BinanceC2cTradeHistoryRequest {
    type Error = Error;

    fn try_from(value: WireBinanceC2cTradeHistoryRequest) -> Result<Self, Self::Error> {
        let trade_type = match value.trade_type.as_str() {
            "BUY" => BinanceC2cTradeType::Buy,
            "SELL" => BinanceC2cTradeType::Sell,
            _ => return Err(invalid_enum("trade_type", &value.trade_type)),
        };
        Ok(Self {
            trade_type,
            start_timestamp: timestamp_option_from_wire(value.start_timestamp, "start_timestamp")?,
            end_timestamp: timestamp_option_from_wire(value.end_timestamp, "end_timestamp")?,
            page: value.page,
            rows: value.rows,
            recv_window: value
                .recv_window
                .as_deref()
                .map(|value| u64_from_wire(value, "recv_window"))
                .transpose()?,
        })
    }
}

impl TryFrom<BinanceC2cTrade> for WireBinanceC2cTrade {
    type Error = Error;

    fn try_from(value: BinanceC2cTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            order_number: value.order_number,
            adv_no: value.adv_no,
            trade_type: value.trade_type,
            asset: value.asset,
            fiat: value.fiat,
            fiat_symbol: value.fiat_symbol,
            amount: decimal_option_to_wire(value.amount),
            total_price: decimal_option_to_wire(value.total_price),
            unit_price: decimal_option_to_wire(value.unit_price),
            order_status: value.order_status,
            created_at: timestamp_option_to_wire(value.created_at),
            commission: decimal_option_to_wire(value.commission),
            counterparty_nickname: value.counterparty_nickname,
            pay_method_name: value.pay_method_name,
            additional_kyc_verify: value.additional_kyc_verify,
            taker_commission_rate: decimal_option_to_wire(value.taker_commission_rate),
            taker_commission: decimal_option_to_wire(value.taker_commission),
            taker_amount: decimal_option_to_wire(value.taker_amount),
            advertisement_role: value.advertisement_role,
        })
    }
}

impl TryFrom<BinanceC2cTradeHistoryPage> for WireBinanceC2cTradeHistoryPage {
    type Error = Error;

    fn try_from(value: BinanceC2cTradeHistoryPage) -> Result<Self, Self::Error> {
        Ok(Self {
            code: value.code,
            message: value.message,
            data: value
                .data
                .map(|values| values.into_iter().map(TryInto::try_into).collect())
                .transpose()?,
            total: value.total.map(|value| value.to_string()),
            success: value.success,
        })
    }
}

impl TryFrom<WireOrderLookupRequest> for OrderLookupRequest {
    type Error = Error;

    fn try_from(value: WireOrderLookupRequest) -> Result<Self, Self::Error> {
        let mut request = match value.kind.as_str() {
            "exchange" => Self::exchange(value.ids),
            "client" => Self::client(value.ids),
            value => {
                return Err(Error::InvalidRequest {
                    field: "kind".to_owned(),
                    detail: format!("unknown value `{value}`"),
                });
            }
        };
        if let Some(market) = value.market {
            request = request.market(market.try_into()?);
        }
        Ok(request)
    }
}

impl TryFrom<WireCancelOrdersRequest> for CancelOrdersRequest {
    type Error = Error;

    fn try_from(value: WireCancelOrdersRequest) -> Result<Self, Self::Error> {
        match value.kind.as_str() {
            "exchange" => Ok(Self::exchange(value.ids)),
            "client" => Ok(Self::client(value.ids)),
            value => Err(Error::InvalidRequest {
                field: "kind".to_owned(),
                detail: format!("unknown value `{value}`"),
            }),
        }
    }
}

impl TryFrom<WireDepositAddressRequest> for DepositAddressRequest {
    type Error = Error;

    fn try_from(value: WireDepositAddressRequest) -> Result<Self, Self::Error> {
        let mut request = Self::new(value.asset, network_from_wire(&value.network));
        if let Some(amount) = value.amount {
            request = request.amount(decimal_from_wire(&amount, "amount")?);
        }
        Ok(request)
    }
}

impl TryFrom<WireWithdrawRequest> for WithdrawRequest {
    type Error = Error;

    fn try_from(value: WireWithdrawRequest) -> Result<Self, Self::Error> {
        let mut request = Self::new(
            value.asset,
            network_from_wire(&value.network),
            decimal_from_wire(&value.amount, "amount")?,
            value.destination.try_into()?,
        );
        if let Some(client_id) = value.client_id {
            request = request.client_id(client_id);
        }
        Ok(request)
    }
}

impl TryFrom<WireTransferLookupRequest> for TransferLookupRequest {
    type Error = Error;

    fn try_from(value: WireTransferLookupRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset.to_ascii_uppercase(),
            id: value.id,
            tx_id: value.tx_id,
        })
    }
}

impl TryFrom<WireTransferHistoryRequest> for TransferHistoryRequest {
    type Error = Error;

    fn try_from(value: WireTransferHistoryRequest) -> Result<Self, Self::Error> {
        let mut request = Self::new();
        if let Some(asset) = value.asset {
            request = request.asset(asset);
        }
        if let Some(network) = value.network {
            request = request.network(network_from_wire(&network));
        }
        if let Some(cursor) = value.cursor {
            request = request.cursor(Cursor::new(cursor));
        }
        if let Some(limit) = value.limit {
            request = request.limit(limit);
        }
        Ok(request)
    }
}

impl TryFrom<CandleRequest> for WireCandleRequest {
    type Error = Error;

    fn try_from(value: CandleRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            interval: interval_to_wire(value.interval)?.to_owned(),
            from: timestamp_option_to_wire(value.from),
            to: timestamp_option_to_wire(value.to),
            limit: value.limit,
        })
    }
}

impl TryFrom<Size> for WireSize {
    type Error = Error;

    fn try_from(value: Size) -> Result<Self, Self::Error> {
        match value {
            Size::Base(value) => Ok(Self::Base {
                value: decimal_to_wire(value),
            }),
            Size::Quote(value) => Ok(Self::Quote {
                value: decimal_to_wire(value),
            }),
            _ => Err(binding_contract("Size")),
        }
    }
}

impl TryFrom<OrderRequest> for WireOrderRequest {
    type Error = Error;

    fn try_from(value: OrderRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            side: side_to_wire(value.side).to_owned(),
            order_type: order_type_to_wire(value.order_type)?.to_owned(),
            size: value.size.try_into()?,
            price: decimal_option_to_wire(value.price),
            time_in_force: value
                .time_in_force
                .map(time_in_force_to_wire)
                .transpose()?
                .map(str::to_owned),
            reduce_only: value.reduce_only,
            client_id: value.client_id,
        })
    }
}

impl TryFrom<OrderHistoryRequest> for WireOrderHistoryRequest {
    type Error = Error;

    fn try_from(value: OrderHistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            statuses: value
                .statuses
                .into_iter()
                .map(order_status_to_wire)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(str::to_owned)
                .collect(),
            from: timestamp_option_to_wire(value.from),
            to: timestamp_option_to_wire(value.to),
            cursor: value.cursor.map(|cursor| cursor.as_str().to_owned()),
            limit: value.limit,
        })
    }
}

impl TryFrom<OrderLookupRequest> for WireOrderLookupRequest {
    type Error = Error;

    fn try_from(value: OrderLookupRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: match value.kind {
                OrderIdKind::Exchange => "exchange",
                OrderIdKind::Client => "client",
            }
            .to_owned(),
            ids: value.ids,
            market: value.market.map(TryInto::try_into).transpose()?,
        })
    }
}

impl TryFrom<CancelOrdersRequest> for WireCancelOrdersRequest {
    type Error = Error;

    fn try_from(value: CancelOrdersRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: match value.kind {
                OrderIdKind::Exchange => "exchange",
                OrderIdKind::Client => "client",
            }
            .to_owned(),
            ids: value.ids,
        })
    }
}

impl TryFrom<DepositAddressRequest> for WireDepositAddressRequest {
    type Error = Error;

    fn try_from(value: DepositAddressRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            network: network_to_wire(value.network)?,
            amount: decimal_option_to_wire(value.amount),
        })
    }
}

impl TryFrom<WithdrawRequest> for WireWithdrawRequest {
    type Error = Error;

    fn try_from(value: WithdrawRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            network: network_to_wire(value.network)?,
            amount: decimal_to_wire(value.amount),
            destination: value.destination.try_into()?,
            client_id: value.client_id,
        })
    }
}

impl TryFrom<TransferLookupRequest> for WireTransferLookupRequest {
    type Error = Error;

    fn try_from(value: TransferLookupRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            id: value.id,
            tx_id: value.tx_id,
        })
    }
}

impl TryFrom<TransferHistoryRequest> for WireTransferHistoryRequest {
    type Error = Error;

    fn try_from(value: TransferHistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            network: value.network.map(network_to_wire).transpose()?,
            cursor: value.cursor.map(|cursor| cursor.as_str().to_owned()),
            limit: value.limit,
        })
    }
}

impl TryFrom<Feed> for WireFeed {
    type Error = Error;

    fn try_from(value: Feed) -> Result<Self, Self::Error> {
        match value {
            Feed::Trades => Ok(Self::Trades),
            Feed::OrderBook => Ok(Self::OrderBook),
            Feed::Ticker => Ok(Self::Ticker),
            Feed::Candles(interval) => Ok(Self::Candles {
                interval: interval_to_wire(interval)?.to_owned(),
            }),
            _ => Err(binding_contract("Feed")),
        }
    }
}

impl TryFrom<WireFeed> for Feed {
    type Error = Error;

    fn try_from(value: WireFeed) -> Result<Self, Self::Error> {
        match value {
            WireFeed::Trades => Ok(Self::Trades),
            WireFeed::OrderBook => Ok(Self::OrderBook),
            WireFeed::Ticker => Ok(Self::Ticker),
            WireFeed::Candles { interval } => Ok(Self::Candles(interval_from_wire(
                &interval,
                "feeds.interval",
            )?)),
        }
    }
}

impl TryFrom<MarketEvent> for WireMarketEvent {
    type Error = Error;

    fn try_from(value: MarketEvent) -> Result<Self, Self::Error> {
        match value {
            MarketEvent::Trade(trade) => Ok(Self::Trade {
                trade: trade.try_into()?,
            }),
            MarketEvent::OrderBook(order_book) => Ok(Self::OrderBook {
                order_book: order_book.try_into()?,
            }),
            MarketEvent::Ticker(ticker) => Ok(Self::Ticker {
                ticker: ticker.try_into()?,
            }),
            MarketEvent::Candle(candle) => Ok(Self::Candle {
                candle: candle.try_into()?,
            }),
            MarketEvent::Reconnected => Ok(Self::Reconnected),
            _ => Err(binding_contract("MarketEvent")),
        }
    }
}

impl TryFrom<WireMarketEvent> for MarketEvent {
    type Error = Error;

    fn try_from(value: WireMarketEvent) -> Result<Self, Self::Error> {
        match value {
            WireMarketEvent::Trade { trade } => Ok(Self::Trade(trade.try_into()?)),
            WireMarketEvent::OrderBook { order_book } => {
                Ok(Self::OrderBook(order_book.try_into()?))
            }
            WireMarketEvent::Ticker { ticker } => Ok(Self::Ticker(ticker.try_into()?)),
            WireMarketEvent::Candle { candle } => Ok(Self::Candle(candle.try_into()?)),
            WireMarketEvent::Reconnected => Ok(Self::Reconnected),
        }
    }
}

impl TryFrom<AccountEvent> for WireAccountEvent {
    type Error = Error;

    fn try_from(value: AccountEvent) -> Result<Self, Self::Error> {
        match value {
            AccountEvent::Balance(balance) => Ok(Self::Balance {
                balance: balance.try_into()?,
            }),
            AccountEvent::Order(order) => Ok(Self::Order {
                order: order.try_into()?,
            }),
            AccountEvent::Reconnected => Ok(Self::Reconnected),
            _ => Err(binding_contract("AccountEvent")),
        }
    }
}

impl TryFrom<WireAccountEvent> for AccountEvent {
    type Error = Error;

    fn try_from(value: WireAccountEvent) -> Result<Self, Self::Error> {
        match value {
            WireAccountEvent::Balance { balance } => Ok(Self::Balance(balance.try_into()?)),
            WireAccountEvent::Order { order } => Ok(Self::Order(order.try_into()?)),
            WireAccountEvent::Reconnected => Ok(Self::Reconnected),
        }
    }
}

pub(crate) fn market_stream_item(item: maxt::Result<MarketEvent>) -> WireMarketStreamItem {
    match item {
        Ok(event) => match event.try_into() {
            Ok(event) => WireMarketStreamItem::Event {
                event: Box::new(event),
            },
            Err(error) => WireMarketStreamItem::Error {
                error: error.into(),
            },
        },
        Err(error) => WireMarketStreamItem::Error {
            error: error.into(),
        },
    }
}

pub(crate) fn account_stream_item(item: maxt::Result<AccountEvent>) -> WireAccountStreamItem {
    match item {
        Ok(event) => match event.try_into() {
            Ok(event) => WireAccountStreamItem::Event { event },
            Err(error) => WireAccountStreamItem::Error {
                error: error.into(),
            },
        },
        Err(error) => WireAccountStreamItem::Error {
            error: error.into(),
        },
    }
}

impl TryFrom<Subscription> for WireSubscription {
    type Error = Error;

    fn try_from(value: Subscription) -> Result<Self, Self::Error> {
        Ok(Self {
            markets: value
                .markets()
                .iter()
                .cloned()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
            feeds: value
                .feeds()
                .iter()
                .copied()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
        })
    }
}

impl TryFrom<WireSubscription> for Subscription {
    type Error = Error;

    fn try_from(value: WireSubscription) -> Result<Self, Self::Error> {
        let markets = value
            .markets
            .into_iter()
            .map(TryInto::try_into)
            .collect::<maxt::Result<Vec<_>>>()?;
        value
            .feeds
            .into_iter()
            .try_fold(Self::new().markets_iter(markets), |subscription, feed| {
                Ok(subscription.feed(feed.try_into()?))
            })
    }
}

impl TryFrom<maxt::UpbitListedSubscription> for WireUpbitListedSubscription {
    type Error = Error;

    fn try_from(value: maxt::UpbitListedSubscription) -> Result<Self, Self::Error> {
        Ok(Self {
            feed_type: value.feed_type,
            markets: value
                .markets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
            level: value.level.map(decimal_to_wire),
        })
    }
}

impl TryFrom<maxt::UpbitSubscriptionList> for WireUpbitSubscriptionList {
    type Error = Error;

    fn try_from(value: maxt::UpbitSubscriptionList) -> Result<Self, Self::Error> {
        Ok(Self {
            ticket: value.ticket,
            subscriptions: value
                .subscriptions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
        })
    }
}

impl TryFrom<StreamConfig> for WireStreamConfig {
    type Error = Error;

    fn try_from(value: StreamConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            max_reconnect_attempts: value.max_reconnect_attempts,
            initial_reconnect_delay_ms: safe_u64_to_wire(
                value.initial_reconnect_delay_ms,
                "initial_reconnect_delay_ms",
            )?,
            max_reconnect_delay_ms: safe_u64_to_wire(
                value.max_reconnect_delay_ms,
                "max_reconnect_delay_ms",
            )?,
            idle_timeout_ms: safe_u64_to_wire(value.idle_timeout_ms, "idle_timeout_ms")?,
            buffer_size: safe_usize_to_wire(value.buffer_size, "buffer_size")?,
            overflow: overflow_to_wire(value.overflow)?.to_owned(),
        })
    }
}

impl TryFrom<WireStreamConfig> for StreamConfig {
    type Error = Error;

    fn try_from(value: WireStreamConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            max_reconnect_attempts: value.max_reconnect_attempts,
            initial_reconnect_delay_ms: safe_u64_from_wire(
                &value.initial_reconnect_delay_ms,
                "initial_reconnect_delay_ms",
            )?,
            max_reconnect_delay_ms: safe_u64_from_wire(
                &value.max_reconnect_delay_ms,
                "max_reconnect_delay_ms",
            )?,
            idle_timeout_ms: safe_u64_from_wire(&value.idle_timeout_ms, "idle_timeout_ms")?,
            buffer_size: safe_usize_from_wire(&value.buffer_size, "buffer_size")?,
            overflow: overflow_from_wire(&value.overflow, "overflow")?,
        })
    }
}

impl TryFrom<HistoryRequest> for WireHistoryRequest {
    type Error = Error;

    fn try_from(value: HistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            from: timestamp_option_to_wire(value.from),
            to: timestamp_option_to_wire(value.to),
            cursor: value.cursor.map(|cursor| cursor.as_str().to_owned()),
            limit: value.limit,
        })
    }
}

impl TryFrom<WireHistoryRequest> for HistoryRequest {
    type Error = Error;

    fn try_from(value: WireHistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            from: timestamp_option_from_wire(value.from, "from")?,
            to: timestamp_option_from_wire(value.to, "to")?,
            cursor: value.cursor.map(Cursor::new),
            limit: value.limit,
        })
    }
}

impl TryFrom<MarginRequest> for WireMarginRequest {
    type Error = Error;

    fn try_from(value: MarginRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            leverage: decimal_option_to_wire(value.leverage),
            margin_mode: value
                .margin_mode
                .map(margin_mode_to_wire)
                .transpose()?
                .map(str::to_owned),
        })
    }
}

impl TryFrom<WireMarginRequest> for MarginRequest {
    type Error = Error;

    fn try_from(value: WireMarginRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            leverage: decimal_option_from_wire(value.leverage, "leverage")?,
            margin_mode: value
                .margin_mode
                .as_deref()
                .map(|value| margin_mode_from_wire(value, "margin_mode"))
                .transpose()?,
        })
    }
}

impl TryFrom<UpbitMarketEvent> for WireUpbitMarketEvent {
    type Error = Error;

    fn try_from(value: UpbitMarketEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            warning: value.warning,
            cautions: value.cautions,
        })
    }
}

impl TryFrom<UpbitYearCandle> for WireUpbitYearCandle {
    type Error = Error;

    fn try_from(value: UpbitYearCandle) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            open_time: timestamp_to_wire(value.open_time),
            korea_open_time: timestamp_option_to_wire(value.korea_open_time),
            timestamp: timestamp_to_wire(value.timestamp),
            open: decimal_to_wire(value.open),
            high: decimal_to_wire(value.high),
            low: decimal_to_wire(value.low),
            close: decimal_to_wire(value.close),
            volume: decimal_to_wire(value.volume),
            quote_volume: decimal_to_wire(value.quote_volume),
            first_day_of_period: value.first_day_of_period,
        })
    }
}

impl TryFrom<UpbitOrderBookInstrument> for WireUpbitOrderBookInstrument {
    type Error = Error;

    fn try_from(value: UpbitOrderBookInstrument) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            quote_currency: value.quote_currency,
            tick_size: decimal_to_wire(value.tick_size),
            supported_levels: value
                .supported_levels
                .into_iter()
                .map(decimal_to_wire)
                .collect(),
        })
    }
}

impl TryFrom<WireUpbitOrderDetailRequest> for UpbitOrderDetailRequest {
    type Error = Error;

    fn try_from(value: WireUpbitOrderDetailRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            uuid: value.uuid,
            identifier: value.identifier,
        })
    }
}

impl TryFrom<UpbitOrderDetailTrade> for WireUpbitOrderDetailTrade {
    type Error = Error;

    fn try_from(value: UpbitOrderDetailTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            uuid: value.uuid,
            price: decimal_to_wire(value.price),
            volume: decimal_to_wire(value.volume),
            funds: decimal_to_wire(value.funds),
            trend: value.trend,
            created_at: timestamp_to_wire(value.created_at),
            side: value.side,
        })
    }
}

impl TryFrom<UpbitOrderDetail> for WireUpbitOrderDetail {
    type Error = Error;

    fn try_from(value: UpbitOrderDetail) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            uuid: value.uuid,
            side: value.side,
            order_type: value.order_type,
            price: decimal_option_to_wire(value.price),
            state: value.state,
            created_at: timestamp_to_wire(value.created_at),
            volume: decimal_option_to_wire(value.volume),
            remaining_volume: decimal_to_wire(value.remaining_volume),
            executed_volume: decimal_to_wire(value.executed_volume),
            reserved_fee: decimal_to_wire(value.reserved_fee),
            remaining_fee: decimal_to_wire(value.remaining_fee),
            paid_fee: decimal_to_wire(value.paid_fee),
            locked: decimal_to_wire(value.locked),
            trades_count: value.trades_count,
            prevented_volume: decimal_to_wire(value.prevented_volume),
            prevented_locked: decimal_to_wire(value.prevented_locked),
            time_in_force: value.time_in_force,
            identifier: value.identifier,
            smp_type: value.smp_type,
            trades: value
                .trades
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<WireUpbitClosedOrdersRequest> for UpbitClosedOrdersRequest {
    type Error = Error;

    fn try_from(value: WireUpbitClosedOrdersRequest) -> Result<Self, Self::Error> {
        let state = value
            .state
            .as_deref()
            .map(|value| match value {
                "done" => Ok(UpbitClosedOrderState::Done),
                "cancel" => Ok(UpbitClosedOrderState::Cancel),
                _ => Err(invalid_enum("state", value)),
            })
            .transpose()?;
        let states = value
            .states
            .into_iter()
            .map(|value| match value.as_str() {
                "done" => Ok(UpbitClosedOrderState::Done),
                "cancel" => Ok(UpbitClosedOrderState::Cancel),
                _ => Err(invalid_enum("states", &value)),
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            state,
            states,
            start_time: value
                .start_time
                .as_deref()
                .map(|value| timestamp_from_wire(value, "start_time"))
                .transpose()?,
            end_time: value
                .end_time
                .as_deref()
                .map(|value| timestamp_from_wire(value, "end_time"))
                .transpose()?,
            limit: value.limit,
            order_by: value
                .order_by
                .as_deref()
                .map(|value| match value {
                    "asc" => Ok(UpbitOrderDirection::Ascending),
                    "desc" => Ok(UpbitOrderDirection::Descending),
                    _ => Err(invalid_enum("order_by", value)),
                })
                .transpose()?,
        })
    }
}

impl TryFrom<UpbitClosedOrder> for WireUpbitClosedOrder {
    type Error = Error;

    fn try_from(value: UpbitClosedOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            uuid: value.uuid,
            side: value.side,
            ord_type: value.ord_type,
            state: value.state,
            created_at: timestamp_to_wire(value.created_at),
            volume: decimal_option_to_wire(value.volume),
            price: decimal_option_to_wire(value.price),
            remaining_volume: decimal_to_wire(value.remaining_volume),
            executed_volume: decimal_to_wire(value.executed_volume),
            executed_funds: decimal_option_to_wire(value.executed_funds),
            reserved_fee: decimal_to_wire(value.reserved_fee),
            remaining_fee: decimal_to_wire(value.remaining_fee),
            paid_fee: decimal_to_wire(value.paid_fee),
            locked: decimal_to_wire(value.locked),
            trades_count: value.trades_count,
            prevented_volume: decimal_to_wire(value.prevented_volume),
            prevented_locked: decimal_to_wire(value.prevented_locked),
            time_in_force: value.time_in_force,
            identifier: value.identifier,
            smp_type: value.smp_type,
        })
    }
}

impl TryFrom<UpbitDepositInfo> for WireUpbitDepositInfo {
    type Error = Error;

    fn try_from(value: UpbitDepositInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            network: value.network.map(network_to_wire).transpose()?,
            provider_network: value.provider_network,
            is_deposit_possible: value.is_deposit_possible,
            deposit_impossible_reason: value.deposit_impossible_reason,
            minimum_deposit_amount: decimal_to_wire(value.minimum_deposit_amount),
            minimum_deposit_confirmations: value.minimum_deposit_confirmations.to_string(),
            decimal_precision: value.decimal_precision.to_string(),
        })
    }
}

impl TryFrom<UpbitWithdrawalAddress> for WireUpbitWithdrawalAddress {
    type Error = Error;

    fn try_from(value: UpbitWithdrawalAddress) -> Result<Self, Self::Error> {
        Ok(Self {
            currency: value.currency,
            net_type: value.net_type,
            network_name: value.network_name,
            withdraw_address: value.withdraw_address,
            secondary_address: value.secondary_address,
            beneficiary_name: value.beneficiary_name,
            beneficiary_company_name: value.beneficiary_company_name,
            beneficiary_type: value.beneficiary_type,
            exchange_name: value.exchange_name,
            wallet_type: value.wallet_type,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<UpbitTravelRuleVasp> for WireUpbitTravelRuleVasp {
    type Error = Error;

    fn try_from(value: UpbitTravelRuleVasp) -> Result<Self, Self::Error> {
        Ok(Self {
            vasp_name: value.vasp_name,
            vasp_uuid: value.vasp_uuid,
            depositable: value.depositable,
            withdrawable: value.withdrawable,
        })
    }
}

impl TryFrom<UpbitTravelRuleVerification> for WireUpbitTravelRuleVerification {
    type Error = Error;

    fn try_from(value: UpbitTravelRuleVerification) -> Result<Self, Self::Error> {
        Ok(Self {
            deposit_uuid: value.deposit_uuid,
            deposit_state: value.deposit_state,
            verification_result: value.verification_result,
        })
    }
}

impl TryFrom<WireUpbitKrwTransferRequest> for UpbitKrwTransferRequest {
    type Error = Error;

    fn try_from(value: WireUpbitKrwTransferRequest) -> Result<Self, Self::Error> {
        let two_factor_type = match value.two_factor_type.as_str() {
            "kakao" => UpbitKrwTwoFactorType::Kakao,
            "naver" => UpbitKrwTwoFactorType::Naver,
            "hana" => UpbitKrwTwoFactorType::Hana,
            value => return Err(invalid_enum("two_factor_type", value)),
        };
        Ok(Self::new(
            decimal_from_wire(&value.amount, "amount")?,
            two_factor_type,
        ))
    }
}

impl TryFrom<UpbitPocket> for WireUpbitPocket {
    type Error = Error;

    fn try_from(value: UpbitPocket) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: value.uuid,
            name: value.name,
            kind: value.kind,
        })
    }
}

impl TryFrom<UpbitPocketApiKey> for WireUpbitPocketApiKey {
    type Error = Error;

    fn try_from(value: UpbitPocketApiKey) -> Result<Self, Self::Error> {
        Ok(Self {
            access_key: value.access_key,
            permissions: value.permissions,
            allowed_ips: value.allowed_ips,
            created_at: timestamp_to_wire(value.created_at),
            expired_at: timestamp_to_wire(value.expired_at),
        })
    }
}

impl TryFrom<UpbitPocketApiKeyGroup> for WireUpbitPocketApiKeyGroup {
    type Error = Error;

    fn try_from(value: UpbitPocketApiKeyGroup) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: value.uuid,
            keys: value
                .keys
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<WireUpbitPocketApiKeysRequest> for UpbitPocketApiKeysRequest {
    type Error = Error;

    fn try_from(value: WireUpbitPocketApiKeysRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            uuids: value.uuids,
            include_expired: value.include_expired,
        })
    }
}

impl TryFrom<UpbitPocketBalance> for WireUpbitPocketBalance {
    type Error = Error;

    fn try_from(value: UpbitPocketBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            currency: value.currency,
            balance: decimal_to_wire(value.balance),
            locked: decimal_to_wire(value.locked),
            avg_buy_price: decimal_to_wire(value.avg_buy_price),
            avg_buy_price_modified: value.avg_buy_price_modified,
            unit_currency: value.unit_currency,
        })
    }
}

impl TryFrom<WireUpbitPocketTransferQuery> for UpbitPocketTransferQuery {
    type Error = Error;

    fn try_from(value: WireUpbitPocketTransferQuery) -> Result<Self, Self::Error> {
        let direction = value
            .direction
            .as_deref()
            .map(|value| match value {
                "in" => Ok(UpbitPocketTransferDirection::Incoming),
                "out" => Ok(UpbitPocketTransferDirection::Outgoing),
                "all" => Ok(UpbitPocketTransferDirection::All),
                _ => Err(invalid_enum("direction", value)),
            })
            .transpose()?;
        let states = value
            .states
            .into_iter()
            .map(|value| match value.as_str() {
                "submitted" => Ok(UpbitPocketTransferState::Submitted),
                "processing" => Ok(UpbitPocketTransferState::Processing),
                "done" => Ok(UpbitPocketTransferState::Done),
                "failed" => Ok(UpbitPocketTransferState::Failed),
                _ => Err(invalid_enum("states", &value)),
            })
            .collect::<Result<_, _>>()?;
        let order_by = value
            .order_by
            .as_deref()
            .map(|value| match value {
                "asc" => Ok(UpbitPocketTransferOrder::Ascending),
                "desc" => Ok(UpbitPocketTransferOrder::Descending),
                _ => Err(invalid_enum("order_by", value)),
            })
            .transpose()?;
        Ok(UpbitPocketTransferQuery {
            from: value.from,
            to: value.to,
            direction,
            states,
            uuids: value.uuids,
            identifiers: value.identifiers,
            start_time: timestamp_option_from_wire(value.start_time, "start_time")?,
            end_time: timestamp_option_from_wire(value.end_time, "end_time")?,
            currency: value.currency,
            limit: value.limit,
            order_by,
        })
    }
}

impl TryFrom<WireUpbitPocketUniversalTransferRequest> for UpbitPocketUniversalTransferRequest {
    type Error = Error;

    fn try_from(value: WireUpbitPocketUniversalTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            from: value.from,
            to: value.to,
            currency: value.currency,
            amount: decimal_from_wire(&value.amount, "amount")?,
            identifier: value.identifier,
        })
    }
}

impl TryFrom<WireUpbitPocketTransferRequest> for UpbitPocketTransferRequest {
    type Error = Error;

    fn try_from(value: WireUpbitPocketTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            to: value.to,
            currency: value.currency,
            amount: decimal_from_wire(&value.amount, "amount")?,
            identifier: value.identifier,
        })
    }
}

impl TryFrom<UpbitPocketTransfer> for WireUpbitPocketTransfer {
    type Error = Error;

    fn try_from(value: UpbitPocketTransfer) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: value.uuid,
            identifier: value.identifier,
            from: value.from,
            to: value.to,
            state: value.state,
            currency: value.currency,
            amount: decimal_to_wire(value.amount),
            created_at: timestamp_to_wire(value.created_at),
        })
    }
}

impl TryFrom<UpbitKrwDeposit> for WireUpbitKrwDeposit {
    type Error = Error;

    fn try_from(value: UpbitKrwDeposit) -> Result<Self, Self::Error> {
        Ok(Self {
            transfer_type: value.transfer_type,
            uuid: value.uuid,
            currency: value.currency,
            net_type: value.net_type,
            txid: value.txid,
            state: value.state,
            created_at: timestamp_to_wire(value.created_at),
            done_at: timestamp_option_to_wire(value.done_at),
            amount: decimal_to_wire(value.amount),
            fee: decimal_to_wire(value.fee),
            transaction_type: value.transaction_type,
        })
    }
}

impl TryFrom<UpbitKrwWithdrawal> for WireUpbitKrwWithdrawal {
    type Error = Error;

    fn try_from(value: UpbitKrwWithdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            transfer_type: value.transfer_type,
            uuid: value.uuid,
            currency: value.currency,
            net_type: value.net_type,
            txid: value.txid,
            state: value.state,
            created_at: timestamp_to_wire(value.created_at),
            done_at: timestamp_option_to_wire(value.done_at),
            amount: decimal_to_wire(value.amount),
            fee: decimal_to_wire(value.fee),
            transaction_type: value.transaction_type,
            is_cancelable: value.is_cancelable,
        })
    }
}

impl TryFrom<UpbitApiKey> for WireUpbitApiKey {
    type Error = Error;

    fn try_from(value: UpbitApiKey) -> Result<Self, Self::Error> {
        Ok(Self {
            access_key: value.access_key,
            expires_at: timestamp_to_wire(value.expires_at),
        })
    }
}

impl TryFrom<BithumbMarketAlert> for WireBithumbMarketAlert {
    type Error = Error;

    fn try_from(value: BithumbMarketAlert) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: value.kind,
            step: bithumb_alert_step_to_wire(value.step)?.to_owned(),
            ends_at: timestamp_to_wire(value.ends_at),
        })
    }
}

impl TryFrom<BithumbNotice> for WireBithumbNotice {
    type Error = Error;

    fn try_from(value: BithumbNotice) -> Result<Self, Self::Error> {
        Ok(Self {
            categories: value.categories,
            title: value.title,
            url: value.url,
            published_at: timestamp_to_wire(value.published_at),
            modified_at: timestamp_to_wire(value.modified_at),
        })
    }
}

impl TryFrom<BithumbApiKey> for WireBithumbApiKey {
    type Error = Error;

    fn try_from(value: BithumbApiKey) -> Result<Self, Self::Error> {
        Ok(Self {
            access_key: value.access_key,
            expires_at: timestamp_to_wire(value.expires_at),
        })
    }
}

impl TryFrom<BithumbKrwWithdrawal> for WireBithumbKrwWithdrawal {
    type Error = Error;

    fn try_from(value: BithumbKrwWithdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            transfer_type: value.transfer_type,
            uuid: value.uuid,
            currency: value.currency,
            net_type: value.net_type,
            txid: value.txid,
            state: value.state,
            created_at: timestamp_option_to_wire(value.created_at),
            done_at: timestamp_option_to_wire(value.done_at),
            amount: decimal_to_wire(value.amount),
            fee: decimal_to_wire(value.fee),
            transaction_type: value.transaction_type,
        })
    }
}

impl TryFrom<BithumbKrwDeposit> for WireBithumbKrwDeposit {
    type Error = Error;

    fn try_from(value: BithumbKrwDeposit) -> Result<Self, Self::Error> {
        Ok(Self {
            transfer_type: value.transfer_type,
            uuid: value.uuid,
            currency: value.currency,
            net_type: value.net_type,
            txid: value.txid,
            state: value.state,
            created_at: timestamp_option_to_wire(value.created_at),
            done_at: timestamp_option_to_wire(value.done_at),
            amount: decimal_to_wire(value.amount),
            fee: decimal_to_wire(value.fee),
            transaction_type: value.transaction_type,
        })
    }
}

impl TryFrom<BithumbAssetFee> for WireBithumbAssetFee {
    type Error = Error;

    fn try_from(value: BithumbAssetFee) -> Result<Self, Self::Error> {
        Ok(Self {
            display_name: value.display_name,
            asset: value.asset,
            networks: value
                .networks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<BithumbNetworkFee> for WireBithumbNetworkFee {
    type Error = Error;

    fn try_from(value: BithumbNetworkFee) -> Result<Self, Self::Error> {
        Ok(Self {
            network: network_to_wire(value.network)?,
            provider_name: value.provider_name,
            deposit_fee: decimal_to_wire(value.deposit_fee),
            minimum_deposit: decimal_to_wire(value.minimum_deposit),
            withdrawal_fee: value.withdrawal_fee.try_into()?,
            minimum_withdrawal: decimal_to_wire(value.minimum_withdrawal),
        })
    }
}

impl TryFrom<BithumbWithdrawalAddress> for WireBithumbWithdrawalAddress {
    type Error = Error;

    fn try_from(value: BithumbWithdrawalAddress) -> Result<Self, Self::Error> {
        Ok(Self {
            currency: value.currency,
            net_type: value.net_type,
            network_name: value.network_name,
            withdraw_address: value.withdraw_address,
            secondary_address: value.secondary_address,
            exchange_name: value.exchange_name,
            owner_type: value.owner_type,
            owner_ko_name: value.owner_ko_name,
            owner_en_name: value.owner_en_name,
            owner_corp_ko_name: value.owner_corp_ko_name,
            owner_corp_en_name: value.owner_corp_en_name,
        })
    }
}

impl TryFrom<WireBithumbOrderDetailRequest> for BithumbOrderDetailRequest {
    type Error = Error;

    fn try_from(value: WireBithumbOrderDetailRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            uuid: value.uuid,
            client_order_id: value.client_order_id,
        })
    }
}

impl TryFrom<WireBithumbOrderListRequest> for BithumbOrderListRequest {
    type Error = Error;

    fn try_from(value: WireBithumbOrderListRequest) -> Result<Self, Self::Error> {
        let state = value
            .state
            .as_deref()
            .map(|value| bithumb_order_list_state_from_wire(value, "state"))
            .transpose()?;
        Ok(Self {
            market: value.market.map(TryInto::try_into).transpose()?,
            state,
            states: value
                .states
                .iter()
                .map(|value| bithumb_order_list_state_from_wire(value, "states"))
                .collect::<Result<_, _>>()?,
            uuids: value.uuids,
            client_order_ids: value.client_order_ids,
            page: value.page,
            limit: value.limit,
            order_by: bithumb_order_direction_from_wire(value.order_by)?,
        })
    }
}

impl TryFrom<BithumbOrderDetailTrade> for WireBithumbOrderDetailTrade {
    type Error = Error;

    fn try_from(value: BithumbOrderDetailTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            uuid: value.uuid,
            price: decimal_to_wire(value.price),
            volume: decimal_to_wire(value.volume),
            funds: decimal_to_wire(value.funds),
            side: value.side,
            created_at: timestamp_to_wire(value.created_at),
        })
    }
}

impl TryFrom<BithumbOrderDetail> for WireBithumbOrderDetail {
    type Error = Error;

    fn try_from(value: BithumbOrderDetail) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: value.uuid,
            client_order_id: value.client_order_id,
            side: value.side,
            order_type: value.order_type,
            price: decimal_to_wire(value.price),
            state: value.state,
            market: value.market.try_into()?,
            created_at: timestamp_to_wire(value.created_at),
            volume: decimal_to_wire(value.volume),
            remaining_volume: decimal_to_wire(value.remaining_volume),
            reserved_fee: decimal_to_wire(value.reserved_fee),
            remaining_fee: decimal_to_wire(value.remaining_fee),
            paid_fee: decimal_to_wire(value.paid_fee),
            locked: decimal_to_wire(value.locked),
            executed_volume: decimal_to_wire(value.executed_volume),
            executed_funds: decimal_to_wire(value.executed_funds),
            trades_count: value.trades_count,
            trades: value
                .trades
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            stp_type: value.stp_type,
            cancel_type: value.cancel_type,
            canceling_uuid: value.canceling_uuid,
            time_in_force: value.time_in_force,
        })
    }
}

impl TryFrom<BithumbOrderListItem> for WireBithumbOrderListItem {
    type Error = Error;

    fn try_from(value: BithumbOrderListItem) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: value.uuid,
            client_order_id: value.client_order_id,
            side: value.side,
            order_type: value.order_type,
            price: decimal_to_wire(value.price),
            state: value.state,
            market: value.market.try_into()?,
            created_at: timestamp_to_wire(value.created_at),
            volume: decimal_to_wire(value.volume),
            remaining_volume: decimal_to_wire(value.remaining_volume),
            reserved_fee: decimal_to_wire(value.reserved_fee),
            remaining_fee: decimal_to_wire(value.remaining_fee),
            paid_fee: decimal_to_wire(value.paid_fee),
            locked: decimal_to_wire(value.locked),
            executed_volume: decimal_to_wire(value.executed_volume),
            executed_funds: decimal_to_wire(value.executed_funds),
            trades_count: value.trades_count,
            stp_type: value.stp_type,
            time_in_force: value.time_in_force,
        })
    }
}

impl TryFrom<BithumbTwapOrder> for WireBithumbTwapOrder {
    type Error = Error;

    fn try_from(value: BithumbTwapOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            side: side_to_wire(value.side).to_owned(),
            price: decimal_to_wire(value.price),
            state: bithumb_twap_state_to_wire(value.state)?.to_owned(),
            market: value.market.try_into()?,
            created_at: timestamp_to_wire(value.created_at),
            volume: decimal_to_wire(value.volume),
            finished_at: timestamp_option_to_wire(value.finished_at),
            total_order_count: value.total_order_count,
            total_trades_count: value.total_trades_count,
            progress_count: value.progress_count,
            total_executed_amount: decimal_to_wire(value.total_executed_amount),
            total_executed_volume: decimal_to_wire(value.total_executed_volume),
            avg_trade_price: decimal_to_wire(value.avg_trade_price),
            wallet_id: value.wallet_id,
            canceled_at: timestamp_option_to_wire(value.canceled_at),
            cancel_type: value.cancel_type,
        })
    }
}

impl TryFrom<BinanceSymbolFilters> for WireBinanceSymbolFilters {
    type Error = Error;

    fn try_from(value: BinanceSymbolFilters) -> Result<Self, Self::Error> {
        Ok(Self {
            symbol: value.symbol,
            tick_size: decimal_option_to_wire(value.tick_size),
            min_price: decimal_option_to_wire(value.min_price),
            max_price: decimal_option_to_wire(value.max_price),
            step_size: decimal_option_to_wire(value.step_size),
            min_quantity: decimal_option_to_wire(value.min_quantity),
            max_quantity: decimal_option_to_wire(value.max_quantity),
            min_notional: decimal_option_to_wire(value.min_notional),
        })
    }
}

impl TryFrom<BinanceSpotOrderDetail> for WireBinanceSpotOrderDetail {
    type Error = Error;

    fn try_from(value: BinanceSpotOrderDetail) -> Result<Self, Self::Error> {
        Ok(Self {
            order: value.order.try_into()?,
            client_order_id: value.client_order_id,
            order_type: value.order_type,
            time_in_force: value.time_in_force,
            filled_quote_quantity: decimal_to_wire(value.filled_quote_quantity),
            updated_at: timestamp_option_to_wire(value.updated_at),
        })
    }
}

impl TryFrom<maxt::adapters::BinanceSpotAveragePrice> for WireBinanceSpotAveragePrice {
    type Error = Error;

    fn try_from(value: maxt::adapters::BinanceSpotAveragePrice) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            minutes: value.minutes,
            price: decimal_to_wire(value.price),
            close_time: timestamp_to_wire(value.close_time),
        })
    }
}

impl TryFrom<WireBinanceDepositHistoryRequest> for BinanceDepositHistoryRequest {
    type Error = Error;

    fn try_from(value: WireBinanceDepositHistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            status: value.status,
            start_time: timestamp_option_from_wire(value.start_time, "start_time")?,
            end_time: timestamp_option_from_wire(value.end_time, "end_time")?,
            offset: value
                .offset
                .as_deref()
                .map(|value| u64_from_wire(value, "offset"))
                .transpose()?,
            limit: value.limit,
            tx_id: value.tx_id,
            include_source: value.include_source,
        })
    }
}

impl TryFrom<WireBinanceWithdrawHistoryRequest> for BinanceWithdrawHistoryRequest {
    type Error = Error;

    fn try_from(value: WireBinanceWithdrawHistoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            withdraw_order_id: value.withdraw_order_id,
            status: value.status,
            offset: value
                .offset
                .as_deref()
                .map(|value| u64_from_wire(value, "offset"))
                .transpose()?,
            limit: value.limit,
            id_list: value.id_list,
            start_time: timestamp_option_from_wire(value.start_time, "start_time")?,
            end_time: timestamp_option_from_wire(value.end_time, "end_time")?,
        })
    }
}

impl TryFrom<BinanceSpotCommissionRates> for WireBinanceSpotCommissionRates {
    type Error = Error;

    fn try_from(value: BinanceSpotCommissionRates) -> Result<Self, Self::Error> {
        Ok(Self {
            maker: decimal_to_wire(value.maker),
            taker: decimal_to_wire(value.taker),
            buyer: decimal_to_wire(value.buyer),
            seller: decimal_to_wire(value.seller),
        })
    }
}

impl TryFrom<BinanceSpotAccountBalance> for WireBinanceSpotAccountBalance {
    type Error = Error;

    fn try_from(value: BinanceSpotAccountBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            free: decimal_to_wire(value.free),
            locked: decimal_to_wire(value.locked),
        })
    }
}

impl TryFrom<BinanceSpotAccountInformation> for WireBinanceSpotAccountInformation {
    type Error = Error;

    fn try_from(value: BinanceSpotAccountInformation) -> Result<Self, Self::Error> {
        Ok(Self {
            maker_commission: value.maker_commission.to_string(),
            taker_commission: value.taker_commission.to_string(),
            buyer_commission: value.buyer_commission.to_string(),
            seller_commission: value.seller_commission.to_string(),
            commission_rates: value.commission_rates.try_into()?,
            can_trade: value.can_trade,
            can_withdraw: value.can_withdraw,
            can_deposit: value.can_deposit,
            update_time: timestamp_to_wire(value.update_time),
            account_type: value.account_type,
            balances: value
                .balances
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            permissions: value.permissions,
            uid: value.uid.map(|value| value.to_string()),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceSpotCancelledOrder> for WireBinanceSpotCancelledOrder {
    type Error = Error;

    fn try_from(value: BinanceSpotCancelledOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            symbol: value.symbol,
            original_client_order_id: value.original_client_order_id,
            order_id: value.order_id,
            client_order_id: value.client_order_id,
            status: value.status,
            price: decimal_option_to_wire(value.price),
            original_quantity: decimal_option_to_wire(value.original_quantity),
            executed_quantity: decimal_option_to_wire(value.executed_quantity),
            cumulative_quote_quantity: decimal_option_to_wire(value.cumulative_quote_quantity),
            transact_time: timestamp_option_to_wire(value.transact_time),
            order_list_id: value.order_list_id,
            contingency_type: value.contingency_type,
            list_status_type: value.list_status_type,
            list_order_status: value.list_order_status,
            list_client_order_id: value.list_client_order_id,
            transaction_time: timestamp_option_to_wire(value.transaction_time),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceSpotCancelAllOpenOrders> for WireBinanceSpotCancelAllOpenOrders {
    type Error = Error;

    fn try_from(value: BinanceSpotCancelAllOpenOrders) -> Result<Self, Self::Error> {
        Ok(Self {
            reports: value
                .reports
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceExchangeSymbol> for WireBinanceExchangeSymbol {
    type Error = Error;

    fn try_from(value: BinanceExchangeSymbol) -> Result<Self, Self::Error> {
        Ok(Self {
            symbol: value.symbol,
            status: value.status,
            base_asset: value.base_asset,
            quote_asset: value.quote_asset,
            contract_type: value.contract_type,
            margin_asset: value.margin_asset,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceExchangeInfo> for WireBinanceExchangeInfo {
    type Error = Error;

    fn try_from(value: BinanceExchangeInfo) -> Result<Self, Self::Error> {
        let venue = match value.venue {
            BinanceMarket::Spot => "spot",
            BinanceMarket::UsdMFutures => "usd_m",
            _ => return Err(Error::adapter("unsupported Binance venue")),
        };
        Ok(Self {
            venue: venue.to_owned(),
            timezone: value.timezone,
            server_time: timestamp_option_to_wire(value.server_time),
            symbols: value
                .symbols
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceUsdMAccountAsset> for WireBinanceUsdMAccountAsset {
    type Error = Error;

    fn try_from(value: BinanceUsdMAccountAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            wallet_balance: decimal_to_wire(value.wallet_balance),
            unrealized_profit: decimal_to_wire(value.unrealized_profit),
            margin_balance: decimal_to_wire(value.margin_balance),
            maintenance_margin: decimal_to_wire(value.maintenance_margin),
            initial_margin: decimal_to_wire(value.initial_margin),
            position_initial_margin: decimal_to_wire(value.position_initial_margin),
            open_order_initial_margin: decimal_to_wire(value.open_order_initial_margin),
            cross_wallet_balance: decimal_to_wire(value.cross_wallet_balance),
            cross_unrealized_profit: decimal_to_wire(value.cross_unrealized_profit),
            available_balance: decimal_to_wire(value.available_balance),
            max_withdraw_amount: decimal_to_wire(value.max_withdraw_amount),
            update_time: timestamp_to_wire(value.update_time),
        })
    }
}

impl TryFrom<BinanceUsdMAccountPosition> for WireBinanceUsdMAccountPosition {
    type Error = Error;

    fn try_from(value: BinanceUsdMAccountPosition) -> Result<Self, Self::Error> {
        Ok(Self {
            symbol: value.symbol,
            position_side: value.position_side,
            position_amount: decimal_to_wire(value.position_amount),
            unrealized_profit: decimal_to_wire(value.unrealized_profit),
            isolated_margin: decimal_to_wire(value.isolated_margin),
            notional: decimal_to_wire(value.notional),
            isolated_wallet: decimal_to_wire(value.isolated_wallet),
            initial_margin: decimal_to_wire(value.initial_margin),
            maintenance_margin: decimal_to_wire(value.maintenance_margin),
            update_time: timestamp_to_wire(value.update_time),
        })
    }
}

impl TryFrom<BinanceUsdMAccountInformation> for WireBinanceUsdMAccountInformation {
    type Error = Error;

    fn try_from(value: BinanceUsdMAccountInformation) -> Result<Self, Self::Error> {
        Ok(Self {
            total_initial_margin: decimal_to_wire(value.total_initial_margin),
            total_maintenance_margin: decimal_to_wire(value.total_maintenance_margin),
            total_wallet_balance: decimal_to_wire(value.total_wallet_balance),
            total_unrealized_profit: decimal_to_wire(value.total_unrealized_profit),
            total_margin_balance: decimal_to_wire(value.total_margin_balance),
            total_position_initial_margin: decimal_to_wire(value.total_position_initial_margin),
            total_open_order_initial_margin: decimal_to_wire(value.total_open_order_initial_margin),
            total_cross_wallet_balance: decimal_to_wire(value.total_cross_wallet_balance),
            total_cross_unrealized_profit: decimal_to_wire(value.total_cross_unrealized_profit),
            available_balance: decimal_to_wire(value.available_balance),
            max_withdraw_amount: decimal_to_wire(value.max_withdraw_amount),
            assets: value
                .assets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            positions: value
                .positions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceUsdMPositionInformation> for WireBinanceUsdMPositionInformation {
    type Error = Error;

    fn try_from(value: BinanceUsdMPositionInformation) -> Result<Self, Self::Error> {
        Ok(Self {
            symbol: value.symbol,
            position_side: value.position_side,
            position_amount: decimal_to_wire(value.position_amount),
            entry_price: decimal_to_wire(value.entry_price),
            break_even_price: decimal_to_wire(value.break_even_price),
            mark_price: decimal_to_wire(value.mark_price),
            unrealized_profit: decimal_to_wire(value.unrealized_profit),
            liquidation_price: decimal_to_wire(value.liquidation_price),
            isolated_margin: decimal_to_wire(value.isolated_margin),
            notional: decimal_to_wire(value.notional),
            margin_asset: value.margin_asset,
            isolated_wallet: decimal_to_wire(value.isolated_wallet),
            initial_margin: decimal_to_wire(value.initial_margin),
            maintenance_margin: decimal_to_wire(value.maintenance_margin),
            position_initial_margin: decimal_to_wire(value.position_initial_margin),
            open_order_initial_margin: decimal_to_wire(value.open_order_initial_margin),
            adl: value.adl.to_string(),
            bid_notional: decimal_to_wire(value.bid_notional),
            ask_notional: decimal_to_wire(value.ask_notional),
            update_time: timestamp_to_wire(value.update_time),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceCoinNetworkInformation> for WireBinanceCoinNetworkInformation {
    type Error = Error;

    fn try_from(value: BinanceCoinNetworkInformation) -> Result<Self, Self::Error> {
        Ok(Self {
            network: value.network,
            deposit_enabled: value.deposit_enabled,
            withdraw_enabled: value.withdraw_enabled,
            busy: value.busy,
            withdrawal_integer_multiple: decimal_option_to_wire(value.withdrawal_integer_multiple),
            withdrawal_fee: decimal_option_to_wire(value.withdrawal_fee),
            minimum_withdrawal: decimal_option_to_wire(value.minimum_withdrawal),
            maximum_withdrawal: decimal_option_to_wire(value.maximum_withdrawal),
            withdrawal_tag: value.withdrawal_tag,
            is_default: value.is_default,
            minimum_confirmations: value.minimum_confirmations.map(|value| value.to_string()),
            unlock_confirmations: value.unlock_confirmations.map(|value| value.to_string()),
            contract_address: value.contract_address,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceCoinInformation> for WireBinanceCoinInformation {
    type Error = Error;

    fn try_from(value: BinanceCoinInformation) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            deposit_all_enabled: value.deposit_all_enabled,
            withdraw_all_enabled: value.withdraw_all_enabled,
            name: value.name,
            free: decimal_option_to_wire(value.free),
            locked: decimal_option_to_wire(value.locked),
            freeze: decimal_option_to_wire(value.freeze),
            withdrawing: decimal_option_to_wire(value.withdrawing),
            is_legal_money: value.is_legal_money,
            trading: value.trading,
            networks: value
                .networks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceApiKeyPermissions> for WireBinanceApiKeyPermissions {
    type Error = Error;

    fn try_from(value: BinanceApiKeyPermissions) -> Result<Self, Self::Error> {
        Ok(Self {
            ip_restrict: value.ip_restrict,
            create_time: timestamp_option_to_wire(value.create_time),
            enable_reading: value.enable_reading,
            enable_withdrawals: value.enable_withdrawals,
            enable_internal_transfer: value.enable_internal_transfer,
            enable_margin: value.enable_margin,
            enable_spot_and_margin_trading: value.enable_spot_and_margin_trading,
            enable_futures: value.enable_futures,
            permits_universal_transfer: value.permits_universal_transfer,
            enable_vanilla_options: value.enable_vanilla_options,
            enable_fix_api_trade: value.enable_fix_api_trade,
            enable_fix_read_only: value.enable_fix_read_only,
            enable_portfolio_margin_trading: value.enable_portfolio_margin_trading,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceDepositHistoryEntry> for WireBinanceDepositHistoryEntry {
    type Error = Error;

    fn try_from(value: BinanceDepositHistoryEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            amount: decimal_to_wire(value.amount),
            coin: value.coin,
            network: value.network,
            status: value.status,
            address: value.address,
            address_tag: value.address_tag,
            tx_id: value.tx_id,
            insert_time: timestamp_to_wire(value.insert_time),
            complete_time: timestamp_option_to_wire(value.complete_time),
            transfer_type: value.transfer_type,
            source_address: value.source_address,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceDepositHistory> for WireBinanceDepositHistory {
    type Error = Error;

    fn try_from(value: BinanceDepositHistory) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: value
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceQuestionnaireRequirements> for WireBinanceQuestionnaireRequirements {
    type Error = Error;

    fn try_from(value: BinanceQuestionnaireRequirements) -> Result<Self, Self::Error> {
        Ok(Self {
            questionnaire_country_code: value.questionnaire_country_code,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceWithdrawalAddress> for WireBinanceWithdrawalAddress {
    type Error = Error;

    fn try_from(value: BinanceWithdrawalAddress) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.address,
            address_tag: value.address_tag,
            coin: value.coin,
            network: value.network,
            white_status: value.white_status,
            name: value.name,
            origin: value.origin,
            origin_type: value.origin_type,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceWithdrawHistoryEntry> for WireBinanceWithdrawHistoryEntry {
    type Error = Error;

    fn try_from(value: BinanceWithdrawHistoryEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            amount: decimal_to_wire(value.amount),
            transaction_fee: decimal_to_wire(value.transaction_fee),
            coin: value.coin,
            status: value.status,
            address: value.address,
            tx_id: value.tx_id,
            apply_time: value.apply_time,
            network: value.network,
            withdraw_order_id: value.withdraw_order_id,
            info: value.info,
            transfer_type: value.transfer_type,
            confirm_no: value.confirm_no,
            wallet_type: value.wallet_type,
            tx_key: value.tx_key,
            complete_time: value.complete_time,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceWithdrawHistory> for WireBinanceWithdrawHistory {
    type Error = Error;

    fn try_from(value: BinanceWithdrawHistory) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: value
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<BinanceMarkPrice> for WireBinanceMarkPrice {
    type Error = Error;

    fn try_from(value: BinanceMarkPrice) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            mark_price: decimal_to_wire(value.mark_price),
            index_price: decimal_to_wire(value.index_price),
            estimated_settle_price: decimal_option_to_wire(value.estimated_settle_price),
            last_funding_rate: decimal_to_wire(value.last_funding_rate),
            interest_rate: decimal_to_wire(value.interest_rate),
            next_funding_time: timestamp_to_wire(value.next_funding_time),
            time: timestamp_to_wire(value.time),
        })
    }
}

impl TryFrom<BinanceOpenInterest> for WireBinanceOpenInterest {
    type Error = Error;

    fn try_from(value: BinanceOpenInterest) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            open_interest: decimal_to_wire(value.open_interest),
            time: timestamp_to_wire(value.time),
        })
    }
}

impl TryFrom<HyperliquidLedgerEntry> for WireHyperliquidLedgerEntry {
    type Error = Error;

    fn try_from(value: HyperliquidLedgerEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: hyperliquid_ledger_kind_to_wire(value.kind)?,
            time: timestamp_to_wire(value.time),
            hash: value.hash,
            asset: value.asset,
            amount: decimal_option_to_wire(value.amount),
            fee: decimal_option_to_wire(value.fee),
            counterparty: value.counterparty,
        })
    }
}

impl TryFrom<HyperliquidAssetContext> for WireHyperliquidAssetContext {
    type Error = Error;

    fn try_from(value: HyperliquidAssetContext) -> Result<Self, Self::Error> {
        Ok(Self {
            mid_price: decimal_option_to_wire(value.mid_price),
            mark_price: decimal_option_to_wire(value.mark_price),
            oracle_price: decimal_option_to_wire(value.oracle_price),
            funding_rate: decimal_option_to_wire(value.funding_rate),
            open_interest: decimal_option_to_wire(value.open_interest),
            size_decimals: value.size_decimals,
            price_decimals: value.price_decimals,
        })
    }
}

impl TryFrom<HyperliquidCandleSnapshot> for WireHyperliquidCandleSnapshot {
    type Error = Error;

    fn try_from(value: HyperliquidCandleSnapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            market: value.market.try_into()?,
            interval: value.interval,
            open_time: timestamp_to_wire(value.open_time),
            close_time: timestamp_to_wire(value.close_time),
            open: decimal_to_wire(value.open),
            high: decimal_to_wire(value.high),
            low: decimal_to_wire(value.low),
            close: decimal_to_wire(value.close),
            volume: decimal_to_wire(value.volume),
            trade_count: value.trade_count.map(|value| value.to_string()),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidBookLevel> for WireHyperliquidBookLevel {
    type Error = Error;

    fn try_from(value: HyperliquidBookLevel) -> Result<Self, Self::Error> {
        Ok(Self {
            price: decimal_to_wire(value.price),
            size: decimal_to_wire(value.size),
            order_count: value.order_count.map(|value| value.to_string()),
        })
    }
}

impl TryFrom<HyperliquidL2Book> for WireHyperliquidL2Book {
    type Error = Error;

    fn try_from(value: HyperliquidL2Book) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            market: value.market.try_into()?,
            time: timestamp_to_wire(value.time),
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
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidRecentTrade> for WireHyperliquidRecentTrade {
    type Error = Error;

    fn try_from(value: HyperliquidRecentTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            market: value.market.try_into()?,
            side: value.side,
            price: decimal_to_wire(value.price),
            size: decimal_to_wire(value.size),
            time: timestamp_to_wire(value.time),
            trade_id: value.trade_id,
            hash: value.hash,
            users: value.users,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidFundingHistoryEntry> for WireHyperliquidFundingHistoryEntry {
    type Error = Error;

    fn try_from(value: HyperliquidFundingHistoryEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            market: value.market.try_into()?,
            funding_rate: decimal_to_wire(value.funding_rate),
            premium: decimal_option_to_wire(value.premium),
            time: timestamp_to_wire(value.time),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidUserFunding> for WireHyperliquidUserFunding {
    type Error = Error;

    fn try_from(value: HyperliquidUserFunding) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: value.kind,
            coin: value.coin,
            market: value.market.try_into()?,
            usdc: decimal_to_wire(value.usdc),
            funding_rate: decimal_to_wire(value.funding_rate),
            position_size: decimal_option_to_wire(value.position_size),
            sample_count: value.sample_count.map(|value| value.to_string()),
            hash: value.hash,
            time: timestamp_to_wire(value.time),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidSpotBalance> for WireHyperliquidSpotBalance {
    type Error = Error;

    fn try_from(value: HyperliquidSpotBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            token: value.token,
            total: decimal_to_wire(value.total),
            hold: decimal_to_wire(value.hold),
            entry_notional: decimal_option_to_wire(value.entry_notional),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidSpotClearinghouseState> for WireHyperliquidSpotClearinghouseState {
    type Error = Error;

    fn try_from(value: HyperliquidSpotClearinghouseState) -> Result<Self, Self::Error> {
        Ok(Self {
            balances: value
                .balances
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidEvmContract> for WireHyperliquidEvmContract {
    type Error = Error;

    fn try_from(value: HyperliquidEvmContract) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.address,
            extra_wei_decimals: value.extra_wei_decimals,
        })
    }
}

impl TryFrom<HyperliquidSpotToken> for WireHyperliquidSpotToken {
    type Error = Error;

    fn try_from(value: HyperliquidSpotToken) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            size_decimals: value.size_decimals,
            wei_decimals: value.wei_decimals,
            index: value.index,
            token_id: value.token_id,
            is_canonical: value.is_canonical,
            evm_contract: value.evm_contract.map(TryInto::try_into).transpose()?,
            full_name: value.full_name,
            deployer_trading_fee_share: decimal_option_to_wire(value.deployer_trading_fee_share),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidSpotPair> for WireHyperliquidSpotPair {
    type Error = Error;

    fn try_from(value: HyperliquidSpotPair) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            tokens: value.tokens,
            index: value.index,
            is_canonical: value.is_canonical,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidSpotMeta> for WireHyperliquidSpotMeta {
    type Error = Error;

    fn try_from(value: HyperliquidSpotMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            tokens: value
                .tokens
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            universe: value
                .universe
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidSpotAssetContext> for WireHyperliquidSpotAssetContext {
    type Error = Error;

    fn try_from(value: HyperliquidSpotAssetContext) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            mid_price: decimal_option_to_wire(value.mid_price),
            mark_price: decimal_option_to_wire(value.mark_price),
            previous_day_price: decimal_option_to_wire(value.previous_day_price),
            day_base_volume: decimal_option_to_wire(value.day_base_volume),
            day_notional_volume: decimal_option_to_wire(value.day_notional_volume),
            circulating_supply: decimal_option_to_wire(value.circulating_supply),
            total_supply: decimal_option_to_wire(value.total_supply),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidSpotMetaAndAssetContexts> for WireHyperliquidSpotMetaAndAssetContexts {
    type Error = Error;

    fn try_from(value: HyperliquidSpotMetaAndAssetContexts) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta.try_into()?,
            contexts: value
                .contexts
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidMidPrice> for WireHyperliquidMidPrice {
    type Error = Error;

    fn try_from(value: HyperliquidMidPrice) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            price: decimal_to_wire(value.price),
        })
    }
}

impl TryFrom<HyperliquidUserRateLimit> for WireHyperliquidUserRateLimit {
    type Error = Error;

    fn try_from(value: HyperliquidUserRateLimit) -> Result<Self, Self::Error> {
        Ok(Self {
            cumulative_volume: decimal_to_wire(value.cumulative_volume),
            requests_used: value.requests_used.to_string(),
            requests_cap: value.requests_cap.to_string(),
            requests_surplus: value.requests_surplus.to_string(),
        })
    }
}

impl TryFrom<HyperliquidUserRole> for WireHyperliquidUserRole {
    type Error = Error;

    fn try_from(value: HyperliquidUserRole) -> Result<Self, Self::Error> {
        match value {
            HyperliquidUserRole::User => Ok(Self::User),
            HyperliquidUserRole::Agent { user } => Ok(Self::Agent { user }),
            HyperliquidUserRole::Vault => Ok(Self::Vault),
            HyperliquidUserRole::SubAccount { master } => Ok(Self::SubAccount { master }),
            HyperliquidUserRole::Missing => Ok(Self::Missing),
            HyperliquidUserRole::Other { role, data_json } => Ok(Self::Other { role, data_json }),
            _ => Err(binding_contract("HyperliquidUserRole")),
        }
    }
}

impl TryFrom<HyperliquidReferrer> for WireHyperliquidReferrer {
    type Error = Error;

    fn try_from(value: HyperliquidReferrer) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.address,
            code: value.code,
        })
    }
}

impl TryFrom<HyperliquidReferral> for WireHyperliquidReferral {
    type Error = Error;

    fn try_from(value: HyperliquidReferral) -> Result<Self, Self::Error> {
        Ok(Self {
            referred_by: value.referred_by.map(TryInto::try_into).transpose()?,
            cumulative_volume: decimal_to_wire(value.cumulative_volume),
            unclaimed_rewards: decimal_to_wire(value.unclaimed_rewards),
            claimed_rewards: decimal_to_wire(value.claimed_rewards),
            builder_rewards: decimal_to_wire(value.builder_rewards),
            referrer_state_json: value.referrer_state_json,
            reward_history_json: value.reward_history_json,
            token_to_state_json: value.token_to_state_json,
        })
    }
}

impl TryFrom<HyperliquidDailyVolume> for WireHyperliquidDailyVolume {
    type Error = Error;

    fn try_from(value: HyperliquidDailyVolume) -> Result<Self, Self::Error> {
        Ok(Self {
            date: value.date,
            user_cross: decimal_to_wire(value.user_cross),
            user_add: decimal_to_wire(value.user_add),
            exchange: decimal_to_wire(value.exchange),
        })
    }
}

impl TryFrom<HyperliquidUserFees> for WireHyperliquidUserFees {
    type Error = Error;

    fn try_from(value: HyperliquidUserFees) -> Result<Self, Self::Error> {
        Ok(Self {
            daily_volumes: value
                .daily_volumes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            fee_schedule_json: value.fee_schedule_json,
            user_cross_rate: decimal_to_wire(value.user_cross_rate),
            user_add_rate: decimal_to_wire(value.user_add_rate),
            user_spot_cross_rate: decimal_option_to_wire(value.user_spot_cross_rate),
            user_spot_add_rate: decimal_option_to_wire(value.user_spot_add_rate),
            active_referral_discount: decimal_option_to_wire(value.active_referral_discount),
            details_json: value.details_json,
        })
    }
}

impl TryFrom<HyperliquidUserFill> for WireHyperliquidUserFill {
    type Error = Error;

    fn try_from(value: HyperliquidUserFill) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            price: decimal_to_wire(value.price),
            size: decimal_to_wire(value.size),
            side: value.side,
            time: timestamp_to_wire(value.time),
            start_position: decimal_to_wire(value.start_position),
            direction: value.direction,
            closed_pnl: decimal_to_wire(value.closed_pnl),
            hash: value.hash,
            order_id: value.order_id.to_string(),
            crossed: value.crossed,
            fee: decimal_to_wire(value.fee),
            builder_fee: decimal_option_to_wire(value.builder_fee),
            trade_id: value.trade_id.to_string(),
            fee_token: value.fee_token,
            twap_id: value.twap_id.map(|value| value.to_string()),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<WireHyperliquidOrderReference> for HyperliquidOrderReference {
    type Error = Error;

    fn try_from(value: WireHyperliquidOrderReference) -> Result<Self, Self::Error> {
        Ok(match value {
            WireHyperliquidOrderReference::OrderId { value } => {
                Self::OrderId(u64_from_wire(&value, "reference.value")?)
            }
            WireHyperliquidOrderReference::ClientOrderId { value } => Self::ClientOrderId(value),
        })
    }
}

impl TryFrom<HyperliquidOpenOrder> for WireHyperliquidOpenOrder {
    type Error = Error;

    fn try_from(value: HyperliquidOpenOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            limit_price: decimal_to_wire(value.limit_price),
            order_id: value.order_id.to_string(),
            side: value.side,
            size: decimal_to_wire(value.size),
            timestamp: timestamp_to_wire(value.timestamp),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidOrderDetail> for WireHyperliquidOrderDetail {
    type Error = Error;

    fn try_from(value: HyperliquidOrderDetail) -> Result<Self, Self::Error> {
        Ok(Self {
            coin: value.coin,
            side: value.side,
            limit_price: decimal_to_wire(value.limit_price),
            size: decimal_to_wire(value.size),
            order_id: value.order_id.to_string(),
            timestamp: timestamp_to_wire(value.timestamp),
            trigger_condition: value.trigger_condition,
            is_trigger: value.is_trigger,
            trigger_price: decimal_to_wire(value.trigger_price),
            children_json: value.children_json,
            is_position_tpsl: value.is_position_tpsl,
            reduce_only: value.reduce_only,
            order_type: value.order_type,
            original_size: decimal_to_wire(value.original_size),
            time_in_force: value.time_in_force,
            client_order_id: value.client_order_id,
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidOrderInfo> for WireHyperliquidOrderInfo {
    type Error = Error;

    fn try_from(value: HyperliquidOrderInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            order: value.order.try_into()?,
            status: value.status,
            status_timestamp: timestamp_to_wire(value.status_timestamp),
            raw_json: value.raw_json,
        })
    }
}

impl TryFrom<HyperliquidOrderStatusResponse> for WireHyperliquidOrderStatusResponse {
    type Error = Error;

    fn try_from(value: HyperliquidOrderStatusResponse) -> Result<Self, Self::Error> {
        Ok(match value {
            HyperliquidOrderStatusResponse::Order(value) => Self::Order {
                value: (*value).try_into()?,
            },
            HyperliquidOrderStatusResponse::UnknownOrder => Self::UnknownOrder,
            HyperliquidOrderStatusResponse::Other { status, raw_json } => {
                Self::Other { status, raw_json }
            }
            _ => {
                return Err(Error::adapter(
                    "unsupported Hyperliquid order status response",
                ));
            }
        })
    }
}

impl TryFrom<HyperliquidPortfolioPoint> for WireHyperliquidPortfolioPoint {
    type Error = Error;

    fn try_from(value: HyperliquidPortfolioPoint) -> Result<Self, Self::Error> {
        Ok(Self {
            time: timestamp_to_wire(value.time),
            value: decimal_to_wire(value.value),
        })
    }
}

impl TryFrom<HyperliquidPortfolioPeriod> for WireHyperliquidPortfolioPeriod {
    type Error = Error;

    fn try_from(value: HyperliquidPortfolioPeriod) -> Result<Self, Self::Error> {
        Ok(Self {
            period: value.period,
            account_value_history: value
                .account_value_history
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            pnl_history: value
                .pnl_history
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            volume: decimal_to_wire(value.volume),
        })
    }
}

impl TryFrom<HyperliquidSubAccount> for WireHyperliquidSubAccount {
    type Error = Error;

    fn try_from(value: HyperliquidSubAccount) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            user: value.user,
            master: value.master,
            perpetual_state_json: value.perpetual_state_json,
            spot_state_json: value.spot_state_json,
        })
    }
}

impl TryFrom<HyperliquidVaultEquity> for WireHyperliquidVaultEquity {
    type Error = Error;

    fn try_from(value: HyperliquidVaultEquity) -> Result<Self, Self::Error> {
        Ok(Self {
            vault_address: value.vault_address,
            equity: decimal_to_wire(value.equity),
            locked_until: timestamp_option_to_wire(value.locked_until),
        })
    }
}

impl TryFrom<Market> for WireMarket {
    type Error = Error;

    fn try_from(value: Market) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: value.exchange.id().to_owned(),
            kind: market_kind_to_wire(value.kind)?.to_owned(),
            base: value.base,
            quote: value.quote,
        })
    }
}

impl TryFrom<MarketInfo> for WireMarketInfo {
    type Error = Error;

    fn try_from(value: MarketInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            native_symbol: value.native_symbol,
            status: market_status_to_wire(value.status)?.to_owned(),
            korean_name: value.korean_name,
            english_name: value.english_name,
        })
    }
}

impl TryFrom<WireMarketInfo> for MarketInfo {
    type Error = Error;

    fn try_from(value: WireMarketInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            native_symbol: value.native_symbol,
            status: market_status_from_wire(&value.status, "status")?,
            korean_name: value.korean_name,
            english_name: value.english_name,
        })
    }
}

impl TryFrom<Trade> for WireTrade {
    type Error = Error;

    fn try_from(value: Trade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_to_wire(value.timestamp),
            price: decimal_to_wire(value.price),
            quantity: decimal_to_wire(value.quantity),
            taker_side: side_to_wire(value.taker_side).to_owned(),
            id: value.id,
        })
    }
}

impl TryFrom<WireTrade> for Trade {
    type Error = Error;

    fn try_from(value: WireTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_from_wire(&value.timestamp, "timestamp")?,
            price: decimal_from_wire(&value.price, "price")?,
            quantity: decimal_from_wire(&value.quantity, "quantity")?,
            taker_side: side_from_wire(&value.taker_side, "taker_side")?,
            id: value.id,
        })
    }
}

impl TryFrom<Level> for WireLevel {
    type Error = Error;

    fn try_from(value: Level) -> Result<Self, Self::Error> {
        Ok(Self {
            price: decimal_to_wire(value.price),
            quantity: decimal_to_wire(value.quantity),
        })
    }
}

impl TryFrom<WireLevel> for Level {
    type Error = Error;

    fn try_from(value: WireLevel) -> Result<Self, Self::Error> {
        Ok(Self {
            price: decimal_from_wire(&value.price, "price")?,
            quantity: decimal_from_wire(&value.quantity, "quantity")?,
        })
    }
}

impl TryFrom<OrderBook> for WireOrderBook {
    type Error = Error;

    fn try_from(value: OrderBook) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_to_wire(value.timestamp),
            bids: value
                .bids
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
            asks: value
                .asks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
        })
    }
}

impl TryFrom<WireOrderBook> for OrderBook {
    type Error = Error;

    fn try_from(value: WireOrderBook) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_from_wire(&value.timestamp, "timestamp")?,
            bids: value
                .bids
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
            asks: value
                .asks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
        })
    }
}

impl TryFrom<Ticker> for WireTicker {
    type Error = Error;

    fn try_from(value: Ticker) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_to_wire(value.timestamp),
            last_trade_time: timestamp_option_to_wire(value.last_trade_time),
            last_price: decimal_to_wire(value.last_price),
            change: decimal_option_to_wire(value.change),
            change_rate: decimal_option_to_wire(value.change_rate),
            high: decimal_option_to_wire(value.high),
            low: decimal_option_to_wire(value.low),
            volume: decimal_option_to_wire(value.volume),
            quote_volume: decimal_option_to_wire(value.quote_volume),
        })
    }
}

impl TryFrom<WireTicker> for Ticker {
    type Error = Error;

    fn try_from(value: WireTicker) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_from_wire(&value.timestamp, "timestamp")?,
            last_trade_time: timestamp_option_from_wire(value.last_trade_time, "last_trade_time")?,
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

impl TryFrom<Candle> for WireCandle {
    type Error = Error;

    fn try_from(value: Candle) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            interval: interval_to_wire(value.interval)?.to_owned(),
            open_time: timestamp_to_wire(value.open_time),
            open: decimal_to_wire(value.open),
            high: decimal_to_wire(value.high),
            low: decimal_to_wire(value.low),
            close: decimal_to_wire(value.close),
            volume: decimal_to_wire(value.volume),
            quote_volume: decimal_option_to_wire(value.quote_volume),
            closed: value.closed,
        })
    }
}

impl TryFrom<WireCandle> for Candle {
    type Error = Error;

    fn try_from(value: WireCandle) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            interval: interval_from_wire(&value.interval, "interval")?,
            open_time: timestamp_from_wire(&value.open_time, "open_time")?,
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

impl TryFrom<Balance> for WireBalance {
    type Error = Error;

    fn try_from(value: Balance) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            available: decimal_to_wire(value.available),
            locked: decimal_to_wire(value.locked),
        })
    }
}

impl TryFrom<WireBalance> for Balance {
    type Error = Error;

    fn try_from(value: WireBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            available: decimal_from_wire(&value.available, "available")?,
            locked: decimal_from_wire(&value.locked, "locked")?,
        })
    }
}

impl TryFrom<OrderAccount> for WireOrderAccount {
    type Error = Error;

    fn try_from(value: OrderAccount) -> Result<Self, Self::Error> {
        Ok(Self {
            balance: value.balance.try_into()?,
            average_buy_price: decimal_to_wire(value.average_buy_price),
            average_buy_price_modified: value.average_buy_price_modified,
            average_buy_price_unit: value.average_buy_price_unit,
        })
    }
}

impl TryFrom<WireOrderAccount> for OrderAccount {
    type Error = Error;

    fn try_from(value: WireOrderAccount) -> Result<Self, Self::Error> {
        Ok(Self {
            balance: value.balance.try_into()?,
            average_buy_price: decimal_from_wire(&value.average_buy_price, "average_buy_price")?,
            average_buy_price_modified: value.average_buy_price_modified,
            average_buy_price_unit: value.average_buy_price_unit,
        })
    }
}

impl TryFrom<OrderOption> for WireOrderOption {
    type Error = Error;

    fn try_from(value: OrderOption) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: value.provider_id,
            order_type: value
                .order_type
                .map(order_type_to_wire)
                .transpose()?
                .map(str::to_owned),
            time_in_force: value
                .time_in_force
                .map(time_in_force_to_wire)
                .transpose()?
                .map(str::to_owned),
        })
    }
}

impl TryFrom<WireOrderOption> for OrderOption {
    type Error = Error;

    fn try_from(value: WireOrderOption) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: value.provider_id,
            order_type: value
                .order_type
                .as_deref()
                .map(|value| order_type_from_wire(value, "order_type"))
                .transpose()?,
            time_in_force: value
                .time_in_force
                .as_deref()
                .map(|value| time_in_force_from_wire(value, "time_in_force"))
                .transpose()?,
        })
    }
}

impl TryFrom<OrderRules> for WireOrderRules {
    type Error = Error;

    fn try_from(value: OrderRules) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            market_name: value.market_name,
            status: market_status_to_wire(value.status)?.to_owned(),
            buy_fee_rate: decimal_to_wire(value.buy_fee_rate),
            sell_fee_rate: decimal_to_wire(value.sell_fee_rate),
            maker_buy_fee_rate: decimal_to_wire(value.maker_buy_fee_rate),
            maker_sell_fee_rate: decimal_to_wire(value.maker_sell_fee_rate),
            sides: value
                .sides
                .into_iter()
                .map(side_to_wire)
                .map(str::to_owned)
                .collect(),
            buy_options: value
                .buy_options
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            sell_options: value
                .sell_options
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            buy_price_unit: value.buy_price_unit.map(decimal_to_wire),
            sell_price_unit: value.sell_price_unit.map(decimal_to_wire),
            minimum_buy_total: decimal_to_wire(value.minimum_buy_total),
            minimum_sell_total: decimal_to_wire(value.minimum_sell_total),
            maximum_total: decimal_to_wire(value.maximum_total),
            quote_account: value.quote_account.try_into()?,
            base_account: value.base_account.try_into()?,
        })
    }
}

impl TryFrom<WireOrderRules> for OrderRules {
    type Error = Error;

    fn try_from(value: WireOrderRules) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            market_name: value.market_name,
            status: market_status_from_wire(&value.status, "status")?,
            buy_fee_rate: decimal_from_wire(&value.buy_fee_rate, "buy_fee_rate")?,
            sell_fee_rate: decimal_from_wire(&value.sell_fee_rate, "sell_fee_rate")?,
            maker_buy_fee_rate: decimal_from_wire(&value.maker_buy_fee_rate, "maker_buy_fee_rate")?,
            maker_sell_fee_rate: decimal_from_wire(
                &value.maker_sell_fee_rate,
                "maker_sell_fee_rate",
            )?,
            sides: value
                .sides
                .iter()
                .map(|value| side_from_wire(value, "sides"))
                .collect::<Result<_, _>>()?,
            buy_options: value
                .buy_options
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            sell_options: value
                .sell_options
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            buy_price_unit: value
                .buy_price_unit
                .as_deref()
                .map(|value| decimal_from_wire(value, "buy_price_unit"))
                .transpose()?,
            sell_price_unit: value
                .sell_price_unit
                .as_deref()
                .map(|value| decimal_from_wire(value, "sell_price_unit"))
                .transpose()?,
            minimum_buy_total: decimal_from_wire(&value.minimum_buy_total, "minimum_buy_total")?,
            minimum_sell_total: decimal_from_wire(&value.minimum_sell_total, "minimum_sell_total")?,
            maximum_total: decimal_from_wire(&value.maximum_total, "maximum_total")?,
            quote_account: value.quote_account.try_into()?,
            base_account: value.base_account.try_into()?,
        })
    }
}

impl TryFrom<WithdrawalFee> for WireWithdrawalFee {
    type Error = Error;

    fn try_from(value: WithdrawalFee) -> Result<Self, Self::Error> {
        match value {
            WithdrawalFee::Fixed(value) => Ok(Self::Fixed {
                value: decimal_to_wire(value),
            }),
            WithdrawalFee::Rate {
                rate,
                minimum,
                maximum,
            } => Ok(Self::Rate {
                rate: decimal_to_wire(rate),
                minimum: decimal_option_to_wire(minimum),
                maximum: decimal_option_to_wire(maximum),
            }),
            _ => Err(binding_contract("WithdrawalFee")),
        }
    }
}

impl TryFrom<WireWithdrawalFee> for WithdrawalFee {
    type Error = Error;

    fn try_from(value: WireWithdrawalFee) -> Result<Self, Self::Error> {
        match value {
            WireWithdrawalFee::Fixed { value } => Ok(Self::Fixed(decimal_from_wire(
                &value,
                "withdrawal_fee.value",
            )?)),
            WireWithdrawalFee::Rate {
                rate,
                minimum,
                maximum,
            } => Ok(Self::Rate {
                rate: decimal_from_wire(&rate, "withdrawal_fee.rate")?,
                minimum: decimal_option_from_wire(minimum, "withdrawal_fee.minimum")?,
                maximum: decimal_option_from_wire(maximum, "withdrawal_fee.maximum")?,
            }),
        }
    }
}

impl TryFrom<AssetNetwork> for WireAssetNetwork {
    type Error = Error;

    fn try_from(value: AssetNetwork) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: value.exchange.id().to_owned(),
            asset: value.asset,
            network: network_to_wire(value.network)?,
            provider_id: value.provider_id,
            deposit_enabled: value.deposit_enabled,
            withdrawal_enabled: value.withdrawal_enabled,
            withdrawal_fee: value.withdrawal_fee.map(TryInto::try_into).transpose()?,
            minimum_withdrawal: decimal_option_to_wire(value.minimum_withdrawal),
            maximum_withdrawal: decimal_option_to_wire(value.maximum_withdrawal),
            memo_required: value.memo_required,
        })
    }
}

impl TryFrom<WireAssetNetwork> for AssetNetwork {
    type Error = Error;

    fn try_from(value: WireAssetNetwork) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: exchange_from_wire(&value.exchange, "exchange")?,
            asset: value.asset,
            network: network_from_wire(&value.network),
            provider_id: value.provider_id,
            deposit_enabled: value.deposit_enabled,
            withdrawal_enabled: value.withdrawal_enabled,
            withdrawal_fee: value.withdrawal_fee.map(TryInto::try_into).transpose()?,
            minimum_withdrawal: decimal_option_from_wire(
                value.minimum_withdrawal,
                "minimum_withdrawal",
            )?,
            maximum_withdrawal: decimal_option_from_wire(
                value.maximum_withdrawal,
                "maximum_withdrawal",
            )?,
            memo_required: value.memo_required,
        })
    }
}

impl TryFrom<DepositAddress> for WireDepositAddress {
    type Error = Error;

    fn try_from(value: DepositAddress) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: value.exchange.id().to_owned(),
            asset: value.asset,
            network: network_to_wire(value.network)?,
            address: value.address,
            memo: value.memo,
        })
    }
}

impl TryFrom<WireDepositAddress> for DepositAddress {
    type Error = Error;

    fn try_from(value: WireDepositAddress) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: exchange_from_wire(&value.exchange, "exchange")?,
            asset: value.asset,
            network: network_from_wire(&value.network),
            address: value.address,
            memo: value.memo,
        })
    }
}

impl TryFrom<DepositAddressEntry> for WireDepositAddressEntry {
    type Error = Error;

    fn try_from(value: DepositAddressEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: value.exchange.id().to_owned(),
            asset: value.asset,
            network: value.network.map(network_to_wire).transpose()?,
            provider_network: value.provider_network,
            address: value.address,
            memo: value.memo,
        })
    }
}

impl TryFrom<WireDepositAddressEntry> for DepositAddressEntry {
    type Error = Error;

    fn try_from(value: WireDepositAddressEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: exchange_from_wire(&value.exchange, "exchange")?,
            asset: value.asset,
            network: value.network.as_deref().map(network_from_wire),
            provider_network: value.provider_network,
            address: value.address,
            memo: value.memo,
        })
    }
}

impl TryFrom<ExchangeDestination> for WireExchangeDestination {
    type Error = Error;

    fn try_from(value: ExchangeDestination) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: value.exchange.id().to_owned(),
            asset: value.asset,
            network: network_to_wire(value.network)?,
            address: value.address,
            memo: value.memo,
        })
    }
}

impl TryFrom<WireExchangeDestination> for ExchangeDestination {
    type Error = Error;

    fn try_from(value: WireExchangeDestination) -> Result<Self, Self::Error> {
        Ok(Self {
            exchange: exchange_from_wire(&value.exchange, "destination.exchange")?,
            asset: value.asset,
            network: network_from_wire(&value.network),
            address: value.address,
            memo: value.memo,
        })
    }
}

impl TryFrom<ChainDestination> for WireChainDestination {
    type Error = Error;

    fn try_from(value: ChainDestination) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            network: network_to_wire(value.network)?,
            address: value.address,
            memo: value.memo,
        })
    }
}

impl From<WireChainDestination> for ChainDestination {
    fn from(value: WireChainDestination) -> Self {
        Self {
            asset: value.asset,
            network: network_from_wire(&value.network),
            address: value.address,
            memo: value.memo,
        }
    }
}

impl TryFrom<ExchangeTransferRequest> for WireExchangeTransferRequest {
    type Error = Error;

    fn try_from(value: ExchangeTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            source_network: value.source_network.map(network_to_wire).transpose()?,
            destination_network: value.destination_network.map(network_to_wire).transpose()?,
            amount: decimal_to_wire(value.amount),
        })
    }
}

impl TryFrom<WireExchangeTransferRequest> for ExchangeTransferRequest {
    type Error = Error;

    fn try_from(value: WireExchangeTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            source_network: value.source_network.map(|value| network_from_wire(&value)),
            destination_network: value
                .destination_network
                .map(|value| network_from_wire(&value)),
            amount: decimal_from_wire(&value.amount, "amount")?,
        })
    }
}

impl TryFrom<ChainTransferRequest> for WireChainTransferRequest {
    type Error = Error;

    fn try_from(value: ChainTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            source_network: value.source_network.map(network_to_wire).transpose()?,
            destination: value.destination.try_into()?,
            amount: decimal_to_wire(value.amount),
        })
    }
}

impl TryFrom<WireChainTransferRequest> for ChainTransferRequest {
    type Error = Error;

    fn try_from(value: WireChainTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            source_network: value.source_network.map(|value| network_from_wire(&value)),
            destination: value.destination.into(),
            amount: decimal_from_wire(&value.amount, "amount")?,
        })
    }
}

impl TryFrom<TransferDestination> for WireTransferDestination {
    type Error = Error;

    fn try_from(value: TransferDestination) -> Result<Self, Self::Error> {
        match value {
            TransferDestination::Exchange(value) => Ok(Self::Exchange {
                value: value.try_into()?,
            }),
            TransferDestination::Chain(value) => Ok(Self::Chain {
                value: value.try_into()?,
            }),
            _ => Err(binding_contract("TransferDestination")),
        }
    }
}

impl TryFrom<WireTransferDestination> for TransferDestination {
    type Error = Error;

    fn try_from(value: WireTransferDestination) -> Result<Self, Self::Error> {
        match value {
            WireTransferDestination::Exchange { value } => Ok(Self::Exchange(value.try_into()?)),
            WireTransferDestination::Chain { value } => Ok(Self::Chain(value.into())),
        }
    }
}

impl TryFrom<TravelRuleRequirement> for WireTravelRuleRequirement {
    type Error = Error;

    fn try_from(value: TravelRuleRequirement) -> Result<Self, Self::Error> {
        match value {
            TravelRuleRequirement::NotRequired => Ok(Self::NotRequired),
            TravelRuleRequirement::Required { consent_url } => Ok(Self::Required { consent_url }),
            _ => Err(binding_contract("TravelRuleRequirement")),
        }
    }
}

impl From<WireTravelRuleRequirement> for TravelRuleRequirement {
    fn from(value: WireTravelRuleRequirement) -> Self {
        match value {
            WireTravelRuleRequirement::NotRequired => Self::NotRequired,
            WireTravelRuleRequirement::Required { consent_url } => Self::Required { consent_url },
        }
    }
}

impl TryFrom<WithdrawalQuote> for WireWithdrawalQuote {
    type Error = Error;

    fn try_from(value: WithdrawalQuote) -> Result<Self, Self::Error> {
        Ok(Self {
            fee: decimal_option_to_wire(value.fee),
            expected_receive: decimal_option_to_wire(value.expected_receive),
            minimum_amount: decimal_option_to_wire(value.minimum_amount),
            maximum_amount: decimal_option_to_wire(value.maximum_amount),
            address_allowed: value.address_allowed,
            travel_rule: value.travel_rule.try_into()?,
            expires_at: timestamp_option_to_wire(value.expires_at),
        })
    }
}

impl TryFrom<WireWithdrawalQuote> for WithdrawalQuote {
    type Error = Error;

    fn try_from(value: WireWithdrawalQuote) -> Result<Self, Self::Error> {
        Ok(Self {
            fee: decimal_option_from_wire(value.fee, "fee")?,
            expected_receive: decimal_option_from_wire(value.expected_receive, "expected_receive")?,
            minimum_amount: decimal_option_from_wire(value.minimum_amount, "minimum_amount")?,
            maximum_amount: decimal_option_from_wire(value.maximum_amount, "maximum_amount")?,
            address_allowed: value.address_allowed,
            travel_rule: value.travel_rule.into(),
            expires_at: timestamp_option_from_wire(value.expires_at, "expires_at")?,
        })
    }
}

impl TryFrom<TransferPlan> for WireTransferPlan {
    type Error = Error;

    fn try_from(value: TransferPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            source: value.source.id().to_owned(),
            destination: value.destination.map(|exchange| exchange.id().to_owned()),
            request: value.request.try_into()?,
            quote: value.quote.try_into()?,
            created_at: timestamp_to_wire(value.created_at),
            expires_at: timestamp_to_wire(value.expires_at),
        })
    }
}

impl TryFrom<WireTransferPlan> for TransferPlan {
    type Error = Error;

    fn try_from(value: WireTransferPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            source: exchange_from_wire(&value.source, "source")?,
            destination: value
                .destination
                .map(|exchange| exchange_from_wire(&exchange, "destination"))
                .transpose()?,
            request: value.request.try_into()?,
            quote: value.quote.try_into()?,
            created_at: timestamp_from_wire(&value.created_at, "created_at")?,
            expires_at: timestamp_from_wire(&value.expires_at, "expires_at")?,
        })
    }
}

impl TryFrom<Withdrawal> for WireWithdrawal {
    type Error = Error;

    fn try_from(value: Withdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            asset: value.asset,
            network: value.network.map(network_to_wire).transpose()?,
            provider_network: value.provider_network,
            amount: decimal_to_wire(value.amount),
            fee: decimal_option_to_wire(value.fee),
            destination: value.destination.map(TryInto::try_into).transpose()?,
            status: withdrawal_status_to_wire(value.status)?.to_owned(),
            provider_status: value.provider_status,
            tx_id: value.tx_id,
            created_at: timestamp_option_to_wire(value.created_at),
        })
    }
}

impl TryFrom<WireWithdrawal> for Withdrawal {
    type Error = Error;

    fn try_from(value: WireWithdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            asset: value.asset,
            network: value.network.as_deref().map(network_from_wire),
            provider_network: value.provider_network,
            amount: decimal_from_wire(&value.amount, "amount")?,
            fee: decimal_option_from_wire(value.fee, "fee")?,
            destination: value.destination.map(TryInto::try_into).transpose()?,
            status: withdrawal_status_from_wire(&value.status, "status")?,
            provider_status: value.provider_status,
            tx_id: value.tx_id,
            created_at: timestamp_option_from_wire(value.created_at, "created_at")?,
        })
    }
}

impl TryFrom<Deposit> for WireDeposit {
    type Error = Error;

    fn try_from(value: Deposit) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            asset: value.asset,
            network: value.network.map(network_to_wire).transpose()?,
            provider_network: value.provider_network,
            amount: decimal_to_wire(value.amount),
            address: value.address,
            memo: value.memo,
            status: deposit_status_to_wire(value.status)?.to_owned(),
            provider_status: value.provider_status,
            tx_id: value.tx_id,
            created_at: timestamp_option_to_wire(value.created_at),
        })
    }
}

impl TryFrom<WireDeposit> for Deposit {
    type Error = Error;

    fn try_from(value: WireDeposit) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            asset: value.asset,
            network: value.network.as_deref().map(network_from_wire),
            provider_network: value.provider_network,
            amount: decimal_from_wire(&value.amount, "amount")?,
            address: value.address,
            memo: value.memo,
            status: deposit_status_from_wire(&value.status, "status")?,
            provider_status: value.provider_status,
            tx_id: value.tx_id,
            created_at: timestamp_option_from_wire(value.created_at, "created_at")?,
        })
    }
}

impl TryFrom<Order> for WireOrder {
    type Error = Error;

    fn try_from(value: Order) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            market: value.market.try_into()?,
            side: side_to_wire(value.side).to_owned(),
            status: order_status_to_wire(value.status)?.to_owned(),
            filled_quantity: decimal_to_wire(value.filled_quantity),
            remaining_quantity: decimal_to_wire(value.remaining_quantity),
            price: decimal_option_to_wire(value.price),
            created_at: timestamp_option_to_wire(value.created_at),
        })
    }
}

impl TryFrom<WireOrder> for Order {
    type Error = Error;

    fn try_from(value: WireOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            market: value.market.try_into()?,
            side: side_from_wire(&value.side, "side")?,
            status: order_status_from_wire(&value.status, "status")?,
            filled_quantity: decimal_from_wire(&value.filled_quantity, "filled_quantity")?,
            remaining_quantity: decimal_from_wire(&value.remaining_quantity, "remaining_quantity")?,
            price: decimal_option_from_wire(value.price, "price")?,
            created_at: timestamp_option_from_wire(value.created_at, "created_at")?,
        })
    }
}

impl TryFrom<CancelledOrder> for WireCancelledOrder {
    type Error = Error;

    fn try_from(value: CancelledOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: value.order_id,
            client_id: value.client_id,
            market: value.market.map(TryInto::try_into).transpose()?,
            cancelled_at: timestamp_option_to_wire(value.cancelled_at),
        })
    }
}

impl TryFrom<WireCancelledOrder> for CancelledOrder {
    type Error = Error;

    fn try_from(value: WireCancelledOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: value.order_id,
            client_id: value.client_id,
            market: value.market.map(TryInto::try_into).transpose()?,
            cancelled_at: timestamp_option_from_wire(value.cancelled_at, "cancelled_at")?,
        })
    }
}

impl TryFrom<OrderCancelFailure> for WireOrderCancelFailure {
    type Error = Error;

    fn try_from(value: OrderCancelFailure) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: value.order_id,
            client_id: value.client_id,
            market: value.market.map(TryInto::try_into).transpose()?,
            code: value.code,
            message: value.message,
        })
    }
}

impl TryFrom<WireOrderCancelFailure> for OrderCancelFailure {
    type Error = Error;

    fn try_from(value: WireOrderCancelFailure) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: value.order_id,
            client_id: value.client_id,
            market: value.market.map(TryInto::try_into).transpose()?,
            code: value.code,
            message: value.message,
        })
    }
}

impl TryFrom<CancelOrdersResult> for WireCancelOrdersResult {
    type Error = Error;

    fn try_from(value: CancelOrdersResult) -> Result<Self, Self::Error> {
        Ok(Self {
            cancelled: value
                .cancelled
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            failed: value
                .failed
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<WireCancelOrdersResult> for CancelOrdersResult {
    type Error = Error;

    fn try_from(value: WireCancelOrdersResult) -> Result<Self, Self::Error> {
        Ok(Self {
            cancelled: value
                .cancelled
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            failed: value
                .failed
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<Position> for WirePosition {
    type Error = Error;

    fn try_from(value: Position) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            side: value.side.map(side_to_wire).map(str::to_owned),
            quantity: decimal_to_wire(value.quantity),
            entry_price: decimal_option_to_wire(value.entry_price),
            mark_price: decimal_option_to_wire(value.mark_price),
            notional: decimal_option_to_wire(value.notional),
            unrealized_pnl: decimal_option_to_wire(value.unrealized_pnl),
            leverage: decimal_option_to_wire(value.leverage),
            margin_mode: value
                .margin_mode
                .map(margin_mode_to_wire)
                .transpose()?
                .map(str::to_owned),
        })
    }
}

impl TryFrom<WirePosition> for Position {
    type Error = Error;

    fn try_from(value: WirePosition) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            side: value
                .side
                .as_deref()
                .map(|value| side_from_wire(value, "side"))
                .transpose()?,
            quantity: decimal_from_wire(&value.quantity, "quantity")?,
            entry_price: decimal_option_from_wire(value.entry_price, "entry_price")?,
            mark_price: decimal_option_from_wire(value.mark_price, "mark_price")?,
            notional: decimal_option_from_wire(value.notional, "notional")?,
            unrealized_pnl: decimal_option_from_wire(value.unrealized_pnl, "unrealized_pnl")?,
            leverage: decimal_option_from_wire(value.leverage, "leverage")?,
            margin_mode: value
                .margin_mode
                .as_deref()
                .map(|value| margin_mode_from_wire(value, "margin_mode"))
                .transpose()?,
        })
    }
}

impl TryFrom<MarginSummary> for WireMarginSummary {
    type Error = Error;

    fn try_from(value: MarginSummary) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.asset,
            equity: decimal_option_to_wire(value.equity),
            margin_balance: decimal_option_to_wire(value.margin_balance),
            available_balance: decimal_option_to_wire(value.available_balance),
        })
    }
}

impl TryFrom<WireMarginSummary> for MarginSummary {
    type Error = Error;

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

impl TryFrom<FundingRate> for WireFundingRate {
    type Error = Error;

    fn try_from(value: FundingRate) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_to_wire(value.timestamp),
            rate: decimal_to_wire(value.rate),
            mark_price: decimal_option_to_wire(value.mark_price),
        })
    }
}

impl TryFrom<WireFundingRate> for FundingRate {
    type Error = Error;

    fn try_from(value: WireFundingRate) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_from_wire(&value.timestamp, "timestamp")?,
            rate: decimal_from_wire(&value.rate, "rate")?,
            mark_price: decimal_option_from_wire(value.mark_price, "mark_price")?,
        })
    }
}

impl TryFrom<FundingPayment> for WireFundingPayment {
    type Error = Error;

    fn try_from(value: FundingPayment) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_to_wire(value.timestamp),
            amount: decimal_to_wire(value.amount),
            rate: decimal_option_to_wire(value.rate),
            id: value.id,
        })
    }
}

impl TryFrom<WireFundingPayment> for FundingPayment {
    type Error = Error;

    fn try_from(value: WireFundingPayment) -> Result<Self, Self::Error> {
        Ok(Self {
            market: value.market.try_into()?,
            timestamp: timestamp_from_wire(&value.timestamp, "timestamp")?,
            amount: decimal_from_wire(&value.amount, "amount")?,
            rate: decimal_option_from_wire(value.rate, "rate")?,
            id: value.id,
        })
    }
}

impl<T, W> TryFrom<Page<T>> for WirePage<W>
where
    W: TryFrom<T, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Page<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            items: value
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
            next: value.next.map(|cursor| cursor.as_str().to_owned()),
        })
    }
}

impl<T, W> TryFrom<WirePage<W>> for Page<T>
where
    T: TryFrom<W, Error = Error>,
{
    type Error = Error;

    fn try_from(value: WirePage<W>) -> Result<Self, Self::Error> {
        Ok(Self {
            items: value
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()?,
            next: value.next.map(Cursor::new),
        })
    }
}

pub(crate) fn decimal_to_wire(value: Decimal) -> String {
    value.to_string()
}

fn decimal_option_to_wire(value: Option<Decimal>) -> Option<String> {
    value.map(decimal_to_wire)
}

fn decimal_option_from_wire(value: Option<String>, field: &str) -> maxt::Result<Option<Decimal>> {
    value
        .as_deref()
        .map(|value| decimal_from_wire(value, field))
        .transpose()
}

fn timestamp_option_to_wire(value: Option<Timestamp>) -> Option<String> {
    value.map(timestamp_to_wire)
}

fn timestamp_option_from_wire(
    value: Option<String>,
    field: &str,
) -> maxt::Result<Option<Timestamp>> {
    value
        .as_deref()
        .map(|value| timestamp_from_wire(value, field))
        .transpose()
}

fn exchange_from_wire(value: &str, field: &str) -> maxt::Result<Exchange> {
    exchange_from_id(value).ok_or_else(|| invalid_enum(field, value))
}

pub(crate) fn network_from_wire(value: &str) -> Network {
    match value {
        "bitcoin" => Network::Bitcoin,
        "ethereum" => Network::Ethereum,
        "arbitrum" => Network::Arbitrum,
        "bnb_smart_chain" => Network::BnbSmartChain,
        "tron" => Network::Tron,
        "solana" => Network::Solana,
        "polygon" => Network::Polygon,
        "base" => Network::Base,
        "optimism" => Network::Optimism,
        "avalanche_c" => Network::AvalancheC,
        "xrp_ledger" => Network::XrpLedger,
        "stellar" => Network::Stellar,
        "cosmos" => Network::Cosmos,
        "aptos" => Network::Aptos,
        "sui" => Network::Sui,
        "ton" => Network::Ton,
        "near" => Network::Near,
        "polkadot" => Network::Polkadot,
        value => Network::Other(value.to_owned()),
    }
}

fn network_to_wire(value: Network) -> maxt::Result<String> {
    Ok(value.id().to_owned())
}

fn withdrawal_status_from_wire(value: &str, field: &str) -> maxt::Result<WithdrawalStatus> {
    match value {
        "pending" => Ok(WithdrawalStatus::Pending),
        "processing" => Ok(WithdrawalStatus::Processing),
        "completed" => Ok(WithdrawalStatus::Completed),
        "cancelled" => Ok(WithdrawalStatus::Cancelled),
        "failed" => Ok(WithdrawalStatus::Failed),
        "unknown" => Ok(WithdrawalStatus::Unknown),
        _ => Err(invalid_enum(field, value)),
    }
}

fn withdrawal_status_to_wire(value: WithdrawalStatus) -> maxt::Result<&'static str> {
    match value {
        WithdrawalStatus::Pending => Ok("pending"),
        WithdrawalStatus::Processing => Ok("processing"),
        WithdrawalStatus::Completed => Ok("completed"),
        WithdrawalStatus::Cancelled => Ok("cancelled"),
        WithdrawalStatus::Failed => Ok("failed"),
        WithdrawalStatus::Unknown => Ok("unknown"),
        _ => Err(binding_contract("WithdrawalStatus")),
    }
}

fn deposit_status_from_wire(value: &str, field: &str) -> maxt::Result<DepositStatus> {
    match value {
        "pending" => Ok(DepositStatus::Pending),
        "completed" => Ok(DepositStatus::Completed),
        "failed" => Ok(DepositStatus::Failed),
        "unknown" => Ok(DepositStatus::Unknown),
        _ => Err(invalid_enum(field, value)),
    }
}

fn deposit_status_to_wire(value: DepositStatus) -> maxt::Result<&'static str> {
    match value {
        DepositStatus::Pending => Ok("pending"),
        DepositStatus::Completed => Ok("completed"),
        DepositStatus::Failed => Ok("failed"),
        DepositStatus::Unknown => Ok("unknown"),
        _ => Err(binding_contract("DepositStatus")),
    }
}

pub(crate) fn market_kind_from_wire(value: &str, field: &str) -> maxt::Result<MarketKind> {
    match value {
        "spot" => Ok(MarketKind::Spot),
        "perpetual" => Ok(MarketKind::Perpetual),
        _ => Err(invalid_enum(field, value)),
    }
}

fn market_kind_to_wire(value: MarketKind) -> maxt::Result<&'static str> {
    match value {
        MarketKind::Spot => Ok("spot"),
        MarketKind::Perpetual => Ok("perpetual"),
        _ => Err(binding_contract("MarketKind")),
    }
}

fn market_status_from_wire(value: &str, field: &str) -> maxt::Result<MarketStatus> {
    match value {
        "active" => Ok(MarketStatus::Active),
        "paused" => Ok(MarketStatus::Paused),
        "delisted" => Ok(MarketStatus::Delisted),
        "unknown" => Ok(MarketStatus::Unknown),
        _ => Err(invalid_enum(field, value)),
    }
}

fn market_status_to_wire(value: MarketStatus) -> maxt::Result<&'static str> {
    match value {
        MarketStatus::Active => Ok("active"),
        MarketStatus::Paused => Ok("paused"),
        MarketStatus::Delisted => Ok("delisted"),
        MarketStatus::Unknown => Ok("unknown"),
        _ => Err(binding_contract("MarketStatus")),
    }
}

fn side_from_wire(value: &str, field: &str) -> maxt::Result<Side> {
    match value {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        _ => Err(invalid_enum(field, value)),
    }
}

fn side_to_wire(value: Side) -> &'static str {
    match value {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn interval_from_wire(value: &str, field: &str) -> maxt::Result<Interval> {
    Ok(match value {
        "sec1" => Interval::Sec1,
        "min1" => Interval::Min1,
        "min3" => Interval::Min3,
        "min5" => Interval::Min5,
        "min10" => Interval::Min10,
        "min15" => Interval::Min15,
        "min30" => Interval::Min30,
        "hour1" => Interval::Hour1,
        "hour2" => Interval::Hour2,
        "hour4" => Interval::Hour4,
        "hour6" => Interval::Hour6,
        "hour8" => Interval::Hour8,
        "hour12" => Interval::Hour12,
        "day1" => Interval::Day1,
        "day3" => Interval::Day3,
        "week1" => Interval::Week1,
        "month1" => Interval::Month1,
        _ => return Err(invalid_enum(field, value)),
    })
}

fn interval_to_wire(value: Interval) -> maxt::Result<&'static str> {
    match value {
        Interval::Sec1 => Ok("sec1"),
        Interval::Min1 => Ok("min1"),
        Interval::Min3 => Ok("min3"),
        Interval::Min5 => Ok("min5"),
        Interval::Min10 => Ok("min10"),
        Interval::Min15 => Ok("min15"),
        Interval::Min30 => Ok("min30"),
        Interval::Hour1 => Ok("hour1"),
        Interval::Hour2 => Ok("hour2"),
        Interval::Hour4 => Ok("hour4"),
        Interval::Hour6 => Ok("hour6"),
        Interval::Hour8 => Ok("hour8"),
        Interval::Hour12 => Ok("hour12"),
        Interval::Day1 => Ok("day1"),
        Interval::Day3 => Ok("day3"),
        Interval::Week1 => Ok("week1"),
        Interval::Month1 => Ok("month1"),
        _ => Err(binding_contract("Interval")),
    }
}

fn order_type_from_wire(value: &str, field: &str) -> maxt::Result<OrderType> {
    match value {
        "market" => Ok(OrderType::Market),
        "limit" => Ok(OrderType::Limit),
        "best" => Ok(OrderType::Best),
        _ => Err(invalid_enum(field, value)),
    }
}

fn order_type_to_wire(value: OrderType) -> maxt::Result<&'static str> {
    match value {
        OrderType::Market => Ok("market"),
        OrderType::Limit => Ok("limit"),
        OrderType::Best => Ok("best"),
        _ => Err(binding_contract("OrderType")),
    }
}

fn order_status_from_wire(value: &str, field: &str) -> maxt::Result<OrderStatus> {
    match value {
        "accepted" => Ok(OrderStatus::Accepted),
        "open" => Ok(OrderStatus::Open),
        "partially_filled" => Ok(OrderStatus::PartiallyFilled),
        "filled" => Ok(OrderStatus::Filled),
        "cancelled" => Ok(OrderStatus::Cancelled),
        "rejected" => Ok(OrderStatus::Rejected),
        "unknown" => Ok(OrderStatus::Unknown),
        _ => Err(invalid_enum(field, value)),
    }
}

fn order_status_to_wire(value: OrderStatus) -> maxt::Result<&'static str> {
    match value {
        OrderStatus::Accepted => Ok("accepted"),
        OrderStatus::Open => Ok("open"),
        OrderStatus::PartiallyFilled => Ok("partially_filled"),
        OrderStatus::Filled => Ok("filled"),
        OrderStatus::Cancelled => Ok("cancelled"),
        OrderStatus::Rejected => Ok("rejected"),
        OrderStatus::Unknown => Ok("unknown"),
        _ => Err(binding_contract("OrderStatus")),
    }
}

fn time_in_force_from_wire(value: &str, field: &str) -> maxt::Result<TimeInForce> {
    match value {
        "good_til_cancelled" => Ok(TimeInForce::GoodTilCancelled),
        "immediate_or_cancel" => Ok(TimeInForce::ImmediateOrCancel),
        "fill_or_kill" => Ok(TimeInForce::FillOrKill),
        "post_only" => Ok(TimeInForce::PostOnly),
        _ => Err(invalid_enum(field, value)),
    }
}

fn time_in_force_to_wire(value: TimeInForce) -> maxt::Result<&'static str> {
    match value {
        TimeInForce::GoodTilCancelled => Ok("good_til_cancelled"),
        TimeInForce::ImmediateOrCancel => Ok("immediate_or_cancel"),
        TimeInForce::FillOrKill => Ok("fill_or_kill"),
        TimeInForce::PostOnly => Ok("post_only"),
        _ => Err(binding_contract("TimeInForce")),
    }
}

fn margin_mode_from_wire(value: &str, field: &str) -> maxt::Result<MarginMode> {
    match value {
        "cross" => Ok(MarginMode::Cross),
        "isolated" => Ok(MarginMode::Isolated),
        _ => Err(invalid_enum(field, value)),
    }
}

fn margin_mode_to_wire(value: MarginMode) -> maxt::Result<&'static str> {
    match value {
        MarginMode::Cross => Ok("cross"),
        MarginMode::Isolated => Ok("isolated"),
        _ => Err(binding_contract("MarginMode")),
    }
}

fn bithumb_twap_state_to_wire(value: BithumbTwapState) -> maxt::Result<&'static str> {
    match value {
        BithumbTwapState::Progress => Ok("progress"),
        BithumbTwapState::Done => Ok("done"),
        BithumbTwapState::Cancel => Ok("cancel"),
    }
}

fn overflow_from_wire(value: &str, field: &str) -> maxt::Result<Overflow> {
    match value {
        "backpressure" => Ok(Overflow::Backpressure),
        "drop_newest" => Ok(Overflow::DropNewest),
        _ => Err(invalid_enum(field, value)),
    }
}

fn overflow_to_wire(value: Overflow) -> maxt::Result<&'static str> {
    match value {
        Overflow::Backpressure => Ok("backpressure"),
        Overflow::DropNewest => Ok("drop_newest"),
        _ => Err(binding_contract("Overflow")),
    }
}

#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
fn bithumb_alert_step_to_wire(value: BithumbAlertStep) -> maxt::Result<&'static str> {
    match value {
        BithumbAlertStep::Caution => Ok("caution"),
        BithumbAlertStep::Warning => Ok("warning"),
        BithumbAlertStep::Danger => Ok("danger"),
        BithumbAlertStep::Unknown => Ok("unknown"),
        _ => Err(binding_contract("BithumbAlertStep")),
    }
}

#[allow(dead_code, reason = "Reserved for Task 9 provider bridge input")]
fn hyperliquid_ledger_kind_to_wire(value: HyperliquidLedgerKind) -> maxt::Result<String> {
    Ok(match value {
        HyperliquidLedgerKind::Deposit => "deposit".to_owned(),
        HyperliquidLedgerKind::Withdraw => "withdraw".to_owned(),
        HyperliquidLedgerKind::InternalTransfer => "internal_transfer".to_owned(),
        HyperliquidLedgerKind::SubAccountTransfer => "sub_account_transfer".to_owned(),
        HyperliquidLedgerKind::SpotTransfer => "spot_transfer".to_owned(),
        HyperliquidLedgerKind::AccountClassTransfer => "account_class_transfer".to_owned(),
        HyperliquidLedgerKind::VaultDeposit => "vault_deposit".to_owned(),
        HyperliquidLedgerKind::VaultWithdraw => "vault_withdraw".to_owned(),
        HyperliquidLedgerKind::VaultDistribution => "vault_distribution".to_owned(),
        HyperliquidLedgerKind::Liquidation => "liquidation".to_owned(),
        HyperliquidLedgerKind::Other(value) => value,
        _ => return Err(binding_contract("HyperliquidLedgerKind")),
    })
}

fn invalid_enum(field: &str, value: &str) -> Error {
    Error::InvalidRequest {
        field: field.to_owned(),
        detail: format!("unknown value `{value}`"),
    }
}

fn binding_contract(type_name: &str) -> Error {
    Error::adapter(format!(
        "maxt binding contract does not map a new {type_name} variant"
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireError {
    InvalidRequest {
        field: String,
        detail: String,
    },
    Transfer {
        transfer_kind: String,
        detail: String,
    },
    Unsupported {
        feature: String,
        exchange: String,
        detail: String,
    },
    Adapter {
        detail: String,
    },
    Auth {
        detail: String,
    },
    Exchange {
        exchange: String,
        code: String,
        message: String,
        #[serde(deserialize_with = "explicit_option")]
        status: Option<u16>,
        exchange_kind: String,
    },
    Transport {
        detail: String,
    },
    Decode {
        detail: String,
    },
}

impl From<Error> for WireError {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidRequest { field, detail } => Self::InvalidRequest { field, detail },
            Error::Transfer { kind, detail } => match transfer_error_kind_to_wire(kind) {
                Ok(transfer_kind) => Self::Transfer {
                    transfer_kind: transfer_kind.to_owned(),
                    detail,
                },
                Err(_) => Self::Adapter {
                    detail: format!("maxt error has unknown transfer classification: {kind:?}"),
                },
            },
            Error::Unsupported {
                feature,
                exchange,
                detail,
            } => match exchange_from_id(exchange) {
                Some(exchange) => Self::Unsupported {
                    feature: feature.id().to_owned(),
                    exchange: exchange.id().to_owned(),
                    detail,
                },
                None => Self::Adapter {
                    detail: unsupported_error_diagnostic(
                        "maxt error has unknown metadata",
                        feature.id(),
                        exchange,
                        &detail,
                    ),
                },
            },
            Error::Adapter { detail } => Self::Adapter { detail },
            Error::Auth { detail } => Self::Auth { detail },
            Error::Exchange {
                exchange,
                code,
                message,
                status,
                kind,
            } => {
                let exchange_id = exchange_from_id(exchange);
                let exchange_kind = exchange_error_kind_to_wire(kind);
                match (exchange_id, exchange_kind) {
                    (Some(exchange), Ok(exchange_kind)) => Self::Exchange {
                        exchange: exchange.id().to_owned(),
                        code,
                        message,
                        status,
                        exchange_kind: exchange_kind.to_owned(),
                    },
                    (_, exchange_kind) => Self::Adapter {
                        detail: exchange_error_diagnostic(
                            "maxt error has unknown metadata",
                            exchange,
                            &code,
                            &message,
                            status,
                            exchange_kind
                                .map(str::to_owned)
                                .unwrap_or_else(|_| format!("{kind:?}"))
                                .as_str(),
                        ),
                    },
                }
            }
            Error::Transport { detail } => Self::Transport { detail },
            Error::Decode { detail } => Self::Decode { detail },
            _ => Self::Adapter {
                detail: "unrecognized maxt error variant".to_owned(),
            },
        }
    }
}

impl TryFrom<WireError> for Error {
    type Error = Error;

    fn try_from(value: WireError) -> Result<Self, Self::Error> {
        match value {
            WireError::InvalidRequest { field, detail } => {
                Ok(Self::InvalidRequest { field, detail })
            }
            WireError::Transfer {
                transfer_kind,
                detail,
            } => transfer_error_kind_from_wire(&transfer_kind)
                .map(|kind| Self::Transfer { kind, detail })
                .ok_or_else(|| {
                    Self::adapter(format!(
                        "foreign transfer error has unknown classification: {transfer_kind:?}"
                    ))
                }),
            WireError::Unsupported {
                feature,
                exchange,
                detail,
            } => match (feature_from_id(&feature), exchange_from_id(&exchange)) {
                (Some(feature), Some(exchange)) => Ok(Self::Unsupported {
                    feature,
                    exchange: exchange.id(),
                    detail,
                }),
                _ => Err(Self::adapter(unsupported_error_diagnostic(
                    "foreign unsupported error has unknown metadata",
                    &feature,
                    &exchange,
                    &detail,
                ))),
            },
            WireError::Adapter { detail } => Ok(Self::Adapter { detail }),
            WireError::Auth { detail } => Ok(Self::Auth { detail }),
            WireError::Exchange {
                exchange,
                code,
                message,
                status,
                exchange_kind,
            } => match (
                exchange_from_id(&exchange),
                exchange_error_kind_from_wire(&exchange_kind),
            ) {
                (Some(exchange), Some(kind)) => Ok(Self::Exchange {
                    exchange: exchange.id(),
                    code,
                    message,
                    status,
                    kind,
                }),
                _ => Err(Self::adapter(exchange_error_diagnostic(
                    "foreign exchange error has unknown metadata",
                    &exchange,
                    &code,
                    &message,
                    status,
                    &exchange_kind,
                ))),
            },
            WireError::Transport { detail } => Ok(Self::Transport { detail }),
            WireError::Decode { detail } => Ok(Self::Decode { detail }),
        }
    }
}

fn unsupported_error_diagnostic(
    context: &str,
    feature: &str,
    exchange: &str,
    detail: &str,
) -> String {
    format!("{context}: feature={feature:?}, exchange={exchange:?}, detail={detail:?}")
}

fn exchange_error_diagnostic(
    context: &str,
    exchange: &str,
    code: &str,
    message: &str,
    status: Option<u16>,
    classification: &str,
) -> String {
    format!(
        "{context}: exchange={exchange:?}, code={code:?}, message={message:?}, status={status:?}, classification={classification:?}"
    )
}

fn exchange_from_id(value: &str) -> Option<Exchange> {
    Exchange::ALL
        .iter()
        .copied()
        .find(|exchange| exchange.id() == value)
}

pub(crate) fn feature_from_id(value: &str) -> Option<Feature> {
    Feature::ALL
        .iter()
        .copied()
        .find(|feature| feature.id() == value)
}

fn exchange_error_kind_to_wire(value: ExchangeErrorKind) -> maxt::Result<&'static str> {
    match value {
        ExchangeErrorKind::Rejected => Ok("rejected"),
        ExchangeErrorKind::RateLimited => Ok("rate_limited"),
        ExchangeErrorKind::Unavailable => Ok("unavailable"),
        ExchangeErrorKind::Unknown => Ok("unknown"),
        _ => Err(binding_contract("ExchangeErrorKind")),
    }
}

fn exchange_error_kind_from_wire(value: &str) -> Option<ExchangeErrorKind> {
    Some(match value {
        "rejected" => ExchangeErrorKind::Rejected,
        "rate_limited" => ExchangeErrorKind::RateLimited,
        "unavailable" => ExchangeErrorKind::Unavailable,
        "unknown" => ExchangeErrorKind::Unknown,
        _ => return None,
    })
}

fn transfer_error_kind_to_wire(value: TransferErrorKind) -> maxt::Result<&'static str> {
    match value {
        TransferErrorKind::AssetMismatch => Ok("asset_mismatch"),
        TransferErrorKind::NetworkMismatch => Ok("network_mismatch"),
        TransferErrorKind::AmbiguousNetwork => Ok("ambiguous_network"),
        TransferErrorKind::NetworkUnavailable => Ok("network_unavailable"),
        TransferErrorKind::MemoRequired => Ok("memo_required"),
        TransferErrorKind::DestinationUnavailable => Ok("destination_unavailable"),
        TransferErrorKind::AddressNotAllowed => Ok("address_not_allowed"),
        TransferErrorKind::TravelRuleRequired => Ok("travel_rule_required"),
        TransferErrorKind::AmountOutOfRange => Ok("amount_out_of_range"),
        TransferErrorKind::PlanExpired => Ok("plan_expired"),
        _ => Err(binding_contract("TransferErrorKind")),
    }
}

fn transfer_error_kind_from_wire(value: &str) -> Option<TransferErrorKind> {
    Some(match value {
        "asset_mismatch" => TransferErrorKind::AssetMismatch,
        "network_mismatch" => TransferErrorKind::NetworkMismatch,
        "ambiguous_network" => TransferErrorKind::AmbiguousNetwork,
        "network_unavailable" => TransferErrorKind::NetworkUnavailable,
        "memo_required" => TransferErrorKind::MemoRequired,
        "destination_unavailable" => TransferErrorKind::DestinationUnavailable,
        "address_not_allowed" => TransferErrorKind::AddressNotAllowed,
        "travel_rule_required" => TransferErrorKind::TravelRuleRequired,
        "amount_out_of_range" => TransferErrorKind::AmountOutOfRange,
        "plan_expired" => TransferErrorKind::PlanExpired,
        _ => return None,
    })
}

pub(crate) fn decimal_from_wire(value: &str, field: &str) -> maxt::Result<Decimal> {
    maxt::parse_decimal_exact(value).map_err(|error| Error::InvalidRequest {
        field: field.to_owned(),
        detail: format!("`{value}` is not an exact decimal: {error}"),
    })
}

pub(crate) fn timestamp_to_wire(value: Timestamp) -> String {
    value.as_nanos().to_string()
}

pub(crate) fn timestamp_from_wire(value: &str, field: &str) -> maxt::Result<Timestamp> {
    value
        .parse::<i64>()
        .map(Timestamp::from_nanos)
        .map_err(|error| Error::InvalidRequest {
            field: field.to_owned(),
            detail: format!("`{value}` is not a signed 64-bit nanosecond timestamp: {error}"),
        })
}

include!("generated_provider_convert.rs");

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    fn assert_wire_round_trip<C, W>(core: C)
    where
        C: Clone + Debug + PartialEq + TryFrom<W, Error = Error>,
        W: TryFrom<C, Error = Error> + Serialize + DeserializeOwned,
    {
        let wire = W::try_from(core.clone()).unwrap();
        let json = serde_json::to_value(wire).unwrap();
        let wire = serde_json::from_value(json).unwrap();
        assert_eq!(C::try_from(wire).unwrap(), core);
    }

    fn foreign_from_wire<T, W>(value: Value, context: &str) -> maxt::Result<T>
    where
        W: DeserializeOwned,
        T: TryFrom<W, Error = Error>,
    {
        let wire = serde_json::from_value::<W>(value).map_err(|error| {
            Error::adapter(format!(
                "foreign adapter returned invalid {context} wire data: {error}"
            ))
        })?;
        T::try_from(wire).map_err(|error| {
            Error::adapter(format!(
                "foreign adapter returned invalid {context} value: {error}"
            ))
        })
    }
    #[test]
    fn decimal_and_timestamp_wire_boundaries_are_lossless() {
        let decimal = decimal_from_wire("1.2300", "price").unwrap();
        assert_eq!(decimal.to_string(), "1.2300");
        assert!(decimal_from_wire("2.5e-28", "price").is_err());

        for nanos in [i64::MIN, -1, 0, 1, i64::MAX] {
            let wire = timestamp_to_wire(Timestamp::from_nanos(nanos));
            assert_eq!(
                timestamp_from_wire(&wire, "timestamp").unwrap().as_nanos(),
                nanos
            );
        }
        assert!(timestamp_from_wire("9223372036854775808", "timestamp").is_err());
    }

    #[test]
    fn json_text_inputs_reject_non_json_as_structured_invalid_requests() {
        for text in ["undefined", "NaN", "Infinity", "-Infinity"] {
            assert!(matches!(
                from_wire_text::<Value>(text, "request"),
                Err(Error::InvalidRequest { field, .. }) if field == "request"
            ));
        }
    }

    #[test]
    fn json_text_inputs_enforce_the_explicit_nesting_limit() {
        let at_limit = format!("{}null{}", "[".repeat(64), "]".repeat(64));
        assert!(from_wire_text::<Value>(&at_limit, "request").is_ok());

        let too_deep = format!("{}null{}", "[".repeat(65), "]".repeat(65));
        let error = from_wire_text::<Value>(&too_deep, "request").unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidRequest { field, detail }
                if field == "request" && detail.contains("maximum JSON nesting depth is 64")
        ));
    }

    #[test]
    fn every_core_error_round_trips_with_its_fields() {
        let errors = [
            Error::InvalidRequest {
                field: "limit".to_owned(),
                detail: "must be positive".to_owned(),
            },
            Error::Transfer {
                kind: TransferErrorKind::NetworkMismatch,
                detail: "chains differ".to_owned(),
            },
            Error::Unsupported {
                feature: maxt::Feature::Markets,
                exchange: "upbit",
                detail: "not mapped".to_owned(),
            },
            Error::adapter("JavaScript stack: boom"),
            Error::Auth {
                detail: "bad key".to_owned(),
            },
            Error::Exchange {
                exchange: "binance",
                code: "-1003".to_owned(),
                message: "too many requests".to_owned(),
                status: Some(429),
                kind: maxt::ExchangeErrorKind::RateLimited,
            },
            Error::Transport {
                detail: "socket closed".to_owned(),
            },
            Error::Decode {
                detail: "bad frame".to_owned(),
            },
        ];

        for error in errors {
            let wire = WireError::from(error.clone());
            assert_eq!(Error::try_from(wire).unwrap(), error);
        }
    }

    #[test]
    fn stable_exchange_feature_and_error_kind_ids_are_exhaustive() {
        for exchange in Exchange::ALL {
            assert_eq!(exchange_from_id(exchange.id()), Some(exchange));
        }
        for feature in Feature::ALL {
            assert_eq!(feature_from_id(feature.id()), Some(feature));
        }
        for kind in [
            ExchangeErrorKind::Rejected,
            ExchangeErrorKind::RateLimited,
            ExchangeErrorKind::Unavailable,
            ExchangeErrorKind::Unknown,
        ] {
            assert_eq!(
                exchange_error_kind_from_wire(exchange_error_kind_to_wire(kind).unwrap()),
                Some(kind)
            );
        }
        for kind in [
            TransferErrorKind::AssetMismatch,
            TransferErrorKind::NetworkMismatch,
            TransferErrorKind::AmbiguousNetwork,
            TransferErrorKind::NetworkUnavailable,
            TransferErrorKind::MemoRequired,
            TransferErrorKind::DestinationUnavailable,
            TransferErrorKind::AddressNotAllowed,
            TransferErrorKind::TravelRuleRequired,
            TransferErrorKind::AmountOutOfRange,
            TransferErrorKind::PlanExpired,
        ] {
            assert_eq!(
                transfer_error_kind_from_wire(transfer_error_kind_to_wire(kind).unwrap()),
                Some(kind)
            );
        }
        assert_eq!(exchange_from_id("future_exchange"), None);
        assert_eq!(feature_from_id("future_feature"), None);
        assert_eq!(exchange_error_kind_from_wire("future_kind"), None);
        assert_eq!(transfer_error_kind_from_wire("future_kind"), None);
    }

    #[test]
    fn core_errors_never_emit_an_unstable_exchange_id() {
        for error in [
            Error::Unsupported {
                feature: Feature::Markets,
                exchange: "future_exchange",
                detail: "not mapped".to_owned(),
            },
            Error::Exchange {
                exchange: "future_exchange",
                code: "-1".to_owned(),
                message: "boom".to_owned(),
                status: None,
                kind: ExchangeErrorKind::Unknown,
            },
        ] {
            assert!(matches!(WireError::from(error), WireError::Adapter { .. }));
        }
    }

    #[test]
    fn unknown_error_metadata_downgrades_without_losing_original_diagnostics() {
        let unsupported = WireError::from(Error::Unsupported {
            feature: Feature::Markets,
            exchange: "future_exchange",
            detail: "provider-specific capability".to_owned(),
        });
        let WireError::Adapter { detail } = unsupported else {
            panic!("unknown exchange must downgrade to an adapter error");
        };
        for expected in ["markets", "future_exchange", "provider-specific capability"] {
            assert!(
                detail.contains(expected),
                "missing `{expected}` in `{detail}`"
            );
        }

        let exchange = WireError::from(Error::Exchange {
            exchange: "future_exchange",
            code: "-9000".to_owned(),
            message: "future failure".to_owned(),
            status: Some(599),
            kind: ExchangeErrorKind::RateLimited,
        });
        let WireError::Adapter { detail } = exchange else {
            panic!("unknown exchange must downgrade to an adapter error");
        };
        for expected in [
            "future_exchange",
            "-9000",
            "future failure",
            "599",
            "rate_limited",
        ] {
            assert!(
                detail.contains(expected),
                "missing `{expected}` in `{detail}`"
            );
        }

        let inbound = Error::try_from(WireError::Exchange {
            exchange: "binance".to_owned(),
            code: "FUTURE_CODE".to_owned(),
            message: "future classification".to_owned(),
            status: Some(418),
            exchange_kind: "future_kind".to_owned(),
        })
        .unwrap_err();
        let Error::Adapter { detail } = inbound else {
            panic!("unknown classification must downgrade to an adapter error");
        };
        for expected in [
            "binance",
            "FUTURE_CODE",
            "future classification",
            "418",
            "future_kind",
        ] {
            assert!(
                detail.contains(expected),
                "missing `{expected}` in `{detail}`"
            );
        }

        assert!(matches!(
            Error::try_from(WireError::Transfer {
                transfer_kind: "future_kind".to_owned(),
                detail: "future transfer failure".to_owned(),
            }),
            Err(Error::Adapter { detail }) if detail.contains("future_kind")
        ));
    }

    #[test]
    fn request_wire_rejects_unknown_missing_and_out_of_range_fields() {
        let valid = serde_json::json!({
            "market": {
                "exchange": "binance",
                "kind": "spot",
                "base": "BTC",
                "quote": "USDT"
            },
            "side": "buy",
            "order_type": "market",
            "size": { "kind": "quote", "value": "10.00" },
            "price": null,
            "time_in_force": null,
            "reduce_only": false,
            "client_id": null
        });
        let request =
            OrderRequest::try_from(from_wire_value::<WireOrderRequest>(valid, "request").unwrap())
                .unwrap();
        assert!(matches!(request.size, maxt::Size::Quote(value) if value.to_string() == "10.00"));

        let invalid = [
            serde_json::json!({
                "market": { "exchange": "binance", "kind": "spot", "base": "BTC", "quote": "USDT" },
                "side": "buy", "order_type": "market",
                "size": { "kind": "contracts", "value": "1" },
                "price": null, "time_in_force": null, "reduce_only": false, "client_id": null
            }),
            serde_json::json!({
                "market": { "exchange": "binance", "kind": "spot", "base": "BTC", "quote": "USDT" },
                "side": "buy", "order_type": "market",
                "size": { "kind": "quote", "value": "1" },
                "price": null, "reduce_only": false
            }),
        ];
        for value in invalid {
            assert!(matches!(
                from_wire_value::<WireOrderRequest>(value, "request"),
                Err(Error::InvalidRequest { field, .. }) if field == "request"
            ));
        }

        let too_large = serde_json::json!({
            "market": { "exchange": "binance", "kind": "spot", "base": "BTC", "quote": "USDT" },
            "interval": "min1", "from": null, "to": null, "limit": 4_294_967_296_u64
        });
        assert!(matches!(
            from_wire_value::<WireCandleRequest>(too_large, "request"),
            Err(Error::InvalidRequest { field, .. }) if field == "request"
        ));
    }

    #[test]
    fn every_common_response_dto_round_trips_without_loss() {
        use maxt::{
            AssetNetwork, Balance, Candle, ChainDestination, Cursor, Deposit, DepositAddress,
            DepositAddressEntry, DepositStatus, FundingPayment, FundingRate, Level, MarginMode,
            MarginSummary, MarketInfo, MarketStatus, Network, Order, OrderBook, OrderStatus, Page,
            Position, Ticker, Trade, TransferDestination, TravelRuleRequirement, Withdrawal,
            WithdrawalFee, WithdrawalQuote, WithdrawalStatus,
        };

        let market = Market::perpetual(Exchange::Binance, "btc", "usdt");
        let decimal = |text| maxt::parse_decimal_exact(text).unwrap();
        let timestamp = Timestamp::from_nanos(i64::MAX);

        assert_wire_round_trip::<_, WireMarket>(market.clone());
        assert_wire_round_trip::<_, WireMarketInfo>(MarketInfo {
            market: market.clone(),
            native_symbol: "BTCUSDT".to_owned(),
            status: MarketStatus::Active,
            korean_name: None,
            english_name: Some("Bitcoin".to_owned()),
        });
        assert_wire_round_trip::<_, WireTrade>(Trade {
            market: market.clone(),
            timestamp,
            price: decimal("1.2300"),
            quantity: decimal("0.00000010"),
            taker_side: Side::Buy,
            id: None,
        });
        assert_wire_round_trip::<_, WireOrderBook>(OrderBook {
            market: market.clone(),
            timestamp,
            bids: vec![Level {
                price: decimal("1.2300"),
                quantity: decimal("2.00"),
            }],
            asks: vec![],
        });
        assert_wire_round_trip::<_, WireTicker>(Ticker {
            market: market.clone(),
            timestamp,
            last_trade_time: Some(Timestamp::from_nanos(i64::MIN)),
            last_price: decimal("1.2300"),
            change: None,
            change_rate: Some(decimal("-0.01")),
            high: None,
            low: None,
            volume: Some(decimal("3.00")),
            quote_volume: None,
        });
        assert_wire_round_trip::<_, WireCandle>(Candle {
            market: market.clone(),
            interval: Interval::Min1,
            open_time: timestamp,
            open: decimal("1.00"),
            high: decimal("2.00"),
            low: decimal("0.50"),
            close: decimal("1.50"),
            volume: decimal("4.00"),
            quote_volume: None,
            closed: true,
        });
        assert_wire_round_trip::<_, WireBalance>(Balance {
            asset: "USDT".to_owned(),
            available: decimal("1.00"),
            locked: decimal("2.00"),
        });
        assert_wire_round_trip::<_, WireAssetNetwork>(AssetNetwork {
            exchange: Exchange::Binance,
            asset: "BTC".to_owned(),
            network: Network::Other("future_chain".to_owned()),
            provider_id: "FUTURE".to_owned(),
            deposit_enabled: true,
            withdrawal_enabled: false,
            withdrawal_fee: Some(WithdrawalFee::Rate {
                rate: decimal("0.001"),
                minimum: Some(decimal("0.0001")),
                maximum: None,
            }),
            minimum_withdrawal: Some(decimal("0.01")),
            maximum_withdrawal: None,
            memo_required: true,
        });
        assert_wire_round_trip::<_, WireDepositAddress>(DepositAddress {
            exchange: Exchange::Binance,
            asset: "BTC".to_owned(),
            network: Network::Bitcoin,
            address: Some("bc1qdestination".to_owned()),
            memo: None,
        });
        assert_wire_round_trip::<_, WireDepositAddressEntry>(DepositAddressEntry {
            exchange: Exchange::Bithumb,
            asset: "XRP".to_owned(),
            network: None,
            provider_network: None,
            address: None,
            memo: Some("tag-7".to_owned()),
        });
        let destination = TransferDestination::Chain(ChainDestination {
            asset: "BTC".to_owned(),
            network: Network::Bitcoin,
            address: "bc1qdestination".to_owned(),
            memo: None,
        });
        assert_wire_round_trip::<_, WireWithdrawalQuote>(WithdrawalQuote {
            fee: Some(decimal("0.0001")),
            expected_receive: Some(decimal("0.9999")),
            minimum_amount: None,
            maximum_amount: None,
            address_allowed: Some(true),
            travel_rule: TravelRuleRequirement::Required {
                consent_url: Some("https://example.test/consent".to_owned()),
            },
            expires_at: Some(timestamp),
        });
        assert_wire_round_trip::<_, WireWithdrawal>(Withdrawal {
            id: "withdrawal-1".to_owned(),
            asset: "BTC".to_owned(),
            network: Some(Network::Bitcoin),
            provider_network: Some("BTC".to_owned()),
            amount: decimal("1.00"),
            fee: Some(decimal("0.0001")),
            destination: Some(destination),
            status: WithdrawalStatus::Processing,
            provider_status: "processing".to_owned(),
            tx_id: None,
            created_at: Some(timestamp),
        });
        assert_wire_round_trip::<_, WireDeposit>(Deposit {
            id: "deposit-1".to_owned(),
            asset: "BTC".to_owned(),
            network: Some(Network::Bitcoin),
            provider_network: Some("BTC".to_owned()),
            amount: decimal("0.9999"),
            address: Some("bc1qdestination".to_owned()),
            memo: None,
            status: DepositStatus::Completed,
            provider_status: "credited".to_owned(),
            tx_id: Some("tx-1".to_owned()),
            created_at: Some(timestamp),
        });
        let order = Order {
            id: "order-1".to_owned(),
            market: market.clone(),
            side: Side::Sell,
            status: OrderStatus::PartiallyFilled,
            filled_quantity: decimal("1.00"),
            remaining_quantity: decimal("2.00"),
            price: Some(decimal("3.00")),
            created_at: None,
        };
        assert_wire_round_trip::<_, WireOrder>(order);
        assert_wire_round_trip::<_, WirePosition>(Position {
            market: market.clone(),
            side: Some(Side::Buy),
            quantity: decimal("1.00"),
            entry_price: None,
            mark_price: Some(decimal("2.00")),
            notional: None,
            unrealized_pnl: Some(decimal("-0.50")),
            leverage: Some(decimal("10.0")),
            margin_mode: Some(MarginMode::Isolated),
        });
        assert_wire_round_trip::<_, WireMarginSummary>(MarginSummary {
            asset: "USDT".to_owned(),
            equity: Some(decimal("3.00")),
            margin_balance: None,
            available_balance: Some(decimal("1.00")),
        });
        assert_wire_round_trip::<_, WirePage<WireFundingRate>>(Page {
            items: vec![FundingRate {
                market: market.clone(),
                timestamp,
                rate: decimal("0.0001"),
                mark_price: None,
            }],
            next: Some(Cursor::new("rate-page")),
        });
        assert_wire_round_trip::<_, WirePage<WireFundingPayment>>(Page {
            items: vec![FundingPayment {
                market,
                timestamp,
                amount: decimal("-1.00"),
                rate: None,
                id: Some("payment-1".to_owned()),
            }],
            next: None,
        });
    }

    #[test]
    fn every_common_request_dto_round_trips_without_loss() {
        use maxt::{
            ChainDestination, DepositAddressRequest, Feed, HistoryRequest, MarginMode,
            MarginRequest, Network, Overflow, StreamConfig, Subscription, TransferDestination,
            TransferHistoryRequest, TransferLookupRequest, WithdrawRequest,
        };

        let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
        let decimal = |text| maxt::parse_decimal_exact(text).unwrap();
        let from = Timestamp::from_nanos(i64::MIN);
        let to = Timestamp::from_nanos(i64::MAX);

        assert_wire_round_trip::<_, WireCandleRequest>(CandleRequest {
            market: market.clone(),
            interval: Interval::Hour1,
            from: Some(from),
            to: Some(to),
            limit: None,
        });
        assert_wire_round_trip::<_, WireOrderRequest>(
            OrderRequest::limit(
                market.clone(),
                Side::Sell,
                Size::Base(decimal("1.2300")),
                decimal("100.00"),
            )
            .time_in_force(TimeInForce::PostOnly)
            .reduce_only(),
        );
        assert_wire_round_trip::<_, WireDepositAddressRequest>(
            DepositAddressRequest::new("BTC", Network::Bitcoin).amount(decimal("1.00")),
        );
        assert_wire_round_trip::<_, WireWithdrawRequest>(
            WithdrawRequest::new(
                "BTC",
                Network::Bitcoin,
                decimal("1.00"),
                TransferDestination::Chain(ChainDestination {
                    asset: "BTC".to_owned(),
                    network: Network::Bitcoin,
                    address: "bc1qdestination".to_owned(),
                    memo: None,
                }),
            )
            .client_id("client-1"),
        );
        assert_wire_round_trip::<_, WireTransferHistoryRequest>(
            TransferHistoryRequest::new()
                .asset("BTC")
                .network(Network::Bitcoin)
                .cursor(Cursor::new("page-2"))
                .limit(100),
        );
        assert_wire_round_trip::<_, WireTransferLookupRequest>(TransferLookupRequest::by_tx_id(
            "BTC", "tx-1",
        ));
        assert_wire_round_trip::<_, WireHistoryRequest>(HistoryRequest {
            market: market.clone(),
            from: Some(from),
            to: None,
            cursor: Some(Cursor::new("page-2")),
            limit: Some(100),
        });
        assert_wire_round_trip::<_, WireMarginRequest>(MarginRequest {
            market: market.clone(),
            leverage: Some(decimal("10.0")),
            margin_mode: Some(MarginMode::Cross),
        });
        assert_wire_round_trip::<_, WireSubscription>(
            Subscription::new()
                .market(market)
                .feed(Feed::Trades)
                .feed(Feed::Candles(Interval::Min1)),
        );
        assert_wire_round_trip::<_, WireStreamConfig>(StreamConfig {
            max_reconnect_attempts: None,
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: MAX_JSON_SAFE_INTEGER,
            idle_timeout_ms: 30_000,
            buffer_size: 1,
            overflow: Overflow::DropNewest,
        });
    }

    #[test]
    fn stream_config_wire_preserves_safe_integer_boundaries_as_decimal_strings() {
        for boundary in ["4294967296", "9007199254740991"] {
            let wire = WireStreamConfig {
                max_reconnect_attempts: None,
                initial_reconnect_delay_ms: boundary.to_owned(),
                max_reconnect_delay_ms: boundary.to_owned(),
                idle_timeout_ms: boundary.to_owned(),
                buffer_size: boundary.to_owned(),
                overflow: "backpressure".to_owned(),
            };
            let core = StreamConfig::try_from(wire).unwrap();
            assert_eq!(
                core.initial_reconnect_delay_ms,
                boundary.parse::<u64>().unwrap()
            );
            assert_eq!(core.buffer_size, boundary.parse::<usize>().unwrap());

            let restored = WireStreamConfig::try_from(core).unwrap();
            assert_eq!(restored.initial_reconnect_delay_ms, boundary);
            assert_eq!(restored.buffer_size, boundary);
        }

        let too_large = WireStreamConfig {
            max_reconnect_attempts: None,
            initial_reconnect_delay_ms: "9007199254740992".to_owned(),
            max_reconnect_delay_ms: "1".to_owned(),
            idle_timeout_ms: "1".to_owned(),
            buffer_size: "1".to_owned(),
            overflow: "backpressure".to_owned(),
        };
        assert!(matches!(
            StreamConfig::try_from(too_large),
            Err(Error::InvalidRequest { field, .. }) if field == "initial_reconnect_delay_ms"
        ));

        let numeric_wire = serde_json::json!({
            "max_reconnect_attempts": null,
            "initial_reconnect_delay_ms": 4294967296_u64,
            "max_reconnect_delay_ms": "1",
            "idle_timeout_ms": "1",
            "buffer_size": "1",
            "overflow": "backpressure"
        });
        assert!(matches!(
            from_wire_value::<WireStreamConfig>(numeric_wire, "config"),
            Err(Error::InvalidRequest { field, .. }) if field == "config"
        ));
    }

    #[test]
    fn every_stream_variant_is_structural_and_lossless() {
        use maxt::{
            AccountEvent, Balance, Candle, Level, MarketEvent, Order, OrderBook, Ticker, Trade,
        };

        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let decimal = |text| maxt::parse_decimal_exact(text).unwrap();
        let timestamp = Timestamp::from_nanos(123);
        let trade = Trade {
            market: market.clone(),
            timestamp,
            price: decimal("1.00"),
            quantity: decimal("2.00"),
            taker_side: Side::Buy,
            id: None,
        };
        let order_book = OrderBook {
            market: market.clone(),
            timestamp,
            bids: vec![Level {
                price: decimal("1.00"),
                quantity: decimal("2.00"),
            }],
            asks: vec![],
        };
        let ticker = Ticker {
            market: market.clone(),
            timestamp,
            last_trade_time: None,
            last_price: decimal("1.00"),
            change: None,
            change_rate: None,
            high: None,
            low: None,
            volume: None,
            quote_volume: None,
        };
        let candle = Candle {
            market: market.clone(),
            interval: Interval::Min1,
            open_time: timestamp,
            open: decimal("1.00"),
            high: decimal("1.00"),
            low: decimal("1.00"),
            close: decimal("1.00"),
            volume: decimal("2.00"),
            quote_volume: None,
            closed: false,
        };
        for event in [
            MarketEvent::Trade(trade),
            MarketEvent::OrderBook(order_book),
            MarketEvent::Ticker(ticker),
            MarketEvent::Candle(candle),
            MarketEvent::Reconnected,
        ] {
            assert_wire_round_trip::<_, WireMarketEvent>(event);
        }

        let balance = Balance {
            asset: "KRW".to_owned(),
            available: decimal("1.00"),
            locked: decimal("0.00"),
        };
        let order = Order {
            id: "order-1".to_owned(),
            market,
            side: Side::Sell,
            status: OrderStatus::Open,
            filled_quantity: decimal("0.00"),
            remaining_quantity: decimal("1.00"),
            price: None,
            created_at: None,
        };
        for event in [
            AccountEvent::Balance(balance),
            AccountEvent::Order(order),
            AccountEvent::Reconnected,
        ] {
            assert_wire_round_trip::<_, WireAccountEvent>(event);
        }

        assert!(matches!(
            market_stream_item(Err(Error::Transport {
                detail: "socket closed".to_owned(),
            })),
            WireMarketStreamItem::Error {
                error: WireError::Transport { .. }
            }
        ));
        assert!(matches!(
            account_stream_item(Ok(AccountEvent::Reconnected)),
            WireAccountStreamItem::Event {
                event: WireAccountEvent::Reconnected
            }
        ));

        assert!(matches!(
            from_wire_value::<WireMarketEvent>(
                serde_json::json!({ "kind": "liquidation" }),
                "event"
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "event"
        ));
    }

    #[test]
    fn provider_wire_dtos_keep_strings_nulls_and_known_variants() {
        use maxt::adapters::{
            BithumbAlertStep, HyperliquidLedgerKind, HyperliquidUserRole, UpbitMarketEvent,
        };

        let upbit = WireUpbitMarketEvent::try_from(UpbitMarketEvent::default()).unwrap();
        assert_eq!(
            serde_json::to_value(upbit).unwrap(),
            serde_json::json!({ "warning": false, "cautions": [] })
        );
        assert_eq!(
            bithumb_alert_step_to_wire(BithumbAlertStep::Danger).unwrap(),
            "danger"
        );
        assert_eq!(
            hyperliquid_ledger_kind_to_wire(HyperliquidLedgerKind::Other(
                "futureMovement".to_owned()
            ))
            .unwrap(),
            "futureMovement"
        );

        let values = [
            serde_json::to_value(WireBithumbMarketAlert {
                kind: "PRICE_FLUCTUATION".to_owned(),
                step: "warning".to_owned(),
                ends_at: i64::MIN.to_string(),
            })
            .unwrap(),
            serde_json::to_value(WireBinanceSymbolFilters {
                symbol: "BTCUSDT".to_owned(),
                tick_size: Some("0.0100".to_owned()),
                min_price: None,
                max_price: None,
                step_size: None,
                min_quantity: None,
                max_quantity: None,
                min_notional: None,
            })
            .unwrap(),
            serde_json::to_value(WireBinanceSpotOrderDetail {
                order: WireOrder {
                    id: "order-1".to_owned(),
                    market: WireMarket {
                        exchange: "binance".to_owned(),
                        kind: "spot".to_owned(),
                        base: "BTC".to_owned(),
                        quote: "USDT".to_owned(),
                    },
                    side: "buy".to_owned(),
                    status: "filled".to_owned(),
                    filled_quantity: "1.00".to_owned(),
                    remaining_quantity: "0.00".to_owned(),
                    price: None,
                    created_at: None,
                },
                client_order_id: "client-1".to_owned(),
                order_type: "MARKET".to_owned(),
                time_in_force: "GTC".to_owned(),
                filled_quote_quantity: "100.00".to_owned(),
                updated_at: None,
            })
            .unwrap(),
            serde_json::to_value(WireHyperliquidLedgerEntry {
                kind: "futureMovement".to_owned(),
                time: i64::MAX.to_string(),
                hash: "0x1".to_owned(),
                asset: None,
                amount: Some("1.2300".to_owned()),
                fee: None,
                counterparty: None,
            })
            .unwrap(),
            serde_json::to_value(WireHyperliquidAssetContext {
                mid_price: None,
                mark_price: Some("1.2300".to_owned()),
                oracle_price: None,
                funding_rate: None,
                open_interest: None,
                size_decimals: 8,
                price_decimals: 2,
            })
            .unwrap(),
            serde_json::to_value(WireHyperliquidMidPrice {
                market: WireMarket {
                    exchange: "hyperliquid".to_owned(),
                    kind: "perpetual".to_owned(),
                    base: "BTC".to_owned(),
                    quote: "USDC".to_owned(),
                },
                price: "113376.5".to_owned(),
            })
            .unwrap(),
            serde_json::to_value(WireUpbitKrwTransferRequest {
                amount: "10000.00".to_owned(),
                two_factor_type: "kakao".to_owned(),
            })
            .unwrap(),
            serde_json::to_value(WireBithumbWithdrawalAddress {
                currency: "XRP".to_owned(),
                net_type: "XRP".to_owned(),
                network_name: None,
                withdraw_address: "rAddress".to_owned(),
                secondary_address: Some("1234".to_owned()),
                exchange_name: None,
                owner_type: None,
                owner_ko_name: None,
                owner_en_name: None,
                owner_corp_ko_name: None,
                owner_corp_en_name: None,
            })
            .unwrap(),
            serde_json::to_value(WireBinanceAccountTrade {
                market: WireMarket {
                    exchange: "binance".to_owned(),
                    kind: "spot".to_owned(),
                    base: "BTC".to_owned(),
                    quote: "USDT".to_owned(),
                },
                id: "1".to_owned(),
                order_id: "2".to_owned(),
                timestamp: i64::MAX.to_string(),
                side: "buy".to_owned(),
                maker: true,
                best_match: Some(true),
                order_list_id: Some("-1".to_owned()),
                price: "1.2300".to_owned(),
                quantity: "0.0100".to_owned(),
                quote_quantity: Some("0.0123".to_owned()),
                commission: "0.0001".to_owned(),
                commission_asset: "BNB".to_owned(),
                realized_pnl: None,
                position_side: None,
                pair: None,
                base_quantity: None,
                margin_asset: None,
            })
            .unwrap(),
            serde_json::to_value(WireHyperliquidUserRole::Other {
                role: "futureRole".to_owned(),
                data_json: Some("null".to_owned()),
            })
            .unwrap(),
            serde_json::to_value(WireHyperliquidUserRateLimit {
                cumulative_volume: "1.2300".to_owned(),
                requests_used: u64::MAX.to_string(),
                requests_cap: u64::MAX.to_string(),
                requests_surplus: "0".to_owned(),
            })
            .unwrap(),
        ];
        for value in values {
            assert!(
                value
                    .as_object()
                    .unwrap()
                    .values()
                    .all(|value| { !value.is_number() || value.as_u64().is_some() })
            );
        }

        assert!(matches!(
            from_wire_value::<WireBinanceSymbolFilters>(
                serde_json::json!({ "symbol": "BTCUSDT" }),
                "filters"
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "filters"
        ));
        assert!(matches!(
            UpbitKrwTransferRequest::try_from(WireUpbitKrwTransferRequest {
                amount: "1".to_owned(),
                two_factor_type: "unknown".to_owned(),
            }),
            Err(Error::InvalidRequest { field, .. }) if field == "two_factor_type"
        ));
        assert!(matches!(
            WireHyperliquidUserRole::try_from(HyperliquidUserRole::Other {
                role: "futureRole".to_owned(),
                data_json: None,
            }),
            Ok(WireHyperliquidUserRole::Other { .. })
        ));
    }

    #[test]
    fn outcome_and_foreign_errors_never_escape_as_raw_napi_failures() {
        assert_eq!(
            outcome::<Value>(Ok(serde_json::json!({ "answer": "1.2300" }))),
            serde_json::json!({ "ok": true, "value": { "answer": "1.2300" } })
        );
        assert_eq!(
            outcome::<Value>(Err(Error::Decode {
                detail: "bad frame".to_owned()
            })),
            serde_json::json!({
                "ok": false,
                "error": { "kind": "decode", "detail": "bad frame" }
            })
        );

        let unknown = WireError::Unsupported {
            feature: "future_feature".to_owned(),
            exchange: "upbit".to_owned(),
            detail: "not mapped".to_owned(),
        };
        assert!(matches!(
            Error::try_from(unknown),
            Err(Error::Adapter { .. })
        ));

        for unknown in [
            WireError::Unsupported {
                feature: "markets".to_owned(),
                exchange: "future_exchange".to_owned(),
                detail: "not mapped".to_owned(),
            },
            WireError::Exchange {
                exchange: "future_exchange".to_owned(),
                code: "-1".to_owned(),
                message: "boom".to_owned(),
                status: None,
                exchange_kind: "unknown".to_owned(),
            },
            WireError::Exchange {
                exchange: "binance".to_owned(),
                code: "-1".to_owned(),
                message: "boom".to_owned(),
                status: None,
                exchange_kind: "future_kind".to_owned(),
            },
        ] {
            assert!(matches!(
                Error::try_from(unknown),
                Err(Error::Adapter { .. })
            ));
        }

        for value in [
            serde_json::json!({ "kind": "future_error", "detail": "boom" }),
            serde_json::json!({
                "kind": "exchange", "exchange": "binance", "code": "-1",
                "message": "boom", "exchange_kind": "unknown"
            }),
        ] {
            assert!(matches!(
                foreign_from_wire::<Error, WireError>(value, "error"),
                Err(Error::Adapter { .. })
            ));
        }

        let invalid_trade = serde_json::json!({
            "market": {
                "exchange": "upbit", "kind": "spot", "base": "BTC", "quote": "KRW"
            },
            "timestamp": "0",
            "price": "2.5e-28",
            "quantity": "1",
            "taker_side": "buy",
            "id": null
        });
        assert!(matches!(
            foreign_from_wire::<Trade, WireTrade>(invalid_trade, "trade"),
            Err(Error::Adapter { .. })
        ));
    }
}
