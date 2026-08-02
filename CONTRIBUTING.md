# Contributing

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## Setup

`maxt` uses Rust edition 2024 and requires Rust 1.85 or newer. CI uses the
current stable Rust toolchain, Python 3.14.2 with uv 0.10.4, and Flutter stable.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test --workspace --all-targets --no-default-features --locked
```

Tests use fixtures and local mock servers. Live exchange tests are ignored by
default.

## CI checks

Run the commands for every area you change. The following commands match
`.github/workflows/ci.yml`.

### Rust workspace

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p maxt --lib --locked -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic
cargo test --workspace --all-targets --no-default-features --locked
cargo test --workspace --doc --no-default-features --locked
cargo build -p maxt --examples --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo package -p maxt --locked
```

The documentation test compiles Rust code blocks from the English and Korean
repository Markdown included by `src/lib.rs`.

### Python binding

Run from `bindings/python`:

```sh
uv lock --check
uv sync --frozen --all-groups
cargo clippy -p maxt-python --all-targets --locked -- -D warnings
cargo test -p maxt-python --all-targets --no-default-features --locked
uv run --frozen maturin develop --locked
MAXT_REQUIRE_NATIVE_TESTS=1 uv run --frozen pytest
uv run --frozen mypy python/maxt
uv build --out-dir dist
uv run --frozen twine check dist/*
```

### Dart and Flutter binding

Run from `bindings/dart`:

```sh
cargo clippy -p maxt_dart_bridge --all-targets --locked -- -D warnings
cargo test -p maxt_dart_bridge --all-targets --locked
dart pub get
cargo install flutter_rust_bridge_codegen --version 2.12.0 --locked
flutter_rust_bridge_codegen generate
perl -pi -e 's/[ \t]+$//' lib/src/rust/*.freezed.dart
cargo fmt --all
git diff --exit-code -- lib/src/rust rust/src/frb_generated.rs
test -z "$(git status --porcelain --untracked-files=all -- lib/src/rust rust/src/frb_generated.rs)"
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-warnings
dart test --chain-stack-traces
flutter test
dart pub publish --dry-run
```

CI also rejects contributor-machine absolute paths in Markdown, Rust, and
TOML files.

## Layout

| Path | Role |
| --- | --- |
| `src/adapter.rs` | Rust `Adapter` contract |
| `src/client.rs` | Common behavior and normalization |
| `src/types/` | Shared market, account, order, and stream values |
| `src/adapters/<provider>/` | Provider REST, stream, private, and parsing code |
| `src/transport/` | Shared HTTP and WebSocket transport |
| `bindings/common/` | Cross-language inventory and mapping checks |
| `bindings/python/` | Python package and PyO3 bridge |
| `bindings/dart/` | Dart package, Rust bridge, and generated bridge code |
| `tests/` | Rust contract, integration, and ignored live tests |
| `docs/` | Common and provider references |

Match the nearest adapter. Do not add empty files for structural symmetry.

## Adapter checklist

1. Add or reuse the Rust `Exchange` variant and export the adapter module.
2. Implement `exchange()` and report only configured capabilities from
   `supports()`, including credential state.
3. Override supported `Adapter` methods. Defaults return `Error::Unsupported`.
4. Preserve common ordering, range, `Option`, `Decimal`, `Timestamp`, error,
   and stream-lifecycle contracts.
5. Add fixture tests for parsing, request construction, signing, capabilities,
   and provider limits.
6. Map the exchange constructor, configuration, public values, methods, and
   provider-specific API in both Python and Dart. Regenerate the Dart bridge.
7. Run the exact mapping check:

   ```sh
   cargo test -p maxt-bindings-common --test language_binding_inventory --locked
   ```

8. Update capability tests, both language versions of the provider reference,
   and affected examples.
9. Run all applicable CI checks above.

The public adapter contracts also support mocks, backtests, and recorded-data
adapters. See [External adapters](docs/common-api.md#external-adapters).

## Documentation

- Write for developers and state contracts with identifiers and operators.
- Define common behavior once in `docs/common-api.md`; keep provider limits and
  native mappings in the provider reference.
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
