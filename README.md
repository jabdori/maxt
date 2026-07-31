# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` (Multi-Asset eXchange Toolkit) is one Rust API for market data,
accounts, and orders across four cryptocurrency exchanges: Upbit, Bithumb,
Binance (spot and USD-margined perpetual futures), and Hyperliquid (spot and
perpetual futures).

## Quick start

`maxt` is not published to a package registry.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Reading a price needs no credentials.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let upbit = Client::new(UpbitAdapter::new());
    let btc_krw = Market::spot(Exchange::Upbit, "BTC", "KRW");
    let ticker = upbit.ticker(&btc_krw).await?;
    println!("{btc_krw} last {}", ticker.last_price);

    Ok(())
}
```

`cargo run --example public_rest` runs
[the whole program](examples/public_rest.rs).

## Documentation

- [Getting started](docs/getting-started.md)
- [The common API](docs/common-api.md)
- [Choosing an exchange](docs/providers.md)
- [`examples/`](examples/)
- [Contributing](CONTRIBUTING.md)

## License

MIT. See [LICENSE](LICENSE).
