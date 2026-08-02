//! Core and foreign adapter method inventory parity.

use std::collections::BTreeSet;

use syn::{ImplItem, Item, TraitItem, Type};

#[test]
fn foreign_adapter_explicitly_implements_every_core_adapter_method() {
    let core = syn::parse_file(include_str!("../../../src/adapter.rs"))
        .expect("핵심 Adapter 소스를 Rust 문법으로 파싱해야 합니다");
    let bridge = syn::parse_file(include_str!("../src/foreign.rs"))
        .expect("외부 Adapter bridge 소스를 Rust 문법으로 파싱해야 합니다");

    let core_methods = core
        .items
        .iter()
        .find_map(|item| match item {
            Item::Trait(item) if item.ident == "Adapter" => Some(
                item.items
                    .iter()
                    .filter_map(|item| match item {
                        TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .expect("핵심 Adapter trait이 존재해야 합니다");

    let bridge_methods = bridge
        .items
        .iter()
        .find_map(|item| match item {
            Item::Impl(item)
                if item
                    .trait_
                    .as_ref()
                    .and_then(|(_, path, _)| path.segments.last())
                    .is_some_and(|segment| segment.ident == "Adapter")
                    && matches!(
                        item.self_ty.as_ref(),
                        Type::Path(path)
                            if path.path.segments.last().is_some_and(
                                |segment| segment.ident == "ForeignAdapter"
                            )
                    ) =>
            {
                Some(
                    item.items
                        .iter()
                        .filter_map(|item| match item {
                            ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
                            _ => None,
                        })
                        .collect::<BTreeSet<_>>(),
                )
            }
            _ => None,
        })
        .expect("ForeignAdapter의 Adapter 구현이 존재해야 합니다");

    assert_eq!(bridge_methods, core_methods);
}
