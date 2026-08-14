//! Official API coverage invariants.

#![cfg(feature = "codegen")]

use std::collections::{BTreeMap, BTreeSet};

const BINANCE_CATALOG: &str = include_str!("../catalog/binance/manifest.tsv");
const BINANCE_COVERAGE_BRIDGE: &str = include_str!("../catalog/binance/coverage.tsv");
const BINANCE_PRODUCT_NORMALIZATION: &str = include_str!("../catalog/binance/products.tsv");
const HYPERLIQUID_CATALOG: &str = include_str!("../catalog/hyperliquid/manifest.tsv");
const HYPERLIQUID_COVERAGE_BRIDGE: &str = include_str!("../catalog/hyperliquid/coverage.tsv");
const HYPERLIQUID_UNRESOLVED: &str = include_str!("../catalog/hyperliquid/unresolved.tsv");
const UPBIT_CATALOG: &str = include_str!("../catalog/upbit/manifest.tsv");
const UPBIT_KOREA_CATALOG: &str = include_str!("../catalog/upbit/korea.tsv");
const BITHUMB_CATALOG: &str = include_str!("../catalog/bithumb/manifest.tsv");
const UPBIT_COVERAGE_BRIDGE: &str = include_str!("../catalog/upbit/coverage.tsv");
const BITHUMB_COVERAGE_BRIDGE: &str = include_str!("../catalog/bithumb/coverage.tsv");
const UPBIT_CLASSIFICATION: &str = include_str!("../catalog/upbit/classification.tsv");
const BITHUMB_CLASSIFICATION: &str = include_str!("../catalog/bithumb/classification.tsv");
const AUDIT_LEDGER: &str = include_str!("../catalog/audit/ledger.tsv");
const AUDIT_QUEUE: &str = include_str!("../catalog/audit/queue.tsv");
const IMPLEMENTATION_WORKLIST: &str = include_str!("../catalog/audit/worklist.tsv");
const EXECUTION_CHECKLIST: &str = include_str!("../catalog/audit/execution-checklist.tsv");
const PLATFORM_SERVICE_WORKLIST: &str =
    include_str!("../catalog/audit/platform-service-worklist.tsv");
const AUDIT_REVIEWS: &str = include_str!("../catalog/audit/reviews.tsv");

const EXPOSURES: &[&str] = &[
    "common_existing",
    "common_and_provider",
    "provider_typed",
    "platform_limited",
    "deprecated_excluded",
];

fn exposure_for_mapping(mapping: OperationMapping) -> &'static str {
    match mapping {
        OperationMapping::Common(_) | OperationMapping::CommonMany(_) => "common_existing",
        OperationMapping::Provider(_) => "provider_typed",
        OperationMapping::CommonAndProvider { .. } => "common_and_provider",
        OperationMapping::PlatformLimited { .. } => "platform_limited",
    }
}

fn catalog_rows(source: &'static str) -> Vec<Vec<&'static str>> {
    source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect())
        .collect()
}

fn catalog_counts(catalog: &[Vec<&'static str>]) -> BTreeMap<&'static str, usize> {
    catalog.iter().fold(BTreeMap::new(), |mut counts, fields| {
        *counts.entry(fields[0]).or_default() += 1;
        counts
    })
}

fn is_deprecated_lifecycle(lifecycle: &str) -> bool {
    matches!(lifecycle, "deprecated" | "documented_deprecated")
}

fn audit_rows(source: &'static str) -> Vec<Vec<&'static str>> {
    catalog_rows(source)
        .into_iter()
        .filter(|row| !matches!(row.first().copied(), Some("exchange" | "local_operation")))
        .collect()
}

#[test]
fn frozen_active_audit_ledger_and_derived_queues_are_consistent() {
    let ledger = audit_rows(AUDIT_LEDGER);
    let queue = audit_rows(AUDIT_QUEUE);
    let work = audit_rows(IMPLEMENTATION_WORKLIST);
    let execution = audit_rows(EXECUTION_CHECKLIST);
    let platform = audit_rows(PLATFORM_SERVICE_WORKLIST);
    let reviews = audit_rows(AUDIT_REVIEWS);
    assert_eq!(ledger.len(), 1_374);
    assert_eq!(queue.len(), 937);
    assert_eq!(work.len(), 1);
    assert_eq!(execution.len(), 1);
    assert_eq!(platform.len(), 437);
    assert_eq!(reviews.len(), 886);
    assert!(ledger.iter().all(|row| row.len() == 31));
    assert!(queue.iter().all(|row| row.len() == 31));
    assert!(work.iter().all(|row| row.len() == 31));
    assert!(execution.iter().all(|row| row.len() == 14));
    assert!(platform.iter().all(|row| row.len() == 31));
    assert!(reviews.iter().all(|row| row.len() == 9));

    let ledger_keys = ledger
        .iter()
        .map(|row| row[..6].iter().copied().collect::<Vec<_>>().join("\t"))
        .collect::<BTreeSet<_>>();
    assert_eq!(ledger_keys.len(), ledger.len());
    assert!(ledger.iter().all(|row| {
        matches!(
            row[6],
            "documented_active" | "documented_active_korea_only" | "documented_testnet"
        ) && matches!(
            (row[28], row[29]),
            ("verified", "none")
                | ("gap_found", "needs_approval")
                | ("needs_design", "service_or_contract_decision")
                | ("needs_evidence", "continue_audit")
                | ("not_checked", "continue_audit")
        ) && !row[30].is_empty()
    }));
    assert!(ledger.iter().all(|row| {
        !matches!(row[28], "verified" | "gap_found")
            || (row[11] == "connected"
                && row[13] == "Implemented"
                && row[14] == "present"
                && row[16] == "present"
                && row[18] == "present"
                && row[20] == "present"
                && row[22] == "present"
                && row[24] == "present"
                && !row[26].is_empty())
    }));
    assert!(
        ledger.iter().all(|row| {
            row[7] != "platform_limited" || row[9] == "pending_service_scope_decision"
        })
    );
    assert!(queue.iter().all(|row| row[7] != "platform_limited"
        && ledger_keys.contains(&row[..6].iter().copied().collect::<Vec<_>>().join("\t"))));
    assert!(queue.iter().all(|row| row[7] != "platform_limited"));
    assert!(work.iter().all(|row| row[28] == "gap_found"
        && row[29] == "needs_approval"
        && row[7] != "platform_limited"
        && ledger_keys.contains(&row[..6].iter().copied().collect::<Vec<_>>().join("\t"))));
    assert!(platform.iter().all(|row| row[7] == "platform_limited"
        && ledger_keys.contains(&row[..6].iter().copied().collect::<Vec<_>>().join("\t"))));

    let expected_local_operations = work
        .iter()
        .flat_map(|row| row[10].split(';'))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let actual_local_operations = execution.iter().map(|row| row[0]).collect::<BTreeSet<_>>();
    assert_eq!(actual_local_operations, expected_local_operations);
    assert_eq!(actual_local_operations.len(), execution.len());
    assert!(execution.iter().all(|row| {
        (row[1].contains(';') && row[2] == "common_contract")
            || (!row[1].contains(';') && row[2].ends_with("_owner"))
    }));
    assert_eq!(
        execution
            .iter()
            .map(|row| row[5].parse::<usize>().unwrap())
            .sum::<usize>(),
        work.len()
    );
    assert!(
        execution
            .iter()
            .all(|row| { row[11] == "gap_found" && row[12] == "needs_approval" })
    );

    let review_keys = reviews
        .iter()
        .map(|row| row[..6].iter().copied().collect::<Vec<_>>().join("\t"))
        .collect::<BTreeSet<_>>();
    assert_eq!(review_keys.len(), reviews.len());
    assert!(review_keys.is_subset(&ledger_keys));
    assert!(reviews.iter().all(|row| matches!(
        (row[6], row[7]),
        ("verified", "none")
            | ("gap_found", "needs_approval")
            | ("needs_design", "service_or_contract_decision")
            | ("needs_evidence", "continue_audit")
            | ("not_checked", "continue_audit")
    ) && !row[8].is_empty()));
}

use maxt::Exchange;
use maxt_bindings_common::{
    coverage::{
        CatalogScope, CoverageStage, Implementation, OPERATIONS, OperationMapping, PRODUCTS,
        REGIONAL_PRODUCT_COUNTS,
    },
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
fn operation_inventory_matches_the_global_catalogs() {
    let counts = PRODUCTS.iter().fold(
        std::collections::BTreeMap::<(Exchange, &str), usize>::new(),
        |mut counts, product| {
            let mapped = product.mapped_operations_for(CatalogScope::Global);
            if mapped > 0 {
                counts.insert((product.exchange, product.id), mapped);
            }
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
            ((Exchange::Binance, "spot"), 20),
            ((Exchange::Binance, "usd_m"), 28),
            ((Exchange::Binance, "wallet"), 8),
            ((Exchange::Binance, "c2c"), 1),
            ((Exchange::Hyperliquid, "info"), 26),
            ((Exchange::Hyperliquid, "exchange"), 3),
            ((Exchange::Hyperliquid, "subscriptions"), 6),
        ])
    );
}

#[test]
fn operation_inventory_matches_the_korea_catalog() {
    let counts = PRODUCTS.iter().fold(
        std::collections::BTreeMap::<(Exchange, &str), usize>::new(),
        |mut counts, product| {
            if product.exchange != Exchange::Upbit {
                return counts;
            }
            let mapped = product.mapped_operations_for(CatalogScope::Korea);
            if mapped > 0 {
                counts.insert((product.exchange, product.id), mapped);
            }
            counts
        },
    );
    assert_eq!(
        counts,
        std::collections::BTreeMap::from([
            ((Exchange::Upbit, "quotation"), 17),
            ((Exchange::Upbit, "exchange"), 14),
            ((Exchange::Upbit, "wallet"), 16),
            ((Exchange::Upbit, "travel_rule"), 3),
            ((Exchange::Upbit, "pockets"), 7),
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
fn upbit_regional_product_counts_match_the_pinned_official_catalogs() {
    assert_eq!(
        REGIONAL_PRODUCT_COUNTS,
        &[
            (Exchange::Upbit, "quotation", CatalogScope::Global, 17),
            (Exchange::Upbit, "quotation", CatalogScope::Korea, 17),
            (Exchange::Upbit, "exchange", CatalogScope::Global, 14),
            (Exchange::Upbit, "exchange", CatalogScope::Korea, 14),
            (Exchange::Upbit, "wallet", CatalogScope::Global, 13),
            (Exchange::Upbit, "wallet", CatalogScope::Korea, 16),
            (Exchange::Upbit, "travel_rule", CatalogScope::Global, 3),
            (Exchange::Upbit, "travel_rule", CatalogScope::Korea, 3),
            (Exchange::Upbit, "pockets", CatalogScope::Korea, 7),
        ]
        .map(|(exchange, product, scope, endpoint_count)| {
            maxt_bindings_common::coverage::RegionalProductCount {
                exchange,
                product,
                scope,
                endpoint_count,
            }
        })
    );
}

#[test]
fn upbit_and_bithumb_catalogs_pin_the_official_operation_sets() {
    let upbit = catalog_rows(UPBIT_CATALOG);
    let upbit_korea = catalog_rows(UPBIT_KOREA_CATALOG);
    let bithumb = catalog_rows(BITHUMB_CATALOG);
    assert!(upbit.iter().all(|fields| fields.len() == 6));
    assert!(upbit_korea.iter().all(|fields| fields.len() == 6));
    assert!(bithumb.iter().all(|fields| fields.len() == 6));
    assert_eq!(upbit.len(), 47);
    assert_eq!(upbit_korea.len(), 10);
    assert_eq!(bithumb.len(), 47);
    assert_eq!(upbit.len() + upbit_korea.len(), 57);

    assert_eq!(
        catalog_counts(&upbit),
        BTreeMap::from([
            ("Deposits and withdrawals", 13),
            ("Exchange", 14),
            ("Quotation", 17),
            ("Travel Rule", 3),
        ]),
    );
    assert_eq!(
        catalog_counts(&upbit_korea),
        BTreeMap::from([("Korea deposits and withdrawals", 3), ("Korea pockets", 7),]),
    );
    assert_eq!(
        catalog_counts(&bithumb),
        BTreeMap::from([
            ("Deposits and withdrawals", 13),
            ("Exchange", 13),
            ("KRW deposits and withdrawals", 4),
            ("Quotation", 14),
            ("TWAP", 3),
        ]),
    );
    assert!(UPBIT_KOREA_CATALOG.contains("supported_levels is intentionally excluded"));
}

#[test]
fn korean_curated_coverage_maps_to_the_pinned_official_catalogs() {
    let upbit_official = [UPBIT_CATALOG, UPBIT_KOREA_CATALOG]
        .into_iter()
        .flat_map(catalog_rows)
        .map(|fields| (fields[0], fields[1], fields[2], fields[3], fields[4]))
        .collect::<BTreeSet<_>>();
    let bithumb_official = catalog_rows(BITHUMB_CATALOG)
        .iter()
        .map(|fields| (fields[0], fields[1], fields[2], fields[3], fields[4]))
        .collect::<BTreeSet<_>>();
    let verify = |source: &'static str, exchange, official: &BTreeSet<_>, expected_len| {
        let bridge = catalog_rows(source);
        assert!(bridge.iter().all(|fields| fields.len() == 8));
        assert_eq!(bridge.len(), expected_len);
        let bridge_local = bridge
            .iter()
            .map(|fields| (fields[0], fields[1]))
            .collect::<BTreeSet<_>>();
        assert_eq!(bridge_local.len(), bridge.len());
        let current = OPERATIONS
            .iter()
            .filter(|operation| operation.exchange == exchange)
            .map(|operation| (operation.product, operation.id))
            .collect::<BTreeSet<_>>();
        assert_eq!(bridge_local, current);
        for fields in bridge {
            assert!(
                official.contains(&(fields[2], fields[3], fields[4], fields[5], fields[6])),
                "{}.{} maps to an operation missing from the pinned official catalog",
                fields[0],
                fields[1],
            );
        }
    };
    verify(UPBIT_COVERAGE_BRIDGE, Exchange::Upbit, &upbit_official, 57);
    verify(
        BITHUMB_COVERAGE_BRIDGE,
        Exchange::Bithumb,
        &bithumb_official,
        47,
    );
}

#[test]
fn korean_operation_classifications_preserve_the_pinned_sources() {
    let upbit_source = [UPBIT_CATALOG, UPBIT_KOREA_CATALOG]
        .into_iter()
        .flat_map(catalog_rows)
        .map(|fields| {
            (
                fields[0], fields[1], fields[2], fields[3], fields[4], fields[5],
            )
        })
        .collect::<Vec<_>>();
    let bithumb_source = catalog_rows(BITHUMB_CATALOG)
        .into_iter()
        .map(|fields| {
            (
                fields[0], fields[1], fields[2], fields[3], fields[4], fields[5],
            )
        })
        .collect::<Vec<_>>();
    let verify =
        |source: &'static str, expected: &Vec<(&str, &str, &str, &str, &str, &str)>| {
            let rows = catalog_rows(source);
            assert!(rows.iter().all(|fields| fields.len() == 7));
            assert!(rows.iter().all(|fields| EXPOSURES.contains(&fields[6])));
            assert_eq!(
                rows.iter()
                    .map(|fields| {
                        (
                            fields[0], fields[1], fields[2], fields[3], fields[4], fields[5],
                        )
                    })
                    .collect::<Vec<_>>(),
                *expected,
            );
            assert!(rows.iter().all(|fields| {
                (fields[5] == "deprecated") == (fields[6] == "deprecated_excluded")
            }));
        };
    verify(UPBIT_CLASSIFICATION, &upbit_source);
    verify(BITHUMB_CLASSIFICATION, &bithumb_source);
}

#[test]
fn manifest_lifecycle_exposure_and_implementation_axes_remain_separate() {
    let sources = [
        ("Upbit", catalog_rows(UPBIT_CLASSIFICATION), 5, 6, 57, 0),
        ("Bithumb", catalog_rows(BITHUMB_CLASSIFICATION), 5, 6, 47, 0),
        ("Binance", catalog_rows(BINANCE_CATALOG), 7, 8, 713, 340),
        (
            "Hyperliquid",
            catalog_rows(HYPERLIQUID_CATALOG),
            5,
            6,
            120,
            97,
        ),
    ];

    let mut general_sdk_total = 0;
    let mut platform_total = 0;
    let mut active_total = 0;
    for (exchange, rows, lifecycle, exposure, expected_general, expected_platform) in sources {
        let active = rows
            .iter()
            .filter(|fields| !is_deprecated_lifecycle(fields[lifecycle]))
            .collect::<Vec<_>>();
        let general = active
            .iter()
            .filter(|fields| fields[exposure] != "platform_limited")
            .count();
        let platform = active
            .iter()
            .filter(|fields| fields[exposure] == "platform_limited")
            .count();
        assert_eq!(
            general, expected_general,
            "{exchange} general-SDK exposure total drifted"
        );
        assert_eq!(
            platform, expected_platform,
            "{exchange} platform/protocol exposure total drifted"
        );
        assert!(
            active
                .iter()
                .all(|fields| fields[exposure] != "deprecated_excluded"),
            "{exchange} marks an active operation as deprecated_excluded"
        );
        general_sdk_total += general;
        platform_total += platform;
        active_total += active.len();
    }

    assert_eq!(general_sdk_total, 937);
    assert_eq!(platform_total, 437);
    assert_eq!(active_total, 1_374);

    assert_eq!(
        OPERATIONS
            .iter()
            .filter(|operation| matches!(
                operation.mapping,
                OperationMapping::PlatformLimited { .. }
            ))
            .count(),
        0,
        "platform-limited manifest rows require an explicit service decision before a coverage bridge exists",
    );
}

#[test]
fn local_bridge_rows_and_implementation_status_are_not_remaining_work_counts() {
    let expected = [
        (Exchange::Upbit, 57, 42, 15, 0),
        (Exchange::Bithumb, 47, 32, 15, 0),
        (Exchange::Binance, 57, 42, 14, 1),
        (Exchange::Hyperliquid, 35, 28, 7, 0),
    ];

    let mut totals = (0, 0, 0, 0);
    for (exchange, mapped, implemented, partial, planned) in expected {
        let operations = OPERATIONS
            .iter()
            .filter(|operation| operation.exchange == exchange)
            .collect::<Vec<_>>();
        let state_count = |state| {
            operations
                .iter()
                .filter(|operation| operation.implementation == state)
                .count()
        };
        assert_eq!(
            operations.len(),
            mapped,
            "{exchange} local bridge count drifted"
        );
        assert_eq!(state_count(Implementation::Implemented), implemented);
        assert_eq!(state_count(Implementation::Partial), partial);
        assert_eq!(state_count(Implementation::Planned), planned);
        assert_eq!(implemented + partial + planned, mapped);
        totals.0 += mapped;
        totals.1 += implemented;
        totals.2 += partial;
        totals.3 += planned;
    }

    assert_eq!(totals, (196, 144, 51, 1));

    let bridge_rows = [
        ("Upbit", UPBIT_COVERAGE_BRIDGE),
        ("Bithumb", BITHUMB_COVERAGE_BRIDGE),
        ("Binance", BINANCE_COVERAGE_BRIDGE),
        ("Hyperliquid", HYPERLIQUID_COVERAGE_BRIDGE),
    ]
    .into_iter()
    .flat_map(|(exchange, source)| {
        catalog_rows(source)
            .into_iter()
            .map(move |fields| (exchange, fields))
    })
    .collect::<Vec<_>>();
    assert_eq!(bridge_rows.len(), totals.0);
    assert_eq!(
        bridge_rows
            .iter()
            .map(|(exchange, fields)| {
                (
                    *exchange, fields[2], fields[3], fields[4], fields[5], fields[6],
                )
            })
            .collect::<BTreeSet<_>>()
            .len(),
        195,
        "local bridge rows must not be reported as unique official operations",
    );
}

#[test]
fn no_korean_operations_remain_planned() {
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
    assert!(actual.is_empty());
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
fn upbit_closed_orders_has_common_and_provider_coverage() {
    let operation = OPERATIONS
        .iter()
        .find(|operation| {
            operation.exchange == Exchange::Upbit
                && operation.product == "exchange"
                && operation.id == "closed_orders"
        })
        .unwrap();
    assert_eq!(operation.implementation, Implementation::Implemented);
    assert_eq!(
        operation.validation,
        maxt_bindings_common::coverage::Validation::Fixture
    );
    assert_eq!(
        operation.mapping,
        OperationMapping::CommonAndProvider {
            common: &["order_history"],
            provider: &["closed_orders"],
        },
    );
}

#[test]
fn bithumb_closed_orders_has_common_and_provider_coverage() {
    let operation = OPERATIONS
        .iter()
        .find(|operation| {
            operation.exchange == Exchange::Bithumb
                && operation.product == "exchange"
                && operation.id == "closed_orders"
        })
        .unwrap();
    assert_eq!(operation.implementation, Implementation::Implemented);
    assert_eq!(
        operation.validation,
        maxt_bindings_common::coverage::Validation::Fixture
    );
    assert_eq!(
        operation.mapping,
        OperationMapping::CommonAndProvider {
            common: &["order_history"],
            provider: &["closed_orders"],
        },
    );
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
    upbit.insert(("GET", "/v1/api_keys"));
    upbit.insert(("POST", "/v1/deposits/krw"));
    upbit.insert(("POST", "/v1/withdraws/krw"));
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
    let catalog = BINANCE_CATALOG
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(catalog.iter().all(|fields| fields.len() == 9));
    assert_eq!(catalog.len(), 1_055);

    let normalization_rows = BINANCE_PRODUCT_NORMALIZATION
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(normalization_rows.iter().all(|fields| fields.len() == 2));
    let normalization = normalization_rows
        .iter()
        .map(|fields| (fields[0], fields[1]))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(normalization.len(), normalization_rows.len());
    assert_eq!(
        normalization.keys().copied().collect::<BTreeSet<_>>(),
        catalog
            .iter()
            .map(|fields| fields[0])
            .collect::<BTreeSet<_>>(),
    );
    assert_eq!(
        catalog
            .iter()
            .filter(|fields| fields[7] == "deprecated")
            .map(|fields| (fields[0], fields[1], fields[2], fields[3], fields[4]))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("Spot REST", "Http", "POST", "/api/v3/order/oco", "orderOco"),
            (
                "Spot WebSocket",
                "WebSocketRequest",
                "POST",
                "/orderList.place",
                "orderListPlace",
            ),
        ])
    );
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
    assert_eq!(
        normalization.values().copied().collect::<BTreeSet<_>>(),
        actual.iter().map(|(id, _, _)| *id).collect::<BTreeSet<_>>(),
    );
    let normalized_counts = catalog.iter().fold(BTreeMap::new(), |mut counts, fields| {
        *counts.entry(normalization[fields[0]]).or_default() += 1;
        counts
    });
    assert_eq!(
        normalized_counts,
        actual
            .iter()
            .map(|(id, _, count)| (*id, usize::from(*count)))
            .collect::<BTreeMap<_, _>>(),
    );
    let product_counts = catalog.iter().fold(
        std::collections::BTreeMap::<&str, usize>::new(),
        |mut counts, fields| {
            *counts.entry(fields[0]).or_default() += 1;
            counts
        },
    );
    assert_eq!(
        product_counts,
        std::collections::BTreeMap::from([
            ("Algo Trading REST", 11),
            ("Alpha Trading REST", 6),
            ("Alpha WebSocket Market Streams", 13),
            ("Binance Pay REST", 1),
            ("C2C REST", 1),
            ("Convert REST", 9),
            ("Copy Trading REST", 2),
            ("Crypto Loan REST", 16),
            ("Discount Buy REST", 4),
            ("Dual Investment REST", 5),
            ("Exchange Link REST", 35),
            ("Fiat REST", 5),
            ("Fund Account REST", 4),
            ("Futures (COIN-M) REST", 64),
            ("Futures (COIN-M) WebSocket", 10),
            ("Futures (COIN-M) WebSocket Market Streams", 19),
            ("Futures (USD-M) REST", 95),
            ("Futures (USD-M) WebSocket", 18),
            ("Futures (USD-M) WebSocket Market Streams", 20),
            ("Gift Card REST", 6),
            ("Institutional Loan REST", 16),
            ("KYC SaaS REST", 12),
            ("Link Plus REST", 8),
            ("Link and Trade REST", 23),
            ("Margin REST", 65),
            ("Mining REST", 13),
            ("Options REST", 44),
            ("Options WebSocket Market Streams", 10),
            ("Portfolio Margin Pro REST", 24),
            ("Portfolio Margin REST", 109),
            ("Prediction Trading REST", 26),
            ("Rebate REST", 1),
            ("Simple Earn REST", 41),
            ("Spot Block Matching REST", 7),
            ("Spot REST", 48),
            ("Spot WebSocket", 55),
            ("Spot WebSocket Market Streams", 15),
            ("Staking REST", 37),
            ("Stocks Trading REST", 16),
            ("Stocks Trading WebSocket Streams", 7),
            ("Sub Account REST", 49),
            ("VIP CAAS REST", 10),
            ("VIP Loan REST", 14),
            ("VIP Service REST", 11),
            ("Wallet REST", 50),
        ])
    );
}

#[test]
fn binance_operation_exposure_is_complete_and_matches_current_coverage() {
    let catalog = catalog_rows(BINANCE_CATALOG);
    assert!(catalog.iter().all(|fields| fields.len() == 9));
    assert!(catalog.iter().all(|fields| EXPOSURES.contains(&fields[8])));
    assert_eq!(
        catalog.iter().fold(BTreeMap::new(), |mut counts, fields| {
            *counts.entry(fields[8]).or_default() += 1;
            counts
        }),
        BTreeMap::from([
            ("common_existing", 28),
            ("common_and_provider", 11),
            ("provider_typed", 674),
            ("platform_limited", 340),
            ("deprecated_excluded", 2),
        ]),
    );
    assert!(
        catalog
            .iter()
            .all(|fields| { (fields[7] == "deprecated") == (fields[8] == "deprecated_excluded") })
    );

    let platform_products = BTreeSet::from([
        "Copy Trading REST",
        "Exchange Link REST",
        "Fund Account REST",
        "Institutional Loan REST",
        "KYC SaaS REST",
        "Link Plus REST",
        "Link and Trade REST",
        "Portfolio Margin Pro REST",
        "Portfolio Margin REST",
        "Stocks Trading REST",
        "Stocks Trading WebSocket Streams",
        "Sub Account REST",
        "VIP CAAS REST",
        "VIP Loan REST",
        "VIP Service REST",
    ]);
    assert!(
        catalog
            .iter()
            .filter(|fields| fields[8] == "platform_limited")
            .all(|fields| platform_products.contains(fields[0]),)
    );
    assert!(
        catalog
            .iter()
            .filter(|fields| platform_products.contains(fields[0]))
            .all(|fields| fields[8] == "platform_limited",)
    );

    let source_by_tuple = catalog
        .iter()
        .map(|fields| {
            (
                (fields[0], fields[1], fields[2], fields[3], fields[4]),
                fields[8],
            )
        })
        .collect::<BTreeMap<_, _>>();
    let local_exposures = OPERATIONS
        .iter()
        .filter(|operation| operation.exchange == Exchange::Binance)
        .map(|operation| {
            (
                (operation.product, operation.id),
                exposure_for_mapping(operation.mapping),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for fields in catalog_rows(BINANCE_COVERAGE_BRIDGE) {
        let expected = local_exposures[&(fields[0], fields[1])];
        assert_eq!(
            source_by_tuple[&(fields[2], fields[3], fields[4], fields[5], fields[6])],
            expected,
            "{}.{} must keep its current public exposure",
            fields[0],
            fields[1],
        );
    }
}

#[test]
fn binance_curated_coverage_maps_to_the_pinned_official_catalog() {
    let catalog = BINANCE_CATALOG
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let official = catalog
        .iter()
        .map(|fields| (fields[0], fields[1], fields[2], fields[3], fields[4]))
        .collect::<BTreeSet<_>>();

    let bridge = BINANCE_COVERAGE_BRIDGE
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(bridge.iter().all(|fields| fields.len() == 8));
    assert_eq!(bridge.len(), 57);

    let bridge_local = bridge
        .iter()
        .map(|fields| (fields[0], fields[1]))
        .collect::<BTreeSet<_>>();
    assert_eq!(bridge_local.len(), bridge.len());
    let current_local = OPERATIONS
        .iter()
        .filter(|operation| operation.exchange == Exchange::Binance)
        .map(|operation| (operation.product, operation.id))
        .collect::<BTreeSet<_>>();
    assert_eq!(bridge_local, current_local);

    for fields in &bridge {
        assert!(
            official.contains(&(fields[2], fields[3], fields[4], fields[5], fields[6])),
            "{}.{} maps to an operation missing from the pinned Binance catalog",
            fields[0],
            fields[1],
        );
    }

    let duplicate_targets = bridge.iter().fold(BTreeMap::new(), |mut targets, fields| {
        targets
            .entry((fields[2], fields[3], fields[4], fields[5], fields[6]))
            .or_insert_with(BTreeSet::new)
            .insert((fields[0], fields[1]));
        targets
    });
    assert_eq!(
        duplicate_targets
            .into_iter()
            .filter(|(_, local)| local.len() > 1)
            .collect::<BTreeMap<_, _>>(),
        BTreeMap::from([(
            (
                "Futures (USD-M) REST",
                "Http",
                "GET",
                "/fapi/v1/premiumIndex",
                "markPrice",
            ),
            BTreeSet::from([("usd_m", "mark_price"), ("usd_m", "mark_prices")]),
        )]),
    );
}

#[test]
fn hyperliquid_catalog_pins_documented_operations_without_guessing_explorer() {
    let catalog = HYPERLIQUID_CATALOG
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(catalog.iter().all(|fields| fields.len() == 7));
    assert_eq!(catalog.len(), 219);
    assert_eq!(
        catalog
            .iter()
            .map(|fields| (fields[0], fields[1], fields[2], fields[3], fields[4]))
            .collect::<BTreeSet<_>>()
            .len(),
        catalog.len(),
    );
    assert_eq!(
        catalog
            .iter()
            .filter(|fields| fields[5] == "documented_deprecated")
            .map(|fields| fields[4])
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["agentEnableDexAbstraction", "userDexAbstraction"]),
    );
    assert_eq!(
        catalog.iter().fold(BTreeMap::new(), |mut counts, fields| {
            *counts.entry(fields[0]).or_default() += 1;
            counts
        }),
        BTreeMap::from([
            ("CoreWriter", 16),
            ("Exchange endpoint", 38),
            ("Exchange supplementary docs", 4),
            ("HIP-1/HIP-2 deployer actions", 10),
            ("HIP-3 deployer actions", 14),
            ("HIP-4 deployer actions", 5),
            ("HIP-4 info", 1),
            ("HyperEVM JSON-RPC", 28),
            ("Info endpoint", 29),
            ("Info rate limits", 1),
            ("L1Read precompile", 21),
            ("Perpetuals info", 17),
            ("Spot info", 8),
            ("WebSocket control", 3),
            ("WebSocket subscriptions", 24),
        ]),
    );
    assert!(HYPERLIQUID_CATALOG.contains("excludes Explorer"));
    assert!(HYPERLIQUID_CATALOG.contains("SDK-only-unverified"));
}

#[test]
fn hyperliquid_operation_exposure_is_complete_and_matches_current_coverage() {
    let catalog = catalog_rows(HYPERLIQUID_CATALOG);
    assert!(catalog.iter().all(|fields| fields.len() == 7));
    assert!(catalog.iter().all(|fields| EXPOSURES.contains(&fields[6])));
    assert_eq!(
        catalog.iter().fold(BTreeMap::new(), |mut counts, fields| {
            *counts.entry(fields[6]).or_default() += 1;
            counts
        }),
        BTreeMap::from([
            ("common_existing", 7),
            ("common_and_provider", 15),
            ("provider_typed", 98),
            ("platform_limited", 97),
            ("deprecated_excluded", 2),
        ]),
    );
    assert!(catalog.iter().all(|fields| {
        (fields[5] == "documented_deprecated") == (fields[6] == "deprecated_excluded")
    }));

    let platform_products = BTreeSet::from([
        "CoreWriter",
        "HIP-1/HIP-2 deployer actions",
        "HIP-3 deployer actions",
        "HIP-4 deployer actions",
        "HIP-4 info",
        "HyperEVM JSON-RPC",
        "L1Read precompile",
    ]);
    assert!(
        catalog
            .iter()
            .filter(|fields| fields[6] == "platform_limited")
            .all(|fields| {
                platform_products.contains(fields[0])
                    || matches!(fields[4], "validatorL1Stream" | "authorizeAqav2Role")
            },)
    );
    assert!(
        catalog
            .iter()
            .filter(|fields| platform_products.contains(fields[0]))
            .all(|fields| fields[6] == "platform_limited",)
    );

    let source_by_tuple = catalog
        .iter()
        .map(|fields| {
            (
                (fields[0], fields[1], fields[2], fields[3], fields[4]),
                fields[6],
            )
        })
        .collect::<BTreeMap<_, _>>();
    let local_exposures = OPERATIONS
        .iter()
        .filter(|operation| operation.exchange == Exchange::Hyperliquid)
        .map(|operation| {
            (
                (operation.product, operation.id),
                exposure_for_mapping(operation.mapping),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for fields in catalog_rows(HYPERLIQUID_COVERAGE_BRIDGE) {
        let expected = local_exposures[&(fields[0], fields[1])];
        assert_eq!(
            source_by_tuple[&(fields[2], fields[3], fields[4], fields[5], fields[6])],
            expected,
            "{}.{} must keep its current public exposure",
            fields[0],
            fields[1],
        );
    }
}

#[test]
fn hyperliquid_unresolved_inventory_is_explicitly_bounded() {
    let unresolved = catalog_rows(HYPERLIQUID_UNRESOLVED);
    assert!(unresolved.iter().all(|fields| fields.len() == 5));
    assert_eq!(unresolved.len(), 9);
    assert_eq!(
        unresolved
            .iter()
            .filter(|fields| fields[0] == "explorer")
            .map(|fields| (fields[0], fields[1], fields[2], fields[3]))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([("explorer", "needs_evidence", "continue_audit", "blockList",)]),
    );
    assert_eq!(
        unresolved
            .iter()
            .filter(|fields| fields[0] == "sdk")
            .map(|fields| fields[3])
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "CSignerAction",
            "CValidatorAction",
            "createSubAccount",
            "extraAgents",
            "setReferrer",
            "subAccountSpotTransfer",
            "subAccountTransfer",
            "userToMultiSigSigners",
        ]),
    );
    assert!(
        unresolved
            .iter()
            .all(|fields| { fields[1] == "needs_evidence" && fields[2] == "continue_audit" })
    );
}

#[test]
fn hyperliquid_curated_coverage_maps_to_the_documented_catalog() {
    let catalog = HYPERLIQUID_CATALOG
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let official = catalog
        .iter()
        .map(|fields| (fields[0], fields[1], fields[2], fields[3], fields[4]))
        .collect::<BTreeSet<_>>();
    let bridge = HYPERLIQUID_COVERAGE_BRIDGE
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(bridge.iter().all(|fields| fields.len() == 8));
    assert_eq!(bridge.len(), 35);

    let bridge_local = bridge
        .iter()
        .map(|fields| (fields[0], fields[1]))
        .collect::<BTreeSet<_>>();
    assert_eq!(bridge_local.len(), bridge.len());
    let current_local = OPERATIONS
        .iter()
        .filter(|operation| operation.exchange == Exchange::Hyperliquid)
        .map(|operation| (operation.product, operation.id))
        .collect::<BTreeSet<_>>();
    assert_eq!(bridge_local, current_local);

    for fields in &bridge {
        assert!(
            official.contains(&(fields[2], fields[3], fields[4], fields[5], fields[6])),
            "{}.{} maps to an operation missing from the pinned Hyperliquid catalog",
            fields[0],
            fields[1],
        );
    }

    assert!(
        !current_local.contains(&("subscriptions", "ping")),
        "the Hyperliquid keepalive is internal transport control, not a public subscription",
    );
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
