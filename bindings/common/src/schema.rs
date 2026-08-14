//! Repository binding schema used by `maxt-bindings-codegen`.

use maxt::{Exchange, Feature};

use crate::coverage::{PRODUCTS, ProductCoverage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Boolean,
    Number,
    UnsignedInteger,
    Decimal,
    Timestamp,
    Identifier(&'static str),
    Named(&'static str),
    Optional(Box<Type>),
    List(Box<Type>),
    Tuple(Vec<Type>),
}

impl Type {
    fn named(name: &'static str) -> Self {
        Self::Named(name)
    }

    fn optional(value: Self) -> Self {
        Self::Optional(Box::new(value))
    }

    fn list(value: Self) -> Self {
        Self::List(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub name: &'static str,
    pub type_parameters: &'static [&'static str],
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub name: &'static str,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedUnion {
    pub name: &'static str,
    pub type_parameters: &'static [&'static str],
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifierVariant {
    pub rust_name: &'static str,
    pub wire_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifier {
    pub name: &'static str,
    pub variants: &'static [IdentifierVariant],
    pub open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation {
    pub rust_name: &'static str,
    pub language_name: &'static str,
    pub feature: &'static str,
    pub arguments: &'static [Argument],
    pub result: ApiType,
    pub client_methods: &'static [ClientMethod],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argument {
    pub name: &'static str,
    pub ty: ApiType,
    pub default: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiType {
    Client,
    String,
    Boolean,
    Number,
    Named(&'static str),
    OptionalString,
    OptionalNumber,
    OptionalNamed(&'static str),
    List(&'static str),
    PairList(&'static str, &'static str),
    Page(&'static str),
    Handle(&'static str),
    HandleToken(&'static str),
    MarketStream,
    AccountStream,
    /// A provider-owned market stream with one named tagged event type.
    ///
    /// This is a schema marker, not a public cross-provider generic stream.
    ProviderMarketStream(&'static str),
    /// A provider-owned account stream with one named tagged event type.
    ///
    /// This is a schema marker, not a public cross-provider generic stream.
    ProviderAccountStream(&'static str),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientMethod {
    pub name: &'static str,
    pub native_name: &'static str,
    pub arguments: &'static [Argument],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientComposition {
    pub rust_name: &'static str,
    pub language_name: &'static str,
    pub arguments: &'static [Argument],
    pub result: ApiType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderMethodKind {
    Property,
    Async,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderMethod {
    pub rust_name: &'static str,
    pub name: &'static str,
    pub kind: ProviderMethodKind,
    pub arguments: &'static [Argument],
    pub result: ApiType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderOptionValue {
    Argument(&'static str),
    Identifier {
        name: &'static str,
        variant: &'static str,
    },
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderOption {
    pub name: &'static str,
    pub value: ProviderOptionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderConstructor {
    pub name: &'static str,
    pub arguments: &'static [Argument],
    pub options: &'static [ProviderOption],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub exchange: &'static str,
    pub adapter: &'static str,
    pub native_handle: &'static str,
    pub native_factory: &'static str,
    pub options_wire: &'static str,
    pub constructors: &'static [ProviderConstructor],
    pub methods: &'static [ProviderMethod],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub native_api_version: u32,
    pub exchanges: Vec<&'static str>,
    pub features: Vec<&'static str>,
    pub identifiers: &'static [Identifier],
    pub models: &'static [&'static str],
    pub errors: &'static [&'static str],
    pub adapter_operations: &'static [Operation],
    pub client_compositions: &'static [ClientComposition],
    pub client_members: &'static [&'static str],
    pub providers: &'static [Provider],
    pub products: &'static [ProductCoverage],
    pub records: Vec<Record>,
    pub unions: Vec<TaggedUnion>,
}

impl Schema {
    pub fn identifier(&self, name: &str) -> Option<&Identifier> {
        self.identifiers.iter().find(|value| value.name == name)
    }

    pub fn has_identifier(&self, name: &str) -> bool {
        self.identifier(name).is_some()
    }
}

const fn argument(name: &'static str, ty: ApiType, default: Option<&'static str>) -> Argument {
    Argument { name, ty, default }
}

const KIND: &[Argument] = &[argument("kind", ApiType::Named("MarketKind"), None)];
const MARKET_LIMIT: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("limit", ApiType::OptionalNumber, Some("null")),
];
const MARKET_DEPTH: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("depth", ApiType::OptionalNumber, Some("null")),
];
const MARKET: &[Argument] = &[argument("market", ApiType::Named("Market"), None)];
const OPTIONAL_MARKET: &[Argument] = &[argument(
    "market",
    ApiType::OptionalNamed("Market"),
    Some("null"),
)];
const CANDLE_REQUEST: &[Argument] = &[argument("request", ApiType::Named("CandleRequest"), None)];
const SUBSCRIPTION: &[Argument] = &[argument(
    "subscription",
    ApiType::Named("Subscription"),
    None,
)];
const SUBSCRIPTION_CONFIG: &[Argument] = &[
    argument("subscription", ApiType::Named("Subscription"), None),
    argument("config", ApiType::Named("StreamConfig"), None),
];
const UPBIT_SUBSCRIPTION: &[Argument] = &[argument(
    "subscription",
    ApiType::Named("Subscription"),
    None,
)];
const CONFIG: &[Argument] = &[argument("config", ApiType::Named("StreamConfig"), None)];
const ORDER_REQUEST: &[Argument] = &[argument("request", ApiType::Named("OrderRequest"), None)];
const ORDER_HISTORY_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("OrderHistoryRequest"),
    None,
)];
const ORDER_LOOKUP_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("OrderLookupRequest"),
    None,
)];
const CANCEL_ORDERS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("CancelOrdersRequest"),
    None,
)];
const UPBIT_BATCH_CANCEL_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitBatchCancelRequest"),
    None,
)];
const UPBIT_CANCEL_AND_NEW_ORDER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitCancelAndNewOrderRequest"),
    None,
)];
const UPBIT_ORDER_DETAIL_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitOrderDetailRequest"),
    None,
)];
const UPBIT_CLOSED_ORDERS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitClosedOrdersRequest"),
    None,
)];
const UPBIT_TRAVEL_RULE_UUID: &[Argument] = &[
    argument("depositUuid", ApiType::String, None),
    argument("vaspUuid", ApiType::String, None),
];
const UPBIT_TRAVEL_RULE_TXID: &[Argument] = &[
    argument("txid", ApiType::String, None),
    argument("vaspUuid", ApiType::String, None),
    argument("currency", ApiType::String, None),
    argument("netType", ApiType::String, None),
];
const ASSET: &[Argument] = &[argument("asset", ApiType::String, None)];
const ASSET_NETWORK: &[Argument] = &[
    argument("asset", ApiType::String, None),
    argument("network", ApiType::Named("Network"), None),
];
const DEPOSIT_ADDRESS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("DepositAddressRequest"),
    None,
)];
const WITHDRAW_REQUEST: &[Argument] =
    &[argument("request", ApiType::Named("WithdrawRequest"), None)];
const TRANSFER_HISTORY_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("TransferHistoryRequest"),
    None,
)];
const TRANSFER_LOOKUP_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("TransferLookupRequest"),
    None,
)];
const WITHDRAWAL_ID: &[Argument] = &[argument("withdrawalId", ApiType::String, None)];
const EXCHANGE_TRANSFER_REQUEST: &[Argument] = &[
    argument("destination", ApiType::Client, None),
    argument("request", ApiType::Named("ExchangeTransferRequest"), None),
];
const CHAIN_TRANSFER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("ChainTransferRequest"),
    None,
)];
const TRANSFER_PLAN: &[Argument] = &[argument("plan", ApiType::Named("TransferPlan"), None)];
const CANCEL_ORDER: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("orderId", ApiType::String, None),
];
const ORDER: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("orderId", ApiType::String, None),
];
const ORDER_BY_CLIENT_ID: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("clientId", ApiType::String, None),
];
const CANCEL_ORDER_BY_CLIENT_ID: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("clientId", ApiType::String, None),
];
const HISTORY_REQUEST: &[Argument] = &[argument("request", ApiType::Named("HistoryRequest"), None)];
const MARGIN_REQUEST: &[Argument] = &[argument("request", ApiType::Named("MarginRequest"), None)];

const CLIENT_MARKETS: &[ClientMethod] = &[ClientMethod {
    name: "markets",
    native_name: "markets",
    arguments: KIND,
}];
const CLIENT_TRADES: &[ClientMethod] = &[ClientMethod {
    name: "trades",
    native_name: "trades",
    arguments: MARKET_LIMIT,
}];
const CLIENT_ORDER_BOOK: &[ClientMethod] = &[ClientMethod {
    name: "orderBook",
    native_name: "orderBook",
    arguments: MARKET_DEPTH,
}];
const CLIENT_TICKER: &[ClientMethod] = &[ClientMethod {
    name: "ticker",
    native_name: "ticker",
    arguments: MARKET,
}];
const CLIENT_CANDLES: &[ClientMethod] = &[ClientMethod {
    name: "candles",
    native_name: "candles",
    arguments: CANDLE_REQUEST,
}];
const CLIENT_SUBSCRIBE: &[ClientMethod] = &[
    ClientMethod {
        name: "subscribe",
        native_name: "subscribe",
        arguments: SUBSCRIPTION,
    },
    ClientMethod {
        name: "subscribeWith",
        native_name: "subscribeWith",
        arguments: SUBSCRIPTION_CONFIG,
    },
];
const CLIENT_BALANCES: &[ClientMethod] = &[ClientMethod {
    name: "balances",
    native_name: "balances",
    arguments: &[],
}];
const CLIENT_ORDER_RULES: &[ClientMethod] = &[ClientMethod {
    name: "orderRules",
    native_name: "orderRules",
    arguments: MARKET,
}];
const CLIENT_ASSET_NETWORKS: &[ClientMethod] = &[ClientMethod {
    name: "assetNetworks",
    native_name: "assetNetworks",
    arguments: ASSET,
}];
const CLIENT_DEPOSIT_ADDRESSES: &[ClientMethod] = &[ClientMethod {
    name: "depositAddresses",
    native_name: "depositAddresses",
    arguments: &[],
}];
const CLIENT_DEPOSIT_ADDRESS: &[ClientMethod] = &[ClientMethod {
    name: "depositAddress",
    native_name: "depositAddress",
    arguments: DEPOSIT_ADDRESS_REQUEST,
}];
const CLIENT_CREATE_DEPOSIT_ADDRESS: &[ClientMethod] = &[ClientMethod {
    name: "createDepositAddress",
    native_name: "createDepositAddress",
    arguments: DEPOSIT_ADDRESS_REQUEST,
}];
const CLIENT_PREPARE_WITHDRAWAL: &[ClientMethod] = &[ClientMethod {
    name: "prepareWithdrawal",
    native_name: "prepareWithdrawal",
    arguments: WITHDRAW_REQUEST,
}];
const CLIENT_WITHDRAW: &[ClientMethod] = &[ClientMethod {
    name: "withdraw",
    native_name: "withdraw",
    arguments: WITHDRAW_REQUEST,
}];
const CLIENT_DEPOSIT: &[ClientMethod] = &[ClientMethod {
    name: "deposit",
    native_name: "deposit",
    arguments: TRANSFER_LOOKUP_REQUEST,
}];
const CLIENT_WITHDRAWAL: &[ClientMethod] = &[ClientMethod {
    name: "withdrawal",
    native_name: "withdrawal",
    arguments: TRANSFER_LOOKUP_REQUEST,
}];
const CLIENT_CANCEL_WITHDRAWAL: &[ClientMethod] = &[ClientMethod {
    name: "cancelWithdrawal",
    native_name: "cancelWithdrawal",
    arguments: WITHDRAWAL_ID,
}];
const CLIENT_DEPOSITS: &[ClientMethod] = &[ClientMethod {
    name: "deposits",
    native_name: "deposits",
    arguments: TRANSFER_HISTORY_REQUEST,
}];
const CLIENT_WITHDRAWALS: &[ClientMethod] = &[ClientMethod {
    name: "withdrawals",
    native_name: "withdrawals",
    arguments: TRANSFER_HISTORY_REQUEST,
}];
const CLIENT_OPEN_ORDERS: &[ClientMethod] = &[
    ClientMethod {
        name: "openOrders",
        native_name: "openOrders",
        arguments: &[],
    },
    ClientMethod {
        name: "openOrdersOn",
        native_name: "openOrdersOn",
        arguments: MARKET,
    },
];
const CLIENT_ORDER: &[ClientMethod] = &[ClientMethod {
    name: "order",
    native_name: "order",
    arguments: ORDER,
}];
const CLIENT_ORDER_BY_CLIENT_ID: &[ClientMethod] = &[ClientMethod {
    name: "orderByClientId",
    native_name: "orderByClientId",
    arguments: ORDER_BY_CLIENT_ID,
}];
const CLIENT_ORDERS_BY_IDS: &[ClientMethod] = &[ClientMethod {
    name: "ordersByIds",
    native_name: "ordersByIds",
    arguments: ORDER_LOOKUP_REQUEST,
}];
const CLIENT_ORDER_HISTORY: &[ClientMethod] = &[ClientMethod {
    name: "orderHistory",
    native_name: "orderHistory",
    arguments: ORDER_HISTORY_REQUEST,
}];
const CLIENT_ACCOUNT_STREAM: &[ClientMethod] = &[
    ClientMethod {
        name: "subscribeAccount",
        native_name: "subscribeAccount",
        arguments: &[],
    },
    ClientMethod {
        name: "subscribeAccountWith",
        native_name: "subscribeAccountWith",
        arguments: CONFIG,
    },
];
const CLIENT_PLACE_ORDER: &[ClientMethod] = &[ClientMethod {
    name: "placeOrder",
    native_name: "placeOrder",
    arguments: ORDER_REQUEST,
}];
const CLIENT_CANCEL_ORDER: &[ClientMethod] = &[ClientMethod {
    name: "cancelOrder",
    native_name: "cancelOrder",
    arguments: CANCEL_ORDER,
}];
const CLIENT_CANCEL_ORDER_BY_CLIENT_ID: &[ClientMethod] = &[ClientMethod {
    name: "cancelOrderByClientId",
    native_name: "cancelOrderByClientId",
    arguments: CANCEL_ORDER_BY_CLIENT_ID,
}];
const CLIENT_CANCEL_ORDERS: &[ClientMethod] = &[ClientMethod {
    name: "cancelOrders",
    native_name: "cancelOrders",
    arguments: CANCEL_ORDERS_REQUEST,
}];
const CLIENT_POSITIONS: &[ClientMethod] = &[
    ClientMethod {
        name: "positions",
        native_name: "positions",
        arguments: &[],
    },
    ClientMethod {
        name: "positionsOn",
        native_name: "positionsOn",
        arguments: MARKET,
    },
];
const CLIENT_MARGIN_SUMMARY: &[ClientMethod] = &[ClientMethod {
    name: "marginSummary",
    native_name: "marginSummary",
    arguments: &[],
}];
const CLIENT_FUNDING_RATES: &[ClientMethod] = &[ClientMethod {
    name: "fundingRates",
    native_name: "fundingRates",
    arguments: HISTORY_REQUEST,
}];
const CLIENT_FUNDING_PAYMENTS: &[ClientMethod] = &[ClientMethod {
    name: "fundingPayments",
    native_name: "fundingPayments",
    arguments: HISTORY_REQUEST,
}];
const CLIENT_SET_MARGIN: &[ClientMethod] = &[ClientMethod {
    name: "setMargin",
    native_name: "setMargin",
    arguments: MARGIN_REQUEST,
}];

const ADAPTER_OPERATIONS: &[Operation] = &[
    Operation {
        rust_name: "markets",
        language_name: "markets",
        feature: "markets",
        arguments: KIND,
        result: ApiType::List("MarketInfo"),
        client_methods: CLIENT_MARKETS,
    },
    Operation {
        rust_name: "trades",
        language_name: "trades",
        feature: "trades",
        arguments: MARKET_LIMIT,
        result: ApiType::List("Trade"),
        client_methods: CLIENT_TRADES,
    },
    Operation {
        rust_name: "order_book",
        language_name: "orderBook",
        feature: "order_book",
        arguments: MARKET_DEPTH,
        result: ApiType::Named("OrderBook"),
        client_methods: CLIENT_ORDER_BOOK,
    },
    Operation {
        rust_name: "ticker",
        language_name: "ticker",
        feature: "ticker",
        arguments: MARKET,
        result: ApiType::Named("Ticker"),
        client_methods: CLIENT_TICKER,
    },
    Operation {
        rust_name: "candles",
        language_name: "candles",
        feature: "candles",
        arguments: CANDLE_REQUEST,
        result: ApiType::List("Candle"),
        client_methods: CLIENT_CANDLES,
    },
    Operation {
        rust_name: "subscribe",
        language_name: "subscribe",
        feature: "trade_stream",
        arguments: SUBSCRIPTION_CONFIG,
        result: ApiType::MarketStream,
        client_methods: CLIENT_SUBSCRIBE,
    },
    Operation {
        rust_name: "balances",
        language_name: "balances",
        feature: "balances",
        arguments: &[],
        result: ApiType::List("Balance"),
        client_methods: CLIENT_BALANCES,
    },
    Operation {
        rust_name: "order_rules",
        language_name: "orderRules",
        feature: "trading",
        arguments: MARKET,
        result: ApiType::Named("OrderRules"),
        client_methods: CLIENT_ORDER_RULES,
    },
    Operation {
        rust_name: "asset_networks",
        language_name: "assetNetworks",
        feature: "asset_networks",
        arguments: ASSET,
        result: ApiType::List("AssetNetwork"),
        client_methods: CLIENT_ASSET_NETWORKS,
    },
    Operation {
        rust_name: "deposit_addresses",
        language_name: "depositAddresses",
        feature: "deposit_addresses",
        arguments: &[],
        result: ApiType::List("DepositAddressEntry"),
        client_methods: CLIENT_DEPOSIT_ADDRESSES,
    },
    Operation {
        rust_name: "deposit_address",
        language_name: "depositAddress",
        feature: "deposit_addresses",
        arguments: DEPOSIT_ADDRESS_REQUEST,
        result: ApiType::Named("DepositAddress"),
        client_methods: CLIENT_DEPOSIT_ADDRESS,
    },
    Operation {
        rust_name: "create_deposit_address",
        language_name: "createDepositAddress",
        feature: "deposit_addresses",
        arguments: DEPOSIT_ADDRESS_REQUEST,
        result: ApiType::Named("DepositAddress"),
        client_methods: CLIENT_CREATE_DEPOSIT_ADDRESS,
    },
    Operation {
        rust_name: "prepare_withdrawal",
        language_name: "prepareWithdrawal",
        feature: "withdrawal_quotes",
        arguments: WITHDRAW_REQUEST,
        result: ApiType::Named("WithdrawalQuote"),
        client_methods: CLIENT_PREPARE_WITHDRAWAL,
    },
    Operation {
        rust_name: "withdraw",
        language_name: "withdraw",
        feature: "withdrawals",
        arguments: WITHDRAW_REQUEST,
        result: ApiType::Named("Withdrawal"),
        client_methods: CLIENT_WITHDRAW,
    },
    Operation {
        rust_name: "deposit",
        language_name: "deposit",
        feature: "deposit_lookup",
        arguments: TRANSFER_LOOKUP_REQUEST,
        result: ApiType::Named("Deposit"),
        client_methods: CLIENT_DEPOSIT,
    },
    Operation {
        rust_name: "withdrawal",
        language_name: "withdrawal",
        feature: "withdrawal_lookup",
        arguments: TRANSFER_LOOKUP_REQUEST,
        result: ApiType::Named("Withdrawal"),
        client_methods: CLIENT_WITHDRAWAL,
    },
    Operation {
        rust_name: "cancel_withdrawal",
        language_name: "cancelWithdrawal",
        feature: "withdrawal_cancellation",
        arguments: WITHDRAWAL_ID,
        result: ApiType::Unit,
        client_methods: CLIENT_CANCEL_WITHDRAWAL,
    },
    Operation {
        rust_name: "deposits",
        language_name: "deposits",
        feature: "deposit_history",
        arguments: TRANSFER_HISTORY_REQUEST,
        result: ApiType::Page("Deposit"),
        client_methods: CLIENT_DEPOSITS,
    },
    Operation {
        rust_name: "withdrawals",
        language_name: "withdrawals",
        feature: "withdrawal_history",
        arguments: TRANSFER_HISTORY_REQUEST,
        result: ApiType::Page("Withdrawal"),
        client_methods: CLIENT_WITHDRAWALS,
    },
    Operation {
        rust_name: "open_orders",
        language_name: "openOrders",
        feature: "open_orders",
        arguments: OPTIONAL_MARKET,
        result: ApiType::List("Order"),
        client_methods: CLIENT_OPEN_ORDERS,
    },
    Operation {
        rust_name: "order",
        language_name: "order",
        feature: "order_history",
        arguments: ORDER,
        result: ApiType::Named("Order"),
        client_methods: CLIENT_ORDER,
    },
    Operation {
        rust_name: "order_by_client_id",
        language_name: "orderByClientId",
        feature: "order_history",
        arguments: ORDER_BY_CLIENT_ID,
        result: ApiType::Named("Order"),
        client_methods: CLIENT_ORDER_BY_CLIENT_ID,
    },
    Operation {
        rust_name: "orders_by_ids",
        language_name: "ordersByIds",
        feature: "order_history",
        arguments: ORDER_LOOKUP_REQUEST,
        result: ApiType::List("Order"),
        client_methods: CLIENT_ORDERS_BY_IDS,
    },
    Operation {
        rust_name: "order_history",
        language_name: "orderHistory",
        feature: "order_history",
        arguments: ORDER_HISTORY_REQUEST,
        result: ApiType::Page("Order"),
        client_methods: CLIENT_ORDER_HISTORY,
    },
    Operation {
        rust_name: "subscribe_account",
        language_name: "subscribeAccount",
        feature: "account_stream",
        arguments: CONFIG,
        result: ApiType::AccountStream,
        client_methods: CLIENT_ACCOUNT_STREAM,
    },
    Operation {
        rust_name: "place_order",
        language_name: "placeOrder",
        feature: "trading",
        arguments: ORDER_REQUEST,
        result: ApiType::Named("Order"),
        client_methods: CLIENT_PLACE_ORDER,
    },
    Operation {
        rust_name: "cancel_order",
        language_name: "cancelOrder",
        feature: "trading",
        arguments: CANCEL_ORDER,
        result: ApiType::Unit,
        client_methods: CLIENT_CANCEL_ORDER,
    },
    Operation {
        rust_name: "cancel_order_by_client_id",
        language_name: "cancelOrderByClientId",
        feature: "trading",
        arguments: CANCEL_ORDER_BY_CLIENT_ID,
        result: ApiType::Unit,
        client_methods: CLIENT_CANCEL_ORDER_BY_CLIENT_ID,
    },
    Operation {
        rust_name: "cancel_orders",
        language_name: "cancelOrders",
        feature: "trading",
        arguments: CANCEL_ORDERS_REQUEST,
        result: ApiType::Named("CancelOrdersResult"),
        client_methods: CLIENT_CANCEL_ORDERS,
    },
    Operation {
        rust_name: "positions",
        language_name: "positions",
        feature: "positions",
        arguments: OPTIONAL_MARKET,
        result: ApiType::List("Position"),
        client_methods: CLIENT_POSITIONS,
    },
    Operation {
        rust_name: "margin_summary",
        language_name: "marginSummary",
        feature: "margin",
        arguments: &[],
        result: ApiType::Named("MarginSummary"),
        client_methods: CLIENT_MARGIN_SUMMARY,
    },
    Operation {
        rust_name: "funding_rates",
        language_name: "fundingRates",
        feature: "funding_rates",
        arguments: HISTORY_REQUEST,
        result: ApiType::Page("FundingRate"),
        client_methods: CLIENT_FUNDING_RATES,
    },
    Operation {
        rust_name: "funding_payments",
        language_name: "fundingPayments",
        feature: "funding_payments",
        arguments: HISTORY_REQUEST,
        result: ApiType::Page("FundingPayment"),
        client_methods: CLIENT_FUNDING_PAYMENTS,
    },
    Operation {
        rust_name: "set_margin",
        language_name: "setMargin",
        feature: "margin_config",
        arguments: MARGIN_REQUEST,
        result: ApiType::Unit,
        client_methods: CLIENT_SET_MARGIN,
    },
];

const CLIENT_COMPOSITIONS: &[ClientComposition] = &[
    ClientComposition {
        rust_name: "prepare_exchange_transfer",
        language_name: "prepareTransferTo",
        arguments: EXCHANGE_TRANSFER_REQUEST,
        result: ApiType::Named("TransferPlan"),
    },
    ClientComposition {
        rust_name: "prepare_chain_transfer",
        language_name: "prepareTransferToChain",
        arguments: CHAIN_TRANSFER_REQUEST,
        result: ApiType::Named("TransferPlan"),
    },
    ClientComposition {
        rust_name: "execute_transfer_plan",
        language_name: "executeTransfer",
        arguments: TRANSFER_PLAN,
        result: ApiType::Named("Withdrawal"),
    },
];

const CLIENT_MEMBERS: &[&str] = &[
    "exchange",
    "supports",
    "adapter",
    "markets",
    "trades",
    "orderBook",
    "ticker",
    "candles",
    "subscribe",
    "subscribeWith",
    "balances",
    "orderRules",
    "assetNetworks",
    "depositAddresses",
    "depositAddress",
    "createDepositAddress",
    "prepareWithdrawal",
    "withdraw",
    "deposit",
    "withdrawal",
    "cancelWithdrawal",
    "deposits",
    "withdrawals",
    "prepareTransferTo",
    "prepareTransferToChain",
    "executeTransfer",
    "openOrders",
    "openOrdersOn",
    "order",
    "orderByClientId",
    "ordersByIds",
    "orderHistory",
    "subscribeAccount",
    "subscribeAccountWith",
    "placeOrder",
    "cancelOrder",
    "cancelOrderByClientId",
    "cancelOrders",
    "positions",
    "positionsOn",
    "marginSummary",
    "fundingRates",
    "fundingPayments",
    "setMargin",
];

const ERRORS: &[&str] = &[
    "InvalidRequest",
    "Transfer",
    "Unsupported",
    "Adapter",
    "Auth",
    "Exchange",
    "Transport",
    "Decode",
];

const fn identifier_variant(rust_name: &'static str, wire_name: &'static str) -> IdentifierVariant {
    IdentifierVariant {
        rust_name,
        wire_name,
    }
}

const fn identifier(
    name: &'static str,
    variants: &'static [IdentifierVariant],
    open: bool,
) -> Identifier {
    Identifier {
        name,
        variants,
        open,
    }
}

const EXCHANGE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Upbit", "upbit"),
    identifier_variant("Bithumb", "bithumb"),
    identifier_variant("Binance", "binance"),
    identifier_variant("Hyperliquid", "hyperliquid"),
];
const FEATURE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Markets", "markets"),
    identifier_variant("Trades", "trades"),
    identifier_variant("OrderBook", "order_book"),
    identifier_variant("Ticker", "ticker"),
    identifier_variant("Candles", "candles"),
    identifier_variant("TradeStream", "trade_stream"),
    identifier_variant("OrderBookStream", "order_book_stream"),
    identifier_variant("TickerStream", "ticker_stream"),
    identifier_variant("CandleStream", "candle_stream"),
    identifier_variant("Balances", "balances"),
    identifier_variant("AssetNetworks", "asset_networks"),
    identifier_variant("DepositAddresses", "deposit_addresses"),
    identifier_variant("DepositHistory", "deposit_history"),
    identifier_variant("DepositLookup", "deposit_lookup"),
    identifier_variant("TravelRule", "travel_rule"),
    identifier_variant("WithdrawalQuotes", "withdrawal_quotes"),
    identifier_variant("Withdrawals", "withdrawals"),
    identifier_variant("WithdrawalHistory", "withdrawal_history"),
    identifier_variant("WithdrawalLookup", "withdrawal_lookup"),
    identifier_variant("WithdrawalCancellation", "withdrawal_cancellation"),
    identifier_variant("OpenOrders", "open_orders"),
    identifier_variant("OrderHistory", "order_history"),
    identifier_variant("AccountStream", "account_stream"),
    identifier_variant("Trading", "trading"),
    identifier_variant("Positions", "positions"),
    identifier_variant("Margin", "margin"),
    identifier_variant("FundingRates", "funding_rates"),
    identifier_variant("FundingPayments", "funding_payments"),
    identifier_variant("MarginConfig", "margin_config"),
    identifier_variant("ReduceOnlyOrders", "reduce_only_orders"),
];
const MARKET_KIND_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Spot", "spot"),
    identifier_variant("Perpetual", "perpetual"),
];
const MARKET_STATUS_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Active", "active"),
    identifier_variant("Paused", "paused"),
    identifier_variant("Delisted", "delisted"),
    identifier_variant("Unknown", "unknown"),
];
const SIDE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Buy", "buy"),
    identifier_variant("Sell", "sell"),
];
const INTERVAL_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Sec1", "sec1"),
    identifier_variant("Min1", "min1"),
    identifier_variant("Min3", "min3"),
    identifier_variant("Min5", "min5"),
    identifier_variant("Min10", "min10"),
    identifier_variant("Min15", "min15"),
    identifier_variant("Min30", "min30"),
    identifier_variant("Hour1", "hour1"),
    identifier_variant("Hour2", "hour2"),
    identifier_variant("Hour4", "hour4"),
    identifier_variant("Hour6", "hour6"),
    identifier_variant("Hour8", "hour8"),
    identifier_variant("Hour12", "hour12"),
    identifier_variant("Day1", "day1"),
    identifier_variant("Day3", "day3"),
    identifier_variant("Week1", "week1"),
    identifier_variant("Month1", "month1"),
];
const OVERFLOW_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Backpressure", "backpressure"),
    identifier_variant("DropNewest", "drop_newest"),
];
const MARGIN_MODE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Cross", "cross"),
    identifier_variant("Isolated", "isolated"),
];
const ORDER_STATUS_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Accepted", "accepted"),
    identifier_variant("Open", "open"),
    identifier_variant("PartiallyFilled", "partially_filled"),
    identifier_variant("Filled", "filled"),
    identifier_variant("Cancelled", "cancelled"),
    identifier_variant("Rejected", "rejected"),
    identifier_variant("Unknown", "unknown"),
];
const ORDER_ID_KIND_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Exchange", "exchange"),
    identifier_variant("Client", "client"),
];
const ORDER_TYPE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Market", "market"),
    identifier_variant("Limit", "limit"),
    identifier_variant("Best", "best"),
];
const TIME_IN_FORCE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("GoodTilCancelled", "good_til_cancelled"),
    identifier_variant("ImmediateOrCancel", "immediate_or_cancel"),
    identifier_variant("FillOrKill", "fill_or_kill"),
    identifier_variant("PostOnly", "post_only"),
];
const SIZE_KIND_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Base", "base"),
    identifier_variant("Quote", "quote"),
];
const UPBIT_REGION_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Korea", "korea"),
    identifier_variant("Singapore", "singapore"),
    identifier_variant("Indonesia", "indonesia"),
    identifier_variant("Thailand", "thailand"),
];
const UPBIT_ORDER_DIRECTION_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Ascending", "asc"),
    identifier_variant("Descending", "desc"),
];
const UPBIT_CLOSED_ORDER_STATE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Done", "done"),
    identifier_variant("Cancel", "cancel"),
];
const UPBIT_SMP_TYPE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("CancelMaker", "cancel_maker"),
    identifier_variant("CancelTaker", "cancel_taker"),
    identifier_variant("Reduce", "reduce"),
];
const UPBIT_KRW_TWO_FACTOR_TYPE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Kakao", "kakao"),
    identifier_variant("Naver", "naver"),
    identifier_variant("Hana", "hana"),
];
const UPBIT_POCKET_TRANSFER_STATE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Submitted", "submitted"),
    identifier_variant("Processing", "processing"),
    identifier_variant("Done", "done"),
    identifier_variant("Failed", "failed"),
];
const UPBIT_POCKET_TRANSFER_DIRECTION_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Incoming", "in"),
    identifier_variant("Outgoing", "out"),
    identifier_variant("All", "all"),
];
const UPBIT_POCKET_TRANSFER_ORDER_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Ascending", "asc"),
    identifier_variant("Descending", "desc"),
];
const BITHUMB_ALERT_STEP_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Caution", "caution"),
    identifier_variant("Warning", "warning"),
    identifier_variant("Danger", "danger"),
    identifier_variant("Unknown", "unknown"),
];
const BITHUMB_PENDING_ORDER_STATE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Wait", "wait"),
    identifier_variant("Watch", "watch"),
];
const BITHUMB_CLOSED_ORDER_STATE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Done", "done"),
    identifier_variant("Cancel", "cancel"),
];
const BITHUMB_ORDER_DIRECTION_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Ascending", "asc"),
    identifier_variant("Descending", "desc"),
];
const BITHUMB_ORDER_LIST_STATE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Wait", "wait"),
    identifier_variant("Watch", "watch"),
    identifier_variant("Done", "done"),
    identifier_variant("Cancel", "cancel"),
];
const BITHUMB_TWAP_STATE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Progress", "progress"),
    identifier_variant("Done", "done"),
    identifier_variant("Cancel", "cancel"),
];
const BITHUMB_TWAP_ORDER_DIRECTION_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Ascending", "asc"),
    identifier_variant("Descending", "desc"),
];
const BINANCE_MARKET_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Spot", "spot"),
    identifier_variant("UsdMFutures", "usd_m"),
];
const BINANCE_C2C_TRADE_TYPE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Buy", "BUY"),
    identifier_variant("Sell", "SELL"),
];
const HYPERLIQUID_LEDGER_KIND_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Deposit", "deposit"),
    identifier_variant("Withdraw", "withdraw"),
    identifier_variant("InternalTransfer", "internal_transfer"),
    identifier_variant("SubAccountTransfer", "sub_account_transfer"),
    identifier_variant("SpotTransfer", "spot_transfer"),
    identifier_variant("AccountClassTransfer", "account_class_transfer"),
    identifier_variant("VaultDeposit", "vault_deposit"),
    identifier_variant("VaultWithdraw", "vault_withdraw"),
    identifier_variant("VaultDistribution", "vault_distribution"),
    identifier_variant("Liquidation", "liquidation"),
];
const EXCHANGE_ERROR_KIND_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Rejected", "rejected"),
    identifier_variant("RateLimited", "rate_limited"),
    identifier_variant("Unavailable", "unavailable"),
    identifier_variant("Unknown", "unknown"),
];
const NETWORK_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Bitcoin", "bitcoin"),
    identifier_variant("Ethereum", "ethereum"),
    identifier_variant("Arbitrum", "arbitrum"),
    identifier_variant("BnbSmartChain", "bnb_smart_chain"),
    identifier_variant("Tron", "tron"),
    identifier_variant("Solana", "solana"),
    identifier_variant("Polygon", "polygon"),
    identifier_variant("Base", "base"),
    identifier_variant("Optimism", "optimism"),
    identifier_variant("AvalancheC", "avalanche_c"),
    identifier_variant("XrpLedger", "xrp_ledger"),
    identifier_variant("Stellar", "stellar"),
    identifier_variant("Cosmos", "cosmos"),
    identifier_variant("Aptos", "aptos"),
    identifier_variant("Sui", "sui"),
    identifier_variant("Ton", "ton"),
    identifier_variant("Near", "near"),
    identifier_variant("Polkadot", "polkadot"),
];
const WITHDRAWAL_STATUS_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Pending", "pending"),
    identifier_variant("Processing", "processing"),
    identifier_variant("Completed", "completed"),
    identifier_variant("Cancelled", "cancelled"),
    identifier_variant("Failed", "failed"),
    identifier_variant("Unknown", "unknown"),
];
const DEPOSIT_STATUS_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Pending", "pending"),
    identifier_variant("Completed", "completed"),
    identifier_variant("Failed", "failed"),
    identifier_variant("Unknown", "unknown"),
];
const TRANSFER_ERROR_KIND_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("AssetMismatch", "asset_mismatch"),
    identifier_variant("NetworkMismatch", "network_mismatch"),
    identifier_variant("AmbiguousNetwork", "ambiguous_network"),
    identifier_variant("NetworkUnavailable", "network_unavailable"),
    identifier_variant("MemoRequired", "memo_required"),
    identifier_variant("DestinationUnavailable", "destination_unavailable"),
    identifier_variant("AddressNotAllowed", "address_not_allowed"),
    identifier_variant("TravelRuleRequired", "travel_rule_required"),
    identifier_variant("AmountOutOfRange", "amount_out_of_range"),
    identifier_variant("PlanExpired", "plan_expired"),
];

const IDENTIFIERS: &[Identifier] = &[
    identifier("Exchange", EXCHANGE_VARIANTS, false),
    identifier("Feature", FEATURE_VARIANTS, false),
    identifier("MarketKind", MARKET_KIND_VARIANTS, false),
    identifier("MarketStatus", MARKET_STATUS_VARIANTS, false),
    identifier("Side", SIDE_VARIANTS, false),
    identifier("Interval", INTERVAL_VARIANTS, false),
    identifier("Overflow", OVERFLOW_VARIANTS, false),
    identifier("MarginMode", MARGIN_MODE_VARIANTS, false),
    identifier("OrderStatus", ORDER_STATUS_VARIANTS, false),
    identifier("OrderIdKind", ORDER_ID_KIND_VARIANTS, false),
    identifier("OrderType", ORDER_TYPE_VARIANTS, false),
    identifier("TimeInForce", TIME_IN_FORCE_VARIANTS, false),
    identifier("SizeKind", SIZE_KIND_VARIANTS, false),
    identifier("UpbitRegion", UPBIT_REGION_VARIANTS, false),
    identifier("UpbitOrderDirection", UPBIT_ORDER_DIRECTION_VARIANTS, false),
    identifier(
        "UpbitClosedOrderState",
        UPBIT_CLOSED_ORDER_STATE_VARIANTS,
        false,
    ),
    identifier("UpbitSmpType", UPBIT_SMP_TYPE_VARIANTS, false),
    identifier(
        "UpbitKrwTwoFactorType",
        UPBIT_KRW_TWO_FACTOR_TYPE_VARIANTS,
        false,
    ),
    identifier(
        "UpbitPocketTransferState",
        UPBIT_POCKET_TRANSFER_STATE_VARIANTS,
        false,
    ),
    identifier(
        "UpbitPocketTransferDirection",
        UPBIT_POCKET_TRANSFER_DIRECTION_VARIANTS,
        false,
    ),
    identifier(
        "UpbitPocketTransferOrder",
        UPBIT_POCKET_TRANSFER_ORDER_VARIANTS,
        false,
    ),
    identifier("BithumbAlertStep", BITHUMB_ALERT_STEP_VARIANTS, false),
    identifier(
        "BithumbPendingOrderState",
        BITHUMB_PENDING_ORDER_STATE_VARIANTS,
        false,
    ),
    identifier(
        "BithumbClosedOrderState",
        BITHUMB_CLOSED_ORDER_STATE_VARIANTS,
        false,
    ),
    identifier(
        "BithumbOrderDirection",
        BITHUMB_ORDER_DIRECTION_VARIANTS,
        false,
    ),
    identifier(
        "BithumbOrderListState",
        BITHUMB_ORDER_LIST_STATE_VARIANTS,
        false,
    ),
    identifier("BithumbTwapState", BITHUMB_TWAP_STATE_VARIANTS, false),
    identifier(
        "BithumbTwapOrderDirection",
        BITHUMB_TWAP_ORDER_DIRECTION_VARIANTS,
        false,
    ),
    identifier("BinanceMarket", BINANCE_MARKET_VARIANTS, false),
    identifier(
        "BinanceC2cTradeType",
        BINANCE_C2C_TRADE_TYPE_VARIANTS,
        false,
    ),
    identifier(
        "HyperliquidLedgerKind",
        HYPERLIQUID_LEDGER_KIND_VARIANTS,
        true,
    ),
    identifier("ExchangeErrorKind", EXCHANGE_ERROR_KIND_VARIANTS, false),
    identifier("Network", NETWORK_VARIANTS, true),
    identifier("WithdrawalStatus", WITHDRAWAL_STATUS_VARIANTS, false),
    identifier("DepositStatus", DEPOSIT_STATUS_VARIANTS, false),
    identifier("TransferErrorKind", TRANSFER_ERROR_KIND_VARIANTS, false),
];

const MODELS: &[&str] = &[
    "Market",
    "MarketInfo",
    "Trade",
    "Level",
    "OrderBook",
    "Ticker",
    "Candle",
    "Balance",
    "OrderAccount",
    "OrderOption",
    "OrderRules",
    "AssetNetwork",
    "DepositAddress",
    "DepositAddressEntry",
    "ExchangeDestination",
    "ChainDestination",
    "ExchangeTransferRequest",
    "ChainTransferRequest",
    "TransferDestination",
    "WithdrawalFee",
    "TravelRuleRequirement",
    "WithdrawalQuote",
    "TransferPlan",
    "Withdrawal",
    "Deposit",
    "Order",
    "CancelledOrder",
    "OrderCancelFailure",
    "CancelOrdersResult",
    "Position",
    "MarginSummary",
    "FundingRate",
    "FundingPayment",
    "CandleRequest",
    "OrderRequest",
    "OrderLookupRequest",
    "CancelOrdersRequest",
    "OrderHistoryRequest",
    "DepositAddressRequest",
    "WithdrawRequest",
    "TransferLookupRequest",
    "TransferHistoryRequest",
    "StreamConfig",
    "Subscription",
    "HistoryRequest",
    "MarginRequest",
    "UpbitMarketEvent",
    "UpbitListedSubscription",
    "UpbitSubscriptionList",
    "UpbitYearCandle",
    "UpbitOrderBookInstrument",
    "UpbitOrderDetailRequest",
    "UpbitOrderDetailTrade",
    "UpbitOrderDetail",
    "UpbitClosedOrdersRequest",
    "UpbitClosedOrder",
    "UpbitDepositInfo",
    "UpbitWithdrawalAddress",
    "UpbitTravelRuleVasp",
    "UpbitTravelRuleVerification",
    "UpbitBatchCancelScope",
    "UpbitBatchCancelRequest",
    "UpbitOrderReference",
    "UpbitOrderVolume",
    "UpbitCancelAndNewOrder",
    "UpbitCancelAndNewOrderRequest",
    "UpbitCancelAndNewOrderResult",
    "UpbitKrwTransferRequest",
    "UpbitKrwDeposit",
    "UpbitKrwWithdrawal",
    "UpbitApiKey",
    "UpbitPocket",
    "UpbitPocketApiKey",
    "UpbitPocketApiKeyGroup",
    "UpbitPocketApiKeysRequest",
    "UpbitPocketBalance",
    "UpbitPocketTransferQuery",
    "UpbitPocketUniversalTransferRequest",
    "UpbitPocketTransferRequest",
    "UpbitPocketTransfer",
    "BithumbMarketAlert",
    "BithumbNotice",
    "BithumbApiKey",
    "BithumbKrwWithdrawalsRequest",
    "BithumbKrwDepositsRequest",
    "BithumbKrwTransferRequest",
    "BithumbKrwWithdrawal",
    "BithumbKrwDeposit",
    "BithumbPendingOrdersRequest",
    "BithumbClosedOrdersRequest",
    "BithumbClosedOrder",
    "BithumbBatchOrdersRequest",
    "BithumbBatchOrder",
    "BithumbBatchOrderFailure",
    "BithumbBatchOrderOutcome",
    "BithumbBatchOrdersResult",
    "BithumbTwapOrdersRequest",
    "BithumbTwapOrderRequest",
    "BithumbTwapOrder",
    "BithumbAssetFee",
    "BithumbNetworkFee",
    "BithumbWithdrawalAddress",
    "BithumbOrderDetailRequest",
    "BithumbOrderDetailTrade",
    "BithumbOrderDetail",
    "BithumbOrderListRequest",
    "BithumbOrderListItem",
    "BinanceSymbolFilters",
    "BinanceSpotOrderDetail",
    "BinanceDepositHistoryRequest",
    "BinanceWithdrawHistoryRequest",
    "BinanceSpotAccountInformation",
    "BinanceSpotCommissionRates",
    "BinanceSpotAccountBalance",
    "BinanceSpotCancelAllOpenOrders",
    "BinanceSpotCancelledOrder",
    "BinanceUsdMAccountInformation",
    "BinanceUsdMAccountAsset",
    "BinanceUsdMAccountPosition",
    "BinanceUsdMPositionInformation",
    "BinanceExchangeInfo",
    "BinanceExchangeSymbol",
    "BinanceCoinInformation",
    "BinanceCoinNetworkInformation",
    "BinanceApiKeyPermissions",
    "BinanceDepositHistory",
    "BinanceDepositHistoryEntry",
    "BinanceQuestionnaireRequirements",
    "BinanceWithdrawalAddress",
    "BinanceWithdrawHistory",
    "BinanceWithdrawHistoryEntry",
    "BinanceSpotAveragePrice",
    "BinanceMarkPrice",
    "BinanceOpenInterest",
    "BinanceAggregateTradesRequest",
    "BinanceAggregateTrade",
    "BinanceAccountTrade",
    "BinanceTestOrderRequest",
    "BinanceTestOrder",
    "BinanceC2cTradeHistoryRequest",
    "BinanceC2cTrade",
    "BinanceC2cTradeHistoryPage",
    "HyperliquidLedgerEntry",
    "HyperliquidCandleSnapshot",
    "HyperliquidBookLevel",
    "HyperliquidL2Book",
    "HyperliquidRecentTrade",
    "HyperliquidTradeEvent",
    "HyperliquidOrderBookEvent",
    "HyperliquidCandleEvent",
    "HyperliquidAssetContextEvent",
    "HyperliquidOrderUpdate",
    "HyperliquidSpotStateBalance",
    "HyperliquidSpotStateEvent",
    "HyperliquidMarketEvent",
    "HyperliquidAccountEvent",
    "HyperliquidFundingHistoryEntry",
    "HyperliquidUserFunding",
    "HyperliquidSpotBalance",
    "HyperliquidSpotClearinghouseState",
    "HyperliquidEvmContract",
    "HyperliquidSpotToken",
    "HyperliquidSpotPair",
    "HyperliquidSpotMeta",
    "HyperliquidSpotAssetContext",
    "HyperliquidSpotMetaAndAssetContexts",
    "HyperliquidMidPrice",
    "HyperliquidAssetContext",
    "HyperliquidUserRateLimit",
    "HyperliquidUserRole",
    "HyperliquidReferral",
    "HyperliquidReferrer",
    "HyperliquidUserFees",
    "HyperliquidUserFill",
    "HyperliquidOrderReference",
    "HyperliquidOpenOrder",
    "HyperliquidOrderDetail",
    "HyperliquidOrderInfo",
    "HyperliquidOrderStatusResponse",
    "HyperliquidDailyVolume",
    "HyperliquidPortfolioPeriod",
    "HyperliquidPortfolioPoint",
    "HyperliquidSubAccount",
    "HyperliquidVaultEquity",
];

const MARKETS_DEPTH: &[Argument] = &[
    argument("markets", ApiType::List("Market"), None),
    argument("depth", ApiType::OptionalNumber, Some("null")),
];
const MARKETS_LEVEL_DEPTH: &[Argument] = &[
    argument("markets", ApiType::List("Market"), None),
    argument("level", ApiType::Named("Decimal"), None),
    argument("depth", ApiType::OptionalNumber, Some("null")),
];
const MARKETS: &[Argument] = &[argument("markets", ApiType::List("Market"), None)];
const QUOTE_CURRENCIES: &[Argument] = &[argument("quoteCurrencies", ApiType::List("String"), None)];
const YEAR_CANDLE_QUERY: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("to", ApiType::OptionalNamed("Timestamp"), None),
    argument("count", ApiType::OptionalNumber, Some("null")),
];
const NOTICE_COUNT: &[Argument] = &[argument("count", ApiType::OptionalNumber, Some("null"))];
const FEE_CURRENCY: &[Argument] = &[argument("currency", ApiType::String, None)];
const BITHUMB_PENDING_ORDERS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbPendingOrdersRequest"),
    None,
)];
const BITHUMB_CLOSED_ORDERS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbClosedOrdersRequest"),
    None,
)];
const BITHUMB_BATCH_ORDERS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbBatchOrdersRequest"),
    None,
)];
const BITHUMB_KRW_WITHDRAWALS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbKrwWithdrawalsRequest"),
    None,
)];
const BITHUMB_KRW_DEPOSITS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbKrwDepositsRequest"),
    None,
)];
const BITHUMB_KRW_TRANSFER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbKrwTransferRequest"),
    None,
)];
const UPBIT_KRW_TRANSFER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitKrwTransferRequest"),
    None,
)];
const UPBIT_POCKET_API_KEYS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitPocketApiKeysRequest"),
    None,
)];
const UPBIT_POCKET_UUID: &[Argument] = &[argument("pocketUuid", ApiType::String, None)];
const UPBIT_POCKET_UNIVERSAL_TRANSFER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitPocketUniversalTransferRequest"),
    None,
)];
const UPBIT_POCKET_TRANSFER_QUERY: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitPocketTransferQuery"),
    None,
)];
const UPBIT_POCKET_TRANSFER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("UpbitPocketTransferRequest"),
    None,
)];
const BITHUMB_ORDER_DETAIL_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbOrderDetailRequest"),
    None,
)];
const BITHUMB_ORDER_LIST_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbOrderListRequest"),
    None,
)];
const BITHUMB_TWAP_ORDERS_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbTwapOrdersRequest"),
    None,
)];
const BITHUMB_TWAP_ORDER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BithumbTwapOrderRequest"),
    None,
)];
const BITHUMB_TWAP_ORDER_ID: &[Argument] = &[argument("algoOrderId", ApiType::String, None)];
const BINANCE_AGGREGATE_TRADES_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BinanceAggregateTradesRequest"),
    None,
)];
const BINANCE_TEST_ORDER_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BinanceTestOrderRequest"),
    None,
)];
const BINANCE_C2C_TRADE_HISTORY_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BinanceC2cTradeHistoryRequest"),
    None,
)];
const BINANCE_DEPOSIT_HISTORY_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BinanceDepositHistoryRequest"),
    None,
)];
const BINANCE_WITHDRAW_HISTORY_REQUEST: &[Argument] = &[argument(
    "request",
    ApiType::Named("BinanceWithdrawHistoryRequest"),
    None,
)];
const HYPERLIQUID_CANDLE_SNAPSHOT: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("interval", ApiType::String, None),
    argument("from", ApiType::Named("Timestamp"), None),
    argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
];
const HYPERLIQUID_TIME_RANGE: &[Argument] = &[
    argument("from", ApiType::Named("Timestamp"), None),
    argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
];
const LEDGER_RANGE: &[Argument] = &[
    argument("from", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("cursor", ApiType::OptionalNamed("Cursor"), Some("null")),
    argument("limit", ApiType::OptionalNumber, Some("null")),
];
const HYPERLIQUID_USER_FILLS: &[Argument] = &[argument("aggregateByTime", ApiType::Boolean, None)];
const HYPERLIQUID_USER_FILLS_BY_TIME: &[Argument] = &[
    argument("from", ApiType::Named("Timestamp"), None),
    argument("to", ApiType::OptionalNamed("Timestamp"), None),
    argument("aggregateByTime", ApiType::Boolean, None),
];
const HYPERLIQUID_ORDER_REFERENCE: &[Argument] = &[argument(
    "reference",
    ApiType::Named("HyperliquidOrderReference"),
    None,
)];
const HYPERLIQUID_DETAILED_SUBSCRIPTION: &[Argument] = &[argument(
    "subscription",
    ApiType::Named("Subscription"),
    None,
)];
const HYPERLIQUID_DETAILED_SUBSCRIPTION_CONFIG: &[Argument] = &[
    argument("subscription", ApiType::Named("Subscription"), None),
    argument("config", ApiType::Named("StreamConfig"), None),
];
const ACCESS_KEY_CREDENTIALS: &[Argument] = &[
    argument("accessKey", ApiType::OptionalString, Some("null")),
    argument("secretKey", ApiType::OptionalString, Some("null")),
];
const UPBIT_REGION_CREDENTIALS: &[Argument] = &[
    argument("region", ApiType::Named("UpbitRegion"), None),
    argument("accessKey", ApiType::OptionalString, Some("null")),
    argument("secretKey", ApiType::OptionalString, Some("null")),
];
const API_KEY_CREDENTIALS: &[Argument] = &[
    argument("apiKey", ApiType::OptionalString, Some("null")),
    argument("secretKey", ApiType::OptionalString, Some("null")),
];
const WALLET_CREDENTIALS: &[Argument] = &[
    argument("address", ApiType::OptionalString, Some("null")),
    argument("privateKey", ApiType::OptionalString, Some("null")),
];

const fn provider_option(name: &'static str, value: ProviderOptionValue) -> ProviderOption {
    ProviderOption { name, value }
}

const UPBIT_OPTIONS: &[ProviderOption] = &[
    provider_option(
        "region",
        ProviderOptionValue::Identifier {
            name: "UpbitRegion",
            variant: "Korea",
        },
    ),
    provider_option("access_key", ProviderOptionValue::Argument("accessKey")),
    provider_option("secret_key", ProviderOptionValue::Argument("secretKey")),
];
const UPBIT_REGION_OPTIONS: &[ProviderOption] = &[
    provider_option("region", ProviderOptionValue::Argument("region")),
    provider_option("access_key", ProviderOptionValue::Argument("accessKey")),
    provider_option("secret_key", ProviderOptionValue::Argument("secretKey")),
];
const BITHUMB_OPTIONS: &[ProviderOption] = &[
    provider_option("access_key", ProviderOptionValue::Argument("accessKey")),
    provider_option("secret_key", ProviderOptionValue::Argument("secretKey")),
];
const BINANCE_SPOT_OPTIONS: &[ProviderOption] = &[
    provider_option(
        "venue",
        ProviderOptionValue::Identifier {
            name: "BinanceMarket",
            variant: "Spot",
        },
    ),
    provider_option("api_key", ProviderOptionValue::Argument("apiKey")),
    provider_option("secret_key", ProviderOptionValue::Argument("secretKey")),
];
const BINANCE_USD_M_OPTIONS: &[ProviderOption] = &[
    provider_option(
        "venue",
        ProviderOptionValue::Identifier {
            name: "BinanceMarket",
            variant: "UsdMFutures",
        },
    ),
    provider_option("api_key", ProviderOptionValue::Argument("apiKey")),
    provider_option("secret_key", ProviderOptionValue::Argument("secretKey")),
];
const HYPERLIQUID_OPTIONS: &[ProviderOption] = &[
    provider_option("testnet", ProviderOptionValue::Boolean(false)),
    provider_option("address", ProviderOptionValue::Argument("address")),
    provider_option("private_key", ProviderOptionValue::Argument("privateKey")),
];
const HYPERLIQUID_TESTNET_OPTIONS: &[ProviderOption] = &[
    provider_option("testnet", ProviderOptionValue::Boolean(true)),
    provider_option("address", ProviderOptionValue::Argument("address")),
    provider_option("private_key", ProviderOptionValue::Argument("privateKey")),
];

const UPBIT_CONSTRUCTORS: &[ProviderConstructor] = &[
    ProviderConstructor {
        name: "constructor",
        arguments: ACCESS_KEY_CREDENTIALS,
        options: UPBIT_OPTIONS,
    },
    ProviderConstructor {
        name: "withRegion",
        arguments: UPBIT_REGION_CREDENTIALS,
        options: UPBIT_REGION_OPTIONS,
    },
];
const BITHUMB_CONSTRUCTORS: &[ProviderConstructor] = &[ProviderConstructor {
    name: "constructor",
    arguments: ACCESS_KEY_CREDENTIALS,
    options: BITHUMB_OPTIONS,
}];
const BINANCE_CONSTRUCTORS: &[ProviderConstructor] = &[
    ProviderConstructor {
        name: "spot",
        arguments: API_KEY_CREDENTIALS,
        options: BINANCE_SPOT_OPTIONS,
    },
    ProviderConstructor {
        name: "usdMFutures",
        arguments: API_KEY_CREDENTIALS,
        options: BINANCE_USD_M_OPTIONS,
    },
];
const HYPERLIQUID_CONSTRUCTORS: &[ProviderConstructor] = &[
    ProviderConstructor {
        name: "constructor",
        arguments: WALLET_CREDENTIALS,
        options: HYPERLIQUID_OPTIONS,
    },
    ProviderConstructor {
        name: "testnet",
        arguments: WALLET_CREDENTIALS,
        options: HYPERLIQUID_TESTNET_OPTIONS,
    },
];
const UPBIT_METHODS: &[ProviderMethod] = &[
    ProviderMethod {
        rust_name: "region",
        name: "region",
        kind: ProviderMethodKind::Property,
        arguments: &[],
        result: ApiType::Named("UpbitRegion"),
    },
    ProviderMethod {
        rust_name: "order_books",
        name: "orderBooks",
        kind: ProviderMethodKind::Async,
        arguments: MARKETS_DEPTH,
        result: ApiType::List("OrderBook"),
    },
    ProviderMethod {
        rust_name: "order_books_at_level",
        name: "orderBooksAtLevel",
        kind: ProviderMethodKind::Async,
        arguments: MARKETS_LEVEL_DEPTH,
        result: ApiType::List("OrderBook"),
    },
    ProviderMethod {
        rust_name: "tickers",
        name: "tickers",
        kind: ProviderMethodKind::Async,
        arguments: MARKETS,
        result: ApiType::List("Ticker"),
    },
    ProviderMethod {
        rust_name: "tickers_by_quote",
        name: "tickersByQuote",
        kind: ProviderMethodKind::Async,
        arguments: QUOTE_CURRENCIES,
        result: ApiType::List("Ticker"),
    },
    ProviderMethod {
        rust_name: "year_candles",
        name: "yearCandles",
        kind: ProviderMethodKind::Async,
        arguments: YEAR_CANDLE_QUERY,
        result: ApiType::List("UpbitYearCandle"),
    },
    ProviderMethod {
        rust_name: "orderbook_instruments",
        name: "orderbookInstruments",
        kind: ProviderMethodKind::Async,
        arguments: MARKETS,
        result: ApiType::List("UpbitOrderBookInstrument"),
    },
    ProviderMethod {
        rust_name: "market_events",
        name: "marketEvents",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::PairList("Market", "UpbitMarketEvent"),
    },
    ProviderMethod {
        rust_name: "list_subscriptions",
        name: "listSubscriptions",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_SUBSCRIPTION,
        result: ApiType::Named("UpbitSubscriptionList"),
    },
    ProviderMethod {
        rust_name: "test_order",
        name: "testOrder",
        kind: ProviderMethodKind::Async,
        arguments: ORDER_REQUEST,
        result: ApiType::Named("Order"),
    },
    ProviderMethod {
        rust_name: "order_detail",
        name: "orderDetail",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_ORDER_DETAIL_REQUEST,
        result: ApiType::Named("UpbitOrderDetail"),
    },
    ProviderMethod {
        rust_name: "closed_orders",
        name: "closedOrders",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_CLOSED_ORDERS_REQUEST,
        result: ApiType::List("UpbitClosedOrder"),
    },
    ProviderMethod {
        rust_name: "deposit_info",
        name: "depositInfo",
        kind: ProviderMethodKind::Async,
        arguments: ASSET_NETWORK,
        result: ApiType::Named("UpbitDepositInfo"),
    },
    ProviderMethod {
        rust_name: "withdrawal_addresses",
        name: "withdrawalAddresses",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("UpbitWithdrawalAddress"),
    },
    ProviderMethod {
        rust_name: "travel_rule_vasps",
        name: "travelRuleVasps",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("UpbitTravelRuleVasp"),
    },
    ProviderMethod {
        rust_name: "verify_travel_rule_by_uuid",
        name: "verifyTravelRuleByUuid",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_TRAVEL_RULE_UUID,
        result: ApiType::Named("UpbitTravelRuleVerification"),
    },
    ProviderMethod {
        rust_name: "verify_travel_rule_by_txid",
        name: "verifyTravelRuleByTxid",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_TRAVEL_RULE_TXID,
        result: ApiType::Named("UpbitTravelRuleVerification"),
    },
    ProviderMethod {
        rust_name: "batch_cancel_open_orders",
        name: "batchCancelOpenOrders",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_BATCH_CANCEL_REQUEST,
        result: ApiType::Named("CancelOrdersResult"),
    },
    ProviderMethod {
        rust_name: "cancel_and_new_order",
        name: "cancelAndNewOrder",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_CANCEL_AND_NEW_ORDER_REQUEST,
        result: ApiType::Named("UpbitCancelAndNewOrderResult"),
    },
    ProviderMethod {
        rust_name: "deposit_krw",
        name: "depositKrw",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_KRW_TRANSFER_REQUEST,
        result: ApiType::Named("UpbitKrwDeposit"),
    },
    ProviderMethod {
        rust_name: "withdraw_krw",
        name: "withdrawKrw",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_KRW_TRANSFER_REQUEST,
        result: ApiType::Named("UpbitKrwWithdrawal"),
    },
    ProviderMethod {
        rust_name: "api_keys",
        name: "apiKeys",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("UpbitApiKey"),
    },
    ProviderMethod {
        rust_name: "list_pockets",
        name: "listPockets",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("UpbitPocket"),
    },
    ProviderMethod {
        rust_name: "list_pocket_api_keys",
        name: "listPocketApiKeys",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_POCKET_API_KEYS_REQUEST,
        result: ApiType::List("UpbitPocketApiKeyGroup"),
    },
    ProviderMethod {
        rust_name: "sub_pocket_balances",
        name: "subPocketBalances",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_POCKET_UUID,
        result: ApiType::List("UpbitPocketBalance"),
    },
    ProviderMethod {
        rust_name: "universal_transfer",
        name: "universalTransfer",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_POCKET_UNIVERSAL_TRANSFER_REQUEST,
        result: ApiType::Named("UpbitPocketTransfer"),
    },
    ProviderMethod {
        rust_name: "universal_transfers",
        name: "universalTransfers",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_POCKET_TRANSFER_QUERY,
        result: ApiType::List("UpbitPocketTransfer"),
    },
    ProviderMethod {
        rust_name: "sub_pocket_transfer",
        name: "subPocketTransfer",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_POCKET_TRANSFER_REQUEST,
        result: ApiType::Named("UpbitPocketTransfer"),
    },
    ProviderMethod {
        rust_name: "sub_pocket_transfers",
        name: "subPocketTransfers",
        kind: ProviderMethodKind::Async,
        arguments: UPBIT_POCKET_TRANSFER_QUERY,
        result: ApiType::List("UpbitPocketTransfer"),
    },
];
const BITHUMB_METHODS: &[ProviderMethod] = &[
    ProviderMethod {
        rust_name: "market_warnings",
        name: "marketWarnings",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::PairList("Market", "String"),
    },
    ProviderMethod {
        rust_name: "market_alerts",
        name: "marketAlerts",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::PairList("Market", "BithumbMarketAlert"),
    },
    ProviderMethod {
        rust_name: "notices",
        name: "notices",
        kind: ProviderMethodKind::Async,
        arguments: NOTICE_COUNT,
        result: ApiType::List("BithumbNotice"),
    },
    ProviderMethod {
        rust_name: "transfer_fees",
        name: "transferFees",
        kind: ProviderMethodKind::Async,
        arguments: FEE_CURRENCY,
        result: ApiType::List("BithumbAssetFee"),
    },
    ProviderMethod {
        rust_name: "api_keys",
        name: "apiKeys",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("BithumbApiKey"),
    },
    ProviderMethod {
        rust_name: "krw_withdrawals",
        name: "krwWithdrawals",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_KRW_WITHDRAWALS_REQUEST,
        result: ApiType::List("BithumbKrwWithdrawal"),
    },
    ProviderMethod {
        rust_name: "withdraw_krw",
        name: "withdrawKrw",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_KRW_TRANSFER_REQUEST,
        result: ApiType::Named("BithumbKrwWithdrawal"),
    },
    ProviderMethod {
        rust_name: "krw_deposits",
        name: "krwDeposits",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_KRW_DEPOSITS_REQUEST,
        result: ApiType::List("BithumbKrwDeposit"),
    },
    ProviderMethod {
        rust_name: "deposit_krw",
        name: "depositKrw",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_KRW_TRANSFER_REQUEST,
        result: ApiType::Named("BithumbKrwDeposit"),
    },
    ProviderMethod {
        rust_name: "pending_orders",
        name: "pendingOrders",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_PENDING_ORDERS_REQUEST,
        result: ApiType::Page("Order"),
    },
    ProviderMethod {
        rust_name: "closed_orders",
        name: "closedOrders",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_CLOSED_ORDERS_REQUEST,
        result: ApiType::Page("BithumbClosedOrder"),
    },
    ProviderMethod {
        rust_name: "batch_orders",
        name: "batchOrders",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_BATCH_ORDERS_REQUEST,
        result: ApiType::Named("BithumbBatchOrdersResult"),
    },
    ProviderMethod {
        rust_name: "twap_orders",
        name: "twapOrders",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_TWAP_ORDERS_REQUEST,
        result: ApiType::Page("BithumbTwapOrder"),
    },
    ProviderMethod {
        rust_name: "create_twap_order",
        name: "createTwapOrder",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_TWAP_ORDER_REQUEST,
        result: ApiType::String,
    },
    ProviderMethod {
        rust_name: "cancel_twap_order",
        name: "cancelTwapOrder",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_TWAP_ORDER_ID,
        result: ApiType::String,
    },
    ProviderMethod {
        rust_name: "withdrawal_addresses",
        name: "withdrawalAddresses",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("BithumbWithdrawalAddress"),
    },
    ProviderMethod {
        rust_name: "order_detail",
        name: "orderDetail",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_ORDER_DETAIL_REQUEST,
        result: ApiType::Named("BithumbOrderDetail"),
    },
    ProviderMethod {
        rust_name: "order_list",
        name: "orderList",
        kind: ProviderMethodKind::Async,
        arguments: BITHUMB_ORDER_LIST_REQUEST,
        result: ApiType::List("BithumbOrderListItem"),
    },
];
const BINANCE_METHODS: &[ProviderMethod] = &[
    ProviderMethod {
        rust_name: "venue",
        name: "venue",
        kind: ProviderMethodKind::Property,
        arguments: &[],
        result: ApiType::Named("BinanceMarket"),
    },
    ProviderMethod {
        rust_name: "spot_symbol_filters",
        name: "spotSymbolFilters",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("BinanceSymbolFilters"),
    },
    ProviderMethod {
        rust_name: "spot_order",
        name: "spotOrder",
        kind: ProviderMethodKind::Async,
        arguments: CANCEL_ORDER,
        result: ApiType::Named("BinanceSpotOrderDetail"),
    },
    ProviderMethod {
        rust_name: "spot_average_price",
        name: "spotAveragePrice",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("BinanceSpotAveragePrice"),
    },
    ProviderMethod {
        rust_name: "spot_account_information",
        name: "spotAccountInformation",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("BinanceSpotAccountInformation"),
    },
    ProviderMethod {
        rust_name: "spot_cancel_all_open_orders",
        name: "spotCancelAllOpenOrders",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("BinanceSpotCancelAllOpenOrders"),
    },
    ProviderMethod {
        rust_name: "spot_exchange_info",
        name: "spotExchangeInfo",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("BinanceExchangeInfo"),
    },
    ProviderMethod {
        rust_name: "usd_m_account_information",
        name: "usdMAccountInformation",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("BinanceUsdMAccountInformation"),
    },
    ProviderMethod {
        rust_name: "usd_m_exchange_info",
        name: "usdMExchangeInfo",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("BinanceExchangeInfo"),
    },
    ProviderMethod {
        rust_name: "usd_m_position_information",
        name: "usdMPositionInformation",
        kind: ProviderMethodKind::Async,
        arguments: OPTIONAL_MARKET,
        result: ApiType::List("BinanceUsdMPositionInformation"),
    },
    ProviderMethod {
        rust_name: "all_coins_information",
        name: "allCoinsInformation",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("BinanceCoinInformation"),
    },
    ProviderMethod {
        rust_name: "api_key_permissions",
        name: "apiKeyPermissions",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("BinanceApiKeyPermissions"),
    },
    ProviderMethod {
        rust_name: "deposit_history",
        name: "depositHistory",
        kind: ProviderMethodKind::Async,
        arguments: BINANCE_DEPOSIT_HISTORY_REQUEST,
        result: ApiType::Named("BinanceDepositHistory"),
    },
    ProviderMethod {
        rust_name: "questionnaire_requirements",
        name: "questionnaireRequirements",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("BinanceQuestionnaireRequirements"),
    },
    ProviderMethod {
        rust_name: "withdraw_address_list",
        name: "withdrawAddressList",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("BinanceWithdrawalAddress"),
    },
    ProviderMethod {
        rust_name: "withdraw_history",
        name: "withdrawHistory",
        kind: ProviderMethodKind::Async,
        arguments: BINANCE_WITHDRAW_HISTORY_REQUEST,
        result: ApiType::Named("BinanceWithdrawHistory"),
    },
    ProviderMethod {
        rust_name: "mark_price",
        name: "markPrice",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("BinanceMarkPrice"),
    },
    ProviderMethod {
        rust_name: "mark_prices",
        name: "markPrices",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("BinanceMarkPrice"),
    },
    ProviderMethod {
        rust_name: "open_interest",
        name: "openInterest",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("BinanceOpenInterest"),
    },
    ProviderMethod {
        rust_name: "aggregate_trades",
        name: "aggregateTrades",
        kind: ProviderMethodKind::Async,
        arguments: BINANCE_AGGREGATE_TRADES_REQUEST,
        result: ApiType::List("BinanceAggregateTrade"),
    },
    ProviderMethod {
        rust_name: "account_trades",
        name: "accountTrades",
        kind: ProviderMethodKind::Async,
        arguments: HISTORY_REQUEST,
        result: ApiType::Page("BinanceAccountTrade"),
    },
    ProviderMethod {
        rust_name: "c2c_trade_history",
        name: "c2cTradeHistory",
        kind: ProviderMethodKind::Async,
        arguments: BINANCE_C2C_TRADE_HISTORY_REQUEST,
        result: ApiType::Named("BinanceC2cTradeHistoryPage"),
    },
    ProviderMethod {
        rust_name: "test_order",
        name: "testOrder",
        kind: ProviderMethodKind::Async,
        arguments: BINANCE_TEST_ORDER_REQUEST,
        result: ApiType::Named("BinanceTestOrder"),
    },
    ProviderMethod {
        rust_name: "cancel_all_open_orders",
        name: "cancelAllOpenOrders",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Unit,
    },
    ProviderMethod {
        rust_name: "usd_m_create_listen_key",
        name: "usdMCreateListenKey",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Handle("BinanceListenKey"),
    },
    ProviderMethod {
        rust_name: "usd_m_keepalive_listen_key",
        name: "usdMKeepaliveListenKey",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Unit,
    },
    ProviderMethod {
        rust_name: "usd_m_close_listen_key",
        name: "usdMCloseListenKey",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Unit,
    },
];
const HYPERLIQUID_METHODS: &[ProviderMethod] = &[
    ProviderMethod {
        rust_name: "is_testnet",
        name: "isTestnet",
        kind: ProviderMethodKind::Property,
        arguments: &[],
        result: ApiType::Boolean,
    },
    ProviderMethod {
        rust_name: "all_mids",
        name: "allMids",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("HyperliquidMidPrice"),
    },
    ProviderMethod {
        rust_name: "subscribe_detailed",
        name: "subscribeDetailed",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_DETAILED_SUBSCRIPTION,
        result: ApiType::ProviderMarketStream("HyperliquidMarketEvent"),
    },
    ProviderMethod {
        rust_name: "subscribe_detailed_with",
        name: "subscribeDetailedWith",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_DETAILED_SUBSCRIPTION_CONFIG,
        result: ApiType::ProviderMarketStream("HyperliquidMarketEvent"),
    },
    ProviderMethod {
        rust_name: "subscribe_detailed_account",
        name: "subscribeDetailedAccount",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::ProviderAccountStream("HyperliquidAccountEvent"),
    },
    ProviderMethod {
        rust_name: "subscribe_detailed_account_with",
        name: "subscribeDetailedAccountWith",
        kind: ProviderMethodKind::Async,
        arguments: CONFIG,
        result: ApiType::ProviderAccountStream("HyperliquidAccountEvent"),
    },
    ProviderMethod {
        rust_name: "user_fills",
        name: "userFills",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_USER_FILLS,
        result: ApiType::List("HyperliquidUserFill"),
    },
    ProviderMethod {
        rust_name: "user_fills_by_time",
        name: "userFillsByTime",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_USER_FILLS_BY_TIME,
        result: ApiType::List("HyperliquidUserFill"),
    },
    ProviderMethod {
        rust_name: "basic_open_orders",
        name: "basicOpenOrders",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("HyperliquidOpenOrder"),
    },
    ProviderMethod {
        rust_name: "order_status",
        name: "orderStatus",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_ORDER_REFERENCE,
        result: ApiType::Named("HyperliquidOrderStatusResponse"),
    },
    ProviderMethod {
        rust_name: "historical_orders",
        name: "historicalOrders",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("HyperliquidOrderInfo"),
    },
    ProviderMethod {
        rust_name: "non_funding_ledger",
        name: "nonFundingLedger",
        kind: ProviderMethodKind::Async,
        arguments: LEDGER_RANGE,
        result: ApiType::Page("HyperliquidLedgerEntry"),
    },
    ProviderMethod {
        rust_name: "asset_context",
        name: "assetContext",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("HyperliquidAssetContext"),
    },
    ProviderMethod {
        rust_name: "candle_snapshot",
        name: "candleSnapshot",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_CANDLE_SNAPSHOT,
        result: ApiType::List("HyperliquidCandleSnapshot"),
    },
    ProviderMethod {
        rust_name: "l2_book",
        name: "l2Book",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::Named("HyperliquidL2Book"),
    },
    ProviderMethod {
        rust_name: "recent_trades",
        name: "recentTrades",
        kind: ProviderMethodKind::Async,
        arguments: MARKET,
        result: ApiType::List("HyperliquidRecentTrade"),
    },
    ProviderMethod {
        rust_name: "funding_history",
        name: "fundingHistory",
        kind: ProviderMethodKind::Async,
        arguments: &[
            argument("market", ApiType::Named("Market"), None),
            argument("from", ApiType::Named("Timestamp"), None),
            argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
        ],
        result: ApiType::List("HyperliquidFundingHistoryEntry"),
    },
    ProviderMethod {
        rust_name: "user_funding",
        name: "userFunding",
        kind: ProviderMethodKind::Async,
        arguments: HYPERLIQUID_TIME_RANGE,
        result: ApiType::List("HyperliquidUserFunding"),
    },
    ProviderMethod {
        rust_name: "spot_clearinghouse_state",
        name: "spotClearinghouseState",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidSpotClearinghouseState"),
    },
    ProviderMethod {
        rust_name: "spot_meta",
        name: "spotMeta",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidSpotMeta"),
    },
    ProviderMethod {
        rust_name: "spot_meta_and_asset_contexts",
        name: "spotMetaAndAssetContexts",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidSpotMetaAndAssetContexts"),
    },
    ProviderMethod {
        rust_name: "user_rate_limit",
        name: "userRateLimit",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidUserRateLimit"),
    },
    ProviderMethod {
        rust_name: "user_role",
        name: "userRole",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidUserRole"),
    },
    ProviderMethod {
        rust_name: "referral",
        name: "referral",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidReferral"),
    },
    ProviderMethod {
        rust_name: "user_fees",
        name: "userFees",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::Named("HyperliquidUserFees"),
    },
    ProviderMethod {
        rust_name: "portfolio",
        name: "portfolio",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("HyperliquidPortfolioPeriod"),
    },
    ProviderMethod {
        rust_name: "sub_accounts",
        name: "subAccounts",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("HyperliquidSubAccount"),
    },
    ProviderMethod {
        rust_name: "user_vault_equities",
        name: "userVaultEquities",
        kind: ProviderMethodKind::Async,
        arguments: &[],
        result: ApiType::List("HyperliquidVaultEquity"),
    },
];

const PROVIDERS: &[Provider] = &[
    Provider {
        exchange: "upbit",
        adapter: "UpbitAdapter",
        native_handle: "NativeUpbitHandle",
        native_factory: "createUpbit",
        options_wire: "UpbitOptionsWire",
        constructors: UPBIT_CONSTRUCTORS,
        methods: UPBIT_METHODS,
    },
    Provider {
        exchange: "bithumb",
        adapter: "BithumbAdapter",
        native_handle: "NativeBithumbHandle",
        native_factory: "createBithumb",
        options_wire: "BithumbOptionsWire",
        constructors: BITHUMB_CONSTRUCTORS,
        methods: BITHUMB_METHODS,
    },
    Provider {
        exchange: "binance",
        adapter: "BinanceAdapter",
        native_handle: "NativeBinanceHandle",
        native_factory: "createBinance",
        options_wire: "BinanceOptionsWire",
        constructors: BINANCE_CONSTRUCTORS,
        methods: BINANCE_METHODS,
    },
    Provider {
        exchange: "hyperliquid",
        adapter: "HyperliquidAdapter",
        native_handle: "NativeHyperliquidHandle",
        native_factory: "createHyperliquid",
        options_wire: "HyperliquidOptionsWire",
        constructors: HYPERLIQUID_CONSTRUCTORS,
        methods: HYPERLIQUID_METHODS,
    },
];

fn field(name: &'static str, ty: Type) -> Field {
    Field { name, ty }
}

fn record(name: &'static str, fields: Vec<Field>) -> Record {
    Record {
        name,
        type_parameters: &[],
        fields,
    }
}

fn generic_record(
    name: &'static str,
    type_parameters: &'static [&'static str],
    fields: Vec<Field>,
) -> Record {
    Record {
        name,
        type_parameters,
        fields,
    }
}

fn variant(name: &'static str, fields: Vec<Field>) -> Variant {
    Variant { name, fields }
}

pub fn binding_schema() -> Schema {
    use Type::{Boolean, Number};

    let market = Type::named("MarketWire");
    let decimal = Type::Decimal;
    let timestamp = Type::Timestamp;
    let records = vec![
        record(
            "MarketWire",
            vec![
                field("exchange", Type::Identifier("Exchange")),
                field("kind", Type::Identifier("MarketKind")),
                field("base", Type::String),
                field("quote", Type::String),
            ],
        ),
        record(
            "MarketInfoWire",
            vec![
                field("market", market.clone()),
                field("native_symbol", Type::String),
                field("status", Type::Identifier("MarketStatus")),
                field("korean_name", Type::optional(Type::String)),
                field("english_name", Type::optional(Type::String)),
            ],
        ),
        record(
            "TradeWire",
            vec![
                field("market", market.clone()),
                field("timestamp", timestamp.clone()),
                field("price", decimal.clone()),
                field("quantity", decimal.clone()),
                field("taker_side", Type::Identifier("Side")),
                field("id", Type::optional(Type::String)),
            ],
        ),
        record(
            "LevelWire",
            vec![
                field("price", decimal.clone()),
                field("quantity", decimal.clone()),
            ],
        ),
        record(
            "OrderBookWire",
            vec![
                field("market", market.clone()),
                field("timestamp", timestamp.clone()),
                field("bids", Type::list(Type::named("LevelWire"))),
                field("asks", Type::list(Type::named("LevelWire"))),
            ],
        ),
        record(
            "TickerWire",
            vec![
                field("market", market.clone()),
                field("timestamp", timestamp.clone()),
                field("last_trade_time", Type::optional(timestamp.clone())),
                field("last_price", decimal.clone()),
                field("change", Type::optional(decimal.clone())),
                field("change_rate", Type::optional(decimal.clone())),
                field("high", Type::optional(decimal.clone())),
                field("low", Type::optional(decimal.clone())),
                field("volume", Type::optional(decimal.clone())),
                field("quote_volume", Type::optional(decimal.clone())),
            ],
        ),
        record(
            "CandleWire",
            vec![
                field("market", market.clone()),
                field("interval", Type::Identifier("Interval")),
                field("open_time", timestamp.clone()),
                field("open", decimal.clone()),
                field("high", decimal.clone()),
                field("low", decimal.clone()),
                field("close", decimal.clone()),
                field("volume", decimal.clone()),
                field("quote_volume", Type::optional(decimal.clone())),
                field("closed", Boolean),
            ],
        ),
        record(
            "BalanceWire",
            vec![
                field("asset", Type::String),
                field("available", decimal.clone()),
                field("locked", decimal.clone()),
            ],
        ),
        record(
            "OrderAccountWire",
            vec![
                field("balance", Type::named("BalanceWire")),
                field("average_buy_price", decimal.clone()),
                field("average_buy_price_modified", Boolean),
                field("average_buy_price_unit", Type::optional(Type::String)),
            ],
        ),
        record(
            "OrderOptionWire",
            vec![
                field("provider_id", Type::String),
                field("order_type", Type::optional(Type::Identifier("OrderType"))),
                field(
                    "time_in_force",
                    Type::optional(Type::Identifier("TimeInForce")),
                ),
            ],
        ),
        record(
            "OrderRulesWire",
            vec![
                field("market", market.clone()),
                field("market_name", Type::String),
                field("status", Type::Identifier("MarketStatus")),
                field("buy_fee_rate", decimal.clone()),
                field("sell_fee_rate", decimal.clone()),
                field("maker_buy_fee_rate", decimal.clone()),
                field("maker_sell_fee_rate", decimal.clone()),
                field("sides", Type::list(Type::Identifier("Side"))),
                field("buy_options", Type::list(Type::named("OrderOptionWire"))),
                field("sell_options", Type::list(Type::named("OrderOptionWire"))),
                field("buy_price_unit", Type::optional(decimal.clone())),
                field("sell_price_unit", Type::optional(decimal.clone())),
                field("minimum_buy_total", decimal.clone()),
                field("minimum_sell_total", decimal.clone()),
                field("maximum_total", decimal.clone()),
                field("quote_account", Type::named("OrderAccountWire")),
                field("base_account", Type::named("OrderAccountWire")),
            ],
        ),
        record(
            "AssetNetworkWire",
            vec![
                field("exchange", Type::Identifier("Exchange")),
                field("asset", Type::String),
                field("network", Type::Identifier("Network")),
                field("provider_id", Type::String),
                field("deposit_enabled", Boolean),
                field("withdrawal_enabled", Boolean),
                field(
                    "withdrawal_fee",
                    Type::optional(Type::named("WithdrawalFeeWire")),
                ),
                field("minimum_withdrawal", Type::optional(decimal.clone())),
                field("maximum_withdrawal", Type::optional(decimal.clone())),
                field("memo_required", Boolean),
            ],
        ),
        record(
            "DepositAddressWire",
            vec![
                field("exchange", Type::Identifier("Exchange")),
                field("asset", Type::String),
                field("network", Type::Identifier("Network")),
                field("address", Type::optional(Type::String)),
                field("memo", Type::optional(Type::String)),
            ],
        ),
        record(
            "DepositAddressEntryWire",
            vec![
                field("exchange", Type::Identifier("Exchange")),
                field("asset", Type::String),
                field("network", Type::optional(Type::Identifier("Network"))),
                field("provider_network", Type::optional(Type::String)),
                field("address", Type::optional(Type::String)),
                field("memo", Type::optional(Type::String)),
            ],
        ),
        record(
            "ExchangeDestinationWire",
            vec![
                field("exchange", Type::Identifier("Exchange")),
                field("asset", Type::String),
                field("network", Type::Identifier("Network")),
                field("address", Type::String),
                field("memo", Type::optional(Type::String)),
            ],
        ),
        record(
            "ChainDestinationWire",
            vec![
                field("asset", Type::String),
                field("network", Type::Identifier("Network")),
                field("address", Type::String),
                field("memo", Type::optional(Type::String)),
            ],
        ),
        record(
            "ExchangeTransferRequestWire",
            vec![
                field("asset", Type::String),
                field(
                    "source_network",
                    Type::optional(Type::Identifier("Network")),
                ),
                field(
                    "destination_network",
                    Type::optional(Type::Identifier("Network")),
                ),
                field("amount", decimal.clone()),
            ],
        ),
        record(
            "ChainTransferRequestWire",
            vec![
                field("asset", Type::String),
                field(
                    "source_network",
                    Type::optional(Type::Identifier("Network")),
                ),
                field("destination", Type::named("ChainDestinationWire")),
                field("amount", decimal.clone()),
            ],
        ),
        record(
            "WithdrawalQuoteWire",
            vec![
                field("fee", Type::optional(decimal.clone())),
                field("expected_receive", Type::optional(decimal.clone())),
                field("minimum_amount", Type::optional(decimal.clone())),
                field("maximum_amount", Type::optional(decimal.clone())),
                field("address_allowed", Type::optional(Boolean)),
                field("travel_rule", Type::named("TravelRuleRequirementWire")),
                field("expires_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "TransferPlanWire",
            vec![
                field("source", Type::Identifier("Exchange")),
                field("destination", Type::optional(Type::Identifier("Exchange"))),
                field("request", Type::named("WithdrawRequestWire")),
                field("quote", Type::named("WithdrawalQuoteWire")),
                field("created_at", timestamp.clone()),
                field("expires_at", timestamp.clone()),
            ],
        ),
        record(
            "WithdrawalWire",
            vec![
                field("id", Type::String),
                field("asset", Type::String),
                field("network", Type::optional(Type::Identifier("Network"))),
                field("provider_network", Type::optional(Type::String)),
                field("amount", decimal.clone()),
                field("fee", Type::optional(decimal.clone())),
                field(
                    "destination",
                    Type::optional(Type::named("TransferDestinationWire")),
                ),
                field("status", Type::Identifier("WithdrawalStatus")),
                field("provider_status", Type::String),
                field("tx_id", Type::optional(Type::String)),
                field("created_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "DepositWire",
            vec![
                field("id", Type::String),
                field("asset", Type::String),
                field("network", Type::optional(Type::Identifier("Network"))),
                field("provider_network", Type::optional(Type::String)),
                field("amount", decimal.clone()),
                field("address", Type::optional(Type::String)),
                field("memo", Type::optional(Type::String)),
                field("status", Type::Identifier("DepositStatus")),
                field("provider_status", Type::String),
                field("tx_id", Type::optional(Type::String)),
                field("created_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "OrderWire",
            vec![
                field("id", Type::String),
                field("market", market.clone()),
                field("side", Type::Identifier("Side")),
                field("status", Type::Identifier("OrderStatus")),
                field("filled_quantity", decimal.clone()),
                field("remaining_quantity", decimal.clone()),
                field("price", Type::optional(decimal.clone())),
                field("created_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "CancelledOrderWire",
            vec![
                field("order_id", Type::String),
                field("client_id", Type::optional(Type::String)),
                field("market", Type::optional(market.clone())),
                field("cancelled_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "OrderCancelFailureWire",
            vec![
                field("order_id", Type::optional(Type::String)),
                field("client_id", Type::optional(Type::String)),
                field("market", Type::optional(market.clone())),
                field("code", Type::optional(Type::String)),
                field("message", Type::optional(Type::String)),
            ],
        ),
        record(
            "CancelOrdersResultWire",
            vec![
                field("cancelled", Type::list(Type::named("CancelledOrderWire"))),
                field("failed", Type::list(Type::named("OrderCancelFailureWire"))),
            ],
        ),
        record(
            "PositionWire",
            vec![
                field("market", market.clone()),
                field("side", Type::optional(Type::Identifier("Side"))),
                field("quantity", decimal.clone()),
                field("entry_price", Type::optional(decimal.clone())),
                field("mark_price", Type::optional(decimal.clone())),
                field("notional", Type::optional(decimal.clone())),
                field("unrealized_pnl", Type::optional(decimal.clone())),
                field("leverage", Type::optional(decimal.clone())),
                field(
                    "margin_mode",
                    Type::optional(Type::Identifier("MarginMode")),
                ),
            ],
        ),
        record(
            "MarginSummaryWire",
            vec![
                field("asset", Type::String),
                field("equity", Type::optional(decimal.clone())),
                field("margin_balance", Type::optional(decimal.clone())),
                field("available_balance", Type::optional(decimal.clone())),
            ],
        ),
        record(
            "FundingRateWire",
            vec![
                field("market", market.clone()),
                field("timestamp", timestamp.clone()),
                field("rate", decimal.clone()),
                field("mark_price", Type::optional(decimal.clone())),
            ],
        ),
        record(
            "FundingPaymentWire",
            vec![
                field("market", market.clone()),
                field("timestamp", timestamp.clone()),
                field("amount", decimal.clone()),
                field("rate", Type::optional(decimal.clone())),
                field("id", Type::optional(Type::String)),
            ],
        ),
        record(
            "CandleRequestWire",
            vec![
                field("market", market.clone()),
                field("interval", Type::Identifier("Interval")),
                field("from", Type::optional(timestamp.clone())),
                field("to", Type::optional(timestamp.clone())),
                field("limit", Type::optional(Number)),
            ],
        ),
        record(
            "OrderRequestWire",
            vec![
                field("market", market.clone()),
                field("side", Type::Identifier("Side")),
                field("order_type", Type::Identifier("OrderType")),
                field("size", Type::named("SizeWire")),
                field("price", Type::optional(decimal.clone())),
                field(
                    "time_in_force",
                    Type::optional(Type::Identifier("TimeInForce")),
                ),
                field("reduce_only", Boolean),
                field("client_id", Type::optional(Type::String)),
            ],
        ),
        record(
            "OrderHistoryRequestWire",
            vec![
                field("market", Type::optional(market.clone())),
                field("statuses", Type::list(Type::Identifier("OrderStatus"))),
                field("from", Type::optional(timestamp.clone())),
                field("to", Type::optional(timestamp.clone())),
                field("cursor", Type::optional(Type::String)),
                field("limit", Type::optional(Number)),
            ],
        ),
        record(
            "OrderLookupRequestWire",
            vec![
                field("kind", Type::Identifier("OrderIdKind")),
                field("ids", Type::list(Type::String)),
                field("market", Type::optional(market.clone())),
            ],
        ),
        record(
            "CancelOrdersRequestWire",
            vec![
                field("kind", Type::Identifier("OrderIdKind")),
                field("ids", Type::list(Type::String)),
            ],
        ),
        record(
            "DepositAddressRequestWire",
            vec![
                field("asset", Type::String),
                field("network", Type::Identifier("Network")),
                field("amount", Type::optional(decimal.clone())),
            ],
        ),
        record(
            "WithdrawRequestWire",
            vec![
                field("asset", Type::String),
                field("network", Type::Identifier("Network")),
                field("amount", decimal.clone()),
                field("destination", Type::named("TransferDestinationWire")),
                field("client_id", Type::optional(Type::String)),
            ],
        ),
        record(
            "TransferLookupRequestWire",
            vec![
                field("asset", Type::String),
                field("id", Type::optional(Type::String)),
                field("tx_id", Type::optional(Type::String)),
            ],
        ),
        record(
            "TransferHistoryRequestWire",
            vec![
                field("asset", Type::optional(Type::String)),
                field("network", Type::optional(Type::Identifier("Network"))),
                field("cursor", Type::optional(Type::String)),
                field("limit", Type::optional(Number)),
            ],
        ),
        record(
            "StreamConfigWire",
            vec![
                field("max_reconnect_attempts", Type::optional(Number)),
                field("initial_reconnect_delay_ms", Type::UnsignedInteger),
                field("max_reconnect_delay_ms", Type::UnsignedInteger),
                field("idle_timeout_ms", Type::UnsignedInteger),
                field("buffer_size", Type::UnsignedInteger),
                field("overflow", Type::Identifier("Overflow")),
            ],
        ),
        record(
            "SubscriptionWire",
            vec![
                field("markets", Type::list(market.clone())),
                field("feeds", Type::list(Type::named("FeedWire"))),
            ],
        ),
        record(
            "HistoryRequestWire",
            vec![
                field("market", market.clone()),
                field("from", Type::optional(timestamp.clone())),
                field("to", Type::optional(timestamp.clone())),
                field("cursor", Type::optional(Type::String)),
                field("limit", Type::optional(Number)),
            ],
        ),
        record(
            "MarginRequestWire",
            vec![
                field("market", market.clone()),
                field("leverage", Type::optional(decimal.clone())),
                field(
                    "margin_mode",
                    Type::optional(Type::Identifier("MarginMode")),
                ),
            ],
        ),
        generic_record(
            "PageWire",
            &["T"],
            vec![
                field("items", Type::list(Type::named("T"))),
                field("next", Type::optional(Type::String)),
            ],
        ),
        record(
            "UpbitMarketEventWire",
            vec![
                field("warning", Boolean),
                field("cautions", Type::list(Type::String)),
            ],
        ),
        record(
            "UpbitListedSubscriptionWire",
            vec![
                field("feed_type", Type::String),
                field("markets", Type::list(market.clone())),
                field("level", Type::optional(decimal.clone())),
            ],
        ),
        record(
            "UpbitSubscriptionListWire",
            vec![
                field("ticket", Type::String),
                field(
                    "subscriptions",
                    Type::list(Type::named("UpbitListedSubscriptionWire")),
                ),
            ],
        ),
        record(
            "UpbitYearCandleWire",
            vec![
                field("market", market.clone()),
                field("open_time", timestamp.clone()),
                field("korea_open_time", Type::optional(timestamp.clone())),
                field("timestamp", timestamp.clone()),
                field("open", decimal.clone()),
                field("high", decimal.clone()),
                field("low", decimal.clone()),
                field("close", decimal.clone()),
                field("volume", decimal.clone()),
                field("quote_volume", decimal.clone()),
                field("first_day_of_period", Type::String),
            ],
        ),
        record(
            "UpbitOrderBookInstrumentWire",
            vec![
                field("market", market.clone()),
                field("quote_currency", Type::String),
                field("tick_size", decimal.clone()),
                field("supported_levels", Type::list(decimal.clone())),
            ],
        ),
        record(
            "UpbitOrderDetailRequestWire",
            vec![
                field("market", market.clone()),
                field("uuid", Type::optional(Type::String)),
                field("identifier", Type::optional(Type::String)),
            ],
        ),
        record(
            "UpbitOrderDetailTradeWire",
            vec![
                field("market", market.clone()),
                field("uuid", Type::String),
                field("price", decimal.clone()),
                field("volume", decimal.clone()),
                field("funds", decimal.clone()),
                field("trend", Type::String),
                field("created_at", timestamp.clone()),
                field("side", Type::String),
            ],
        ),
        record(
            "UpbitOrderDetailWire",
            vec![
                field("market", market.clone()),
                field("uuid", Type::String),
                field("side", Type::String),
                field("order_type", Type::String),
                field("price", Type::optional(decimal.clone())),
                field("state", Type::String),
                field("created_at", timestamp.clone()),
                field("volume", Type::optional(decimal.clone())),
                field("remaining_volume", decimal.clone()),
                field("executed_volume", decimal.clone()),
                field("reserved_fee", decimal.clone()),
                field("remaining_fee", decimal.clone()),
                field("paid_fee", decimal.clone()),
                field("locked", decimal.clone()),
                field("trades_count", Number),
                field("prevented_volume", decimal.clone()),
                field("prevented_locked", decimal.clone()),
                field("time_in_force", Type::optional(Type::String)),
                field("identifier", Type::optional(Type::String)),
                field("smp_type", Type::optional(Type::String)),
                field(
                    "trades",
                    Type::list(Type::named("UpbitOrderDetailTradeWire")),
                ),
            ],
        ),
        record(
            "UpbitClosedOrdersRequestWire",
            vec![
                field("market", Type::optional(market.clone())),
                field(
                    "state",
                    Type::optional(Type::Identifier("UpbitClosedOrderState")),
                ),
                field(
                    "states",
                    Type::list(Type::Identifier("UpbitClosedOrderState")),
                ),
                field("start_time", Type::optional(timestamp.clone())),
                field("end_time", Type::optional(timestamp.clone())),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("UpbitOrderDirection")),
                ),
            ],
        ),
        record(
            "UpbitClosedOrderWire",
            vec![
                field("market", market.clone()),
                field("uuid", Type::String),
                field("side", Type::String),
                field("ord_type", Type::String),
                field("state", Type::String),
                field("created_at", timestamp.clone()),
                field("volume", Type::optional(decimal.clone())),
                field("price", Type::optional(decimal.clone())),
                field("remaining_volume", decimal.clone()),
                field("executed_volume", decimal.clone()),
                field("executed_funds", Type::optional(decimal.clone())),
                field("reserved_fee", decimal.clone()),
                field("remaining_fee", decimal.clone()),
                field("paid_fee", decimal.clone()),
                field("locked", decimal.clone()),
                field("trades_count", Number),
                field("prevented_volume", decimal.clone()),
                field("prevented_locked", decimal.clone()),
                field("time_in_force", Type::optional(Type::String)),
                field("identifier", Type::optional(Type::String)),
                field("smp_type", Type::optional(Type::String)),
            ],
        ),
        record(
            "UpbitDepositInfoWire",
            vec![
                field("asset", Type::String),
                field("network", Type::optional(Type::Identifier("Network"))),
                field("provider_network", Type::optional(Type::String)),
                field("is_deposit_possible", Boolean),
                field("deposit_impossible_reason", Type::optional(Type::String)),
                field("minimum_deposit_amount", decimal.clone()),
                field("minimum_deposit_confirmations", Type::UnsignedInteger),
                field("decimal_precision", Type::UnsignedInteger),
            ],
        ),
        record(
            "UpbitWithdrawalAddressWire",
            vec![
                field("currency", Type::String),
                field("net_type", Type::String),
                field("network_name", Type::String),
                field("withdraw_address", Type::String),
                field("secondary_address", Type::optional(Type::String)),
                field("beneficiary_name", Type::optional(Type::String)),
                field("beneficiary_company_name", Type::optional(Type::String)),
                field("beneficiary_type", Type::optional(Type::String)),
                field("exchange_name", Type::optional(Type::String)),
                field("wallet_type", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "UpbitTravelRuleVaspWire",
            vec![
                field("vasp_name", Type::String),
                field("vasp_uuid", Type::String),
                field("depositable", Boolean),
                field("withdrawable", Boolean),
            ],
        ),
        record(
            "UpbitTravelRuleVerificationWire",
            vec![
                field("deposit_uuid", Type::String),
                field("deposit_state", Type::String),
                field("verification_result", Type::String),
            ],
        ),
        record(
            "UpbitBatchCancelRequestWire",
            vec![
                field("scope", Type::named("UpbitBatchCancelScopeWire")),
                field("excluded_pairs", Type::optional(Type::list(market.clone()))),
                field("side", Type::optional(Type::Identifier("Side"))),
                field("count", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("UpbitOrderDirection")),
                ),
            ],
        ),
        record(
            "UpbitCancelAndNewOrderRequestWire",
            vec![
                field("previous_order", Type::named("UpbitOrderReferenceWire")),
                field("new_order", Type::named("UpbitCancelAndNewOrderWire")),
                field("new_identifier", Type::optional(Type::String)),
                field(
                    "new_smp_type",
                    Type::optional(Type::Identifier("UpbitSmpType")),
                ),
            ],
        ),
        record(
            "UpbitCancelAndNewOrderResultWire",
            vec![
                field("previous_order", Type::named("OrderWire")),
                field("new_order_uuid", Type::optional(Type::String)),
                field("new_order_identifier", Type::optional(Type::String)),
            ],
        ),
        record(
            "UpbitKrwTransferRequestWire",
            vec![
                field("amount", decimal.clone()),
                field("two_factor_type", Type::Identifier("UpbitKrwTwoFactorType")),
            ],
        ),
        record(
            "UpbitKrwDepositWire",
            vec![
                field("transfer_type", Type::String),
                field("uuid", Type::String),
                field("currency", Type::String),
                field("net_type", Type::optional(Type::String)),
                field("txid", Type::String),
                field("state", Type::String),
                field("created_at", timestamp.clone()),
                field("done_at", Type::optional(timestamp.clone())),
                field("amount", decimal.clone()),
                field("fee", decimal.clone()),
                field("transaction_type", Type::String),
            ],
        ),
        record(
            "UpbitKrwWithdrawalWire",
            vec![
                field("transfer_type", Type::String),
                field("uuid", Type::String),
                field("currency", Type::String),
                field("net_type", Type::optional(Type::String)),
                field("txid", Type::optional(Type::String)),
                field("state", Type::String),
                field("created_at", timestamp.clone()),
                field("done_at", Type::optional(timestamp.clone())),
                field("amount", decimal.clone()),
                field("fee", decimal.clone()),
                field("transaction_type", Type::String),
                field("is_cancelable", Type::optional(Boolean)),
            ],
        ),
        record(
            "UpbitApiKeyWire",
            vec![
                field("access_key", Type::String),
                field("expires_at", timestamp.clone()),
            ],
        ),
        record(
            "UpbitPocketWire",
            vec![
                field("uuid", Type::String),
                field("name", Type::String),
                field("kind", Type::String),
            ],
        ),
        record(
            "UpbitPocketApiKeyWire",
            vec![
                field("access_key", Type::String),
                field("permissions", Type::list(Type::String)),
                field("allowed_ips", Type::list(Type::String)),
                field("created_at", timestamp.clone()),
                field("expired_at", timestamp.clone()),
            ],
        ),
        record(
            "UpbitPocketApiKeyGroupWire",
            vec![
                field("uuid", Type::String),
                field("keys", Type::list(Type::named("UpbitPocketApiKeyWire"))),
            ],
        ),
        record(
            "UpbitPocketApiKeysRequestWire",
            vec![
                field("uuids", Type::list(Type::String)),
                field("include_expired", Boolean),
            ],
        ),
        record(
            "UpbitPocketBalanceWire",
            vec![
                field("currency", Type::String),
                field("balance", decimal.clone()),
                field("locked", decimal.clone()),
                field("avg_buy_price", decimal.clone()),
                field("avg_buy_price_modified", Boolean),
                field("unit_currency", Type::String),
            ],
        ),
        record(
            "UpbitPocketTransferQueryWire",
            vec![
                field("from", Type::optional(Type::String)),
                field("to", Type::optional(Type::String)),
                field(
                    "direction",
                    Type::optional(Type::Identifier("UpbitPocketTransferDirection")),
                ),
                field(
                    "states",
                    Type::list(Type::Identifier("UpbitPocketTransferState")),
                ),
                field("uuids", Type::list(Type::String)),
                field("identifiers", Type::list(Type::String)),
                field("start_time", Type::optional(timestamp.clone())),
                field("end_time", Type::optional(timestamp.clone())),
                field("currency", Type::optional(Type::String)),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("UpbitPocketTransferOrder")),
                ),
            ],
        ),
        record(
            "UpbitPocketUniversalTransferRequestWire",
            vec![
                field("from", Type::optional(Type::String)),
                field("to", Type::String),
                field("currency", Type::String),
                field("amount", decimal.clone()),
                field("identifier", Type::optional(Type::String)),
            ],
        ),
        record(
            "UpbitPocketTransferRequestWire",
            vec![
                field("to", Type::String),
                field("currency", Type::String),
                field("amount", decimal.clone()),
                field("identifier", Type::optional(Type::String)),
            ],
        ),
        record(
            "UpbitPocketTransferWire",
            vec![
                field("uuid", Type::String),
                field("identifier", Type::optional(Type::String)),
                field("from", Type::String),
                field("to", Type::String),
                field("state", Type::String),
                field("currency", Type::String),
                field("amount", decimal.clone()),
                field("created_at", timestamp.clone()),
            ],
        ),
        record(
            "BithumbMarketAlertWire",
            vec![
                field("kind", Type::String),
                field("step", Type::Identifier("BithumbAlertStep")),
                field("ends_at", timestamp.clone()),
            ],
        ),
        record(
            "BithumbNoticeWire",
            vec![
                field("categories", Type::list(Type::String)),
                field("title", Type::String),
                field("url", Type::String),
                field("published_at", timestamp.clone()),
                field("modified_at", timestamp.clone()),
            ],
        ),
        record(
            "BithumbAssetFeeWire",
            vec![
                field("display_name", Type::String),
                field("asset", Type::String),
                field("networks", Type::list(Type::named("BithumbNetworkFeeWire"))),
            ],
        ),
        record(
            "BithumbApiKeyWire",
            vec![
                field("access_key", Type::String),
                field("expires_at", timestamp.clone()),
            ],
        ),
        record(
            "BithumbKrwWithdrawalsRequestWire",
            vec![
                field("state", Type::optional(Type::String)),
                field("uuids", Type::list(Type::String)),
                field("txids", Type::list(Type::String)),
                field("page", Type::optional(Number)),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("BithumbOrderDirection")),
                ),
            ],
        ),
        record(
            "BithumbKrwDepositsRequestWire",
            vec![
                field("state", Type::optional(Type::String)),
                field("uuids", Type::list(Type::String)),
                field("txids", Type::list(Type::String)),
                field("page", Type::optional(Number)),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("BithumbOrderDirection")),
                ),
            ],
        ),
        record(
            "BithumbKrwTransferRequestWire",
            vec![field("amount", decimal.clone())],
        ),
        record(
            "BithumbKrwWithdrawalWire",
            vec![
                field("transfer_type", Type::String),
                field("uuid", Type::String),
                field("currency", Type::String),
                field("net_type", Type::optional(Type::String)),
                field("txid", Type::optional(Type::String)),
                field("state", Type::String),
                field("created_at", Type::optional(timestamp.clone())),
                field("done_at", Type::optional(timestamp.clone())),
                field("amount", decimal.clone()),
                field("fee", decimal.clone()),
                field("transaction_type", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbKrwDepositWire",
            vec![
                field("transfer_type", Type::String),
                field("uuid", Type::String),
                field("currency", Type::String),
                field("net_type", Type::optional(Type::String)),
                field("txid", Type::optional(Type::String)),
                field("state", Type::String),
                field("created_at", Type::optional(timestamp.clone())),
                field("done_at", Type::optional(timestamp.clone())),
                field("amount", decimal.clone()),
                field("fee", decimal.clone()),
                field("transaction_type", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbPendingOrdersRequestWire",
            vec![
                field("market", Type::optional(market.clone())),
                field(
                    "state",
                    Type::optional(Type::Identifier("BithumbPendingOrderState")),
                ),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("BithumbOrderDirection")),
                ),
                field("cursor", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbClosedOrdersRequestWire",
            vec![
                field("market", Type::optional(market.clone())),
                field(
                    "state",
                    Type::optional(Type::Identifier("BithumbClosedOrderState")),
                ),
                field(
                    "states",
                    Type::list(Type::Identifier("BithumbClosedOrderState")),
                ),
                field("start_time", Type::optional(timestamp.clone())),
                field("end_time", Type::optional(timestamp.clone())),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("BithumbOrderDirection")),
                ),
                field("cursor", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbClosedOrderWire",
            vec![
                field("order_id", Type::String),
                field("side", Type::String),
                field("order_type", Type::String),
                field("price", Type::optional(decimal.clone())),
                field("state", Type::String),
                field("market", market.clone()),
                field("created_at", Type::optional(timestamp.clone())),
                field("volume", decimal.clone()),
                field("remaining_volume", decimal.clone()),
                field("reserved_fee", decimal.clone()),
                field("remaining_fee", decimal.clone()),
                field("paid_fee", decimal.clone()),
                field("locked", decimal.clone()),
                field("executed_volume", decimal.clone()),
                field("executed_funds", decimal.clone()),
                field("trades_count", Number),
                field("client_order_id", Type::optional(Type::String)),
                field("stp_type", Type::optional(Type::String)),
                field("time_in_force", Type::optional(Type::String)),
                field("cancel_type", Type::optional(Type::String)),
                field("canceling_order_id", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbTwapOrdersRequestWire",
            vec![
                field("market", Type::optional(market.clone())),
                field("uuids", Type::list(Type::String)),
                field(
                    "state",
                    Type::optional(Type::Identifier("BithumbTwapState")),
                ),
                field("cursor", Type::optional(Type::String)),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("BithumbTwapOrderDirection")),
                ),
            ],
        ),
        record(
            "BithumbBatchOrdersRequestWire",
            vec![field("orders", Type::list(Type::named("OrderRequestWire")))],
        ),
        record(
            "BithumbBatchOrderWire",
            vec![
                field("order_id", Type::String),
                field("client_order_id", Type::optional(Type::String)),
                field("market", market.clone()),
                field("side", Type::Identifier("Side")),
                field("order_type", Type::Identifier("OrderType")),
                field("time_in_force", Type::optional(Type::String)),
                field("stp_type", Type::optional(Type::String)),
                field("created_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "BithumbBatchOrderFailureWire",
            vec![
                field("client_order_id", Type::optional(Type::String)),
                field("time_in_force", Type::optional(Type::String)),
                field("code", Type::String),
                field("message", Type::String),
            ],
        ),
        record(
            "BithumbBatchOrdersResultWire",
            vec![field(
                "outcomes",
                Type::list(Type::named("BithumbBatchOrderOutcomeWire")),
            )],
        ),
        record(
            "BithumbTwapOrderRequestWire",
            vec![
                field("market", market.clone()),
                field("side", Type::Identifier("Side")),
                field("volume", Type::optional(decimal.clone())),
                field("price", Type::optional(decimal.clone())),
                field("duration", Number),
                field("frequency", Number),
            ],
        ),
        record(
            "BithumbTwapOrderWire",
            vec![
                field("id", Type::String),
                field("side", Type::Identifier("Side")),
                field("price", decimal.clone()),
                field("state", Type::Identifier("BithumbTwapState")),
                field("market", market.clone()),
                field("created_at", timestamp.clone()),
                field("volume", decimal.clone()),
                field("finished_at", Type::optional(timestamp.clone())),
                field("total_order_count", Number),
                field("total_trades_count", Number),
                field("progress_count", Number),
                field("total_executed_amount", decimal.clone()),
                field("total_executed_volume", decimal.clone()),
                field("avg_trade_price", decimal.clone()),
                field("wallet_id", Type::optional(Type::String)),
                field("canceled_at", Type::optional(timestamp.clone())),
                field("cancel_type", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbNetworkFeeWire",
            vec![
                field("network", Type::Identifier("Network")),
                field("provider_name", Type::String),
                field("deposit_fee", decimal.clone()),
                field("minimum_deposit", decimal.clone()),
                field("withdrawal_fee", Type::named("WithdrawalFeeWire")),
                field("minimum_withdrawal", decimal.clone()),
            ],
        ),
        record(
            "BithumbWithdrawalAddressWire",
            vec![
                field("currency", Type::String),
                field("net_type", Type::String),
                field("network_name", Type::optional(Type::String)),
                field("withdraw_address", Type::String),
                field("secondary_address", Type::optional(Type::String)),
                field("exchange_name", Type::optional(Type::String)),
                field("owner_type", Type::optional(Type::String)),
                field("owner_ko_name", Type::optional(Type::String)),
                field("owner_en_name", Type::optional(Type::String)),
                field("owner_corp_ko_name", Type::optional(Type::String)),
                field("owner_corp_en_name", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbOrderDetailRequestWire",
            vec![
                field("market", market.clone()),
                field("uuid", Type::optional(Type::String)),
                field("client_order_id", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbOrderDetailTradeWire",
            vec![
                field("market", market.clone()),
                field("uuid", Type::String),
                field("price", decimal.clone()),
                field("volume", decimal.clone()),
                field("funds", decimal.clone()),
                field("side", Type::String),
                field("created_at", timestamp.clone()),
            ],
        ),
        record(
            "BithumbOrderDetailWire",
            vec![
                field("uuid", Type::String),
                field("client_order_id", Type::optional(Type::String)),
                field("side", Type::String),
                field("order_type", Type::String),
                field("price", decimal.clone()),
                field("state", Type::String),
                field("market", market.clone()),
                field("created_at", timestamp.clone()),
                field("volume", decimal.clone()),
                field("remaining_volume", decimal.clone()),
                field("reserved_fee", decimal.clone()),
                field("remaining_fee", decimal.clone()),
                field("paid_fee", decimal.clone()),
                field("locked", decimal.clone()),
                field("executed_volume", decimal.clone()),
                field("executed_funds", decimal.clone()),
                field("trades_count", Number),
                field(
                    "trades",
                    Type::list(Type::named("BithumbOrderDetailTradeWire")),
                ),
                field("stp_type", Type::optional(Type::String)),
                field("cancel_type", Type::optional(Type::String)),
                field("canceling_uuid", Type::optional(Type::String)),
                field("time_in_force", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbOrderListRequestWire",
            vec![
                field("market", Type::optional(market.clone())),
                field(
                    "state",
                    Type::optional(Type::Identifier("BithumbOrderListState")),
                ),
                field(
                    "states",
                    Type::list(Type::Identifier("BithumbOrderListState")),
                ),
                field("uuids", Type::list(Type::String)),
                field("client_order_ids", Type::list(Type::String)),
                field("page", Type::optional(Number)),
                field("limit", Type::optional(Number)),
                field(
                    "order_by",
                    Type::optional(Type::Identifier("BithumbOrderDirection")),
                ),
            ],
        ),
        record(
            "BithumbOrderListItemWire",
            vec![
                field("uuid", Type::String),
                field("client_order_id", Type::optional(Type::String)),
                field("side", Type::String),
                field("order_type", Type::String),
                field("price", decimal.clone()),
                field("state", Type::String),
                field("market", market.clone()),
                field("created_at", timestamp.clone()),
                field("volume", decimal.clone()),
                field("remaining_volume", decimal.clone()),
                field("reserved_fee", decimal.clone()),
                field("remaining_fee", decimal.clone()),
                field("paid_fee", decimal.clone()),
                field("locked", decimal.clone()),
                field("executed_volume", decimal.clone()),
                field("executed_funds", decimal.clone()),
                field("trades_count", Number),
                field("stp_type", Type::optional(Type::String)),
                field("time_in_force", Type::optional(Type::String)),
            ],
        ),
        record(
            "BinanceSymbolFiltersWire",
            vec![
                field("symbol", Type::String),
                field("tick_size", Type::optional(decimal.clone())),
                field("min_price", Type::optional(decimal.clone())),
                field("max_price", Type::optional(decimal.clone())),
                field("step_size", Type::optional(decimal.clone())),
                field("min_quantity", Type::optional(decimal.clone())),
                field("max_quantity", Type::optional(decimal.clone())),
                field("min_notional", Type::optional(decimal.clone())),
            ],
        ),
        record(
            "BinanceSpotOrderDetailWire",
            vec![
                field("order", Type::named("OrderWire")),
                field("client_order_id", Type::String),
                field("order_type", Type::String),
                field("time_in_force", Type::String),
                field("filled_quote_quantity", decimal.clone()),
                field("updated_at", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "BinanceSpotAveragePriceWire",
            vec![
                field("market", market.clone()),
                field("minutes", Number),
                field("price", decimal.clone()),
                field("close_time", timestamp.clone()),
            ],
        ),
        record(
            "BinanceDepositHistoryRequestWire",
            vec![
                field("coin", Type::optional(Type::String)),
                field("status", Type::optional(Number)),
                field("start_time", Type::optional(timestamp.clone())),
                field("end_time", Type::optional(timestamp.clone())),
                field("offset", Type::optional(Type::UnsignedInteger)),
                field("limit", Type::optional(Number)),
                field("tx_id", Type::optional(Type::String)),
                field("include_source", Boolean),
            ],
        ),
        record(
            "BinanceWithdrawHistoryRequestWire",
            vec![
                field("coin", Type::optional(Type::String)),
                field("withdraw_order_id", Type::optional(Type::String)),
                field("status", Type::optional(Number)),
                field("offset", Type::optional(Type::UnsignedInteger)),
                field("limit", Type::optional(Number)),
                field("id_list", Type::list(Type::String)),
                field("start_time", Type::optional(timestamp.clone())),
                field("end_time", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "BinanceSpotCommissionRatesWire",
            vec![
                field("maker", decimal.clone()),
                field("taker", decimal.clone()),
                field("buyer", decimal.clone()),
                field("seller", decimal.clone()),
            ],
        ),
        record(
            "BinanceSpotAccountBalanceWire",
            vec![
                field("asset", Type::String),
                field("free", decimal.clone()),
                field("locked", decimal.clone()),
            ],
        ),
        record(
            "BinanceSpotAccountInformationWire",
            vec![
                field("maker_commission", Type::UnsignedInteger),
                field("taker_commission", Type::UnsignedInteger),
                field("buyer_commission", Type::UnsignedInteger),
                field("seller_commission", Type::UnsignedInteger),
                field(
                    "commission_rates",
                    Type::named("BinanceSpotCommissionRatesWire"),
                ),
                field("can_trade", Boolean),
                field("can_withdraw", Boolean),
                field("can_deposit", Boolean),
                field("update_time", timestamp.clone()),
                field("account_type", Type::String),
                field(
                    "balances",
                    Type::list(Type::named("BinanceSpotAccountBalanceWire")),
                ),
                field("permissions", Type::list(Type::String)),
                field("uid", Type::optional(Type::UnsignedInteger)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceSpotCancelledOrderWire",
            vec![
                field("symbol", Type::optional(Type::String)),
                field("original_client_order_id", Type::optional(Type::String)),
                field("order_id", Type::optional(Type::String)),
                field("client_order_id", Type::optional(Type::String)),
                field("status", Type::optional(Type::String)),
                field("price", Type::optional(decimal.clone())),
                field("original_quantity", Type::optional(decimal.clone())),
                field("executed_quantity", Type::optional(decimal.clone())),
                field("cumulative_quote_quantity", Type::optional(decimal.clone())),
                field("transact_time", Type::optional(timestamp.clone())),
                field("order_list_id", Type::optional(Type::String)),
                field("contingency_type", Type::optional(Type::String)),
                field("list_status_type", Type::optional(Type::String)),
                field("list_order_status", Type::optional(Type::String)),
                field("list_client_order_id", Type::optional(Type::String)),
                field("transaction_time", Type::optional(timestamp.clone())),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceSpotCancelAllOpenOrdersWire",
            vec![
                field(
                    "reports",
                    Type::list(Type::named("BinanceSpotCancelledOrderWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceExchangeSymbolWire",
            vec![
                field("symbol", Type::String),
                field("status", Type::String),
                field("base_asset", Type::String),
                field("quote_asset", Type::String),
                field("contract_type", Type::optional(Type::String)),
                field("margin_asset", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceExchangeInfoWire",
            vec![
                field("venue", Type::Identifier("BinanceMarket")),
                field("timezone", Type::optional(Type::String)),
                field("server_time", Type::optional(timestamp.clone())),
                field(
                    "symbols",
                    Type::list(Type::named("BinanceExchangeSymbolWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceUsdMAccountAssetWire",
            vec![
                field("asset", Type::String),
                field("wallet_balance", decimal.clone()),
                field("unrealized_profit", decimal.clone()),
                field("margin_balance", decimal.clone()),
                field("maintenance_margin", decimal.clone()),
                field("initial_margin", decimal.clone()),
                field("position_initial_margin", decimal.clone()),
                field("open_order_initial_margin", decimal.clone()),
                field("cross_wallet_balance", decimal.clone()),
                field("cross_unrealized_profit", decimal.clone()),
                field("available_balance", decimal.clone()),
                field("max_withdraw_amount", decimal.clone()),
                field("update_time", timestamp.clone()),
            ],
        ),
        record(
            "BinanceUsdMAccountPositionWire",
            vec![
                field("symbol", Type::String),
                field("position_side", Type::String),
                field("position_amount", decimal.clone()),
                field("unrealized_profit", decimal.clone()),
                field("isolated_margin", decimal.clone()),
                field("notional", decimal.clone()),
                field("isolated_wallet", decimal.clone()),
                field("initial_margin", decimal.clone()),
                field("maintenance_margin", decimal.clone()),
                field("update_time", timestamp.clone()),
            ],
        ),
        record(
            "BinanceUsdMAccountInformationWire",
            vec![
                field("total_initial_margin", decimal.clone()),
                field("total_maintenance_margin", decimal.clone()),
                field("total_wallet_balance", decimal.clone()),
                field("total_unrealized_profit", decimal.clone()),
                field("total_margin_balance", decimal.clone()),
                field("total_position_initial_margin", decimal.clone()),
                field("total_open_order_initial_margin", decimal.clone()),
                field("total_cross_wallet_balance", decimal.clone()),
                field("total_cross_unrealized_profit", decimal.clone()),
                field("available_balance", decimal.clone()),
                field("max_withdraw_amount", decimal.clone()),
                field(
                    "assets",
                    Type::list(Type::named("BinanceUsdMAccountAssetWire")),
                ),
                field(
                    "positions",
                    Type::list(Type::named("BinanceUsdMAccountPositionWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceUsdMPositionInformationWire",
            vec![
                field("symbol", Type::String),
                field("position_side", Type::String),
                field("position_amount", decimal.clone()),
                field("entry_price", decimal.clone()),
                field("break_even_price", decimal.clone()),
                field("mark_price", decimal.clone()),
                field("unrealized_profit", decimal.clone()),
                field("liquidation_price", decimal.clone()),
                field("isolated_margin", decimal.clone()),
                field("notional", decimal.clone()),
                field("margin_asset", Type::String),
                field("isolated_wallet", decimal.clone()),
                field("initial_margin", decimal.clone()),
                field("maintenance_margin", decimal.clone()),
                field("position_initial_margin", decimal.clone()),
                field("open_order_initial_margin", decimal.clone()),
                field("adl", Type::UnsignedInteger),
                field("bid_notional", decimal.clone()),
                field("ask_notional", decimal.clone()),
                field("update_time", timestamp.clone()),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceCoinNetworkInformationWire",
            vec![
                field("network", Type::String),
                field("deposit_enabled", Boolean),
                field("withdraw_enabled", Boolean),
                field("busy", Boolean),
                field(
                    "withdrawal_integer_multiple",
                    Type::optional(decimal.clone()),
                ),
                field("withdrawal_fee", Type::optional(decimal.clone())),
                field("minimum_withdrawal", Type::optional(decimal.clone())),
                field("maximum_withdrawal", Type::optional(decimal.clone())),
                field("withdrawal_tag", Type::optional(Boolean)),
                field("is_default", Type::optional(Boolean)),
                field(
                    "minimum_confirmations",
                    Type::optional(Type::UnsignedInteger),
                ),
                field(
                    "unlock_confirmations",
                    Type::optional(Type::UnsignedInteger),
                ),
                field("contract_address", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceCoinInformationWire",
            vec![
                field("coin", Type::String),
                field("deposit_all_enabled", Boolean),
                field("withdraw_all_enabled", Boolean),
                field("name", Type::optional(Type::String)),
                field("free", Type::optional(decimal.clone())),
                field("locked", Type::optional(decimal.clone())),
                field("freeze", Type::optional(decimal.clone())),
                field("withdrawing", Type::optional(decimal.clone())),
                field("is_legal_money", Type::optional(Boolean)),
                field("trading", Type::optional(Boolean)),
                field(
                    "networks",
                    Type::list(Type::named("BinanceCoinNetworkInformationWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceApiKeyPermissionsWire",
            vec![
                field("ip_restrict", Boolean),
                field("create_time", Type::optional(timestamp.clone())),
                field("enable_reading", Boolean),
                field("enable_withdrawals", Boolean),
                field("enable_internal_transfer", Boolean),
                field("enable_margin", Boolean),
                field("enable_spot_and_margin_trading", Boolean),
                field("enable_futures", Boolean),
                field("permits_universal_transfer", Boolean),
                field("enable_vanilla_options", Boolean),
                field("enable_fix_api_trade", Boolean),
                field("enable_fix_read_only", Boolean),
                field("enable_portfolio_margin_trading", Boolean),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceDepositHistoryEntryWire",
            vec![
                field("id", Type::String),
                field("amount", decimal.clone()),
                field("coin", Type::String),
                field("network", Type::String),
                field("status", Number),
                field("address", Type::optional(Type::String)),
                field("address_tag", Type::optional(Type::String)),
                field("tx_id", Type::optional(Type::String)),
                field("insert_time", timestamp.clone()),
                field("complete_time", Type::optional(timestamp.clone())),
                field("transfer_type", Type::optional(Number)),
                field("source_address", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceDepositHistoryWire",
            vec![
                field(
                    "entries",
                    Type::list(Type::named("BinanceDepositHistoryEntryWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceQuestionnaireRequirementsWire",
            vec![
                field("questionnaire_country_code", Type::String),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceWithdrawalAddressWire",
            vec![
                field("address", Type::String),
                field("address_tag", Type::optional(Type::String)),
                field("coin", Type::String),
                field("network", Type::String),
                field("white_status", Boolean),
                field("name", Type::optional(Type::String)),
                field("origin", Type::optional(Type::String)),
                field("origin_type", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceWithdrawHistoryEntryWire",
            vec![
                field("id", Type::String),
                field("amount", decimal.clone()),
                field("transaction_fee", decimal.clone()),
                field("coin", Type::String),
                field("status", Number),
                field("address", Type::optional(Type::String)),
                field("tx_id", Type::optional(Type::String)),
                field("apply_time", Type::optional(Type::String)),
                field("network", Type::optional(Type::String)),
                field("withdraw_order_id", Type::optional(Type::String)),
                field("info", Type::optional(Type::String)),
                field("transfer_type", Type::optional(Number)),
                field("confirm_no", Type::optional(Number)),
                field("wallet_type", Type::optional(Number)),
                field("tx_key", Type::optional(Type::String)),
                field("complete_time", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceWithdrawHistoryWire",
            vec![
                field(
                    "entries",
                    Type::list(Type::named("BinanceWithdrawHistoryEntryWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "BinanceMarkPriceWire",
            vec![
                field("market", market.clone()),
                field("mark_price", decimal.clone()),
                field("index_price", decimal.clone()),
                field("estimated_settle_price", Type::optional(decimal.clone())),
                field("last_funding_rate", decimal.clone()),
                field("interest_rate", decimal.clone()),
                field("next_funding_time", timestamp.clone()),
                field("time", timestamp.clone()),
            ],
        ),
        record(
            "BinanceOpenInterestWire",
            vec![
                field("market", market.clone()),
                field("open_interest", decimal.clone()),
                field("time", timestamp.clone()),
            ],
        ),
        record(
            "BinanceAggregateTradesRequestWire",
            vec![
                field("market", market.clone()),
                field("from_id", Type::optional(Type::UnsignedInteger)),
                field("start_time", Type::optional(timestamp.clone())),
                field("end_time", Type::optional(timestamp.clone())),
                field("limit", Type::optional(Number)),
            ],
        ),
        record(
            "BinanceAggregateTradeWire",
            vec![
                field("market", market.clone()),
                field("aggregate_id", Type::UnsignedInteger),
                field("first_trade_id", Type::UnsignedInteger),
                field("last_trade_id", Type::UnsignedInteger),
                field("timestamp", timestamp.clone()),
                field("price", decimal.clone()),
                field("quantity", decimal.clone()),
                field("normal_quantity", Type::optional(decimal.clone())),
                field("taker_side", Type::Identifier("Side")),
            ],
        ),
        record(
            "BinanceAccountTradeWire",
            vec![
                field("market", market.clone()),
                field("id", Type::String),
                field("order_id", Type::String),
                field("timestamp", timestamp.clone()),
                field("side", Type::Identifier("Side")),
                field("maker", Boolean),
                field("best_match", Type::optional(Boolean)),
                field("order_list_id", Type::optional(Type::String)),
                field("price", decimal.clone()),
                field("quantity", decimal.clone()),
                field("quote_quantity", Type::optional(decimal.clone())),
                field("commission", decimal.clone()),
                field("commission_asset", Type::String),
                field("realized_pnl", Type::optional(decimal.clone())),
                field("position_side", Type::optional(Type::String)),
                field("pair", Type::optional(Type::String)),
                field("base_quantity", Type::optional(decimal.clone())),
                field("margin_asset", Type::optional(Type::String)),
            ],
        ),
        record(
            "BinanceTestOrderRequestWire",
            vec![
                field("order", Type::named("OrderRequestWire")),
                field("compute_commission_rates", Boolean),
            ],
        ),
        record(
            "BinanceTestOrderWire",
            vec![field("response_json", Type::String)],
        ),
        record(
            "BinanceC2cTradeHistoryRequestWire",
            vec![
                field("trade_type", Type::Identifier("BinanceC2cTradeType")),
                field("start_timestamp", Type::optional(timestamp.clone())),
                field("end_timestamp", Type::optional(timestamp.clone())),
                field("page", Type::optional(Number)),
                field("rows", Type::optional(Number)),
                field("recv_window", Type::optional(Type::UnsignedInteger)),
            ],
        ),
        record(
            "BinanceC2cTradeWire",
            vec![
                field("order_number", Type::optional(Type::String)),
                field("adv_no", Type::optional(Type::String)),
                field("trade_type", Type::optional(Type::String)),
                field("asset", Type::optional(Type::String)),
                field("fiat", Type::optional(Type::String)),
                field("fiat_symbol", Type::optional(Type::String)),
                field("amount", Type::optional(decimal.clone())),
                field("total_price", Type::optional(decimal.clone())),
                field("unit_price", Type::optional(decimal.clone())),
                field("order_status", Type::optional(Type::String)),
                field("created_at", Type::optional(timestamp.clone())),
                field("commission", Type::optional(decimal.clone())),
                field("counterparty_nickname", Type::optional(Type::String)),
                field("pay_method_name", Type::optional(Type::String)),
                field("additional_kyc_verify", Type::optional(Number)),
                field("taker_commission_rate", Type::optional(decimal.clone())),
                field("taker_commission", Type::optional(decimal.clone())),
                field("taker_amount", Type::optional(decimal.clone())),
                field("advertisement_role", Type::optional(Type::String)),
            ],
        ),
        record(
            "BinanceC2cTradeHistoryPageWire",
            vec![
                field("code", Type::optional(Type::String)),
                field("message", Type::optional(Type::String)),
                field(
                    "data",
                    Type::optional(Type::list(Type::named("BinanceC2cTradeWire"))),
                ),
                field("total", Type::optional(Type::UnsignedInteger)),
                field("success", Type::optional(Boolean)),
            ],
        ),
        record(
            "BinanceListenKeyWire",
            vec![field("id", Type::String), field("value", Type::String)],
        ),
        record(
            "HyperliquidLedgerEntryWire",
            vec![
                field("kind", Type::Identifier("HyperliquidLedgerKind")),
                field("time", timestamp.clone()),
                field("hash", Type::String),
                field("asset", Type::optional(Type::String)),
                field("amount", Type::optional(decimal.clone())),
                field("fee", Type::optional(decimal.clone())),
                field("counterparty", Type::optional(Type::String)),
            ],
        ),
        record(
            "HyperliquidCandleSnapshotWire",
            vec![
                field("coin", Type::String),
                field("market", market.clone()),
                field("interval", Type::String),
                field("open_time", timestamp.clone()),
                field("close_time", timestamp.clone()),
                field("open", decimal.clone()),
                field("high", decimal.clone()),
                field("low", decimal.clone()),
                field("close", decimal.clone()),
                field("volume", decimal.clone()),
                field("trade_count", Type::optional(Type::UnsignedInteger)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidBookLevelWire",
            vec![
                field("price", decimal.clone()),
                field("size", decimal.clone()),
                field("order_count", Type::optional(Type::UnsignedInteger)),
            ],
        ),
        record(
            "HyperliquidL2BookWire",
            vec![
                field("coin", Type::String),
                field("market", market.clone()),
                field("time", timestamp.clone()),
                field("bids", Type::list(Type::named("HyperliquidBookLevelWire"))),
                field("asks", Type::list(Type::named("HyperliquidBookLevelWire"))),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidRecentTradeWire",
            vec![
                field("coin", Type::String),
                field("market", market.clone()),
                field("side", Type::String),
                field("price", decimal.clone()),
                field("size", decimal.clone()),
                field("time", timestamp.clone()),
                field("trade_id", Type::String),
                field("hash", Type::optional(Type::String)),
                field("users", Type::list(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidTradeEventWire",
            vec![
                field("common", Type::named("TradeWire")),
                field("provider", Type::named("HyperliquidRecentTradeWire")),
            ],
        ),
        record(
            "HyperliquidOrderBookEventWire",
            vec![
                field("common", Type::named("OrderBookWire")),
                field("provider", Type::named("HyperliquidL2BookWire")),
            ],
        ),
        record(
            "HyperliquidCandleEventWire",
            vec![
                field("common", Type::named("CandleWire")),
                field("provider", Type::named("HyperliquidCandleSnapshotWire")),
            ],
        ),
        record(
            "HyperliquidAssetContextEventWire",
            vec![
                field("common", Type::named("TickerWire")),
                field("coin", Type::String),
                field("mid_price", Type::optional(decimal.clone())),
                field("mark_price", Type::optional(decimal.clone())),
                field("previous_day_price", Type::optional(decimal.clone())),
                field("day_base_volume", Type::optional(decimal.clone())),
                field("day_notional_volume", Type::optional(decimal.clone())),
                field("oracle_price", Type::optional(decimal.clone())),
                field("funding_rate", Type::optional(decimal.clone())),
                field("open_interest", Type::optional(decimal.clone())),
                field("circulating_supply", Type::optional(decimal.clone())),
                field("total_supply", Type::optional(decimal.clone())),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidOrderUpdateWire",
            vec![
                field("common", Type::named("OrderWire")),
                field("coin", Type::String),
                field("side", Type::String),
                field("limit_price", decimal.clone()),
                field("remaining_size", decimal.clone()),
                field("original_size", decimal.clone()),
                field("order_id", Type::UnsignedInteger),
                field("accepted_at", timestamp.clone()),
                field("client_order_id", Type::optional(Type::String)),
                field("status", Type::String),
                field("status_at", Type::optional(timestamp.clone())),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotStateBalanceWire",
            vec![
                field("common", Type::named("BalanceWire")),
                field("provider", Type::named("HyperliquidSpotBalanceWire")),
            ],
        ),
        record(
            "HyperliquidSpotStateEventWire",
            vec![
                field("user", Type::String),
                field(
                    "balances",
                    Type::list(Type::named("HyperliquidSpotStateBalanceWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidFundingHistoryEntryWire",
            vec![
                field("coin", Type::String),
                field("market", market.clone()),
                field("funding_rate", decimal.clone()),
                field("premium", Type::optional(decimal.clone())),
                field("time", timestamp.clone()),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidUserFundingWire",
            vec![
                field("kind", Type::optional(Type::String)),
                field("coin", Type::String),
                field("market", market.clone()),
                field("usdc", decimal.clone()),
                field("funding_rate", decimal.clone()),
                field("position_size", Type::optional(decimal.clone())),
                field("sample_count", Type::optional(Type::UnsignedInteger)),
                field("hash", Type::String),
                field("time", timestamp.clone()),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotBalanceWire",
            vec![
                field("coin", Type::String),
                field("token", Type::optional(Number)),
                field("total", decimal.clone()),
                field("hold", decimal.clone()),
                field("entry_notional", Type::optional(decimal.clone())),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotClearinghouseStateWire",
            vec![
                field(
                    "balances",
                    Type::list(Type::named("HyperliquidSpotBalanceWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidEvmContractWire",
            vec![
                field("address", Type::String),
                field("extra_wei_decimals", Number),
            ],
        ),
        record(
            "HyperliquidSpotTokenWire",
            vec![
                field("name", Type::String),
                field("size_decimals", Number),
                field("wei_decimals", Type::optional(Number)),
                field("index", Number),
                field("token_id", Type::optional(Type::String)),
                field("is_canonical", Type::optional(Boolean)),
                field(
                    "evm_contract",
                    Type::optional(Type::named("HyperliquidEvmContractWire")),
                ),
                field("full_name", Type::optional(Type::String)),
                field(
                    "deployer_trading_fee_share",
                    Type::optional(decimal.clone()),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotPairWire",
            vec![
                field("name", Type::String),
                field("tokens", Type::list(Number)),
                field("index", Number),
                field("is_canonical", Type::optional(Boolean)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotMetaWire",
            vec![
                field(
                    "tokens",
                    Type::list(Type::named("HyperliquidSpotTokenWire")),
                ),
                field(
                    "universe",
                    Type::list(Type::named("HyperliquidSpotPairWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotAssetContextWire",
            vec![
                field("coin", Type::optional(Type::String)),
                field("mid_price", Type::optional(decimal.clone())),
                field("mark_price", Type::optional(decimal.clone())),
                field("previous_day_price", Type::optional(decimal.clone())),
                field("day_base_volume", Type::optional(decimal.clone())),
                field("day_notional_volume", Type::optional(decimal.clone())),
                field("circulating_supply", Type::optional(decimal.clone())),
                field("total_supply", Type::optional(decimal.clone())),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidSpotMetaAndAssetContextsWire",
            vec![
                field("meta", Type::named("HyperliquidSpotMetaWire")),
                field(
                    "contexts",
                    Type::list(Type::named("HyperliquidSpotAssetContextWire")),
                ),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidMidPriceWire",
            vec![
                field("market", market.clone()),
                field("price", decimal.clone()),
            ],
        ),
        record(
            "HyperliquidAssetContextWire",
            vec![
                field("mid_price", Type::optional(decimal.clone())),
                field("mark_price", Type::optional(decimal.clone())),
                field("oracle_price", Type::optional(decimal.clone())),
                field("funding_rate", Type::optional(decimal.clone())),
                field("open_interest", Type::optional(decimal.clone())),
                field("size_decimals", Number),
                field("price_decimals", Number),
            ],
        ),
        record(
            "HyperliquidUserRateLimitWire",
            vec![
                field("cumulative_volume", decimal.clone()),
                field("requests_used", Type::UnsignedInteger),
                field("requests_cap", Type::UnsignedInteger),
                field("requests_surplus", Type::UnsignedInteger),
            ],
        ),
        record(
            "HyperliquidReferrerWire",
            vec![field("address", Type::String), field("code", Type::String)],
        ),
        record(
            "HyperliquidReferralWire",
            vec![
                field(
                    "referred_by",
                    Type::optional(Type::named("HyperliquidReferrerWire")),
                ),
                field("cumulative_volume", decimal.clone()),
                field("unclaimed_rewards", decimal.clone()),
                field("claimed_rewards", decimal.clone()),
                field("builder_rewards", decimal.clone()),
                field("referrer_state_json", Type::String),
                field("reward_history_json", Type::String),
                field("token_to_state_json", Type::String),
            ],
        ),
        record(
            "HyperliquidDailyVolumeWire",
            vec![
                field("date", Type::String),
                field("user_cross", decimal.clone()),
                field("user_add", decimal.clone()),
                field("exchange", decimal.clone()),
            ],
        ),
        record(
            "HyperliquidUserFeesWire",
            vec![
                field(
                    "daily_volumes",
                    Type::list(Type::named("HyperliquidDailyVolumeWire")),
                ),
                field("fee_schedule_json", Type::String),
                field("user_cross_rate", decimal.clone()),
                field("user_add_rate", decimal.clone()),
                field("user_spot_cross_rate", Type::optional(decimal.clone())),
                field("user_spot_add_rate", Type::optional(decimal.clone())),
                field("active_referral_discount", Type::optional(decimal.clone())),
                field("details_json", Type::String),
            ],
        ),
        record(
            "HyperliquidUserFillWire",
            vec![
                field("coin", Type::String),
                field("price", decimal.clone()),
                field("size", decimal.clone()),
                field("side", Type::String),
                field("time", timestamp.clone()),
                field("start_position", decimal.clone()),
                field("direction", Type::String),
                field("closed_pnl", decimal.clone()),
                field("hash", Type::String),
                field("order_id", Type::UnsignedInteger),
                field("crossed", Boolean),
                field("fee", decimal.clone()),
                field("builder_fee", Type::optional(decimal.clone())),
                field("trade_id", Type::UnsignedInteger),
                field("fee_token", Type::String),
                field("twap_id", Type::optional(Type::UnsignedInteger)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidOpenOrderWire",
            vec![
                field("coin", Type::String),
                field("limit_price", decimal.clone()),
                field("order_id", Type::UnsignedInteger),
                field("side", Type::String),
                field("size", decimal.clone()),
                field("timestamp", timestamp.clone()),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidOrderDetailWire",
            vec![
                field("coin", Type::String),
                field("side", Type::String),
                field("limit_price", decimal.clone()),
                field("size", decimal.clone()),
                field("order_id", Type::UnsignedInteger),
                field("timestamp", timestamp.clone()),
                field("trigger_condition", Type::String),
                field("is_trigger", Boolean),
                field("trigger_price", decimal.clone()),
                field("children_json", Type::String),
                field("is_position_tpsl", Boolean),
                field("reduce_only", Boolean),
                field("order_type", Type::String),
                field("original_size", decimal.clone()),
                field("time_in_force", Type::optional(Type::String)),
                field("client_order_id", Type::optional(Type::String)),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidOrderInfoWire",
            vec![
                field("order", Type::named("HyperliquidOrderDetailWire")),
                field("status", Type::String),
                field("status_timestamp", timestamp.clone()),
                field("raw_json", Type::String),
            ],
        ),
        record(
            "HyperliquidPortfolioPointWire",
            vec![
                field("time", timestamp.clone()),
                field("value", decimal.clone()),
            ],
        ),
        record(
            "HyperliquidPortfolioPeriodWire",
            vec![
                field("period", Type::String),
                field(
                    "account_value_history",
                    Type::list(Type::named("HyperliquidPortfolioPointWire")),
                ),
                field(
                    "pnl_history",
                    Type::list(Type::named("HyperliquidPortfolioPointWire")),
                ),
                field("volume", decimal.clone()),
            ],
        ),
        record(
            "HyperliquidSubAccountWire",
            vec![
                field("name", Type::String),
                field("user", Type::String),
                field("master", Type::String),
                field("perpetual_state_json", Type::String),
                field("spot_state_json", Type::String),
            ],
        ),
        record(
            "HyperliquidVaultEquityWire",
            vec![
                field("vault_address", Type::String),
                field("equity", decimal.clone()),
                field("locked_until", Type::optional(timestamp.clone())),
            ],
        ),
        record(
            "UpbitOptionsWire",
            vec![
                field("region", Type::Identifier("UpbitRegion")),
                field("access_key", Type::optional(Type::String)),
                field("secret_key", Type::optional(Type::String)),
            ],
        ),
        record(
            "BithumbOptionsWire",
            vec![
                field("access_key", Type::optional(Type::String)),
                field("secret_key", Type::optional(Type::String)),
            ],
        ),
        record(
            "BinanceOptionsWire",
            vec![
                field("venue", Type::Identifier("BinanceMarket")),
                field("api_key", Type::optional(Type::String)),
                field("secret_key", Type::optional(Type::String)),
            ],
        ),
        record(
            "HyperliquidOptionsWire",
            vec![
                field("testnet", Boolean),
                field("address", Type::optional(Type::String)),
                field("private_key", Type::optional(Type::String)),
            ],
        ),
        record(
            "NativeStreamReferenceWire",
            vec![field("id", Type::String), field("kind", Type::String)],
        ),
    ];
    let unions = vec![
        TaggedUnion {
            name: "UpbitBatchCancelScopeWire",
            type_parameters: &[],
            variants: vec![
                variant("all", vec![]),
                variant(
                    "quote_currencies",
                    vec![field("values", Type::list(Type::String))],
                ),
                variant("pairs", vec![field("values", Type::list(market.clone()))]),
            ],
        },
        TaggedUnion {
            name: "UpbitOrderReferenceWire",
            type_parameters: &[],
            variants: vec![
                variant("uuid", vec![field("value", Type::String)]),
                variant("identifier", vec![field("value", Type::String)]),
            ],
        },
        TaggedUnion {
            name: "UpbitOrderVolumeWire",
            type_parameters: &[],
            variants: vec![
                variant("amount", vec![field("value", Type::Decimal)]),
                variant("remain_only", vec![]),
            ],
        },
        TaggedUnion {
            name: "UpbitCancelAndNewOrderWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "limit",
                    vec![
                        field("volume", Type::named("UpbitOrderVolumeWire")),
                        field("price", Type::Decimal),
                        field(
                            "time_in_force",
                            Type::optional(Type::Identifier("TimeInForce")),
                        ),
                    ],
                ),
                variant("market_buy", vec![field("price", Type::Decimal)]),
                variant(
                    "market_sell",
                    vec![field("volume", Type::named("UpbitOrderVolumeWire"))],
                ),
                variant(
                    "best_buy",
                    vec![
                        field("price", Type::Decimal),
                        field("time_in_force", Type::Identifier("TimeInForce")),
                    ],
                ),
                variant(
                    "best_sell",
                    vec![
                        field("volume", Type::named("UpbitOrderVolumeWire")),
                        field("time_in_force", Type::Identifier("TimeInForce")),
                    ],
                ),
            ],
        },
        TaggedUnion {
            name: "HyperliquidUserRoleWire",
            type_parameters: &[],
            variants: vec![
                variant("user", vec![]),
                variant("agent", vec![field("user", Type::optional(Type::String))]),
                variant("vault", vec![]),
                variant(
                    "sub_account",
                    vec![field("master", Type::optional(Type::String))],
                ),
                variant("missing", vec![]),
                variant(
                    "other",
                    vec![
                        field("role", Type::String),
                        field("data_json", Type::optional(Type::String)),
                    ],
                ),
            ],
        },
        TaggedUnion {
            name: "HyperliquidOrderReferenceWire",
            type_parameters: &[],
            variants: vec![
                variant("order_id", vec![field("value", Type::UnsignedInteger)]),
                variant("client_order_id", vec![field("value", Type::String)]),
            ],
        },
        TaggedUnion {
            name: "HyperliquidOrderStatusResponseWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "order",
                    vec![field("value", Type::named("HyperliquidOrderInfoWire"))],
                ),
                variant("unknown_order", vec![]),
                variant(
                    "other",
                    vec![
                        field("status", Type::String),
                        field("raw_json", Type::String),
                    ],
                ),
            ],
        },
        TaggedUnion {
            name: "BithumbBatchOrderOutcomeWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "accepted",
                    vec![field("value", Type::named("BithumbBatchOrderWire"))],
                ),
                variant(
                    "rejected",
                    vec![field("value", Type::named("BithumbBatchOrderFailureWire"))],
                ),
            ],
        },
        TaggedUnion {
            name: "SizeWire",
            type_parameters: &[],
            variants: vec![
                variant("base", vec![field("value", Type::Decimal)]),
                variant("quote", vec![field("value", Type::Decimal)]),
            ],
        },
        TaggedUnion {
            name: "WithdrawalFeeWire",
            type_parameters: &[],
            variants: vec![
                variant("fixed", vec![field("value", Type::Decimal)]),
                variant(
                    "rate",
                    vec![
                        field("rate", Type::Decimal),
                        field("minimum", Type::optional(Type::Decimal)),
                        field("maximum", Type::optional(Type::Decimal)),
                    ],
                ),
            ],
        },
        TaggedUnion {
            name: "TransferDestinationWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "exchange",
                    vec![field("value", Type::named("ExchangeDestinationWire"))],
                ),
                variant(
                    "chain",
                    vec![field("value", Type::named("ChainDestinationWire"))],
                ),
            ],
        },
        TaggedUnion {
            name: "TravelRuleRequirementWire",
            type_parameters: &[],
            variants: vec![
                variant("not_required", vec![]),
                variant(
                    "required",
                    vec![field("consent_url", Type::optional(Type::String))],
                ),
            ],
        },
        TaggedUnion {
            name: "FeedWire",
            type_parameters: &[],
            variants: vec![
                variant("trades", vec![]),
                variant("order_book", vec![]),
                variant("ticker", vec![]),
                variant(
                    "candles",
                    vec![field("interval", Type::Identifier("Interval"))],
                ),
            ],
        },
        TaggedUnion {
            name: "HyperliquidMarketEventWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "trade",
                    vec![field("value", Type::named("HyperliquidTradeEventWire"))],
                ),
                variant(
                    "order_book",
                    vec![field("value", Type::named("HyperliquidOrderBookEventWire"))],
                ),
                variant(
                    "asset_context",
                    vec![field(
                        "value",
                        Type::named("HyperliquidAssetContextEventWire"),
                    )],
                ),
                variant(
                    "candle",
                    vec![field("value", Type::named("HyperliquidCandleEventWire"))],
                ),
                variant("reconnected", vec![]),
            ],
        },
        TaggedUnion {
            name: "HyperliquidAccountEventWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "order_update",
                    vec![field("value", Type::named("HyperliquidOrderUpdateWire"))],
                ),
                variant(
                    "spot_state",
                    vec![field("value", Type::named("HyperliquidSpotStateEventWire"))],
                ),
                variant("reconnected", vec![]),
            ],
        },
        TaggedUnion {
            name: "MarketEventWire",
            type_parameters: &[],
            variants: vec![
                variant("trade", vec![field("trade", Type::named("TradeWire"))]),
                variant(
                    "order_book",
                    vec![field("order_book", Type::named("OrderBookWire"))],
                ),
                variant("ticker", vec![field("ticker", Type::named("TickerWire"))]),
                variant("candle", vec![field("candle", Type::named("CandleWire"))]),
                variant("reconnected", vec![]),
            ],
        },
        TaggedUnion {
            name: "AccountEventWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "balance",
                    vec![field("balance", Type::named("BalanceWire"))],
                ),
                variant("order", vec![field("order", Type::named("OrderWire"))]),
                variant("reconnected", vec![]),
            ],
        },
        TaggedUnion {
            name: "MarketStreamItemWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "event",
                    vec![field("event", Type::named("MarketEventWire"))],
                ),
                variant("error", vec![field("error", Type::named("ErrorWire"))]),
            ],
        },
        TaggedUnion {
            name: "AccountStreamItemWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "event",
                    vec![field("event", Type::named("AccountEventWire"))],
                ),
                variant("error", vec![field("error", Type::named("ErrorWire"))]),
            ],
        },
        TaggedUnion {
            name: "HyperliquidMarketStreamItemWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "event",
                    vec![field("event", Type::named("HyperliquidMarketEventWire"))],
                ),
                variant("error", vec![field("error", Type::named("ErrorWire"))]),
            ],
        },
        TaggedUnion {
            name: "HyperliquidAccountStreamItemWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "event",
                    vec![field("event", Type::named("HyperliquidAccountEventWire"))],
                ),
                variant("error", vec![field("error", Type::named("ErrorWire"))]),
            ],
        },
        TaggedUnion {
            name: "ErrorWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "invalid_request",
                    vec![field("field", Type::String), field("detail", Type::String)],
                ),
                variant(
                    "transfer",
                    vec![
                        field("transfer_kind", Type::Identifier("TransferErrorKind")),
                        field("detail", Type::String),
                    ],
                ),
                variant(
                    "unsupported",
                    vec![
                        field("feature", Type::String),
                        field("exchange", Type::String),
                        field("detail", Type::String),
                    ],
                ),
                variant("adapter", vec![field("detail", Type::String)]),
                variant("auth", vec![field("detail", Type::String)]),
                variant(
                    "exchange",
                    vec![
                        field("exchange", Type::String),
                        field("code", Type::String),
                        field("message", Type::String),
                        field("status", Type::optional(Number)),
                        field("exchange_kind", Type::String),
                    ],
                ),
                variant("transport", vec![field("detail", Type::String)]),
                variant("decode", vec![field("detail", Type::String)]),
            ],
        },
        TaggedUnion {
            name: "AdapterCallWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "markets",
                    vec![field("market_kind", Type::Identifier("MarketKind"))],
                ),
                variant(
                    "trades",
                    vec![
                        field("market", Type::named("MarketWire")),
                        field("limit", Type::optional(Number)),
                    ],
                ),
                variant(
                    "order_book",
                    vec![
                        field("market", Type::named("MarketWire")),
                        field("depth", Type::optional(Number)),
                    ],
                ),
                variant("ticker", vec![field("market", Type::named("MarketWire"))]),
                variant(
                    "candles",
                    vec![field("request", Type::named("CandleRequestWire"))],
                ),
                variant(
                    "subscribe",
                    vec![
                        field("stream_id", Type::String),
                        field("subscription", Type::named("SubscriptionWire")),
                        field("config", Type::named("StreamConfigWire")),
                    ],
                ),
                variant("balances", vec![]),
                variant(
                    "order_rules",
                    vec![field("market", Type::named("MarketWire"))],
                ),
                variant("asset_networks", vec![field("asset", Type::String)]),
                variant("deposit_addresses", vec![]),
                variant(
                    "deposit_address",
                    vec![field("request", Type::named("DepositAddressRequestWire"))],
                ),
                variant(
                    "create_deposit_address",
                    vec![field("request", Type::named("DepositAddressRequestWire"))],
                ),
                variant(
                    "prepare_withdrawal",
                    vec![field("request", Type::named("WithdrawRequestWire"))],
                ),
                variant(
                    "withdraw",
                    vec![field("request", Type::named("WithdrawRequestWire"))],
                ),
                variant(
                    "deposit",
                    vec![field("request", Type::named("TransferLookupRequestWire"))],
                ),
                variant(
                    "withdrawal",
                    vec![field("request", Type::named("TransferLookupRequestWire"))],
                ),
                variant(
                    "cancel_withdrawal",
                    vec![field("withdrawal_id", Type::String)],
                ),
                variant(
                    "deposits",
                    vec![field("request", Type::named("TransferHistoryRequestWire"))],
                ),
                variant(
                    "withdrawals",
                    vec![field("request", Type::named("TransferHistoryRequestWire"))],
                ),
                variant(
                    "open_orders",
                    vec![field("market", Type::optional(Type::named("MarketWire")))],
                ),
                variant(
                    "order",
                    vec![
                        field("market", Type::named("MarketWire")),
                        field("order_id", Type::String),
                    ],
                ),
                variant(
                    "order_by_client_id",
                    vec![
                        field("market", Type::named("MarketWire")),
                        field("client_id", Type::String),
                    ],
                ),
                variant(
                    "orders_by_ids",
                    vec![field("request", Type::named("OrderLookupRequestWire"))],
                ),
                variant(
                    "order_history",
                    vec![field("request", Type::named("OrderHistoryRequestWire"))],
                ),
                variant(
                    "subscribe_account",
                    vec![
                        field("stream_id", Type::String),
                        field("config", Type::named("StreamConfigWire")),
                    ],
                ),
                variant(
                    "place_order",
                    vec![field("request", Type::named("OrderRequestWire"))],
                ),
                variant(
                    "cancel_order",
                    vec![
                        field("market", Type::named("MarketWire")),
                        field("order_id", Type::String),
                    ],
                ),
                variant(
                    "cancel_order_by_client_id",
                    vec![
                        field("market", Type::named("MarketWire")),
                        field("client_id", Type::String),
                    ],
                ),
                variant(
                    "cancel_orders",
                    vec![field("request", Type::named("CancelOrdersRequestWire"))],
                ),
                variant(
                    "positions",
                    vec![field("market", Type::optional(Type::named("MarketWire")))],
                ),
                variant("margin_summary", vec![]),
                variant(
                    "funding_rates",
                    vec![field("request", Type::named("HistoryRequestWire"))],
                ),
                variant(
                    "funding_payments",
                    vec![field("request", Type::named("HistoryRequestWire"))],
                ),
                variant(
                    "set_margin",
                    vec![field("request", Type::named("MarginRequestWire"))],
                ),
            ],
        },
        TaggedUnion {
            name: "AdapterReplyWire",
            type_parameters: &[],
            variants: vec![
                variant(
                    "markets",
                    vec![field("value", Type::list(Type::named("MarketInfoWire")))],
                ),
                variant(
                    "trades",
                    vec![field("value", Type::list(Type::named("TradeWire")))],
                ),
                variant(
                    "order_book",
                    vec![field("value", Type::named("OrderBookWire"))],
                ),
                variant("ticker", vec![field("value", Type::named("TickerWire"))]),
                variant(
                    "candles",
                    vec![field("value", Type::list(Type::named("CandleWire")))],
                ),
                variant("market_stream", vec![field("stream_id", Type::String)]),
                variant(
                    "balances",
                    vec![field("value", Type::list(Type::named("BalanceWire")))],
                ),
                variant(
                    "order_rules",
                    vec![field("value", Type::named("OrderRulesWire"))],
                ),
                variant(
                    "asset_networks",
                    vec![field("value", Type::list(Type::named("AssetNetworkWire")))],
                ),
                variant(
                    "deposit_addresses",
                    vec![field(
                        "value",
                        Type::list(Type::named("DepositAddressEntryWire")),
                    )],
                ),
                variant(
                    "deposit_address",
                    vec![field("value", Type::named("DepositAddressWire"))],
                ),
                variant(
                    "create_deposit_address",
                    vec![field("value", Type::named("DepositAddressWire"))],
                ),
                variant(
                    "withdrawal_quote",
                    vec![field("value", Type::named("WithdrawalQuoteWire"))],
                ),
                variant(
                    "withdrawal",
                    vec![field("value", Type::named("WithdrawalWire"))],
                ),
                variant("deposit", vec![field("value", Type::named("DepositWire"))]),
                variant(
                    "withdrawal_lookup",
                    vec![field("value", Type::named("WithdrawalWire"))],
                ),
                variant(
                    "deposits",
                    vec![field("value", Type::named("PageWire<DepositWire>"))],
                ),
                variant(
                    "withdrawals",
                    vec![field("value", Type::named("PageWire<WithdrawalWire>"))],
                ),
                variant(
                    "open_orders",
                    vec![field("value", Type::list(Type::named("OrderWire")))],
                ),
                variant("order", vec![field("value", Type::named("OrderWire"))]),
                variant(
                    "orders_by_ids",
                    vec![field("value", Type::list(Type::named("OrderWire")))],
                ),
                variant(
                    "order_history",
                    vec![field("value", Type::named("PageWire<OrderWire>"))],
                ),
                variant("account_stream", vec![field("stream_id", Type::String)]),
                variant(
                    "place_order",
                    vec![field("value", Type::named("OrderWire"))],
                ),
                variant(
                    "cancel_orders",
                    vec![field("value", Type::named("CancelOrdersResultWire"))],
                ),
                variant(
                    "positions",
                    vec![field("value", Type::list(Type::named("PositionWire")))],
                ),
                variant(
                    "margin_summary",
                    vec![field("value", Type::named("MarginSummaryWire"))],
                ),
                variant(
                    "funding_rates",
                    vec![field("value", Type::named("PageWire<FundingRateWire>"))],
                ),
                variant(
                    "funding_payments",
                    vec![field("value", Type::named("PageWire<FundingPaymentWire>"))],
                ),
                variant("unit", vec![]),
            ],
        },
    ];

    Schema {
        native_api_version: 30,
        exchanges: Exchange::ALL.into_iter().map(Exchange::id).collect(),
        features: Feature::ALL.into_iter().map(Feature::id).collect(),
        identifiers: IDENTIFIERS,
        models: MODELS,
        errors: ERRORS,
        adapter_operations: ADAPTER_OPERATIONS,
        client_compositions: CLIENT_COMPOSITIONS,
        client_members: CLIENT_MEMBERS,
        providers: PROVIDERS,
        products: PRODUCTS,
        records,
        unions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_names_are_unique() {
        let schema = binding_schema();
        let mut names = schema
            .records
            .iter()
            .map(|item| item.name)
            .collect::<Vec<_>>();
        names.extend(schema.unions.iter().map(|item| item.name));
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn client_inventory_is_derived_from_operations() {
        let schema = binding_schema();
        let mut generated = schema
            .adapter_operations
            .iter()
            .flat_map(|operation| operation.client_methods)
            .map(|method| method.name)
            .collect::<Vec<_>>();
        generated.extend(
            schema
                .client_compositions
                .iter()
                .map(|method| method.language_name),
        );
        generated.extend(["exchange", "supports", "adapter"]);
        generated.sort_unstable();
        let mut declared = schema.client_members.to_vec();
        declared.sort_unstable();
        assert_eq!(generated, declared);
    }

    #[test]
    fn provider_method_names_are_unique() {
        for provider in binding_schema().providers {
            let mut names = provider
                .methods
                .iter()
                .map(|method| method.name)
                .collect::<Vec<_>>();
            let count = names.len();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), count, "{} methods", provider.adapter);
        }
    }
}
