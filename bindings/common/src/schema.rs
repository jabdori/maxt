//! Repository binding schema used by `maxt-bindings-codegen`.

use maxt::{Exchange, Feature};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Boolean,
    Number,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub exchange: &'static str,
    pub adapter: &'static str,
    pub native_handle: &'static str,
    pub native_factory: &'static str,
    pub options_wire: &'static str,
    pub constructors: &'static [&'static str],
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
    pub client_members: &'static [&'static str],
    pub providers: &'static [Provider],
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
const CANCEL_ORDER: &[Argument] = &[
    argument("market", ApiType::Named("Market"), None),
    argument("orderId", ApiType::String, None),
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
        rust_name: "open_orders",
        language_name: "openOrders",
        feature: "open_orders",
        arguments: OPTIONAL_MARKET,
        result: ApiType::List("Order"),
        client_methods: CLIENT_OPEN_ORDERS,
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
        result: ApiType::Named("Order"),
        client_methods: CLIENT_CANCEL_ORDER,
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
    "openOrders",
    "openOrdersOn",
    "subscribeAccount",
    "subscribeAccountWith",
    "placeOrder",
    "cancelOrder",
    "positions",
    "positionsOn",
    "marginSummary",
    "fundingRates",
    "fundingPayments",
    "setMargin",
];

const ERRORS: &[&str] = &[
    "InvalidRequest",
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
    identifier_variant("OpenOrders", "open_orders"),
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
    identifier_variant("Min15", "min15"),
    identifier_variant("Min30", "min30"),
    identifier_variant("Hour1", "hour1"),
    identifier_variant("Hour2", "hour2"),
    identifier_variant("Hour4", "hour4"),
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
const ORDER_TYPE_VARIANTS: &[IdentifierVariant] = &[
    identifier_variant("Market", "market"),
    identifier_variant("Limit", "limit"),
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
    "Order",
    "Position",
    "MarginSummary",
    "FundingRate",
    "FundingPayment",
    "CandleRequest",
    "OrderRequest",
    "StreamConfig",
    "Subscription",
    "HistoryRequest",
    "MarginRequest",
    "UpbitMarketEvent",
    "BithumbMarketAlert",
    "BinanceSymbolFilters",
    "BinanceSpotOrderDetail",
    "HyperliquidLedgerEntry",
    "HyperliquidAssetContext",
];

const MARKETS_DEPTH: &[Argument] = &[
    argument("markets", ApiType::List("Market"), None),
    argument("depth", ApiType::OptionalNumber, Some("null")),
];
const MARKETS: &[Argument] = &[argument("markets", ApiType::List("Market"), None)];
const LISTEN_KEY: &[Argument] = &[argument(
    "key",
    ApiType::HandleToken("BinanceListenKey"),
    None,
)];
const LEDGER_RANGE: &[Argument] = &[
    argument("from", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("to", ApiType::OptionalNamed("Timestamp"), Some("null")),
    argument("cursor", ApiType::OptionalNamed("Cursor"), Some("null")),
    argument("limit", ApiType::OptionalNumber, Some("null")),
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
        rust_name: "tickers",
        name: "tickers",
        kind: ProviderMethodKind::Async,
        arguments: MARKETS,
        result: ApiType::List("Ticker"),
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
        arguments: LISTEN_KEY,
        result: ApiType::Unit,
    },
    ProviderMethod {
        rust_name: "usd_m_close_listen_key",
        name: "usdMCloseListenKey",
        kind: ProviderMethodKind::Async,
        arguments: LISTEN_KEY,
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
        constructors: &["constructor", "withRegion"],
        methods: UPBIT_METHODS,
    },
    Provider {
        exchange: "bithumb",
        adapter: "BithumbAdapter",
        native_handle: "NativeBithumbHandle",
        native_factory: "createBithumb",
        options_wire: "BithumbOptionsWire",
        constructors: &["constructor"],
        methods: BITHUMB_METHODS,
    },
    Provider {
        exchange: "binance",
        adapter: "BinanceAdapter",
        native_handle: "NativeBinanceHandle",
        native_factory: "createBinance",
        options_wire: "BinanceOptionsWire",
        constructors: &["spot", "usdMFutures"],
        methods: BINANCE_METHODS,
    },
    Provider {
        exchange: "hyperliquid",
        adapter: "HyperliquidAdapter",
        native_handle: "NativeHyperliquidHandle",
        native_factory: "createHyperliquid",
        options_wire: "HyperliquidOptionsWire",
        constructors: &["constructor", "testnet"],
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
                field("kind", Type::named("MarketKindWire")),
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
            ],
        ),
        record(
            "StreamConfigWire",
            vec![
                field("max_reconnect_attempts", Type::optional(Number)),
                field("initial_reconnect_delay_ms", Type::String),
                field("max_reconnect_delay_ms", Type::String),
                field("idle_timeout_ms", Type::String),
                field("buffer_size", Type::String),
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
                field("market", market),
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
            "BithumbMarketAlertWire",
            vec![
                field("kind", Type::String),
                field("step", Type::Identifier("BithumbAlertStep")),
                field("ends_at", timestamp.clone()),
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
                variant("base", vec![field("value", Type::named("DecimalWire"))]),
                variant("quote", vec![field("value", Type::named("DecimalWire"))]),
            ],
        },
        TaggedUnion {
            name: "FeedWire",
            type_parameters: &[],
            variants: vec![
                variant("trades", vec![]),
                variant("order_book", vec![]),
                variant("ticker", vec![]),
                variant("candles", vec![field("interval", Type::String)]),
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
                    vec![field("market_kind", Type::named("MarketKindWire"))],
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
                    "open_orders",
                    vec![field("market", Type::optional(Type::named("MarketWire")))],
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
                    "open_orders",
                    vec![field("value", Type::list(Type::named("OrderWire")))],
                ),
                variant("account_stream", vec![field("stream_id", Type::String)]),
                variant(
                    "place_order",
                    vec![field("value", Type::named("OrderWire"))],
                ),
                variant(
                    "cancel_order",
                    vec![field("value", Type::named("OrderWire"))],
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
        native_api_version: 1,
        exchanges: Exchange::ALL.into_iter().map(Exchange::id).collect(),
        features: Feature::ALL.into_iter().map(Feature::id).collect(),
        identifiers: IDENTIFIERS,
        models: MODELS,
        errors: ERRORS,
        adapter_operations: ADAPTER_OPERATIONS,
        client_members: CLIENT_MEMBERS,
        providers: PROVIDERS,
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
