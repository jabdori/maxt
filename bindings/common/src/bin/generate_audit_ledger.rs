//! Cargo tool that audits the complete official catalog and current public contracts.
//!
//! It uses Rust `OPERATIONS`/`binding_schema()` and pinned TSV files. It keeps
//! mechanically verifiable connection evidence separate from human-reviewed
//! audit results and does not reinterpret Rust source using regular expressions.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process,
};

use maxt::Exchange;
use maxt_bindings_common::{
    coverage::{Implementation, OPERATIONS, OperationMapping, Validation},
    schema::binding_schema,
};

const OUT_LEDGER: &str = "audit/ledger.tsv";
const OUT_QUEUE: &str = "audit/queue.tsv";
const OUT_WORK: &str = "audit/worklist.tsv";
const OUT_EXECUTION: &str = "audit/execution-checklist.tsv";
const OUT_PLATFORM: &str = "audit/platform-service-worklist.tsv";
const REVIEW_INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/catalog/audit/reviews.tsv"
));

// Explicit audit alias for one official operation that supplies multiple provider
// surfaces or configuration overloads for the same subscription.
const PROVIDER_AUDIT_ALIASES: &[(Exchange, &str, &str)] = &[
    (
        Exchange::Hyperliquid,
        "user_non_funding_ledger_updates",
        "non_funding_ledger",
    ),
    (
        Exchange::Hyperliquid,
        "subscriptions",
        "subscribe_detailed_with",
    ),
    (
        Exchange::Hyperliquid,
        "subscriptions",
        "subscribe_detailed_account_with",
    ),
];

#[derive(Clone, Copy)]
struct Catalog {
    exchange: Exchange,
    source: &'static str,
    lifecycle: usize,
    exposure: Option<usize>,
    classification: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct Bridge<'a> {
    local_product: &'a str,
    local_id: &'a str,
}

#[derive(Clone, Copy)]
struct AuditReview<'a> {
    result: &'a str,
    next_action: &'a str,
    reason: &'a str,
}

struct RenderedOutput {
    name: &'static str,
    contents: String,
    rows: usize,
}

#[derive(Default)]
struct ExecutionGroup {
    exchanges: BTreeSet<String>,
    coverage_states: BTreeSet<String>,
    mapping_methods: BTreeSet<String>,
    official_operations: BTreeSet<String>,
    exposures: BTreeSet<String>,
    coverage_locators: BTreeSet<String>,
    rust_locators: BTreeSet<String>,
    verification: BTreeSet<String>,
    audit_results: BTreeSet<String>,
    next_actions: BTreeSet<String>,
    reasons: BTreeSet<String>,
}

fn rows(source: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect())
}

fn key(fields: &[&str]) -> String {
    fields[..5].join("\t")
}

fn exchange_name(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Upbit => "Upbit",
        Exchange::Bithumb => "Bithumb",
        Exchange::Binance => "Binance",
        Exchange::Hyperliquid => "Hyperliquid",
        _ => "Other",
    }
}

fn exchange_slug(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Upbit => "upbit",
        Exchange::Bithumb => "bithumb",
        Exchange::Binance => "binance",
        Exchange::Hyperliquid => "hyperliquid",
        _ => "other",
    }
}

fn exchange_from_name(value: &str) -> Exchange {
    match value {
        "Upbit" => Exchange::Upbit,
        "Bithumb" => Exchange::Bithumb,
        "Binance" => Exchange::Binance,
        "Hyperliquid" => Exchange::Hyperliquid,
        _ => panic!("unknown audit review exchange {value}"),
    }
}

fn valid_audit_result(result: &str, next_action: &str) -> bool {
    matches!(
        (result, next_action),
        ("verified", "none")
            | ("gap_found", "needs_approval")
            | ("not_implemented", "needs_approval")
            | ("needs_design", "service_or_contract_decision")
            | ("blocked", "official_contract_required")
    )
}

fn audit_reviews() -> BTreeMap<(Exchange, String), AuditReview<'static>> {
    let mut reviews = BTreeMap::new();
    for fields in
        rows(REVIEW_INPUT).filter(|fields| !matches!(fields.first().copied(), Some("exchange")))
    {
        assert_eq!(fields.len(), 9, "audit review must have nine columns");
        let review = AuditReview {
            result: fields[6],
            next_action: fields[7],
            reason: fields[8],
        };
        assert!(
            valid_audit_result(review.result, review.next_action),
            "invalid audit result/action for {}",
            fields[5]
        );
        assert!(
            !review.reason.is_empty(),
            "audit review reason must not be empty"
        );
        let review_key = (exchange_from_name(fields[0]), key(&fields[1..6]));
        assert!(
            reviews.insert(review_key, review).is_none(),
            "duplicate audit review for {}",
            fields[5]
        );
    }
    reviews
}

fn snake_to_camel(value: &str) -> String {
    let mut output = String::new();
    for (index, part) in value.split('_').enumerate() {
        if index == 0 {
            output.push_str(part);
        } else {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                output.extend(first.to_uppercase());
                output.push_str(chars.as_str());
            }
        }
    }
    output
}

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

fn emit(header: &[&str], comments: &[&str], records: &[Vec<String>]) -> String {
    let mut output = comments
        .iter()
        .map(|comment| format!("# {comment}"))
        .collect::<Vec<_>>();
    output.push(header.join("\t"));
    output.extend(records.iter().map(|record| {
        record
            .iter()
            .map(|value| sanitize(value))
            .collect::<Vec<_>>()
            .join("\t")
    }));
    format!("{}\n", output.join("\n"))
}

fn operation_mapping(
    operation: &maxt_bindings_common::coverage::OperationCoverage,
) -> Vec<(&'static str, &'static str)> {
    match operation.mapping {
        OperationMapping::Common(name) => vec![(name, "common")],
        OperationMapping::CommonMany(names) => names.iter().map(|name| (*name, "common")).collect(),
        OperationMapping::Provider(name) => vec![(name, "provider")],
        OperationMapping::CommonAndProvider { common, provider } => common
            .iter()
            .map(|name| (*name, "common"))
            .chain(provider.iter().map(|name| (*name, "provider")))
            .collect(),
        OperationMapping::PlatformLimited { service, .. } => vec![(service, "platform")],
    }
}

fn operation_names(
    schema: &maxt_bindings_common::schema::Schema,
    exchange: Exchange,
) -> BTreeSet<&'static str> {
    let mut names = schema
        .adapter_operations
        .iter()
        .map(|operation| operation.rust_name)
        .collect::<BTreeSet<_>>();
    names.extend(schema.client_members.iter().copied());
    if let Some(provider) = schema
        .providers
        .iter()
        .find(|provider| provider.exchange == exchange_slug(exchange))
    {
        names.extend(provider.methods.iter().map(|method| method.rust_name));
    }
    names
}

fn rust_locator(root: &Path, exchange: Exchange, name: &str, kind: &str) -> Option<String> {
    let path = if kind == "common" {
        "src/adapter.rs"
    } else {
        match exchange {
            Exchange::Upbit => "src/adapters/upbit/mod.rs",
            Exchange::Bithumb => "src/adapters/bithumb/mod.rs",
            Exchange::Binance => "src/adapters/binance/mod.rs",
            Exchange::Hyperliquid => "src/adapters/hyperliquid/mod.rs",
            _ => "src/adapters/mod.rs",
        }
    };
    let source = fs::read_to_string(root.join(path)).ok()?;
    let needle = format!("fn {name}(");
    source.lines().enumerate().find_map(|(index, line)| {
        line.contains(&needle)
            .then(|| format!("{path}:{}", index + 1))
    })
}

fn has_all_name_variants(root: &Path, paths: &[&str], names: &BTreeSet<&str>) -> bool {
    let source = paths
        .iter()
        .filter_map(|relative| fs::read_to_string(root.join(relative)).ok())
        .collect::<Vec<_>>()
        .join("\n");
    !names.is_empty()
        && names
            .iter()
            .all(|name| source.contains(name) || source.contains(&snake_to_camel(name)))
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace repository root")
}

fn catalog_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog")
}

fn render(root: &Path) -> Vec<RenderedOutput> {
    let catalogs = [
        Catalog {
            exchange: Exchange::Upbit,
            source: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/upbit/manifest.tsv"
            )),
            lifecycle: 5,
            exposure: None,
            classification: Some(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/upbit/classification.tsv"
            ))),
        },
        Catalog {
            exchange: Exchange::Upbit,
            source: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/upbit/korea.tsv"
            )),
            lifecycle: 5,
            exposure: None,
            classification: Some(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/upbit/classification.tsv"
            ))),
        },
        Catalog {
            exchange: Exchange::Bithumb,
            source: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/bithumb/manifest.tsv"
            )),
            lifecycle: 5,
            exposure: None,
            classification: Some(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/bithumb/classification.tsv"
            ))),
        },
        Catalog {
            exchange: Exchange::Binance,
            source: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/binance/manifest.tsv"
            )),
            lifecycle: 7,
            exposure: Some(8),
            classification: None,
        },
        Catalog {
            exchange: Exchange::Hyperliquid,
            source: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/hyperliquid/manifest.tsv"
            )),
            lifecycle: 5,
            exposure: Some(6),
            classification: None,
        },
    ];
    let bridges = [
        (
            Exchange::Upbit,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/upbit/coverage.tsv"
            )),
        ),
        (
            Exchange::Bithumb,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/bithumb/coverage.tsv"
            )),
        ),
        (
            Exchange::Binance,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/binance/coverage.tsv"
            )),
        ),
        (
            Exchange::Hyperliquid,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/catalog/hyperliquid/coverage.tsv"
            )),
        ),
    ];
    let mut bridge_map = BTreeMap::<(Exchange, String), Vec<Bridge<'_>>>::new();
    for (exchange, source) in bridges {
        for fields in rows(source) {
            bridge_map
                .entry((exchange, key(&fields[2..])))
                .or_default()
                .push(Bridge {
                    local_product: fields[0],
                    local_id: fields[1],
                });
        }
    }
    let mut classification_map = BTreeMap::<(Exchange, String), &'static str>::new();
    for catalog in catalogs {
        if let Some(source) = catalog.classification {
            for fields in rows(source) {
                classification_map.insert((catalog.exchange, key(&fields)), fields[6]);
            }
        }
    }
    let reviews = audit_reviews();
    let schema = binding_schema();
    let schema_names = [
        operation_names(&schema, Exchange::Upbit),
        operation_names(&schema, Exchange::Bithumb),
        operation_names(&schema, Exchange::Binance),
        operation_names(&schema, Exchange::Hyperliquid),
    ];
    let mut operations = BTreeMap::new();
    for operation in OPERATIONS {
        operations.insert(
            (operation.exchange, operation.product, operation.id),
            operation,
        );
    }
    let mapped_provider_methods = OPERATIONS
        .iter()
        .flat_map(|operation| {
            operation_mapping(operation)
                .into_iter()
                .map(move |(name, kind)| (operation.exchange, name, kind))
        })
        .chain(
            PROVIDER_AUDIT_ALIASES
                .iter()
                .map(|(_, _, method)| (Exchange::Hyperliquid, *method, "provider")),
        )
        .collect::<BTreeSet<_>>();
    for provider in schema.providers {
        for method in provider.methods {
            if matches!(
                method.kind,
                maxt_bindings_common::schema::ProviderMethodKind::Async
            ) {
                let mapped = mapped_provider_methods.contains(&(
                    match provider.exchange {
                        "upbit" => Exchange::Upbit,
                        "bithumb" => Exchange::Bithumb,
                        "binance" => Exchange::Binance,
                        "hyperliquid" => Exchange::Hyperliquid,
                        _ => panic!("unknown provider {}", provider.exchange),
                    },
                    method.rust_name,
                    "provider",
                ));
                assert!(
                    mapped,
                    "public provider method {}.{} is outside the official coverage bridge; record it in the fixed audit backlog before implementation",
                    provider.exchange, method.rust_name,
                );
            }
        }
    }
    let mut ledger = Vec::<Vec<String>>::new();
    let mut queue = Vec::<Vec<String>>::new();
    let mut work = Vec::<Vec<String>>::new();
    let mut platform = Vec::<Vec<String>>::new();
    let mut used_reviews = BTreeSet::new();
    for catalog in catalogs {
        for fields in rows(catalog.source) {
            let lifecycle = fields[catalog.lifecycle];
            if matches!(lifecycle, "deprecated" | "documented_deprecated") {
                continue;
            }
            let official = key(&fields);
            let exposure = classification_map
                .get(&(catalog.exchange, official.clone()))
                .copied()
                .or_else(|| catalog.exposure.map(|index| fields[index]))
                .unwrap_or("provider_typed");
            let bridges = bridge_map
                .get(&(catalog.exchange, official.clone()))
                .cloned()
                .unwrap_or_default();
            let local = bridges
                .iter()
                .map(|bridge| format!("{}.{}", bridge.local_product, bridge.local_id))
                .collect::<Vec<_>>();
            let local_ops = bridges
                .iter()
                .filter_map(|bridge| {
                    operations
                        .get(&(catalog.exchange, bridge.local_product, bridge.local_id))
                        .copied()
                })
                .collect::<Vec<_>>();
            let state = if local_ops.is_empty() {
                "Unlinked".to_owned()
            } else if local_ops
                .iter()
                .all(|op| op.implementation == Implementation::Implemented)
            {
                "Implemented".to_owned()
            } else if local_ops
                .iter()
                .any(|op| op.implementation == Implementation::Planned)
            {
                "Planned".to_owned()
            } else {
                "Partial".to_owned()
            };
            let mappings = local_ops
                .iter()
                .flat_map(|op| operation_mapping(op))
                .collect::<Vec<_>>();
            let mapping_names = mappings
                .iter()
                .map(|(name, _)| *name)
                .collect::<BTreeSet<_>>();
            let source_locators = mappings
                .iter()
                .filter_map(|(name, kind)| rust_locator(root, catalog.exchange, name, kind))
                .collect::<BTreeSet<_>>();
            let rust_present =
                !local_ops.is_empty() && source_locators.len() >= mapping_names.len();
            let schema_present = mappings.iter().all(|(name, _)| {
                schema_names[match catalog.exchange {
                    Exchange::Upbit => 0,
                    Exchange::Bithumb => 1,
                    Exchange::Binance => 2,
                    Exchange::Hyperliquid => 3,
                    _ => 0,
                }]
                .contains(name)
            });
            let generated_names = mapping_names.clone();
            let codegen_names_present = has_all_name_variants(
                root,
                &[
                    "bindings/python/python/maxt/_generated_contract.py",
                    "bindings/dart/lib/src/generated_contract.dart",
                    "bindings/typescript/src/generated/contract.ts",
                ],
                &generated_names,
            );
            let python_present = has_all_name_variants(
                root,
                &[
                    "bindings/python/python/maxt/_generated_contract.py",
                    "bindings/python/python/maxt/adapters.py",
                ],
                &generated_names,
            );
            let dart_present = has_all_name_variants(
                root,
                &[
                    "bindings/dart/lib/src/generated_contract.dart",
                    "bindings/dart/lib/src/generated_provider_methods.dart",
                ],
                &generated_names,
            );
            let typescript_present = has_all_name_variants(
                root,
                &[
                    "bindings/typescript/src/generated/contract.ts",
                    "bindings/typescript/src/generated/api.ts",
                ],
                &generated_names,
            );
            let verification = local_ops
                .iter()
                .map(|op| match op.validation {
                    Validation::Documented => "Documented",
                    Validation::Fixture => "Fixture",
                    Validation::Testnet => "Testnet",
                    Validation::LiveRead => "LiveRead",
                    Validation::LiveWrite => "LiveWrite",
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("+");
            let platform_decision = if exposure == "platform_limited" {
                "pending_service_scope_decision"
            } else {
                "not_applicable"
            };
            let default_review = if exposure == "platform_limited" {
                AuditReview {
                    result: "needs_design",
                    next_action: "service_or_contract_decision",
                    reason: "Requires a separate platform or protocol service; release scope requires user approval.",
                }
            } else if state == "Partial" {
                AuditReview {
                    result: "gap_found",
                    next_action: "needs_approval",
                    reason: "Current coverage explicitly marks this connected public contract as Partial.",
                }
            } else {
                AuditReview {
                    result: "not_implemented",
                    next_action: "needs_approval",
                    reason: "No connected Rust, schema, or public binding contract exists for this active operation.",
                }
            };
            let review_key = (catalog.exchange, official.clone());
            let review = if let Some(review) = reviews.get(&review_key).copied() {
                used_reviews.insert(review_key);
                review
            } else {
                default_review
            };
            assert!(
                valid_audit_result(review.result, review.next_action),
                "invalid derived audit result/action for {}",
                fields[4]
            );
            if review.result == "verified" {
                assert!(
                    bridges.len() > 0
                        && state == "Implemented"
                        && rust_present
                        && schema_present
                        && codegen_names_present
                        && python_present
                        && dart_present
                        && typescript_present
                        && !verification.is_empty(),
                    "{} audit review lacks required mechanical evidence for {}",
                    review.result,
                    fields[4]
                );
            }
            if review.result == "gap_found" {
                assert!(
                    bridges.len() > 0
                        && rust_present
                        && schema_present
                        && codegen_names_present
                        && python_present
                        && dart_present
                        && typescript_present
                        && !verification.is_empty(),
                    "gap_found audit review lacks a connected public contract for {}",
                    fields[4]
                );
            }
            if review.result == "not_implemented" {
                assert!(
                    bridges.is_empty() && local_ops.is_empty(),
                    "not_implemented audit review has a connected public contract for {}",
                    fields[4]
                );
            }
            let source_locator_text = source_locators.into_iter().collect::<Vec<_>>().join(";");
            let local_status = if local_ops.is_empty() {
                "absent"
            } else if rust_present {
                "present"
            } else {
                "missing_or_unverified"
            };
            let schema_status = if local_ops.is_empty() {
                "absent"
            } else if schema_present {
                "present"
            } else {
                "missing_or_unverified"
            };
            let schema_locator = if local_ops.is_empty() {
                ""
            } else {
                "bindings/common/src/schema.rs::binding_schema"
            };
            let row = vec![
                exchange_name(catalog.exchange).to_owned(),
                fields[0].to_owned(),
                fields[1].to_owned(),
                fields[2].to_owned(),
                fields[3].to_owned(),
                fields[4].to_owned(),
                lifecycle.to_owned(),
                exposure.to_owned(),
                if exposure == "platform_limited" {
                    "separate_platform_or_protocol_service"
                } else {
                    "general_adapter_or_provider"
                }
                .to_owned(),
                platform_decision.to_owned(),
                local.join(";"),
                if bridges.is_empty() {
                    "not_connected"
                } else {
                    "connected"
                }
                .to_owned(),
                if bridges.is_empty() {
                    ""
                } else {
                    "bindings/common/src/coverage.rs::OPERATIONS"
                }
                .to_owned(),
                state,
                local_status.to_owned(),
                source_locator_text,
                schema_status.to_owned(),
                schema_locator.to_owned(),
                if local_ops.is_empty() {
                    "absent"
                } else if codegen_names_present {
                    "present"
                } else {
                    "missing_or_unverified"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    ""
                } else {
                    "bindings/python/python/maxt/_generated_contract.py;bindings/dart/lib/src/generated_contract.dart;bindings/typescript/src/generated/contract.ts"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    "absent"
                } else if python_present {
                    "present"
                } else {
                    "missing_or_unverified"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    ""
                } else {
                    "bindings/python/python/maxt/adapters.py"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    "absent"
                } else if dart_present {
                    "present"
                } else {
                    "missing_or_unverified"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    ""
                } else {
                    "bindings/dart/lib/src/generated_provider_methods.dart"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    "absent"
                } else if typescript_present {
                    "present"
                } else {
                    "missing_or_unverified"
                }
                .to_owned(),
                if local_ops.is_empty() {
                    ""
                } else {
                    "bindings/typescript/src/generated/api.ts"
                }
                .to_owned(),
                verification,
                mapping_names.into_iter().collect::<Vec<_>>().join(";"),
                review.result.to_owned(),
                review.next_action.to_owned(),
                review.reason.to_owned(),
            ];
            if exposure == "platform_limited" {
                platform.push(row.clone());
            } else {
                queue.push(row.clone());
                if review.result == "gap_found"
                    && review.next_action == "needs_approval"
                    && !local.is_empty()
                {
                    work.push(row.clone());
                }
            }
            ledger.push(row);
        }
    }
    assert_eq!(
        used_reviews.len(),
        reviews.len(),
        "an audit review does not match an active official operation"
    );
    ledger.sort_by(|a, b| a[..6].cmp(&b[..6]));
    queue.sort_by(|a, b| a[..6].cmp(&b[..6]));
    work.sort_by(|a, b| a[..6].cmp(&b[..6]));
    platform.sort_by(|a, b| a[..6].cmp(&b[..6]));
    assert_eq!(ledger.len(), 1_374);
    assert_eq!(platform.len(), 437);
    assert_eq!(queue.len(), 937);
    let mut execution_groups = BTreeMap::<String, ExecutionGroup>::new();
    for row in &work {
        for local_operation in row[10].split(';').filter(|value| !value.is_empty()) {
            let group = execution_groups
                .entry(local_operation.to_owned())
                .or_default();
            group.exchanges.insert(row[0].clone());
            group.coverage_states.insert(row[13].clone());
            group.exposures.insert(row[7].clone());
            group.coverage_locators.insert(row[12].clone());
            group.rust_locators.extend(
                row[15]
                    .split(';')
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
            );
            group.verification.extend(
                row[26]
                    .split('+')
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
            );
            group.audit_results.insert(row[28].clone());
            group.next_actions.insert(row[29].clone());
            group.reasons.insert(row[30].clone());
            group.mapping_methods.extend(
                row[27]
                    .split(';')
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
            );
            group.official_operations.insert(format!(
                "{} | {} | {} | {} | {} | {}",
                row[0], row[1], row[2], row[3], row[4], row[5]
            ));
        }
    }
    let execution = execution_groups
        .into_iter()
        .map(|(local_operation, group)| {
            let exchanges = group.exchanges.into_iter().collect::<Vec<_>>();
            let owner = if exchanges.len() > 1 {
                "common_contract".to_owned()
            } else {
                exchanges
                    .first()
                    .map(|exchange| format!("{}_owner", exchange.to_lowercase()))
                    .unwrap_or_else(|| "unassigned".to_owned())
            };
            vec![
                local_operation,
                exchanges.join(";"),
                owner,
                group
                    .coverage_states
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(";"),
                group
                    .mapping_methods
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(";"),
                group.official_operations.len().to_string(),
                group
                    .official_operations
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(" || "),
                group.exposures.into_iter().collect::<Vec<_>>().join(";"),
                group
                    .coverage_locators
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(";"),
                group
                    .rust_locators
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(";"),
                group.verification.into_iter().collect::<Vec<_>>().join("+"),
                group
                    .audit_results
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(";"),
                group.next_actions.into_iter().collect::<Vec<_>>().join(";"),
                group.reasons.into_iter().collect::<Vec<_>>().join(" || "),
            ]
        })
        .collect::<Vec<_>>();
    let header = [
        "exchange",
        "official_product",
        "interface",
        "method",
        "path_or_message",
        "operation_id",
        "lifecycle",
        "exposure",
        "platform_boundary",
        "platform_decision",
        "local_operations",
        "bridge_status",
        "coverage_locator",
        "coverage_implementation_state",
        "rust_status",
        "rust_locators",
        "schema_status",
        "schema_locator",
        "codegen_status",
        "codegen_locator",
        "Python_status",
        "Python_locator",
        "Dart_status",
        "Dart_locator",
        "TypeScript_status",
        "TypeScript_locator",
        "verification",
        "mapping_names",
        "audit_result",
        "next_action",
        "reason",
    ];
    let execution_header = [
        "local_operation",
        "exchanges",
        "owner",
        "coverage_states",
        "mapping_methods",
        "official_row_count",
        "official_operations",
        "exposures",
        "coverage_locators",
        "rust_locators",
        "verification",
        "audit_result",
        "next_action",
        "reasons",
    ];
    vec![
        RenderedOutput {
            name: OUT_LEDGER,
            contents: emit(
                &header,
                &[
                    "audit ledger for all 1,374 active official rows",
                    "deprecated source rows are retained but excluded from this active ledger",
                    "every row has an explicit audited result; not_implemented means no connected public contract exists",
                    "platform_limited rows remain in the ledger and are separated from general Adapter work",
                ],
                &ledger,
            ),
            rows: ledger.len(),
        },
        RenderedOutput {
            name: OUT_QUEUE,
            contents: emit(
                &header,
                &[
                    "all 937 general-SDK rows; this is a filtered ledger view, not an implementation schedule",
                ],
                &queue,
            ),
            rows: queue.len(),
        },
        RenderedOutput {
            name: OUT_WORK,
            contents: emit(
                &header,
                &[
                    "official rows with gap_found and needs_approval; this is not an approved implementation list",
                ],
                &work,
            ),
            rows: work.len(),
        },
        RenderedOutput {
            name: OUT_EXECUTION,
            contents: emit(
                &execution_header,
                &[
                    "unique local-operation groups derived from the current worklist",
                    "each group keeps all mapped official rows together to prevent duplicate implementation",
                    "needs_approval means implementation remains unauthorized until the user approves it",
                ],
                &execution,
            ),
            rows: execution.len(),
        },
        RenderedOutput {
            name: OUT_PLATFORM,
            contents: emit(
                &header,
                &[
                    "437 platform_limited rows retained at a separate platform/protocol boundary; all decisions are provisional pending user approval",
                ],
                &platform,
            ),
            rows: platform.len(),
        },
    ]
}

fn check(outputs: &[RenderedOutput]) -> Result<(), String> {
    let root = catalog_root();
    for output in outputs {
        let checked_in = fs::read(root.join(output.name))
            .map_err(|error| format!("read {}: {error}", output.name))?;
        if checked_in != output.contents.as_bytes() {
            return Err(format!(
                "{} differs from the rendered audit ledger; run with --write after reviewing the diff",
                output.name
            ));
        }
    }
    Ok(())
}

fn write(outputs: &[RenderedOutput]) -> Result<(), String> {
    let root = catalog_root();
    for output in outputs {
        fs::write(root.join(output.name), &output.contents)
            .map_err(|error| format!("write {}: {error}", output.name))?;
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mode = args
        .next()
        .ok_or_else(|| "usage: generate_audit_ledger --check|--write".to_owned())?;
    if args.next().is_some() {
        return Err("usage: generate_audit_ledger --check|--write".to_owned());
    }
    let outputs = render(workspace_root());
    match mode.as_str() {
        "--check" => check(&outputs)?,
        "--write" => write(&outputs)?,
        _ => return Err("usage: generate_audit_ledger --check|--write".to_owned()),
    }
    println!(
        "audit ledger: {} active; {} audit queue; {} approval candidates; {} execution units; {} platform rows",
        outputs[0].rows, outputs[1].rows, outputs[2].rows, outputs[3].rows, outputs[4].rows
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_is_pure_and_checked_in_outputs_match() {
        let outputs = render(workspace_root());
        let root = catalog_root();
        let before = outputs
            .iter()
            .map(|output| fs::read(root.join(output.name)).expect("read checked-in audit output"))
            .collect::<Vec<_>>();
        check(&outputs).expect("checked-in audit outputs match the in-memory render");
        let after = outputs
            .iter()
            .map(|output| fs::read(root.join(output.name)).expect("read checked-in audit output"))
            .collect::<Vec<_>>();
        assert_eq!(after, before, "--check must not modify audit outputs");
        for (output, (expected_rows, expected_columns)) in
            outputs
                .iter()
                .zip([(1_374, 31), (937, 31), (28, 31), (28, 14), (437, 31)])
        {
            let rows = output
                .contents
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect::<Vec<_>>();
            assert_eq!(output.rows, expected_rows, "{}", output.name);
            assert_eq!(rows.len() - 1, expected_rows, "{}", output.name);
            assert!(
                rows[1..]
                    .iter()
                    .all(|row| row.split('\t').count() == expected_columns),
                "{} width",
                output.name
            );
        }
    }
}
