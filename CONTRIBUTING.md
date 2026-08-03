# Contributing

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## Setup

`maxt` uses Rust edition 2024 and requires Rust 1.85 or newer. CI uses the
current stable Rust toolchain, Python 3.14.2 with uv 0.10.4, and Flutter stable.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test -p maxt -p maxt-bindings-common --all-targets --locked
```

Tests use fixtures and local mock servers. Live exchange tests are ignored by
default.

## CI checks

Run only the commands for the area you change. Each binding has its own CI,
Cargo lockfile, and release tag.

### Rust workspace

```sh
cargo fmt -p maxt -p maxt-bindings-common -p maxt-bindings-codegen --check
cargo clippy -p maxt -p maxt-bindings-common --all-targets --locked -- -D warnings
cargo clippy -p maxt-bindings-codegen --no-default-features --features rust --all-targets --locked -- -D warnings
cargo clippy -p maxt --lib --locked -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic
cargo test -p maxt -p maxt-bindings-common --all-targets --no-default-features --locked
cargo test -p maxt-bindings-codegen --no-default-features --features rust --all-targets --locked
cargo test -p maxt -p maxt-bindings-common --doc --no-default-features --locked
cargo build -p maxt --examples --locked
cargo package -p maxt --locked
```

The documentation test compiles Rust code blocks from the English and Korean
repository Markdown included by `src/lib.rs`.

### Python binding

Run from `bindings/python`:

```sh
uv lock --check
uv sync --frozen --all-groups
cargo run --manifest-path ../codegen/Cargo.toml --no-default-features --features python --locked -- python --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --no-default-features --locked
uv run --frozen maturin develop --locked
MAXT_REQUIRE_NATIVE_TESTS=1 uv run --frozen pytest
uv run --frozen mypy python/maxt
uv build --out-dir dist
uv run --frozen twine check dist/*
```

### Dart and Flutter binding

Run from `bindings/dart`:

```sh
cargo run --manifest-path ../codegen/Cargo.toml --no-default-features --features dart --locked -- dart --check
cargo clippy --manifest-path rust/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path rust/Cargo.toml --all-targets --locked
dart pub get
cargo install flutter_rust_bridge_codegen --version 2.12.0 --locked
flutter_rust_bridge_codegen generate
perl -pi -e 's/[ \t]+$//' lib/src/rust/*.freezed.dart
cargo fmt --manifest-path rust/Cargo.toml
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
| `bindings/common/` | Language-neutral binding contract |
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
6. Update each language binding in a separate change and run that binding's
   generator target. A Rust change does not require every binding in the same pull request.
7. Update capability tests, both language versions of the provider reference,
   and affected examples.
8. Run the CI checks for the changed area.

## Releases

- `rust-vX.Y.Z`: crates.io
- `python-vX.Y.Z`: PyPI
- `dart-vX.Y.Z`: pub.dev
- `typescript-vX.Y.Z`: npm, including Node.js and browser WebAssembly

The first Dart release is published manually before configuring the pub.dev tag
pattern `dart-v{{version}}`. The first npm release requires `NPM_TOKEN`; after
that, configure Trusted Publishing for `release-typescript.yml` and remove the token.

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
