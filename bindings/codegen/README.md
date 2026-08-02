# maxt binding code generator

[한국어](README.ko.md)

`maxt-bindings-codegen` generates language contracts from the binding schema in
`maxt-bindings-common`. It is a repository development tool, not an application
dependency.

## Generate contracts

```sh
cargo run -p maxt-bindings-codegen --locked
```

Generated files cover the shared exchange, feature, error, Adapter, Client,
provider, and wire DTO contracts for TypeScript, Python, and Dart. Do not edit
them directly.

## Check generated files

```sh
cargo run -p maxt-bindings-codegen --locked -- --check
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
```

The first command rejects stale generated files. The second command rejects a
schema that no longer matches the Rust Adapter, Client, or error surface.
