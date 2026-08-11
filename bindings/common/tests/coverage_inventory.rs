//! Official API coverage invariants.

#![cfg(feature = "codegen")]

use std::collections::BTreeSet;

use maxt::Exchange;
use maxt_bindings_common::{
    coverage::{CoverageStage, Implementation, OPERATIONS, OperationMapping, PRODUCTS},
    schema::binding_schema,
};

#[test]
fn product_ids_and_operation_ids_are_unique_and_connected() {
    let mut products = BTreeSet::new();
    for product in PRODUCTS {
        assert!(
            products.insert((product.exchange, product.id)),
            "duplicate product: {}.{}",
            product.exchange,
            product.id,
        );
        assert!(
            !product.interfaces.is_empty(),
            "{}.{} has no interface",
            product.exchange,
            product.id
        );
        assert!(
            !product.encodings.is_empty(),
            "{}.{} has no encoding",
            product.exchange,
            product.id
        );
        assert_eq!(
            product.interfaces.iter().collect::<BTreeSet<_>>().len(),
            product.interfaces.len(),
            "{}.{} repeats an interface",
            product.exchange,
            product.id,
        );
        assert_eq!(
            product.encodings.iter().collect::<BTreeSet<_>>().len(),
            product.encodings.len(),
            "{}.{} repeats an encoding",
            product.exchange,
            product.id,
        );
    }

    let mut operations = BTreeSet::new();
    for operation in OPERATIONS {
        assert!(
            products.contains(&(operation.exchange, operation.product)),
            "{}.{} operation {} names an unknown product",
            operation.exchange,
            operation.product,
            operation.id,
        );
        assert!(
            operations.insert((
                operation.exchange,
                operation.product,
                operation.id,
                operation.interface,
            )),
            "duplicate operation: {}.{}.{} via {}",
            operation.exchange,
            operation.product,
            operation.id,
            operation.interface.id(),
        );
    }
}

#[test]
fn common_coverage_mappings_name_real_schema_operations() {
    let schema = binding_schema();
    let operations = schema
        .adapter_operations
        .iter()
        .map(|operation| operation.rust_name)
        .collect::<BTreeSet<_>>();
    for operation in OPERATIONS {
        if operation.implementation == Implementation::Planned {
            continue;
        }
        let check = |name| {
            assert!(
                operations.contains(name),
                "{}.{}.{} maps to unknown common operation {name}",
                operation.exchange,
                operation.product,
                operation.id,
            );
        };
        let check_provider = |name| {
            assert!(
                schema.providers.iter().any(|provider| {
                    provider.exchange == operation.exchange.id()
                        && provider
                            .methods
                            .iter()
                            .any(|method| method.rust_name == name)
                }),
                "{}.{}.{} maps to unknown provider operation {name}",
                operation.exchange,
                operation.product,
                operation.id,
            );
        };
        match operation.mapping {
            OperationMapping::Common(name) => check(name),
            OperationMapping::CommonMany(names) => names.iter().for_each(|name| check(name)),
            OperationMapping::Provider(name) => check_provider(name),
            OperationMapping::CommonAndProvider { common, provider } => {
                common.iter().for_each(|name| check(name));
                provider.iter().for_each(|name| check_provider(name));
            }
            OperationMapping::PlatformLimited { .. } => {}
        }
    }
}

#[test]
fn operation_inventory_matches_the_pinned_korean_exchange_catalogs() {
    let counts = OPERATIONS.iter().fold(
        std::collections::BTreeMap::<(Exchange, &str), usize>::new(),
        |mut counts, operation| {
            *counts
                .entry((operation.exchange, operation.product))
                .or_default() += 1;
            counts
        },
    );
    assert_eq!(
        counts,
        std::collections::BTreeMap::from([
            ((Exchange::Upbit, "quotation"), 17),
            ((Exchange::Upbit, "exchange"), 14),
            ((Exchange::Upbit, "wallet"), 13),
            ((Exchange::Upbit, "travel_rule"), 3),
            ((Exchange::Bithumb, "quotation"), 14),
            ((Exchange::Bithumb, "exchange"), 13),
            ((Exchange::Bithumb, "wallet"), 13),
            ((Exchange::Bithumb, "twap"), 3),
            ((Exchange::Bithumb, "krw"), 4),
            ((Exchange::Binance, "spot"), 15),
            ((Exchange::Binance, "usd_m"), 24),
            ((Exchange::Binance, "wallet"), 8),
            ((Exchange::Hyperliquid, "info"), 14),
            ((Exchange::Hyperliquid, "exchange"), 3),
            ((Exchange::Hyperliquid, "subscriptions"), 7),
        ])
    );
}

#[test]
fn korean_exchange_product_counts_match_the_pinned_official_catalogs() {
    let actual = PRODUCTS
        .iter()
        .filter(|product| {
            matches!(product.exchange, Exchange::Upbit | Exchange::Bithumb)
                && product.endpoint_count.is_some()
        })
        .map(|product| {
            (
                product.exchange,
                product.id,
                product.endpoint_count.unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            (Exchange::Upbit, "quotation", 17),
            (Exchange::Upbit, "exchange", 14),
            (Exchange::Upbit, "wallet", 13),
            (Exchange::Upbit, "travel_rule", 3),
            (Exchange::Bithumb, "quotation", 14),
            (Exchange::Bithumb, "exchange", 13),
            (Exchange::Bithumb, "wallet", 13),
            (Exchange::Bithumb, "twap", 3),
            (Exchange::Bithumb, "krw", 4),
        ]
    );
    assert_eq!(
        actual
            .iter()
            .filter(|(exchange, _, _)| *exchange == Exchange::Upbit)
            .map(|(_, _, count)| count)
            .sum::<u16>(),
        47
    );
    assert_eq!(
        actual
            .iter()
            .filter(|(exchange, _, _)| *exchange == Exchange::Bithumb)
            .map(|(_, _, count)| count)
            .sum::<u16>(),
        47
    );
}

#[test]
fn planned_korean_operations_pin_official_request_contracts() {
    let actual = OPERATIONS
        .iter()
        .filter(|operation| {
            matches!(operation.exchange, Exchange::Upbit | Exchange::Bithumb)
                && operation.implementation == Implementation::Planned
        })
        .map(|operation| {
            format!(
                "{}|{}|{}|{}|{}|{}|{:?}|{:?}|{:?}",
                operation.exchange.id(),
                operation.product,
                operation.id,
                operation.method,
                operation.path,
                operation.interface.id(),
                operation.authentication,
                operation.risk,
                operation.availability,
            )
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "upbit|quotation|list_subscriptions|LIST_SUBSCRIPTIONS|LIST_SUBSCRIPTIONS|websocket_request|Public|Read|General",
        "upbit|exchange|cancel_and_new_order|POST|/v1/orders/cancel_and_new|http|Jwt|FinancialWrite|General",
        "upbit|travel_rule|travel_rule_vasps|GET|/v1/travel_rule/vasps|http|Jwt|Read|Region(\"Singapore\")",
        "upbit|travel_rule|travel_rule_verify_uuid|POST|/v1/travel_rule/deposit/uuid|http|Jwt|FinancialWrite|Region(\"Singapore\")",
        "upbit|travel_rule|travel_rule_verify_txid|POST|/v1/travel_rule/deposit/txid|http|Jwt|FinancialWrite|Region(\"Singapore\")",
        "bithumb|exchange|batch_orders|POST|/v2/orders/batch|http|Jwt|FinancialWrite|General",
        "bithumb|krw|withdrawals|GET|/v1/withdraws/krw|http|Jwt|Read|General",
        "bithumb|krw|withdraw|POST|/v1/withdraws/krw|http|Jwt|FinancialWrite|General",
        "bithumb|krw|deposits|GET|/v1/deposits/krw|http|Jwt|Read|General",
        "bithumb|krw|deposit|POST|/v1/deposits/krw|http|Jwt|FinancialWrite|General",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn wallet_lookup_and_cancellation_contracts_are_pinned() {
    let actual = OPERATIONS
        .iter()
        .filter(|operation| {
            matches!(operation.exchange, Exchange::Upbit | Exchange::Bithumb)
                && operation.product == "wallet"
                && matches!(operation.id, "deposit" | "withdrawal" | "cancel_withdrawal")
        })
        .map(|operation| {
            format!(
                "{}|{}|{}|{}|{}|{}|{:?}|{:?}|{:?}|{:?}",
                operation.exchange.id(),
                operation.product,
                operation.id,
                operation.method,
                operation.path,
                operation.interface.id(),
                operation.authentication,
                operation.risk,
                operation.implementation,
                operation.validation,
            )
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "upbit|wallet|deposit|GET|/v1/deposit|http|Jwt|Read|Partial|Fixture",
        "upbit|wallet|withdrawal|GET|/v1/withdraw|http|Jwt|Read|Partial|Fixture",
        "upbit|wallet|cancel_withdrawal|DELETE|/v1/withdraws/coin|http|Jwt|FinancialWrite|Partial|Fixture",
        "bithumb|wallet|deposit|GET|/v1/deposit|http|Jwt|Read|Partial|Fixture",
        "bithumb|wallet|withdrawal|GET|/v1/withdraw|http|Jwt|Read|Partial|Fixture",
        "bithumb|wallet|cancel_withdrawal|DELETE|/v1/withdraws/coin|http|Jwt|FinancialWrite|Partial|Fixture",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn implemented_wallet_endpoint_sets_match_the_current_adapters() {
    let paths = |exchange, product| {
        OPERATIONS
            .iter()
            .filter(|operation| {
                operation.exchange == exchange
                    && operation.product == product
                    && operation.implementation == Implementation::Implemented
            })
            .map(|operation| (operation.method, operation.path))
            .collect::<BTreeSet<_>>()
    };
    let upbit_and_bithumb = BTreeSet::from([
        ("GET", "/v1/status/wallet"),
        ("GET", "/v1/deposits/coin_addresses"),
        ("GET", "/v1/deposits/coin_address"),
        ("POST", "/v1/deposits/generate_coin_address"),
        ("GET", "/v1/withdraws/chance"),
        ("GET", "/v1/withdraws/coin_addresses"),
        ("POST", "/v1/withdraws/coin"),
        ("GET", "/v1/deposits"),
        ("GET", "/v1/withdraws"),
    ]);
    let mut upbit = upbit_and_bithumb.clone();
    upbit.insert(("GET", "/v1/deposits/chance/coin"));
    assert_eq!(paths(Exchange::Upbit, "wallet"), upbit);
    let mut bithumb = upbit_and_bithumb.clone();
    bithumb.insert(("GET", "/v1/api_keys"));
    assert_eq!(paths(Exchange::Bithumb, "wallet"), bithumb);
    assert_eq!(
        paths(Exchange::Binance, "wallet"),
        BTreeSet::from([
            ("GET", "/sapi/v1/capital/config/getall"),
            ("GET", "/sapi/v1/capital/deposit/address"),
            ("GET", "/sapi/v1/account/apiRestrictions"),
            ("GET", "/sapi/v1/capital/withdraw/address/list"),
            ("GET", "/sapi/v1/localentity/questionnaire-requirements"),
            ("POST", "/sapi/v1/capital/withdraw/apply"),
            ("GET", "/sapi/v1/capital/deposit/hisrec"),
            ("GET", "/sapi/v1/capital/withdraw/history"),
        ])
    );
}

#[test]
fn binance_catalog_matches_the_pinned_official_exact_set() {
    let actual = PRODUCTS
        .iter()
        .filter(|product| product.exchange == Exchange::Binance)
        .map(|product| (product.id, product.name, product.endpoint_count.unwrap()))
        .collect::<Vec<_>>();
    let expected = vec![
        ("spot", "Spot Trading", 118),
        ("usd_m", "Futures (USDⓈ-M)", 133),
        ("coin_m", "Futures (COIN-M)", 93),
        ("options", "Options", 54),
        ("margin", "Margin", 65),
        ("wallet", "Wallet", 50),
        ("convert", "Convert", 9),
        ("portfolio_margin", "Portfolio Margin", 109),
        ("portfolio_margin_pro", "Portfolio Margin Pro", 24),
        ("algo", "Algo Trading", 11),
        ("copy_trading", "Copy Trading", 2),
        ("institutional_loan", "Institutional Loan", 16),
        ("alpha", "Alpha Trading", 19),
        ("stocks", "Stocks Trading", 23),
        ("sub_account", "Sub Account", 49),
        ("spot_block_matching", "Spot Block Matching", 7),
        ("vip_service", "VIP Service", 11),
        ("caas", "Crypto-as-a-Service (CAAS)", 10),
        ("fund_account", "Fund Account", 4),
        ("link_plus", "Link Plus", 8),
        ("exchange_link", "Exchange Link", 35),
        ("kyc_saas", "KYC SaaS", 12),
        ("link_and_trade", "Link and Trade", 23),
        ("staking", "Staking", 37),
        ("mining", "Mining", 13),
        ("crypto_loan", "Crypto Loan", 16),
        ("vip_loan", "VIP Loan", 14),
        ("c2c", "C2C", 1),
        ("fiat", "Fiat", 5),
        ("gift_card", "Gift Card", 6),
        ("rebate", "Rebate", 1),
        ("simple_earn", "Simple Earn", 41),
        ("discount_buy", "Discount Buy", 4),
        ("dual_investment", "Dual Investment", 5),
        ("pay", "Pay", 1),
        ("prediction", "Prediction Trading", 26),
    ];

    assert_eq!(actual, expected);
    assert_eq!(actual.iter().map(|(_, _, count)| count).sum::<u16>(), 1_055);
}

#[test]
fn complete_means_every_official_endpoint_is_implemented() {
    for product in PRODUCTS {
        if product.stage() == CoverageStage::Complete {
            assert_eq!(
                Some(product.mapped_operations() as u16),
                product.endpoint_count,
                "{}.{} cannot be complete without exact official coverage",
                product.exchange,
                product.id,
            );
            assert_eq!(
                product.implemented_operations(),
                product.mapped_operations(),
                "{}.{} cannot be complete with planned operations",
                product.exchange,
                product.id,
            );
        }
    }
}
