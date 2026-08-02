# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` is a typed async Rust API for market data, accounts, and orders on
Upbit, Bithumb, Binance, and Hyperliquid.

## Why maxt

`maxt` gives every supported exchange the same operations, request and result
types, structured errors, and stream lifecycle. Provider-only capabilities
remain available on the concrete adapter through `Client::adapter()`.

## Install

`maxt` requires Rust 1.85 or newer.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Example

Public REST and market streams require no credentials.

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    let ticker = client.ticker(&market).await?;

    println!("{market}: {}", ticker.last_price);
    Ok(())
}
```

Run the public REST example:

```sh
cargo run --example public_rest
```

## Documentation

- [Getting started](docs/getting-started.md)
- [Common API reference](docs/common-api.md)
- [Provider matrix](docs/providers.md)
- Rust API reference: `cargo doc --open`
- [Python binding](bindings/python/PYPI.md)
- [Dart / Flutter binding](bindings/dart/README.md)
- [Runnable examples](examples/)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)

## Binding roadmap

The Rust API is the reference contract. `Complete` means the binding exposes
the same exchange adapters and common behavior.

| Binding | Upbit | Bithumb | Binance | Hyperliquid |
| --- | --- | --- | --- | --- |
| Rust | Complete | Complete | Complete | Complete |
| Python | Complete | Complete | Complete | Complete |
| Dart / Flutter | Complete | Complete | Complete | Complete |
| TypeScript / Node.js | Planned | Planned | Planned | Planned |
| TypeScript / WebAssembly | Planned | Planned | Planned | Planned |

## License

MIT. See [LICENSE](LICENSE).
