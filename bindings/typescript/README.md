# maxt for TypeScript

[한국어](README.ko.md)

The TypeScript package is under development and is not published yet. The
package name is reserved as `@jabdori/maxt`; installation becomes available
after the Node and browser contract tests pass for release `0.1.0`.

## Development

```sh
npm ci
npm run build
npm run test:unit
npm run build:node
```

Shared wire DTOs and API inventories are generated from
`maxt-bindings-common`:

```sh
cargo run -p maxt-bindings-codegen --locked
cargo run -p maxt-bindings-codegen --locked -- --check
```

The installation command and Binance Spot `BTC/USDT` common/provider example
will be added here when the public Node and browser APIs are executable. This
README intentionally does not present the unfinished facade as released.
