# maxt binding contracts

[한국어](README.ko.md)

`maxt-bindings-common` defines requests, responses, errors, streams, and the
`ForeignAdapter` bridge shared by the language bindings. Applications should
install the Rust, Python, Dart / Flutter, or TypeScript package instead.

## Source of truth

`src/schema.rs` is the source of truth for generated binding public APIs.

| Change | Schema entry |
| --- | --- |
| Common method | `ADAPTER_OPERATIONS` |
| Provider method or constructor | `PROVIDERS` |
| Enum or open identifier | `IDENTIFIERS` |
| Request or response model | `records` in `binding_schema()` |
| Tagged error | `unions` in `binding_schema()` |

The Rust adapter remains the behavioral implementation. The schema describes
the foreign-language boundary and must match it.

## Verify a schema change

```sh
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python --check
```

Replace `python` with the binding being updated. Run its generator without
`--check` first when the schema intentionally changes generated output.

See the [code generator guide](../codegen/README.md),
[common API reference](../../docs/common-api.md), and
[contribution guide](../../CONTRIBUTING.md).
