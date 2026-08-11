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
const ASSET: &[Argument] = &[argument("asset", ApiType::String, None)];
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
const BITHUMB_ALERT_STEP_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Caution", "caution"),
    identifier_variant("Warning", "warning"),
    identifier_variant("Danger", "danger"),
    identifier_variant("Unknown", "unknown"),
];
const BINANCE_MARKET_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Spot", "spot"),
    identifier_variant("UsdMFutures", "usd_m"),
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
    identifier("BithumbAlertStep", BITHUMB_ALERT_STEP_VARIANTS, false),
    identifier("BinanceMarket", BINANCE_MARKET_VARIANTS, false),
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
    "UpbitYearCandle",
    "UpbitOrderBookInstrument",
    "BithumbMarketAlert",
    "BithumbNotice",
    "BinanceSymbolFilters",
    "BinanceSpotOrderDetail",
    "HyperliquidLedgerEntry",
    "HyperliquidAssetContext",
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
    argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("count", ApiType::OptionalNumber, Some("null")),
];
const NOTICE_COUNT: &[Argument] = &[argument("count", ApiType::OptionalNumber, Some("null"))];
const LEDGER_RANGE: &[Argument] = &[
    argument("from", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("cursor", ApiType::OptionalNamed("Cursor"), Some("null")),
    argument("limit", ApiType::OptionalNumber, Some("null")),
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
            "BinanceListenKeyWire",
            vec![field("id", Type::String), field("value", Type::String)],
        ),
        record(
            "HyperliquidLedgerEntryWire",
            vec![
                field("kind", Type::Identifier("HyperliquidLedgerKind")),
                field("time", timestamp),
                field("hash", Type::String),
                field("asset", Type::optional(Type::String)),
                field("amount", Type::optional(decimal.clone())),
                field("fee", Type::optional(decimal.clone())),
                field("counterparty", Type::optional(Type::String)),
            ],
        ),
        record(
            "HyperliquidAssetContextWire",
            vec![
                field("mid_price", Type::optional(decimal.clone())),
                field("mark_price", Type::optional(decimal.clone())),
                field("oracle_price", Type::optional(decimal.clone())),
                field("funding_rate", Type::optional(decimal.clone())),
                field("open_interest", Type::optional(decimal)),
                field("size_decimals", Number),
                field("price_decimals", Number),
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
        native_api_version: 13,
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
