use std::collections::BTreeSet;

use maxt_bindings_common::schema::{
    ApiType, Field, Record, Schema, TaggedUnion, Type, uses_generated_native_provider_bridge,
};

use crate::typescript_contract::HEADER;

/// Renders provider result models which are introduced by the schema-driven
/// native bridge. Core portable models remain in `models.ts`; these generated
/// models are re-exported from that module.
pub(crate) fn render(schema: &Schema) -> String {
    let names = provider_model_names(schema);
    let mut output = String::from(HEADER);
    output.push_str(
        "import type * as Core from \"../models.js\";\n\nconst U32_MAX = 0xffff_ffff;\nconst U64_MAX = (1n << 64n) - 1n;\n\nfunction checkedU32(value: number, field: string): number {\n  if (!Number.isSafeInteger(value) || value < 0 || value > U32_MAX) {\n    throw new RangeError(`${field} must be a u32`);\n  }\n  return value;\n}\n\nfunction checkedU64(value: bigint, field: string): bigint {\n  if (value < 0n || value > U64_MAX) throw new RangeError(`${field} must be a u64`);\n  return value;\n}\n\nfunction freezeRecord<T extends object>(value: T): T { return Object.freeze(value); }\n\n",
    );
    for name in names {
        if let Some(record) = schema
            .records
            .iter()
            .find(|record| record.name == format!("{name}Wire"))
        {
            output.push_str(&render_record(&name, record, schema));
        } else if let Some(union) = schema
            .unions
            .iter()
            .find(|union| union.name == format!("{name}Wire"))
        {
            output.push_str(&render_union(&name, union, schema));
        } else {
            panic!("generated TypeScript provider model {name} has no schema shape");
        }
    }
    output.truncate(output.trim_end().len());
    output.push('\n');
    output
}

pub(crate) fn provider_model_names(schema: &Schema) -> Vec<String> {
    provider_model_names_for(schema, false)
}

/// Models that need Rust wire conversion for provider bridges.
///
/// Existing provider streams may use handwritten TypeScript public models, but
/// their event wire conversion still follows the schema-generated native path.
pub(crate) fn provider_wire_model_names(schema: &Schema) -> Vec<String> {
    provider_model_names_for(schema, true)
}

fn provider_model_names_for(schema: &Schema, include_all_streams: bool) -> Vec<String> {
    let mut names = BTreeSet::new();
    for provider in schema.providers {
        for method in provider.methods.iter().filter(|method| {
            uses_generated_native_provider_bridge(provider.exchange, method.rust_name)
                || (include_all_streams
                    && matches!(
                        method.result,
                        ApiType::ProviderMarketStream(_) | ApiType::ProviderAccountStream(_)
                    ))
        }) {
            match method.result {
                ApiType::Named(name) | ApiType::List(name) | ApiType::Page(name) => {
                    insert_model(schema, name, &mut names);
                }
                ApiType::ProviderMarketStream(event) | ApiType::ProviderAccountStream(event) => {
                    insert_model(schema, event, &mut names);
                }
                _ => {}
            }
        }
    }
    names.into_iter().collect()
}

fn insert_model(schema: &Schema, name: &str, names: &mut BTreeSet<String>) {
    if core_model(name) || !names.insert(name.to_owned()) {
        return;
    }
    let wire = format!("{name}Wire");
    if let Some(record) = schema.records.iter().find(|record| record.name == wire) {
        for field in &record.fields {
            insert_type(schema, &field.ty, names);
        }
    } else if let Some(union) = schema.unions.iter().find(|union| union.name == wire) {
        for variant in &union.variants {
            for field in &variant.fields {
                insert_type(schema, &field.ty, names);
            }
        }
    } else {
        panic!("generated TypeScript provider model {name} has no schema shape");
    }
}

fn insert_type(schema: &Schema, ty: &Type, names: &mut BTreeSet<String>) {
    match ty {
        Type::Named(name) if name.ends_with("Wire") => {
            insert_model(schema, name.trim_end_matches("Wire"), names);
        }
        Type::Optional(inner) | Type::List(inner) => insert_type(schema, inner, names),
        Type::Tuple(items) => {
            for item in items {
                insert_type(schema, item, names);
            }
        }
        _ => {}
    }
}

pub(crate) fn core_model(name: &str) -> bool {
    matches!(
        name,
        "String"
            | "Decimal"
            | "Timestamp"
            | "Cursor"
            | "Market"
            | "MarketInfo"
            | "Trade"
            | "Level"
            | "OrderBook"
            | "Ticker"
            | "Candle"
            | "Balance"
            | "Order"
            | "Position"
            | "MarginSummary"
            | "FundingRate"
            | "FundingPayment"
            | "Deposit"
            | "Withdrawal"
            | "CancelOrdersResult"
            | "HyperliquidBookLevel"
            | "HyperliquidCandleSnapshot"
            | "HyperliquidL2Book"
            | "HyperliquidMidPrice"
            | "HyperliquidRecentTrade"
            | "HyperliquidSpotBalance"
            // This older public model intentionally retains its convenience
            // getter, while generated detail records can reference it.
            | "UpbitCancelAndNewOrderResult"
    )
}

fn render_record(name: &str, record: &Record, schema: &Schema) -> String {
    let generated = provider_model_names(schema)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let declarations = record
        .fields
        .iter()
        .filter(|field| needs_assignment(&field.ty))
        .map(|field| {
            format!(
                "  readonly {}: {};\n",
                camel(field.name),
                typescript_type(&field.ty, &generated)
            )
        })
        .collect::<String>();
    let parameters = record
        .fields
        .iter()
        .map(|field| {
            let field_name = camel(field.name);
            let ty = typescript_type(&field.ty, &generated);
            if needs_assignment(&field.ty) {
                format!("    {field_name}: {ty},\n")
            } else {
                format!("    readonly {field_name}: {ty},\n")
            }
        })
        .collect::<String>();
    let assignments = record
        .fields
        .iter()
        .filter(|field| needs_assignment(&field.ty))
        .map(|field| render_assignment(field))
        .collect::<String>();
    format!(
        "export class {name} {{\n{declarations}  constructor(\n{parameters}  ) {{\n{assignments}    freezeRecord(this);\n  }}\n}}\n\n"
    )
}

fn render_union(name: &str, union: &TaggedUnion, schema: &Schema) -> String {
    let generated = provider_model_names(schema)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let variants = union
        .variants
        .iter()
        .map(|variant| {
            let fields = variant
                .fields
                .iter()
                .map(|field| {
                    format!(
                        "; readonly {}: {}",
                        camel(field.name),
                        typescript_type(&field.ty, &generated)
                    )
                })
                .collect::<String>();
            format!("  | {{ readonly kind: \"{}\"{fields} }}\n", variant.name)
        })
        .collect::<String>();
    format!("export type {name} =\n{variants};\n")
}

fn needs_assignment(ty: &Type) -> bool {
    match ty {
        Type::Number | Type::UnsignedInteger | Type::List(_) => true,
        Type::Optional(inner) => {
            matches!(
                inner.as_ref(),
                Type::Number | Type::UnsignedInteger | Type::List(_)
            )
        }
        _ => false,
    }
}

fn render_assignment(field: &Field) -> String {
    let name = camel(field.name);
    match &field.ty {
        Type::Number => format!("    this.{name} = checkedU32({name}, \"{name}\");\n"),
        Type::UnsignedInteger => {
            format!("    this.{name} = checkedU64({name}, \"{name}\");\n")
        }
        Type::List(_) => format!("    this.{name} = Object.freeze([...{name}]);\n"),
        Type::Optional(inner) if matches!(inner.as_ref(), Type::Number) => {
            format!("    this.{name} = {name} === null ? null : checkedU32({name}, \"{name}\");\n")
        }
        Type::Optional(inner) if matches!(inner.as_ref(), Type::UnsignedInteger) => {
            format!("    this.{name} = {name} === null ? null : checkedU64({name}, \"{name}\");\n")
        }
        Type::Optional(inner) if matches!(inner.as_ref(), Type::List(_)) => {
            format!("    this.{name} = {name} === null ? null : Object.freeze([...{name}]);\n")
        }
        _ => unreachable!("needs_assignment must recognize every rendered assignment"),
    }
}

fn typescript_type(ty: &Type, generated: &BTreeSet<String>) -> String {
    match ty {
        Type::String => "string".to_owned(),
        Type::Boolean => "boolean".to_owned(),
        Type::Number => "number".to_owned(),
        Type::UnsignedInteger => "bigint".to_owned(),
        Type::Decimal => "Core.Decimal".to_owned(),
        Type::Timestamp => "Core.Timestamp".to_owned(),
        Type::Identifier(name) => format!("Core.{name}"),
        Type::Named(name) if name.ends_with("Wire") => {
            let name = name.trim_end_matches("Wire");
            if generated.contains(name) {
                name.to_owned()
            } else {
                format!("Core.{name}")
            }
        }
        Type::Named(name) => format!("Core.{name}"),
        Type::Optional(inner) => format!("{} | null", typescript_type(inner, generated)),
        Type::List(inner) => format!("readonly {}[]", typescript_type(inner, generated)),
        Type::Tuple(items) => format!(
            "readonly [{}]",
            items
                .iter()
                .map(|item| typescript_type(item, generated))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn camel(value: &str) -> String {
    let mut output = String::new();
    let mut upper = false;
    for character in value.chars() {
        if character == '_' {
            upper = true;
        } else if upper {
            output.push(character.to_ascii_uppercase());
            upper = false;
        } else {
            output.push(character);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use maxt_bindings_common::schema::binding_schema;

    use super::render;

    #[test]
    fn provider_results_are_rendered_from_the_shared_registry() {
        let output = render(&binding_schema());
        for expected in [
            "export class UpbitOrderResponse",
            "export class BithumbOrderBookSnapshot",
            "export class BinanceOrderResponse",
            "export class HyperliquidProviderResponse",
        ] {
            assert!(output.contains(expected), "missing {expected}");
        }
    }
}
