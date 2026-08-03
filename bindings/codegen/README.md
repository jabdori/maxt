# maxt binding code generator

[한국어](README.ko.md)

`maxt-bindings-codegen` generates language contracts from the binding schema in
`maxt-bindings-common`. It is a repository development tool, not an application
dependency.

## Generate contracts

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python
```

Targets are `rust`, `python`, `dart`, and `typescript`. Each target updates only
that port. Omitting the target updates every output. Do not edit generated files
directly.

## Check generated files

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python --check
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
```

The first command checks only the selected port. The second command checks the
Rust source schema.
