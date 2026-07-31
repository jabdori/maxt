# Contributing

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## Setup

`maxt` requires Rust 1.85 or newer and uses Rust edition 2024. The repository
does not pin a toolchain; continuous integration (CI) uses the current stable
toolchain.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test
```

Tests use fixtures and local mock servers rather than exchange endpoints. A
clean Cargo cache may still download Rust dependencies.

## Checks

Run the same checks before opening a pull request:

```sh
export RUSTFLAGS="-D warnings"
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --doc
cargo build --examples
cargo doc --no-deps
cargo clippy --lib -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic
```

CI runs on pushes to the main branch and on pull requests. Live exchange tests
are ignored by default and are not part of CI.

## Layout

- `src/adapter.rs`: the public adapter contract
- `src/client.rs`: common API semantics and normalization
- `src/types/`: shared market, account, order, and stream types
- `src/adapters/<provider>/`: provider REST, private, stream, and parsing code
- `src/transport/`: shared HTTP and WebSocket transport
- `tests/`: contract, integration, and ignored live tests
- `docs/`: common reference and provider-specific constraints

Provider internals may add signing or native protocol modules; follow the
nearest adapter's layout instead of creating empty files for symmetry.

## Adapter checklist

For an in-tree exchange adapter:

1. Add or reuse an `Exchange` variant and export the adapter module.
2. Implement `exchange` and report `supports` accurately for the configured
   adapter, including whether credentials are present.
3. Override only supported `Adapter` methods. Optional methods already return
   `Error::Unsupported`.
4. Use the shared transport and preserve the common ordering, validation,
   `Option`, `Decimal`, and `Timestamp` contracts.
5. Add fixture tests for parsing, requests, signing, capability reporting, and
   provider-specific boundary values.
6. Update capability tests, the English and Korean provider pages, and any
   affected examples.
7. Run all checks above. Run the live test only when network access is intended.

The public trait can also implement mocks, recorded-data adapters, and
backtests outside this crate:

```rust
use maxt::{Adapter, BoxFuture, Exchange, Feature, MarketInfo, MarketKind};

struct EmptyUpbit;

impl Adapter for EmptyUpbit {
    fn exchange(&self) -> Exchange {
        Exchange::Upbit
    }

    fn supports(&self, feature: Feature) -> bool {
        matches!(feature, Feature::Markets)
    }

    fn markets(&self, _kind: MarketKind) -> BoxFuture<'_, maxt::Result<Vec<MarketInfo>>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}
```

`exchange` and `supports` are required. Every operation has a default
`Error::Unsupported` implementation. External adapters must still preserve the
ordering, validation, and normalization documented on `Client`; adding a new
real exchange requires a new `Exchange` variant in `maxt`.

## Live test

The ignored conformance test contacts public exchange endpoints and needs no
credentials:

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

As of 2026-07-31 it covers representative public REST and streaming behavior
for Upbit Korea `BTC/KRW`, Bithumb `BTC/KRW`, Binance Spot `BTC/USDT`, Binance
USD-M `BTC/USDT` perpetual, and Hyperliquid mainnet `BTC/USDC` perpetual. It
does not live-test private account or trading operations. Exchange availability,
rate limits, and market changes can still make the test fail.

## Security

- Never commit API keys, secrets, private keys, signed requests, `.env` files,
  or private exchange payloads.
- Use read-only or testnet credentials while developing private paths whenever
  the provider offers them.
- Keep secrets out of fixtures, logs, issues, and pull requests.
- Revoke and rotate any credential that is exposed.
