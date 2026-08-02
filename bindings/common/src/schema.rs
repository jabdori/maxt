//! Repository binding schema used by `maxt-bindings-codegen`.

use maxt::{Exchange, Feature};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Boolean,
    Number,
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
pub struct Operation {
    pub rust_name: &'static str,
    pub language_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub exchange: &'static str,
    pub adapter: &'static str,
    pub methods: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub exchanges: Vec<&'static str>,
    pub features: Vec<&'static str>,
    pub errors: &'static [&'static str],
    pub adapter_operations: &'static [Operation],
    pub client_members: &'static [&'static str],
    pub providers: &'static [Provider],
    pub records: Vec<Record>,
    pub unions: Vec<TaggedUnion>,
}

const ADAPTER_OPERATIONS: &[Operation] = &[
    Operation {
        rust_name: "markets",
        language_name: "markets",
    },
    Operation {
        rust_name: "trades",
        language_name: "trades",
    },
    Operation {
        rust_name: "order_book",
        language_name: "orderBook",
    },
    Operation {
        rust_name: "ticker",
        language_name: "ticker",
    },
    Operation {
        rust_name: "candles",
        language_name: "candles",
    },
    Operation {
        rust_name: "subscribe",
        language_name: "subscribe",
    },
    Operation {
        rust_name: "balances",
        language_name: "balances",
    },
    Operation {
        rust_name: "open_orders",
        language_name: "openOrders",
    },
    Operation {
        rust_name: "subscribe_account",
        language_name: "subscribeAccount",
    },
    Operation {
        rust_name: "place_order",
        language_name: "placeOrder",
    },
    Operation {
        rust_name: "cancel_order",
        language_name: "cancelOrder",
    },
    Operation {
        rust_name: "positions",
        language_name: "positions",
    },
    Operation {
        rust_name: "margin_summary",
        language_name: "marginSummary",
    },
    Operation {
        rust_name: "funding_rates",
        language_name: "fundingRates",
    },
    Operation {
        rust_name: "funding_payments",
        language_name: "fundingPayments",
    },
    Operation {
        rust_name: "set_margin",
        language_name: "setMargin",
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

const PROVIDERS: &[Provider] = &[
    Provider {
        exchange: "upbit",
        adapter: "UpbitAdapter",
        methods: &["region", "orderBooks", "tickers", "marketEvents"],
    },
    Provider {
        exchange: "bithumb",
        adapter: "BithumbAdapter",
        methods: &["marketWarnings", "marketAlerts"],
    },
    Provider {
        exchange: "binance",
        adapter: "BinanceAdapter",
        methods: &[
            "venue",
            "spotSymbolFilters",
            "spotOrder",
            "usdMCreateListenKey",
            "usdMKeepaliveListenKey",
            "usdMCloseListenKey",
        ],
    },
    Provider {
        exchange: "hyperliquid",
        adapter: "HyperliquidAdapter",
        methods: &["isTestnet", "nonFundingLedger", "assetContext"],
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
    let decimal = Type::named("DecimalWire");
    let timestamp = Type::named("TimestampWire");
    let records = vec![
        record(
            "MarketWire",
            vec![
                field("exchange", Type::String),
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
                field("status", Type::String),
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
                field("taker_side", Type::String),
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
                field("interval", Type::String),
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
                field("side", Type::String),
                field("status", Type::String),
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
                field("side", Type::optional(Type::String)),
                field("quantity", decimal.clone()),
                field("entry_price", Type::optional(decimal.clone())),
                field("mark_price", Type::optional(decimal.clone())),
                field("notional", Type::optional(decimal.clone())),
                field("unrealized_pnl", Type::optional(decimal.clone())),
                field("leverage", Type::optional(decimal.clone())),
                field("margin_mode", Type::optional(Type::String)),
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
                field("interval", Type::String),
                field("from", Type::optional(timestamp.clone())),
                field("to", Type::optional(timestamp.clone())),
                field("limit", Type::optional(Number)),
            ],
        ),
        record(
            "OrderRequestWire",
            vec![
                field("market", market.clone()),
                field("side", Type::String),
                field("order_type", Type::String),
                field("size", Type::named("SizeWire")),
                field("price", Type::optional(decimal.clone())),
                field("time_in_force", Type::optional(Type::String)),
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
                field("overflow", Type::String),
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
                field("margin_mode", Type::optional(Type::String)),
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
                field("step", Type::String),
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
                field("kind", Type::String),
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
                field("region", Type::String),
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
                field("venue", Type::String),
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
        exchanges: Exchange::ALL.into_iter().map(Exchange::id).collect(),
        features: Feature::ALL.into_iter().map(Feature::id).collect(),
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
}
