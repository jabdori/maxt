# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` is a typed asynchronous API for market data, accounts, orders, and
streams across Binance, Upbit, Bithumb, and Hyperliquid. Rust is the core;
Python, Dart/Flutter, and TypeScript expose the same generated contract.

## What is supported

| Exchange | General adapter boundary | Start here |
| --- | --- | --- |
| **Binance** | Spot and USD-M perpetual futures | `BinanceAdapter::spot()`; this README's default example |
| **Upbit** | Spot in Korea, Singapore, Indonesia, and Thailand | `UpbitAdapter::new()` or `with_region(...)` |
| **Bithumb** | Spot; provider-specific KRW account and order APIs | `BithumbAdapter::new()` |
| **Hyperliquid** | Spot and perpetual futures on mainnet or testnet | `HyperliquidAdapter::new()` or `testnet()` |

Every built-in adapter supports a documented subset of the common `Client`
API. Public market data and market streams need no account configuration.
Account reads, orders, and transfers use provider-specific configuration and
may be limited by region, venue, or account permissions. Hyperliquid also has
address-scoped Info reads that need a public address but no local signature;
its signed actions need a signer. See [provider support](docs/providers.md) for
the exact constructors and boundaries; see the generated
[endpoint reference](bindings/common/generated/api.md) for per-operation
coverage and validation state.

Binance testnet constructors, Hyperliquid HIP-3 DEXes, and outcome assets are
not exposed. The endpoint reference distinguishes mapped operations from the
other exchange products that remain planned or unmapped.

## Why maxt

`maxt` is designed for applications that work with multiple exchanges. Switching
exchanges or languages should not mean learning another SDK.

- Use common operations with the same API shape, models, errors, and stream contract across exchanges and supported languages.
- Access common operations through `Client` and exchange-specific capabilities through typed adapters.
- Generate language-native public APIs and contracts from one schema, then verify them against the compiled native API.

## Documentation map

Start with the [Binance first-read tutorial](docs/tutorials/binance-first-read.md).
Then choose the document that matches your goal:

| Need | Read |
| --- | --- |
| Run a checked-in program by language or task | [Example guide](docs/examples.md) |
| Read prices, candles, order books, or streams | [Market-data and streams guide](docs/guides/market-data-and-streams.md) |
| Read an account or prepare a financial request safely | [Account-safety guide](docs/guides/account-safety.md) |
| Ship Dart Web or TypeScript Browser WebAssembly | [Browser relay guide](docs/guides/browser-relay.md) |
| Decide between portable and exchange-specific calls | [Common and provider concept](docs/concepts/common-and-provider.md) |
| Find an exact public method and language name | [Generated binding contract](bindings/common/generated/api.md) |

## Quick start: Binance Spot

The default example is a credential-free Binance Spot `BTC/USDT` read. It
combines a common operation (`ticker`) and a Binance-only operation
(`spot_average_price`) without placing an order.

## Install for Rust

Rust 1.85 or newer is required.

```toml
[dependencies]
maxt = "0.3.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");

    let ticker = client.ticker(&market).await?;
    let average = client.adapter().spot_average_price(&market).await?;

    println!("{}", ticker.last_price);
    println!("{}-minute average: {}", average.minutes, average.price);
    Ok(())
}
```

`ticker()` is common. `spot_average_price()` is Binance Spot-specific and is
available through `Client::adapter()`.

Run the first public Binance read:

```sh
cargo run --example binance_first_read
```

Use `cargo run --example public_rest -- upbit BTC KRW` to explore another
supported public exchange without changing code.

## Language packages

| Language | Package guide | Runnable examples |
| --- | --- | --- |
| Rust | This README and [Getting started](docs/getting-started.md) | [Rust examples](examples/README.md) |
| Python | [Python package guide](bindings/python/README.md) | [Python examples](bindings/python/python/maxt/examples/README.md) |
| Dart / Flutter | [Dart package guide](bindings/dart/README.md) | [Dart examples](bindings/dart/example/README.md) |
| TypeScript | [TypeScript package guide](bindings/typescript/README.md) | [TypeScript examples](bindings/typescript/examples/README.md) |

The Dart package supports Android, iOS, Linux, macOS, Windows, and Web. The
TypeScript package supports Node.js and browser WebAssembly.

## Documentation

- [Getting started](docs/getting-started.md)
- [Binance first-read tutorial](docs/tutorials/binance-first-read.md)
- [Task-oriented examples](docs/examples.md)
- [Market data and streams](docs/guides/market-data-and-streams.md)
- [Account safety](docs/guides/account-safety.md)
- [Browser relay guide](docs/guides/browser-relay.md)
- [Common and provider concepts](docs/concepts/common-and-provider.md)
- [Common API](docs/common-api.md)
- [Provider support](docs/providers.md)
- [Endpoint coverage reference](bindings/common/generated/api.md)
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
