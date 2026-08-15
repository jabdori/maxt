use maxt_bindings_common::schema::Schema;

/// A user-facing task that has one or more runnable examples.
///
/// This is intentionally documentation-only metadata. The schema knows method
/// signatures, but it cannot choose safe credentials, markets, or write
/// permissions for executable source examples.
#[derive(Clone, Copy)]
pub(crate) struct ExampleScenario {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) safety: &'static str,
    pub(crate) description: &'static str,
    pub(crate) rust: Option<&'static str>,
    pub(crate) python: Option<&'static str>,
    pub(crate) dart: Option<&'static str>,
    pub(crate) typescript: Option<&'static str>,
}

const SCENARIOS: &[ExampleScenario] = &[
    ExampleScenario {
        id: "market-data",
        title: "Read public market data",
        safety: "public read",
        description: "List markets, then read a ticker, order-book snapshot, and recent trades without account configuration.",
        rust: Some("examples/binance_first_read.rs"),
        python: Some("bindings/python/python/maxt/examples/binance_public_ticker.py"),
        dart: Some("bindings/dart/example/main.dart"),
        typescript: Some("bindings/typescript/examples/binance-public-ticker.mjs"),
    },
    ExampleScenario {
        id: "candles-and-history",
        title: "Read candles and paged history",
        safety: "public or credentialed read",
        description: "Read time-bounded candles and learn the cursor pattern used by private order, transfer, and funding history.",
        rust: Some("examples/candles_history.rs"),
        python: Some("bindings/python/python/maxt/examples/candles_history.py"),
        dart: Some("bindings/dart/example/candles_history.dart"),
        typescript: Some("bindings/typescript/examples/candles-history.mjs"),
    },
    ExampleScenario {
        id: "streams",
        title: "Receive public and account streams",
        safety: "public read or credentialed read",
        description: "Subscribe to a market feed, handle individual stream errors and reconnect gaps, and close the stream deterministically.",
        rust: Some("examples/public_stream.rs"),
        python: Some("bindings/python/python/maxt/examples/streams.py"),
        dart: Some("bindings/dart/example/streams.dart"),
        typescript: Some("bindings/typescript/examples/streams.mjs"),
    },
    ExampleScenario {
        id: "account-and-assets",
        title: "Read balances, rules, and asset configuration",
        safety: "credentialed read",
        description: "Use a read-enabled account to inspect balances and open orders while keeping requests and credentials out of source code.",
        rust: Some("examples/account_and_safety.rs"),
        python: Some("bindings/python/python/maxt/examples/account_and_safety.py"),
        dart: Some("bindings/dart/example/account_and_safety.dart"),
        typescript: Some("bindings/typescript/examples/account-and-safety.mjs"),
    },
    ExampleScenario {
        id: "orders-and-safety",
        title: "Inspect and validate order workflows safely",
        safety: "request only or credentialed read",
        description: "Build order requests locally and use provider dry-run validation when available; the checked-in examples do not submit a live order.",
        rust: Some("examples/account_and_safety.rs"),
        python: Some("bindings/python/python/maxt/examples/account_and_safety.py"),
        dart: Some("bindings/dart/example/account_and_safety.dart"),
        typescript: Some("bindings/typescript/examples/account-and-safety.mjs"),
    },
    ExampleScenario {
        id: "transfers-and-wallet",
        title: "Inspect transfers and wallet configuration",
        safety: "credentialed read or request only",
        description: "Build wallet and transfer requests, inspect configured account data, and leave any financial submission to the application.",
        rust: Some("examples/account_and_safety.rs"),
        python: Some("bindings/python/python/maxt/examples/account_and_safety.py"),
        dart: Some("bindings/dart/example/account_and_safety.dart"),
        typescript: Some("bindings/typescript/examples/account-and-safety.mjs"),
    },
    ExampleScenario {
        id: "derivatives",
        title: "Read funding, positions, and margin",
        safety: "public or credentialed read",
        description: "Read public perpetual market context first, then use read-enabled account operations for positions, margin, and funding payments.",
        rust: Some("examples/derivatives.rs"),
        python: Some("bindings/python/python/maxt/examples/derivatives.py"),
        dart: Some("bindings/dart/example/derivatives.dart"),
        typescript: Some("bindings/typescript/examples/derivatives.mjs"),
    },
    ExampleScenario {
        id: "binance-provider",
        title: "Use Binance Spot, USD-M, and wallet APIs",
        safety: "public, credentialed read, or request only",
        description: "Choose a Spot or USD-M adapter deliberately and use Binance provider calls when their additional response fields matter.",
        rust: Some("examples/binance_provider.rs"),
        python: Some("bindings/python/python/maxt/examples/binance_provider.py"),
        dart: Some("bindings/dart/example/binance_provider.dart"),
        typescript: Some("bindings/typescript/examples/binance-provider.mjs"),
    },
    ExampleScenario {
        id: "upbit-provider",
        title: "Use Upbit region and provider-specific APIs",
        safety: "public, credentialed read, or request only",
        description: "Select an Upbit region before construction and use Korea-only, Travel Rule, pocket, or detailed response APIs only where applicable.",
        rust: Some("examples/upbit_provider.rs"),
        python: Some("bindings/python/python/maxt/examples/upbit_provider.py"),
        dart: Some("bindings/dart/example/upbit_provider.dart"),
        typescript: Some("bindings/typescript/examples/upbit-provider.mjs"),
    },
    ExampleScenario {
        id: "bithumb-provider",
        title: "Use Bithumb market, KRW, and TWAP APIs",
        safety: "public, credentialed read, or request only",
        description: "Start with public warnings, notices, and fee metadata; use account, KRW, and TWAP provider calls only with the required Bithumb permissions.",
        rust: Some("examples/bithumb_provider.rs"),
        python: Some("bindings/python/python/maxt/examples/bithumb_provider.py"),
        dart: Some("bindings/dart/example/bithumb_provider.dart"),
        typescript: Some("bindings/typescript/examples/bithumb-provider.mjs"),
    },
    ExampleScenario {
        id: "hyperliquid-provider",
        title: "Use Hyperliquid market and address-scoped Info APIs",
        safety: "public or address-scoped read",
        description: "Read market Info without credentials, then supply only a public address for address-scoped Info reads; signed actions require a separate signer.",
        rust: Some("examples/hyperliquid_provider.rs"),
        python: Some("bindings/python/python/maxt/examples/hyperliquid_provider.py"),
        dart: Some("bindings/dart/example/hyperliquid_provider.dart"),
        typescript: Some("bindings/typescript/examples/hyperliquid-provider.mjs"),
    },
    ExampleScenario {
        id: "browser-relay",
        title: "Use public reads in a browser and route signed calls through a relay",
        safety: "browser boundary",
        description: "Use direct browser transport for public reads; configure a trusted relay and explicit opt-in before supplying browser credentials.",
        rust: None,
        python: None,
        dart: Some("bindings/dart/example/browser_relay.dart"),
        typescript: Some("bindings/typescript/examples/browser-relay.mjs"),
    },
];

const UPBIT_METHODS: &[&str] = &[
    "region",
    "order_books",
    "order_books_at_level",
    "tickers",
    "tickers_by_quote",
    "year_candles",
    "orderbook_instruments",
    "market_events",
    "list_subscriptions",
    "test_order",
    "order_detail",
    "closed_orders",
    "deposit_info",
    "withdrawal_addresses",
    "travel_rule_vasps",
    "verify_travel_rule_by_uuid",
    "verify_travel_rule_by_txid",
    "batch_cancel_open_orders",
    "cancel_and_new_order",
    "deposit_krw",
    "withdraw_krw",
    "api_keys",
    "list_pockets",
    "list_pocket_api_keys",
    "sub_pocket_balances",
    "universal_transfer",
    "universal_transfers",
    "sub_pocket_transfer",
    "sub_pocket_transfers",
    "subscribe_detailed",
    "subscribe_detailed_with",
    "subscribe_detailed_account",
    "subscribe_detailed_account_with",
    "test_order_detail",
    "place_order_detail",
    "cancel_order_detail",
    "cancel_order_by_client_id_detail",
    "orders_by_ids_detail",
    "cancel_orders_detail",
    "deposit_detail",
    "withdrawal_detail",
    "cancel_withdrawal_detail",
    "cancel_and_new_order_detail",
];

const BITHUMB_METHODS: &[&str] = &[
    "market_warnings",
    "market_alerts",
    "notices",
    "transfer_fees",
    "api_keys",
    "krw_withdrawals",
    "withdraw_krw",
    "krw_deposits",
    "deposit_krw",
    "pending_orders",
    "closed_orders",
    "batch_orders",
    "twap_orders",
    "create_twap_order",
    "cancel_twap_order",
    "withdrawal_addresses",
    "order_detail",
    "order_list",
    "order_book_snapshot",
    "subscribe_detailed",
    "subscribe_detailed_with",
    "subscribe_detailed_account",
    "subscribe_detailed_account_with",
    "orders_by_ids_detail",
    "place_order_detail",
    "cancel_order_detail",
    "cancel_order_by_client_id_detail",
    "cancel_orders_detail",
    "deposit_detail",
    "withdrawal_detail",
    "cancel_withdrawal_detail",
];

const BINANCE_METHODS: &[&str] = &[
    "venue",
    "spot_symbol_filters",
    "spot_order",
    "spot_average_price",
    "spot_account_information",
    "spot_cancel_all_open_orders",
    "spot_exchange_info",
    "usd_m_account_information",
    "usd_m_exchange_info",
    "usd_m_position_information",
    "all_coins_information",
    "api_key_permissions",
    "deposit_history",
    "questionnaire_requirements",
    "withdraw_address_list",
    "withdraw_history",
    "mark_price",
    "mark_prices",
    "open_interest",
    "aggregate_trades",
    "account_trades",
    "c2c_trade_history",
    "test_order",
    "cancel_all_open_orders",
    "usd_m_create_listen_key",
    "usd_m_keepalive_listen_key",
    "usd_m_close_listen_key",
    "place_order_detail",
    "cancel_order_detail",
    "cancel_order_by_client_id_detail",
    "subscribe_detailed",
    "subscribe_detailed_with",
    "subscribe_detailed_account",
    "subscribe_detailed_account_with",
];

const HYPERLIQUID_METHODS: &[&str] = &[
    "is_testnet",
    "all_mids",
    "subscribe_detailed",
    "subscribe_detailed_with",
    "subscribe_detailed_account",
    "subscribe_detailed_account_with",
    "user_fills",
    "user_fills_by_time",
    "basic_open_orders",
    "order_status",
    "historical_orders",
    "non_funding_ledger",
    "asset_context",
    "candle_snapshot",
    "l2_book",
    "recent_trades",
    "funding_history",
    "user_funding",
    "spot_clearinghouse_state",
    "spot_meta",
    "spot_meta_and_asset_contexts",
    "user_rate_limit",
    "user_role",
    "referral",
    "user_fees",
    "portfolio",
    "sub_accounts",
    "user_vault_equities",
    "all_mids_detail",
    "perpetual_meta",
    "perpetual_meta_and_asset_contexts",
    "clearinghouse_state_detail",
    "frontend_open_orders_detail",
    "place_order_detail",
    "cancel_order_detail",
];

pub(crate) fn all() -> &'static [ExampleScenario] {
    SCENARIOS
}

pub(crate) fn common(method: &str) -> Option<&'static ExampleScenario> {
    let id = match method {
        "markets" | "trades" | "order_book" | "ticker" => "market-data",
        "candles" | "deposits" | "withdrawals" | "open_orders" | "order" | "order_by_client_id"
        | "orders_by_ids" | "order_history" | "funding_rates" | "funding_payments" => {
            "candles-and-history"
        }
        "subscribe" | "subscribe_account" => "streams",
        "balances" | "order_rules" | "asset_networks" | "deposit_addresses" | "deposit_address"
        | "positions" | "margin_summary" => "account-and-assets",
        "place_order"
        | "cancel_order"
        | "cancel_order_by_client_id"
        | "cancel_orders"
        | "set_margin" => "orders-and-safety",
        "create_deposit_address"
        | "prepare_withdrawal"
        | "withdraw"
        | "deposit"
        | "withdrawal"
        | "cancel_withdrawal" => "transfers-and-wallet",
        _ => return None,
    };
    scenario(id)
}

pub(crate) fn provider(exchange: &str, method: &str) -> Option<&'static ExampleScenario> {
    let (methods, id) = match exchange {
        "upbit" => (UPBIT_METHODS, "upbit-provider"),
        "bithumb" => (BITHUMB_METHODS, "bithumb-provider"),
        "binance" => (BINANCE_METHODS, "binance-provider"),
        "hyperliquid" => (HYPERLIQUID_METHODS, "hyperliquid-provider"),
        _ => return None,
    };
    methods.contains(&method).then(|| scenario(id)).flatten()
}

pub(crate) fn validate(schema: &Schema) -> Result<(), String> {
    for operation in schema.adapter_operations {
        if common(operation.rust_name).is_none() {
            return Err(format!(
                "missing example scenario for adapter method `{}`",
                operation.rust_name
            ));
        }
    }
    for provider_schema in schema.providers {
        for method in provider_schema.methods {
            if provider(provider_schema.exchange, method.rust_name).is_none() {
                return Err(format!(
                    "missing example scenario for provider method `{}.{}`",
                    provider_schema.exchange, method.rust_name
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn render_catalog(schema: &Schema) -> String {
    validate(schema).expect("every public API must have an example scenario");
    let mut output = String::from(
        "<!-- Generated by `cargo run -p maxt-bindings-codegen -- rust`. Do not edit. -->\n\n# Examples\n\nExamples are organized by user task rather than one file per endpoint. Every current public API is linked to one scenario below; the final tables are generated from the binding schema and fail generation when an API has no scenario.\n\nPublic reads run against the exchange. Credentialed examples stop with setup instructions when credentials are absent. Request-only examples never submit an order, transfer, or withdrawal by default.\n\n## Scenarios\n\n| Scenario | Safety | Rust | Python | Dart | TypeScript |\n| --- | --- | --- | --- | --- | --- |\n",
    );
    for item in all() {
        output.push_str(&format!(
            "| [{}](#{}) | {} | {} | {} | {} | {} |\n",
            item.title,
            item.id,
            item.safety,
            link("..", item.rust),
            link("..", item.python),
            link("..", item.dart),
            link("..", item.typescript),
        ));
    }
    for item in all() {
        output.push_str(&format!(
            "\n### {}\n\n{}\n\nSafety: **{}**.\n",
            item.title, item.description, item.safety
        ));
    }
    output.push_str("\n## API-to-scenario map\n\n### Common API\n\n| Rust / Python | Dart / TypeScript | Scenario |\n| --- | --- | --- |\n");
    for operation in schema.adapter_operations {
        let item = common(operation.rust_name).expect("validated above");
        output.push_str(&format!(
            "| `{}` | `{}` | [{}](#{}) |\n",
            operation.rust_name, operation.language_name, item.title, item.id
        ));
    }
    output.push_str("\n### Provider-specific API\n\n| Exchange | Rust / Python | Dart / TypeScript | Scenario |\n| --- | --- | --- | --- |\n");
    for provider_schema in schema.providers {
        for method in provider_schema.methods {
            let item =
                provider(provider_schema.exchange, method.rust_name).expect("validated above");
            output.push_str(&format!(
                "| {} | `{}` | `{}` | [{}](#{}) |\n",
                provider_schema.exchange, method.rust_name, method.name, item.title, item.id
            ));
        }
    }
    output
}

pub(crate) fn render_directory_index(language: &str) -> String {
    let (heading, run_context, first_read) = match language {
        "rust" => (
            "Rust examples",
            "From a repository checkout, run:",
            "cargo run --example binance_first_read",
        ),
        "python" => (
            "Python examples",
            "After installing the package, run:",
            "python -m maxt.examples.binance_public_ticker",
        ),
        "dart" => (
            "Dart examples",
            "From a repository or package checkout, run:",
            "dart run example/main.dart",
        ),
        "typescript" => (
            "TypeScript examples",
            "From a repository or package checkout, run:",
            "node examples/binance-public-ticker.mjs",
        ),
        _ => panic!("unsupported example language: {language}"),
    };
    let mut output = format!(
        "<!-- Generated by `cargo run -p maxt-bindings-codegen -- rust`. Do not edit. -->\n\n# {heading}\n\nRun the package's first-install steps before using an example. See the repository [example guide](https://github.com/jabdori/maxt/blob/main/docs/examples.md). Public reads run normally; credentialed reads require a read-enabled account; request-only examples do not submit financial writes.\n\n## Run the first public read\n\n{run_context}\n\n```text\n{first_read}\n```\n\n| Task | Safety | Example |\n| --- | --- | --- |\n"
    );
    for item in all() {
        let path = match language {
            "rust" => item.rust,
            "python" => item.python,
            "dart" => item.dart,
            "typescript" => item.typescript,
            _ => unreachable!(),
        };
        if let Some(path) = path {
            output.push_str(&format!(
                "| {} | {} | [{}]({}) |\n",
                item.title,
                item.safety,
                path.rsplit('/')
                    .next()
                    .expect("example path has a file name"),
                relative_path(language, path),
            ));
        }
    }
    output
}

fn scenario(id: &str) -> Option<&'static ExampleScenario> {
    SCENARIOS.iter().find(|item| item.id == id)
}

fn link(prefix: &str, path: Option<&str>) -> String {
    path.map_or_else(
        || "—".to_owned(),
        |path| {
            let file = path
                .rsplit('/')
                .next()
                .expect("example path has a file name");
            format!("[{file}]({prefix}/{path})")
        },
    )
}

fn relative_path(language: &str, path: &str) -> String {
    match language {
        "rust" => path.trim_start_matches("examples/").to_owned(),
        "python" => path
            .trim_start_matches("bindings/python/python/maxt/examples/")
            .to_owned(),
        "dart" => path.trim_start_matches("bindings/dart/example/").to_owned(),
        "typescript" => path
            .trim_start_matches("bindings/typescript/examples/")
            .to_owned(),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maxt_bindings_common::schema::binding_schema;
    use std::path::{Path, PathBuf};

    #[test]
    fn every_schema_method_has_an_explicit_example_scenario() {
        validate(&binding_schema()).expect("missing scenario mapping");
    }

    #[test]
    fn catalog_exposes_each_language_path_when_the_scenario_supports_it() {
        let output = render_catalog(&binding_schema());
        assert!(output.contains("bindings/python/python/maxt/examples/binance_provider.py"));
        assert!(output.contains("bindings/dart/example/browser_relay.dart"));
        assert!(output.contains("bindings/typescript/examples/hyperliquid-provider.mjs"));
    }

    #[test]
    fn every_documented_example_source_exists() {
        let codegen_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = codegen_dir
            .parent()
            .and_then(Path::parent)
            .expect("codegen crate must be below the repository root");
        for scenario in all() {
            for path in [
                scenario.rust,
                scenario.python,
                scenario.dart,
                scenario.typescript,
            ]
            .into_iter()
            .flatten()
            {
                assert!(
                    root.join(path).is_file(),
                    "{} references missing example {path}",
                    scenario.id
                );
            }
        }
    }
}
