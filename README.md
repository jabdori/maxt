# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` is a typed async API for market data, accounts, orders, and streams on
Upbit, Bithumb, Binance, and Hyperliquid.

## Why maxt

- Use the same operations, models, errors, and stream contract across exchanges.
- Keep common operations on `Client` and exchange-specific operations on each adapter.
- Generate language contracts from one schema and verify generated code against the compiled native API.

## Install

Rust 1.85 or newer is required.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Binance example

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");

    let ticker = client.ticker(&market).await?;
    let filters = client.adapter().spot_symbol_filters(&market).await?;

    println!("{}", ticker.last_price);
    println!("{:?}", filters.tick_size);
    Ok(())
}
```

`ticker()` is common. `spot_symbol_filters()` is Binance Spot-specific and is
available through `Client::adapter()`.

Run the public REST example:

```sh
cargo run --example public_rest
```

## Support

- [x] Rust
- [x] Python
- [x] Dart / Flutter native
- [x] TypeScript / Node.js
- [x] TypeScript / Browser WebAssembly

## Documentation

- [Getting started](docs/getting-started.md)
- [Common API](docs/common-api.md)
- [Provider support](docs/providers.md)
- Rust API: `cargo doc --open`
- [Python](bindings/python/README.md)
- [Dart / Flutter](bindings/dart/README.md)
- [TypeScript](bindings/typescript/README.md)
- [Browser relay](relay/README.md)
- [Examples](examples/)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT. See [LICENSE](LICENSE).
