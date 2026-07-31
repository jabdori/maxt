# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` is a typed async Rust API for market data, accounts, and orders on
Upbit, Bithumb, Binance, and Hyperliquid. Applications use the same `Client`
methods and types across exchanges. Provider-specific methods remain on each
adapter.

## Why maxt

Using several exchanges in one application usually introduces
provider-specific branches for request shapes, ordering, time ranges, numeric
formats, missing fields, and errors. `maxt` normalizes those contracts behind
the same `Client` methods and types, while provider-specific capabilities
remain on the concrete adapter.

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
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
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
- [API documentation](https://docs.rs/maxt)
- [Runnable examples](examples/)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT. See [LICENSE](LICENSE).
