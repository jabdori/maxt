# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt` (Multi-Asset eXchange Toolkit) is one Rust API for market data,
accounts, and orders across several cryptocurrency exchanges.

Supported exchanges: Upbit, Bithumb, Binance (spot and USD-margined perpetual
futures), Hyperliquid (spot and perpetual futures).

## Quick start

`maxt` is not published to a package registry. Depend on the repository:

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Reading a price needs no credentials:

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

That is the opening of [`examples/public_rest.rs`](examples/public_rest.rs);
run the whole thing with `cargo run --example public_rest`.

## Documentation

- [Getting started](docs/getting-started.md): public reads, a live feed, then
  account reads.
- [The common API](docs/common-api.md): what `Client` offers.
- [Choosing an exchange](docs/providers.md): which adapter for which job.
- [`examples/`](examples/): four runnable programs.
- [Contributing](CONTRIBUTING.md): the checks, and how an exchange is added.

## License

MIT. See [LICENSE](LICENSE).
