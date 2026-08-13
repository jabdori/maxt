use std::env;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(feature = "dart")]
use std::process::Command;

#[cfg(feature = "rust")]
use maxt_bindings_common::coverage::{
    Authentication, Availability, BASELINE_DATE, CatalogScope, OPERATIONS, OperationMapping,
    OperationRisk, REGIONAL_PRODUCT_COUNTS,
};
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
    let any_target_enabled = cfg!(feature = "rust")
        || cfg!(feature = "python")
        || cfg!(feature = "dart")
        || cfg!(feature = "typescript");
    assert!(
        (target == "all" && any_target_enabled) || target_enabled,
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
        root.join("bindings/python/python/maxt/_generated_models.py"),
        python::render_models(&schema),
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
        root.join("bindings/dart/lib/src/generated_models.dart"),
        dart::render_models(&schema),
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
    #[cfg(feature = "dart")]
    outputs.push((
        "dart",
        root.join("bindings/dart/rust/src/convert/generated_models.rs"),
        dart::render_rust_models(&schema),
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
        #[cfg(feature = "dart")]
        let content = if output_target == "dart" {
            match path.extension().and_then(|extension| extension.to_str()) {
                Some("dart") => format_dart_source(&content),
                Some("rs") => format_rust_source(&content),
                _ => content,
            }
        } else {
            content
        };
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

#[cfg(feature = "dart")]
fn format_dart_source(source: &str) -> String {
    let path = env::temp_dir().join(format!("maxt-bindings-codegen-{}.dart", std::process::id()));
    fs::write(&path, source).expect("temporary Dart source must be writable");
    let result = Command::new("dart")
        .args(["format", "--output=show"])
        .arg(&path)
        .output()
        .expect("Dart SDK is required to generate Dart bindings");
    let _ = fs::remove_file(&path);
    assert!(
        result.status.success(),
        "dart format failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = String::from_utf8(result.stdout)
        .expect("dart format output must be UTF-8")
        .replace("\r\n", "\n");
    let (formatted, summary) = output
        .trim_end_matches('\n')
        .rsplit_once('\n')
        .expect("dart format must return source and a summary");
    assert!(
        summary.starts_with("Formatted "),
        "unexpected dart format output: {summary}"
    );
    format!("{formatted}\n")
}

#[cfg(feature = "dart")]
fn format_rust_source(source: &str) -> String {
    let path = env::temp_dir().join(format!("maxt-bindings-codegen-{}.rs", std::process::id()));
    fs::write(&path, source).expect("temporary Rust source must be writable");
    let result = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(&path)
        .output()
        .expect("rustfmt is required to generate Dart bindings");
    assert!(
        result.status.success(),
        "rustfmt failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = fs::read_to_string(&path).expect("formatted Rust source must be readable");
    let _ = fs::remove_file(&path);
    output.replace("\r\n", "\n")
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
    output.push_str(&format!(
        "\n## Official API products\n\nDocumentation baseline: `{BASELINE_DATE}`.\n\n| Exchange | Product | Mapped / official | Interfaces | Encodings | Status |\n| --- | --- | ---: | --- | --- | --- |\n"
    ));
    for product in schema.products {
        let official = product
            .endpoint_count
            .map_or_else(|| "—".to_owned(), |count| count.to_string());
        let interfaces = product
            .interfaces
            .iter()
            .map(|interface| format!("`{}`", interface.id()))
            .collect::<Vec<_>>()
            .join(", ");
        let encodings = product
            .encodings
            .iter()
            .map(|encoding| format!("`{}`", encoding.id()))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "| {} | {} | {} / {} | {} | {} | {} |\n",
            product.exchange.id(),
            product.name,
            product.mapped_operations(),
            official,
            interfaces,
            encodings,
            product.stage().label(),
        ));
    }
    if !REGIONAL_PRODUCT_COUNTS.is_empty() {
        output.push_str(
            "\n## Regional catalog counts\n\nThe product table above uses the Global catalog. Regional counts include active operations published only for that region.\n\n| Exchange | Scope | Product | Mapped / official | Status |\n| --- | --- | --- | ---: | --- |\n",
        );
        for count in REGIONAL_PRODUCT_COUNTS {
            let product = schema
                .products
                .iter()
                .find(|product| product.exchange == count.exchange && product.id == count.product)
                .expect("regional catalog count must name a product");
            let scope = match count.scope {
                CatalogScope::Global => "Global",
                CatalogScope::Korea => "Korea",
            };
            output.push_str(&format!(
                "| {} | {} | {} | {} / {} | {} |\n",
                count.exchange.id(),
                scope,
                product.name,
                product.mapped_operations_for(count.scope),
                count.endpoint_count,
                product.stage_for(count.scope).label(),
            ));
        }
    }
    output.push_str("\n## Recorded operations\n\n| Exchange | Product | Operation | Method | Path / message | Interface | Authentication | Risk | Availability | Mapping | Implementation | Validation |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for operation in OPERATIONS {
        let mapping = match operation.mapping {
            OperationMapping::Common(name) => format!("common `{name}`"),
            OperationMapping::CommonMany(names) => format!("common `{}`", names.join("`, `")),
            OperationMapping::Provider(name) => format!("provider `{name}`"),
            OperationMapping::CommonAndProvider { common, provider } => format!(
                "common `{}`; provider `{}`",
                common.join("`, `"),
                provider.join("`, `")
            ),
            OperationMapping::PlatformLimited { service, platform } => {
                format!("provider `{service}`; unavailable on {platform}")
            }
        };
        output.push_str(&format!(
            "| {} | {} | `{}` | `{}` | `{}` | `{}` | {} | {} | {} | {} | `{:?}` | `{:?}` |\n",
            operation.exchange.id(),
            operation.product,
            operation.id,
            operation.method,
            operation.path,
            operation.interface.id(),
            authentication_label(operation.authentication),
            risk_label(operation.risk),
            availability_label(operation.availability),
            mapping,
            operation.implementation,
            operation.validation,
        ));
    }
    output
}

#[cfg(feature = "rust")]
const fn authentication_label(authentication: Authentication) -> &'static str {
    match authentication {
        Authentication::Public => "public",
        Authentication::ApiKey => "API key",
        Authentication::Hmac => "HMAC",
        Authentication::Jwt => "JWT",
        Authentication::Rsa => "RSA",
        Authentication::Ed25519 => "Ed25519",
        Authentication::Eip712 => "EIP-712",
        Authentication::OAuth => "OAuth",
        Authentication::Partner => "partner credentials",
    }
}

#[cfg(feature = "rust")]
const fn risk_label(risk: OperationRisk) -> &'static str {
    match risk {
        OperationRisk::Read => "read",
        OperationRisk::AccountWrite => "account write",
        OperationRisk::FinancialWrite => "financial write",
        OperationRisk::AdministrativeWrite => "administrative write",
    }
}

#[cfg(feature = "rust")]
fn availability_label(availability: Availability) -> String {
    match availability {
        Availability::General => "general".to_owned(),
        Availability::Region(region) => format!("{region} only"),
        Availability::Partner => "partners only".to_owned(),
        Availability::Eligibility(condition) => format!("eligibility: {condition}"),
        Availability::Beta => "beta".to_owned(),
        Availability::Testnet => "testnet only".to_owned(),
    }
}

#[cfg(all(
    test,
    any(feature = "rust", feature = "python", feature = "typescript")
))]
mod tests {
    use super::*;

    #[cfg(feature = "rust")]
    #[test]
    fn generated_markdown_includes_endpoint_safety_metadata() {
        let output = render_markdown(&binding_schema());
        assert!(output.contains(
            "| Method | Path / message | Interface | Authentication | Risk | Availability |"
        ));
        assert!(output.contains(
            "| upbit | travel_rule | `travel_rule_vasps` | `GET` | `/v1/travel_rule/vasps` | `http` | JWT | read | Korea or Singapore only |"
        ));
        assert!(
            output.contains("| upbit | Korea | Deposits and withdrawals | 16 / 16 | Partial |")
        );
    }

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
