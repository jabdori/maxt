# maxt binding contracts

[한국어](README.ko.md)

`maxt-bindings-common` contains the language-neutral request, response, error,
and stream contracts used by maxt language bindings. It also provides the
`ForeignAdapter` bridge for adapters implemented outside Rust.

This crate is for binding implementations. Applications should install the
Rust, Python, Dart / Flutter, or TypeScript package instead.

## Verify a binding

The inventory test compares each language's public adapters, methods, models,
enums, and construction options with the Rust API:

```sh
cargo test -p maxt-bindings-common --test language_binding_inventory --locked
```

Run the bridge contract tests after changing request dispatch, replies, or
stream cancellation:

```sh
cargo test -p maxt-bindings-common --locked
```

See the [common API reference](../../docs/common-api.md) and
[contribution guide](../../CONTRIBUTING.md) for the public contract and the
required cross-language checks.

The [generated binding contract](generated/api.md) lists Adapter and
provider-specific method names for every language. Regenerate it with the
[binding code generator](../codegen/README.md).
