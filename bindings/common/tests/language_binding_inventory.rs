//! Cross-language public API inventory parity.

use std::collections::BTreeSet;

use maxt::{Exchange, Feature};
use syn::{ImplItem, Item, TraitItem, Type, Visibility};

const CORE_ADAPTER: &str = include_str!("../../../src/adapter.rs");
const CORE_CLIENT: &str = include_str!("../../../src/client.rs");
const COMMON_CONTRACT: &str = include_str!("../src/contract.rs");
const PYTHON_RUST_ADAPTER: &str = include_str!("../../python/src/adapter.rs");
const PYTHON_MODELS: &str = include_str!("../../python/python/maxt/models.py");
const PYTHON_API: &str = include_str!("../../python/python/maxt/_api.py");
const PYTHON_ADAPTERS: &str = include_str!("../../python/python/maxt/adapters.py");
const DART_RUST_ADAPTER: &str = include_str!("../../dart/rust/src/adapter.rs");
const DART_MODELS: &str = include_str!("../../dart/lib/src/models.dart");
const DART_ADAPTER: &str = include_str!("../../dart/lib/src/adapter.dart");
const DART_CLIENT: &str = include_str!("../../dart/lib/src/client.dart");
const DART_ADAPTERS: &str = include_str!("../../dart/lib/src/adapters.dart");
const DART_ERRORS: &str = include_str!("../../dart/lib/src/errors.dart");
const DART_PROVIDERS: &str = include_str!("../../dart/lib/src/providers.dart");
const DART_GENERATED_CONVERT: &str = include_str!("../../dart/lib/src/rust/convert.dart");

fn rust_trait_methods(source: &str, name: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("Rust trait source must parse")
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Trait(item) if item.ident == name => Some(
                item.items
                    .into_iter()
                    .filter_map(|item| match item {
                        TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Rust trait {name} must exist"))
}

fn rust_enum_variants(source: &str, name: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("Rust enum source must parse")
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == name => Some(
                item.variants
                    .into_iter()
                    .map(|variant| variant.ident.to_string())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Rust enum {name} must exist"))
}

fn qualified_variants(source: &str, prefix: &str) -> BTreeSet<String> {
    let mut variants = BTreeSet::new();
    let mut remainder = source;
    while let Some(index) = remainder.find(prefix) {
        remainder = &remainder[index + prefix.len()..];
        let variant = remainder
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !variant.is_empty() {
            variants.insert(variant);
        }
    }
    variants
}

fn rust_public_methods(source: &str, name: &str, async_only: bool) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("Rust impl source must parse")
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Impl(item)
                if item.trait_.is_none()
                    && matches!(
                        item.self_ty.as_ref(),
                        Type::Path(path)
                            if path.path.segments.last().is_some_and(
                                |segment| segment.ident == name
                            )
                    ) =>
            {
                Some(item.items)
            }
            _ => None,
        })
        .flatten()
        .filter_map(|item| match item {
            ImplItem::Fn(method)
                if matches!(method.vis, Visibility::Public(_))
                    && (!async_only || method.sig.asyncness.is_some()) =>
            {
                Some(method.sig.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

fn python_class_methods(source: &str, name: &str, async_only: bool) -> BTreeSet<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut methods = BTreeSet::new();
    for line in source.lines() {
        if !found {
            found = line.starts_with(&marker);
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        let Some(member) = line.strip_prefix("    ") else {
            continue;
        };
        if member.starts_with(' ') {
            continue;
        }
        let declaration = if let Some(value) = member.strip_prefix("async def ") {
            value
        } else if !async_only {
            match member.strip_prefix("def ") {
                Some(value) => value,
                None => continue,
            }
        } else {
            continue;
        };
        let method = declaration.split('(').next().expect("method has a name");
        if !method.starts_with('_') {
            methods.insert(method.to_owned());
        }
    }
    assert!(found, "Python class {name} must exist");
    methods
}

fn python_enum_values(source: &str, name: &str) -> BTreeSet<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut values = BTreeSet::new();
    for line in source.lines() {
        if !found {
            found = line.starts_with(&marker);
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        let Some(member) = line.strip_prefix("    ") else {
            continue;
        };
        if member.starts_with(' ') {
            continue;
        }
        if let Some((_, value)) = member.split_once(" = ") {
            values.insert(value.trim_matches('"').to_owned());
        }
    }
    assert!(found, "Python enum {name} must exist");
    values
}

fn python_enum_names(source: &str, name: &str) -> BTreeSet<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut names = BTreeSet::new();
    for line in source.lines() {
        if !found {
            found = line.starts_with(&marker);
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        let Some(member) = line.strip_prefix("    ") else {
            continue;
        };
        if member.starts_with(' ') {
            continue;
        }
        if let Some((name, _)) = member.split_once(" = ") {
            names.insert(name.to_ascii_lowercase());
        }
    }
    assert!(found, "Python enum {name} must exist");
    names
}

fn python_function<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("def {name}(");
    let start = source
        .lines()
        .enumerate()
        .find_map(|(index, line)| line.starts_with(&marker).then_some(index))
        .unwrap_or_else(|| panic!("Python function {name} must exist"));
    let lines = source.lines().collect::<Vec<_>>();
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (!line.is_empty() && !line.starts_with(' ')).then_some(index))
        .unwrap_or(lines.len());
    let start_offset: usize = lines[..=start].iter().map(|line| line.len() + 1).sum();
    let end_offset: usize = lines[..end].iter().map(|line| line.len() + 1).sum();
    &source[start_offset..end_offset.min(source.len())]
}

fn python_event_kinds(source: &str, function: &str) -> BTreeSet<String> {
    let function = python_function(source, function);
    let start = function
        .find("model = {")
        .expect("event decoder must contain a model map")
        + "model = {".len();
    let end = function[start..]
        .find("}.get(kind)")
        .expect("event decoder model map must end")
        + start;
    let mut values = function[start..end]
        .split(',')
        .filter_map(|entry| entry.split_once(':').map(|(key, _)| key))
        .map(str::trim)
        .map(|key| key.trim_matches('"').to_owned())
        .collect::<BTreeSet<_>>();
    values.insert("reconnected".to_owned());
    values
}

fn python_error_kinds(source: &str) -> BTreeSet<String> {
    python_function(source, "_error_from_wire")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("if kind == \""))
        .filter_map(|value| value.split_once('"').map(|(kind, _)| kind.to_owned()))
        .collect()
}

fn python_assigned_constructor_values(source: &str, prefix: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter(|line| line.starts_with(prefix))
        .filter_map(|line| line.split_once("(\"").map(|(_, value)| value))
        .filter_map(|value| value.split_once('"').map(|(value, _)| value.to_owned()))
        .collect()
}

fn dart_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing {marker}"));
    let source = &source[start..];
    let open = source
        .find('{')
        .expect("Dart declaration must open a block");
    let mut depth = 0_usize;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("Dart declaration {marker} must close its block")
}

fn dart_enum_values(source: &str, name: &str) -> BTreeSet<String> {
    dart_block(source, &format!("enum {name}"))
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn dart_factory_names(source: &str, marker: &str) -> BTreeSet<String> {
    let type_name = marker.split_whitespace().last().expect("type name");
    let prefix = format!("const factory {type_name}.");
    dart_block(source, marker)
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name.to_owned()))
        .collect()
}

fn dart_static_and_factory_names(source: &str, marker: &str) -> BTreeSet<String> {
    let type_name = marker.split_whitespace().last().expect("type name");
    let factory_prefix = format!("factory {type_name}.");
    dart_block(source, marker)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("static const ") {
                return value.split_whitespace().next().map(ToOwned::to_owned);
            }
            line.strip_prefix(&factory_prefix)
                .and_then(|value| value.split_once('(').map(|(name, _)| name.to_owned()))
        })
        .collect()
}

fn dart_class_members(source: &str, marker: &str, future_only: bool) -> BTreeSet<String> {
    let block = dart_block(source, marker);
    let mut depth = 0_isize;
    let mut members = BTreeSet::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if depth == 0 && !trimmed.is_empty() {
            let name = if !future_only && trimmed.starts_with("final ") && trimmed.ends_with(';') {
                trimmed.trim_end_matches(';').split_whitespace().next_back()
            } else if !future_only {
                trimmed
                    .split_once(" get ")
                    .map(|(_, value)| value.split_whitespace().next().unwrap_or(value))
            } else {
                None
            }
            .or_else(|| {
                if future_only && !trimmed.starts_with("Future<") {
                    return None;
                }
                let prefix = trimmed.split_once('(')?.0;
                prefix.split_whitespace().next_back()
            });
            if let Some(name) = name.filter(|name| {
                !name.starts_with('_')
                    && !name.contains('.')
                    && !matches!(*name, "if" | "for" | "switch" | "while")
                    && !name
                        .chars()
                        .next()
                        .is_some_and(|character| character.is_ascii_uppercase())
            }) {
                members.insert(name.trim_end_matches(';').to_owned());
            }
        }
        depth += line.chars().filter(|value| *value == '{').count() as isize;
        depth -= line.chars().filter(|value| *value == '}').count() as isize;
    }
    members
}

fn camel_case(value: &str) -> String {
    let mut result = String::new();
    let mut uppercase = false;
    for character in value.chars() {
        if character == '_' {
            uppercase = true;
        } else if uppercase {
            result.extend(character.to_uppercase());
            uppercase = false;
        } else {
            result.push(character);
        }
    }
    result
}

fn snake_case(value: &str) -> String {
    let mut result = String::new();
    for character in value.chars() {
        if character.is_ascii_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
        } else {
            result.push(character);
        }
    }
    result
}

fn rust_variant_ids(source: &str, name: &str) -> BTreeSet<String> {
    rust_enum_variants(source, name)
        .into_iter()
        .map(|value| snake_case(&value))
        .collect()
}

fn dart_variant_ids(source: &str, name: &str) -> BTreeSet<String> {
    dart_enum_values(source, name)
        .into_iter()
        .map(|value| snake_case(&value))
        .collect()
}

#[test]
fn exchange_and_feature_inventories_match_every_language() {
    let exchanges = Exchange::ALL
        .map(|value| value.id().to_owned())
        .into_iter()
        .collect::<BTreeSet<_>>();
    let features = Feature::ALL
        .map(|value| value.id().to_owned())
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(python_enum_values(PYTHON_MODELS, "Exchange"), exchanges);
    assert_eq!(python_enum_values(PYTHON_MODELS, "Feature"), features);
    assert_eq!(dart_enum_values(DART_MODELS, "Exchange"), exchanges);
    assert_eq!(
        dart_enum_values(DART_MODELS, "Feature")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        features
    );
}

#[test]
fn adapter_and_client_methods_match_every_language() {
    let adapter = rust_trait_methods(CORE_ADAPTER, "Adapter");
    let mut python_adapter = adapter.clone();
    python_adapter.insert("features".to_owned());
    assert_eq!(
        python_class_methods(PYTHON_API, "Adapter", false),
        python_adapter
    );
    assert_eq!(
        dart_class_members(DART_ADAPTER, "abstract interface class Adapter", false),
        python_adapter
            .iter()
            .map(|value| camel_case(value))
            .collect()
    );

    let mut client = rust_public_methods(CORE_CLIENT, "Client", false);
    client.remove("new");
    assert_eq!(python_class_methods(PYTHON_API, "Client", false), client);
    client.remove("into_adapter");
    assert_eq!(
        dart_class_members(DART_CLIENT, "final class Client", false),
        client.iter().map(|value| camel_case(value)).collect()
    );
}

#[test]
fn adapter_calls_and_replies_match_every_language() {
    let calls = rust_enum_variants(COMMON_CONTRACT, "AdapterCall");
    let python_production = PYTHON_RUST_ADAPTER
        .split("#[cfg(test)]")
        .next()
        .expect("Python adapter production source");
    assert_eq!(
        qualified_variants(python_production, "AdapterCall::"),
        calls
    );

    let mut dart_calls = calls.clone();
    dart_calls.insert("CancelStream".to_owned());
    assert_eq!(
        rust_enum_variants(DART_RUST_ADAPTER, "AdapterCall"),
        dart_calls
    );
    let dart_production = DART_RUST_ADAPTER
        .split("#[cfg(test)]")
        .next()
        .expect("Dart adapter production source");
    assert_eq!(
        qualified_variants(dart_production, "CommonAdapterCall::"),
        calls
    );

    let replies = rust_enum_variants(COMMON_CONTRACT, "AdapterReply");
    assert_eq!(
        rust_enum_variants(PYTHON_RUST_ADAPTER, "ReplyKind"),
        replies
    );
    let mut dart_replies = replies;
    dart_replies.remove("MarketStream");
    dart_replies.remove("AccountStream");
    assert_eq!(
        rust_enum_variants(DART_RUST_ADAPTER, "AdapterReply"),
        dart_replies
    );
}

#[test]
fn stream_event_and_error_variants_match_every_language() {
    let stream_types = include_str!("../../../src/types/stream.rs");
    let market_events = rust_variant_ids(stream_types, "MarketEvent");
    assert_eq!(
        python_event_kinds(PYTHON_ADAPTERS, "_decode_market_event"),
        market_events
    );
    assert_eq!(
        dart_factory_names(DART_MODELS, "sealed class MarketEvent")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        market_events
    );

    let account_events = rust_variant_ids(stream_types, "AccountEvent");
    assert_eq!(
        python_event_kinds(PYTHON_ADAPTERS, "_decode_account_event"),
        account_events
    );
    assert_eq!(
        dart_factory_names(DART_MODELS, "sealed class AccountEvent")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        account_events
    );

    let errors = rust_variant_ids(include_str!("../../../src/error.rs"), "Error");
    assert_eq!(python_error_kinds(PYTHON_API), errors);
    assert_eq!(
        dart_variant_ids(DART_GENERATED_CONVERT, "NativeErrorKind"),
        errors
    );
}

#[test]
fn public_enum_variants_match_every_language() {
    let inventories = [
        (
            "MarketKind",
            include_str!("../../../src/types/market.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "MarketStatus",
            include_str!("../../../src/types/market.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "Side",
            include_str!("../../../src/types/data.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "Interval",
            include_str!("../../../src/types/data.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "Overflow",
            include_str!("../../../src/types/stream.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "MarginMode",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "OrderStatus",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "OrderType",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "TimeInForce",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "ExchangeErrorKind",
            include_str!("../../../src/error.rs"),
            PYTHON_API,
            DART_ERRORS,
        ),
        (
            "UpbitRegion",
            include_str!("../../../src/adapters/upbit/mod.rs"),
            PYTHON_MODELS,
            DART_PROVIDERS,
        ),
        (
            "BithumbAlertStep",
            include_str!("../../../src/adapters/bithumb/mod.rs"),
            PYTHON_MODELS,
            DART_PROVIDERS,
        ),
        (
            "BinanceMarket",
            include_str!("../../../src/adapters/binance/mod.rs"),
            PYTHON_MODELS,
            DART_PROVIDERS,
        ),
    ];

    for (name, rust_source, python_source, dart_source) in inventories {
        let variants = rust_variant_ids(rust_source, name);
        assert_eq!(
            python_enum_names(python_source, name),
            variants,
            "Python {name} variants differ"
        );
        assert_eq!(
            dart_variant_ids(dart_source, name),
            variants,
            "Dart {name} variants differ"
        );
    }

    let sizes = rust_variant_ids(include_str!("../../../src/types/account.rs"), "Size");
    assert_eq!(python_enum_names(PYTHON_MODELS, "SizeKind"), sizes);
    assert_eq!(
        dart_factory_names(DART_MODELS, "sealed class Size")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        sizes
    );

    let feeds = rust_variant_ids(include_str!("../../../src/types/stream.rs"), "Feed");
    let mut python_feeds = python_assigned_constructor_values(PYTHON_MODELS, "Feed.");
    if python_class_methods(PYTHON_MODELS, "Feed", false).contains("candles") {
        python_feeds.insert("candles".to_owned());
    }
    assert_eq!(python_feeds, feeds);
    assert_eq!(dart_variant_ids(DART_MODELS, "FeedKind"), feeds);

    let ledger_kinds = rust_variant_ids(
        include_str!("../../../src/adapters/hyperliquid/native.rs"),
        "HyperliquidLedgerKind",
    );
    let mut python_ledger_kinds = python_enum_names(PYTHON_MODELS, "HyperliquidLedgerKind");
    assert!(PYTHON_MODELS.contains("def _missing_(cls, value: object) -> HyperliquidLedgerKind:"));
    python_ledger_kinds.insert("other".to_owned());
    assert_eq!(python_ledger_kinds, ledger_kinds);
    assert_eq!(
        dart_static_and_factory_names(DART_PROVIDERS, "final class HyperliquidLedgerKind",)
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        ledger_kinds
    );
}

#[test]
fn provider_specific_methods_match_every_language() {
    let providers = [
        (
            "upbit",
            "UpbitAdapter",
            include_str!("../../../src/adapters/upbit/mod.rs"),
        ),
        (
            "bithumb",
            "BithumbAdapter",
            include_str!("../../../src/adapters/bithumb/mod.rs"),
        ),
        (
            "binance",
            "BinanceAdapter",
            include_str!("../../../src/adapters/binance/mod.rs"),
        ),
        (
            "hyperliquid",
            "HyperliquidAdapter",
            include_str!("../../../src/adapters/hyperliquid/mod.rs"),
        ),
    ];
    assert_eq!(
        providers
            .iter()
            .map(|(exchange, _, _)| (*exchange).to_owned())
            .collect::<BTreeSet<_>>(),
        Exchange::ALL
            .map(|value| value.id().to_owned())
            .into_iter()
            .collect()
    );

    for (_, adapter, rust_source) in providers {
        let mut methods = rust_public_methods(rust_source, adapter, false);
        for constructor in [
            "new",
            "with_region",
            "with_credentials",
            "spot",
            "usd_m_futures",
            "testnet",
            "with_wallet",
        ] {
            methods.remove(constructor);
        }

        let mut python_methods = python_class_methods(PYTHON_ADAPTERS, adapter, false);
        for constructor in ["spot", "usd_m_futures", "testnet"] {
            python_methods.remove(constructor);
        }
        assert_eq!(
            python_methods, methods,
            "Python {adapter} provider methods differ"
        );

        let mut dart_methods =
            dart_class_members(DART_ADAPTERS, &format!("final class {adapter}"), false);
        dart_methods.remove("withCredentials");
        dart_methods.remove("withWallet");
        assert_eq!(
            dart_methods,
            methods.iter().map(|value| camel_case(value)).collect(),
            "Dart {adapter} provider methods differ"
        );
    }
}
