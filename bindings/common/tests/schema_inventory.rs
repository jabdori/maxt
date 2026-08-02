//! Generated binding schema parity with the Rust public contract.

#![cfg(feature = "codegen")]

use std::collections::BTreeSet;

use maxt_bindings_common::schema::binding_schema;
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
    let actual = source
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
    let expected = binding_schema()
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
