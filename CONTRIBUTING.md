# Contributing

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## Setup

`maxt` uses Rust edition 2024 and requires Rust 1.85 or newer. CI uses the
current stable toolchain.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test
```

Tests use fixtures and local mock servers. Live exchange tests are ignored by
default.

## Checks

Run all CI checks before opening a pull request:

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

`cargo test --doc` compiles Rust blocks in the English and Korean Markdown
files through the `markdown` module in `src/lib.rs`.

## Layout

| Path | Role |
| --- | --- |
| `src/adapter.rs` | Public `Adapter` contract |
| `src/client.rs` | Common API semantics and normalization |
| `src/types/` | Shared market, account, order, and stream types |
| `src/adapters/<provider>/` | Provider REST, stream, private, and parsing code |
| `src/transport/` | Shared HTTP and WebSocket transport |
| `tests/` | Contract, integration, and ignored live tests |
| `docs/` | Common and provider references |

Match the nearest adapter. Do not add empty files for structural symmetry.

## Adapter checklist

1. Add or reuse an `Exchange` variant and export the adapter module.
2. Implement `exchange` and make `supports` reflect the configured adapter,
   including credential state.
3. Override supported `Adapter` methods only. Defaults return
   `Error::Unsupported`.
4. Preserve common ordering, validation, `Option`, `Decimal`, and `Timestamp`
   contracts.
5. Add fixture tests for parsing, request construction, signing, capability
   reporting, and provider boundaries.
6. Update capability tests, both language versions of the provider reference,
   and affected examples.
7. Run all checks above.

The public trait also supports out-of-tree mocks, backtests, and recorded-data
adapters. See [External adapters](docs/common-api.md#external-adapters).

## Documentation

- Write for experienced Rust developers.
- Prefer identifiers and operators: `from <= open_time < to`, not a prose
  expansion of the same condition.
- Define common contracts once in `common-api`; keep provider limits and native
  behavior in the provider reference.
- Keep English and Korean documents structurally equivalent. Translate meaning,
  not sentence order.

## Live test

The ignored conformance test uses public exchange endpoints and no credentials:

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

Exchange availability, rate limits, and listing changes can make it fail.

## Security

- Never commit API keys, secrets, private keys, signed requests, `.env` files,
  or private exchange payloads.
- Use read-only or testnet credentials where available.
- Keep secrets out of fixtures, logs, issues, and pull requests.
- Revoke and rotate exposed credentials.
