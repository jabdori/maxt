use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "rust")]
use maxt_bindings_common::schema::Schema;
use maxt_bindings_common::schema::binding_schema;

#[cfg(feature = "dart")]
mod dart;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "typescript")]
mod typescript_api;
#[cfg(feature = "typescript")]
mod typescript_codec;
#[cfg(feature = "typescript")]
mod typescript_contract;

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let check = arguments.iter().any(|argument| argument == "--check");
    let targets = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--check")
        .collect::<Vec<_>>();
    assert!(targets.len() <= 1, "only one generation target is allowed");
    let target = targets.first().map_or("all", |target| target.as_str());
    assert!(
        matches!(target, "all" | "rust" | "python" | "dart" | "typescript"),
        "target must be one of: all, rust, python, dart, typescript"
    );
    let target_enabled = (target == "rust" && cfg!(feature = "rust"))
        || (target == "python" && cfg!(feature = "python"))
        || (target == "dart" && cfg!(feature = "dart"))
        || (target == "typescript" && cfg!(feature = "typescript"));
    assert!(
        target == "all" || target_enabled,
        "the requested target feature is disabled"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("codegen crate must be below the repository root")
        .to_owned();
    let schema = binding_schema();
    let mut outputs = Vec::new();
    #[cfg(feature = "typescript")]
    outputs.extend([
        (
            "typescript",
            root.join("bindings/typescript/src/generated/contract.ts"),
            typescript_contract::render(&schema),
        ),
        (
            "typescript",
            root.join("bindings/typescript/src/generated/identifiers.ts"),
            typescript_contract::render_identifiers(&schema),
        ),
        (
            "typescript",
            root.join("bindings/typescript/src/generated/codec.ts"),
            typescript_codec::render(&schema),
        ),
        (
            "typescript",
            root.join("bindings/typescript/src/generated/api.ts"),
            typescript_api::render(&schema),
        ),
    ]);
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/python/maxt/_generated_contract.py"),
        python::render(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/python/maxt/_generated_identifiers.py"),
        python::render_identifiers(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/python/maxt/_generated_api.py"),
        python::render_api(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/python/maxt/_generated_delegate.py"),
        python::render_native_delegate(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/python/maxt/_generated_wire.py"),
        python::render_wire_schema(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/python/maxt/_native.pyi"),
        python::render_native_stub(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/src/generated/client_methods.rs"),
        python::render_rust_client(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/src/generated/adapter_dispatch.rs"),
        python::render_rust_dispatcher(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/src/generated/convert.rs"),
        python::render_rust_convert(&schema),
    ));
    #[cfg(feature = "python")]
    outputs.push((
        "python",
        root.join("bindings/python/src/generated/provider_convert.rs"),
        python::render_rust_provider_convert(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_contract.dart"),
        dart::render(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_identifiers.dart"),
        dart::render_identifiers(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_adapter.dart"),
        dart::render_adapter_api(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_client.dart"),
        dart::render_client_api(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_provider_guard.dart"),
        dart::render_provider_guard(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_provider_methods.dart"),
        dart::render_provider_methods(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_delegate.dart"),
        dart::render_delegate_methods(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/lib/src/generated_wire_converters.dart"),
        dart::render_wire_converters(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/rust/src/api/generated_native_client.rs"),
        dart::render_native_client_api(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/rust/src/adapter/generated_dispatch.rs"),
        dart::render_rust_adapter_dispatch(&schema),
    ));
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/rust/src/convert/generated_shape_guard.rs"),
        dart::render_wire_shape_guard(&schema),
    ));
    #[cfg(feature = "rust")]
    outputs.push((
        "rust",
        root.join("bindings/common/generated/api.md"),
        render_markdown(&schema),
    ));
    for (output_target, path, content) in outputs {
        if target != "all" && target != output_target {
            continue;
        }
        if check {
            let actual = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} is missing: {error}", path.display()));
            assert_eq!(actual, content, "{} is stale", path.display());
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("generated output directory must be writable");
            }
            fs::write(&path, content)
                .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
        }
    }
}

#[cfg(feature = "rust")]
fn render_markdown(schema: &Schema) -> String {
    let mut output = String::from(
        "<!-- Generated by `cargo run -p maxt-bindings-codegen`. Do not edit. -->\n\n# Binding contract\n\n## Adapter operations\n\n| Rust / Python | Dart / TypeScript |\n| --- | --- |\n",
    );
    for operation in schema.adapter_operations {
        output.push_str(&format!(
            "| `{}` | `{}` |\n",
            operation.rust_name, operation.language_name
        ));
    }
    output.push_str("\n## Provider-specific API\n\n| Exchange | Adapter | Python | Dart / TypeScript |\n| --- | --- | --- | --- |\n");
    for provider in schema.providers {
        let methods = provider
            .methods
            .iter()
            .map(|method| format!("`{}`", method.name))
            .collect::<Vec<_>>()
            .join(", ");
        let python_methods = provider
            .methods
            .iter()
            .map(|method| format!("`{}`", method.rust_name))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "| {} | `{}` | {} | {} |\n",
            provider.exchange, provider.adapter, python_methods, methods
        ));
    }
    output
}

#[cfg(all(test, any(feature = "python", feature = "typescript")))]
mod tests {
    use super::*;

    #[cfg(feature = "typescript")]
    #[test]
    fn generated_typescript_contains_every_contract_axis() {
        let output = typescript_contract::render(&binding_schema());
        for expected in [
            "MarketWire",
            "SizeWire",
            "EXCHANGES",
            "FEATURES",
            "ADAPTER_OPERATIONS",
            "CLIENT_MEMBERS",
            "PROVIDER_METHODS",
        ] {
            assert!(output.contains(expected), "missing {expected}");
        }
    }

    #[cfg(feature = "python")]
    #[test]
    fn python_names_are_derived_from_the_same_contract() {
        assert_eq!(python::snake_case("orderBook"), "order_book");
        assert_eq!(
            python::snake_case("usdMCreateListenKey"),
            "usd_m_create_listen_key"
        );
    }
}
