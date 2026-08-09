//! Generated binding schema parity with the Rust public contract.

#![cfg(feature = "codegen")]

use std::collections::BTreeSet;

use maxt::{Exchange, Feature};
use maxt_bindings_common::schema::{ProviderOptionValue, Type, binding_schema};
use syn::{ImplItem, Item, TraitItem};

fn snake_to_camel(value: &str) -> String {
    let mut output = String::new();
    let mut uppercase = false;
    for character in value.chars() {
        if character == '_' {
            uppercase = true;
        } else if uppercase {
            output.push(character.to_ascii_uppercase());
            uppercase = false;
        } else {
            output.push(character);
        }
    }
    output
}

#[test]
fn schema_adapter_operations_match_the_core_trait() {
    let source = syn::parse_file(include_str!("../../../src/adapter.rs")).unwrap();
    let actual = source
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Trait(item) if item.ident == "Adapter" => Some(
                item.items
                    .into_iter()
                    .filter_map(|item| match item {
                        TraitItem::Fn(method)
                            if !matches!(
                                method.sig.ident.to_string().as_str(),
                                "exchange" | "supports"
                            ) =>
                        {
                            Some(method.sig.ident.to_string())
                        }
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .unwrap();
    let expected = binding_schema()
        .adapter_operations
        .iter()
        .map(|operation| operation.rust_name.to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn schema_client_members_match_the_public_binding_surface() {
    let source = syn::parse_file(include_str!("../../../src/client.rs")).unwrap();
    let mut actual = source
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Impl(item) => Some(
                item.items
                    .into_iter()
                    .filter_map(|item| match item {
                        ImplItem::Fn(method)
                            if matches!(method.vis, syn::Visibility::Public(_))
                                && !matches!(
                                    method.sig.ident.to_string().as_str(),
                                    "new" | "into_adapter"
                                ) =>
                        {
                            Some(snake_to_camel(&method.sig.ident.to_string()))
                        }
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .unwrap();
    let schema = binding_schema();
    let composition_source = syn::parse_file(include_str!("../../../src/wallet.rs")).unwrap();
    let composition_functions = composition_source
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Fn(function) if matches!(function.vis, syn::Visibility::Public(_)) => {
                Some(function.sig.ident.to_string())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for composition in schema.client_compositions {
        assert!(
            composition_functions.contains(composition.rust_name),
            "binding composition {} has no Rust function {}",
            composition.language_name,
            composition.rust_name
        );
        actual.insert(composition.language_name.to_owned());
    }
    let expected = schema
        .client_members
        .iter()
        .map(|member| (*member).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn schema_error_variants_match_the_core_error() {
    let source = syn::parse_file(include_str!("../../../src/error.rs")).unwrap();
    let actual = source
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == "Error" => Some(
                item.variants
                    .into_iter()
                    .map(|variant| variant.ident.to_string())
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .unwrap();
    let expected = binding_schema()
        .errors
        .iter()
        .map(|variant| (*variant).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

fn public_inherent_methods(source: &str, adapter: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Impl(item) if item.trait_.is_none() => Some(item),
            _ => None,
        })
        .filter(|item| match item.self_ty.as_ref() {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|value| value.ident == adapter),
            _ => false,
        })
        .flat_map(|item| item.items)
        .filter_map(|item| match item {
            ImplItem::Fn(method) if matches!(method.vis, syn::Visibility::Public(_)) => {
                Some(method.sig.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn schema_covers_every_exchange_provider() {
    let schema = binding_schema();
    let exchanges = schema.exchanges.into_iter().collect::<BTreeSet<_>>();
    let providers = schema
        .providers
        .iter()
        .map(|provider| provider.exchange)
        .collect::<BTreeSet<_>>();
    assert_eq!(providers, exchanges);
}

#[test]
fn schema_provider_methods_match_every_core_adapter() {
    let schema = binding_schema();
    for provider in schema.providers {
        let (source, construction) = match provider.exchange {
            "upbit" => (
                include_str!("../../../src/adapters/upbit/mod.rs"),
                &["new", "with_region", "with_credentials"][..],
            ),
            "bithumb" => (
                include_str!("../../../src/adapters/bithumb/mod.rs"),
                &["new", "with_credentials"][..],
            ),
            "binance" => (
                include_str!("../../../src/adapters/binance/mod.rs"),
                &["spot", "usd_m_futures", "with_credentials"][..],
            ),
            "hyperliquid" => (
                include_str!("../../../src/adapters/hyperliquid/mod.rs"),
                &[
                    "new",
                    "testnet",
                    "with_query_address",
                    "with_signer",
                    "with_wallet",
                ][..],
            ),
            exchange => panic!("provider source is not classified for {exchange}"),
        };
        let mut actual = public_inherent_methods(source, provider.adapter);
        for method in construction {
            assert!(
                actual.remove(*method),
                "{} construction method {method} is missing",
                provider.adapter
            );
        }
        let expected = provider
            .methods
            .iter()
            .map(|method| method.rust_name.to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual, expected,
            "{} provider methods differ",
            provider.adapter
        );
    }
}

#[test]
fn provider_constructors_cover_their_native_options() {
    let schema = binding_schema();
    for provider in schema.providers {
        let option_record = schema
            .records
            .iter()
            .find(|record| record.name == provider.options_wire)
            .unwrap_or_else(|| panic!("{} options record is missing", provider.exchange));
        let expected_options = option_record
            .fields
            .iter()
            .map(|field| field.name)
            .collect::<BTreeSet<_>>();
        for constructor in provider.constructors {
            let arguments = constructor
                .arguments
                .iter()
                .map(|argument| argument.name)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                arguments.len(),
                constructor.arguments.len(),
                "{}.{} argument names must be unique",
                provider.exchange,
                constructor.name,
            );
            let options = constructor
                .options
                .iter()
                .map(|option| option.name)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                options, expected_options,
                "{}.{} must set every native option",
                provider.exchange, constructor.name,
            );
            for option in constructor.options {
                match option.value {
                    ProviderOptionValue::Argument(name) => assert!(
                        arguments.contains(name),
                        "{}.{} option {} references missing argument {name}",
                        provider.exchange,
                        constructor.name,
                        option.name,
                    ),
                    ProviderOptionValue::Identifier { name, variant } => {
                        let identifier = schema.identifier(name).unwrap_or_else(|| {
                            panic!(
                                "{}.{} option {} references missing identifier {name}",
                                provider.exchange, constructor.name, option.name,
                            )
                        });
                        assert!(
                            identifier
                                .variants
                                .iter()
                                .any(|value| value.rust_name == variant),
                            "{}.{} option {} references missing {name}::{variant}",
                            provider.exchange,
                            constructor.name,
                            option.name,
                        );
                    }
                    ProviderOptionValue::Boolean(_) => {}
                }
            }
        }
    }
}

fn enum_variants(source: &str, name: &str) -> Vec<String> {
    syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == name => Some(
                item.variants
                    .into_iter()
                    .filter(|variant| variant.ident != "Other")
                    .map(|variant| variant.ident.to_string())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("enum {name} is missing"))
}

#[test]
fn schema_identifier_variants_match_the_core_enums() {
    let schema = binding_schema();
    let exchange = schema.identifier("Exchange").unwrap();
    assert_eq!(
        exchange
            .variants
            .iter()
            .map(|variant| variant.wire_name)
            .collect::<Vec<_>>(),
        Exchange::ALL.map(Exchange::id),
    );
    let feature = schema.identifier("Feature").unwrap();
    assert_eq!(
        feature
            .variants
            .iter()
            .map(|variant| variant.wire_name)
            .collect::<Vec<_>>(),
        Feature::ALL.map(Feature::id),
    );

    let sources = [
        (
            "MarketKind",
            "MarketKind",
            include_str!("../../../src/types/market.rs"),
        ),
        (
            "MarketStatus",
            "MarketStatus",
            include_str!("../../../src/types/market.rs"),
        ),
        ("Side", "Side", include_str!("../../../src/types/data.rs")),
        (
            "Interval",
            "Interval",
            include_str!("../../../src/types/data.rs"),
        ),
        (
            "Overflow",
            "Overflow",
            include_str!("../../../src/types/stream.rs"),
        ),
        (
            "MarginMode",
            "MarginMode",
            include_str!("../../../src/types/account.rs"),
        ),
        (
            "OrderStatus",
            "OrderStatus",
            include_str!("../../../src/types/account.rs"),
        ),
        (
            "OrderType",
            "OrderType",
            include_str!("../../../src/types/account.rs"),
        ),
        (
            "TimeInForce",
            "TimeInForce",
            include_str!("../../../src/types/account.rs"),
        ),
        (
            "SizeKind",
            "Size",
            include_str!("../../../src/types/account.rs"),
        ),
        (
            "UpbitRegion",
            "UpbitRegion",
            include_str!("../../../src/adapters/upbit/mod.rs"),
        ),
        (
            "BithumbAlertStep",
            "BithumbAlertStep",
            include_str!("../../../src/adapters/bithumb/mod.rs"),
        ),
        (
            "BinanceMarket",
            "BinanceMarket",
            include_str!("../../../src/adapters/binance/mod.rs"),
        ),
        (
            "HyperliquidLedgerKind",
            "HyperliquidLedgerKind",
            include_str!("../../../src/adapters/hyperliquid/native.rs"),
        ),
        (
            "ExchangeErrorKind",
            "ExchangeErrorKind",
            include_str!("../../../src/error.rs"),
        ),
    ];
    for (identifier_name, enum_name, source) in sources {
        let expected = schema
            .identifier(identifier_name)
            .unwrap()
            .variants
            .iter()
            .map(|variant| variant.rust_name.to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            enum_variants(source, enum_name),
            expected,
            "{identifier_name} variants differ",
        );
    }
}

fn referenced_identifiers(value: &Type, names: &mut Vec<&'static str>) {
    match value {
        Type::Identifier(name) => names.push(name),
        Type::Optional(value) | Type::List(value) => referenced_identifiers(value, names),
        Type::Tuple(values) => {
            for value in values {
                referenced_identifiers(value, names);
            }
        }
        _ => {}
    }
}

#[test]
fn every_identifier_reference_has_a_variant_contract() {
    let schema = binding_schema();
    let mut names = Vec::new();
    for record in &schema.records {
        for field in &record.fields {
            referenced_identifiers(&field.ty, &mut names);
        }
    }
    for union in &schema.unions {
        for variant in &union.variants {
            for field in &variant.fields {
                referenced_identifiers(&field.ty, &mut names);
            }
        }
    }
    names.sort_unstable();
    names.dedup();
    assert!(
        names
            .into_iter()
            .all(|name| schema.identifier(name).is_some())
    );
}
