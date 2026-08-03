# maxt binding contracts

[한국어](README.ko.md)

`maxt-bindings-common` contains the language-neutral request, response, error,
and stream contracts used by maxt language bindings. It also provides the
`ForeignAdapter` bridge for adapters implemented outside Rust.

This crate is for binding implementations. Applications should install the
Rust, Python, Dart / Flutter, or TypeScript package instead.

## Verify the Rust contract

Run the contract tests after changing request dispatch, replies, or
stream cancellation:

```sh
cargo test -p maxt-bindings-common --locked
```

See the [common API reference](../../docs/common-api.md) and
[contribution guide](../../CONTRIBUTING.md) for the public contract and the
required checks.

The [generated binding contract](generated/api.md) lists Adapter and
provider-specific method names for every language. Regenerate it with the
[binding code generator](../codegen/README.md).
