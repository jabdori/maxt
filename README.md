# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` is a typed Rust API for market data, accounts, and orders across Upbit,
Bithumb, Binance, and Hyperliquid. Exchange-specific operations remain available
on each adapter.

## Quick start

`maxt` requires Rust 1.85 or newer and is not published to a package registry.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Public market data needs no credentials:

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

Run the complete public REST example with:

```sh
cargo run --example public_rest
```

## Documentation

- [Getting started](docs/getting-started.md): public REST and streaming
- [Common API reference](docs/common-api.md): types, ordering, errors, and private calls
- [Choosing a provider](docs/providers.md): constructors and provider differences
- [Runnable examples](examples/)
- [Contributing](CONTRIBUTING.md)

## Verification scope

On 2026-07-31, the public REST and streaming surface was checked live on one
representative market for Upbit Korea, Bithumb, Binance Spot, Binance USD-M, and
Hyperliquid mainnet. The live check uses no credentials. Private account and
trading paths are tested offline but have not been verified live.

## License

MIT. See [LICENSE](LICENSE).
