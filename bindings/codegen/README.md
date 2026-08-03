# maxt binding code generator

[한국어](README.ko.md)

`maxt-bindings-codegen` generates public APIs, native façades, structural wire
conversions, and contract inventories from `maxt-bindings-common`. It is a
repository tool, not an application dependency.

## Generate one language

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python
```

Targets are `rust`, `python`, `dart`, and `typescript`. The feature and final
argument must name the same target. Omit both only when intentionally updating
every language.

Check without writing:

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python --check
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
```

## Generated files

| Target | Files |
| --- | --- |
| Rust | `bindings/common/generated/api.md` |
| Python | `python/maxt/_generated_contract.py`, `_generated_identifiers.py`, `_generated_api.py`, `_generated_delegate.py`, `_generated_wire.py`, `_native.pyi`; `src/generated/client_methods.rs`, `adapter_dispatch.rs`, `convert.rs`, `provider_convert.rs` |
| Dart | `lib/src/generated_contract.dart`, `generated_identifiers.dart`, `generated_adapter.dart`, `generated_client.dart`, `generated_provider_guard.dart`, `generated_provider_methods.dart`, `generated_delegate.dart`, `generated_wire_converters.dart`; `rust/src/api/generated_native_client.rs`, `adapter/generated_dispatch.rs`, `convert/generated_shape_guard.rs` |
| TypeScript | `src/generated/contract.ts`, `identifiers.ts`, `codec.ts`, `api.ts` |

Paths in the table are relative to that binding directory unless they start
with `bindings/`. Do not edit generated files.

## Change the schema

Edit `bindings/common/src/schema.rs`:

- Add common calls to `ADAPTER_OPERATIONS` and provider-only calls to `PROVIDERS`.
- Add closed or open enums to `IDENTIFIERS`.
- Add request and response shapes to `records`; add tagged errors to `unions`.
- Add constructor metadata when a binding must create the provider adapter.

Then generate and check only the language being updated.

## Generated and handwritten boundaries

The generator owns repeated structure: models, enums, errors, client and
adapter method lists, provider dispatch, option/list/page mapping, and
structural wire conversion.

Runtime policy stays handwritten: Python async/GIL/object lifetime, Dart
isolate and FRB handles, Node callback and Worker lifetime, browser stream
backpressure, cancellation and close behavior, native/WASM loading, credential
and relay security, and precision rules whose meaning differs by language.

For Dart, run the repository generator first, then FRB:

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features dart --locked -- dart
cd bindings/dart
flutter_rust_bridge_codegen generate
```
