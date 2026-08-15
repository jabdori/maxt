# Changelog

## 0.4.2

- Made the bundled Binance public-market example perform its documented read on a normal run.

## 0.4.1

- Added the pub.dev-recognized `example/main.dart` Binance public-market
  example and standardized its code comments and output in English.

## 0.4.0

- Added generated provider APIs, models, native dispatch, and provider stream
  support for the fixed exchange coverage batch.

## 0.3.3

- Reissued 0.3.2 after formatting the release test source with the pinned Dart
  SDK.

## 0.3.2

- Reissued 0.3.1 after aligning release code generation with the pinned Rust
  formatter.

## 0.3.1

- Reissued 0.3.0 with the same Travel Rule and KRW transfer APIs after GitHub
  Actions skipped the multi-tag push event.

## 0.3.0

- Added Upbit Travel Rule VASP lookup and verification APIs for Korea and Singapore.
- Added Bithumb KRW transfer history and transfer request APIs.
- Generated the matching Dart, Flutter Native, and WebAssembly API surfaces.

## 0.2.2

- Fixed release validation by installing and selecting the bridge's pinned
  Rust formatter.

## 0.2.1

- Generated public APIs, identifiers, native client methods, adapter dispatch,
  and structural wire conversions from the shared binding schema.
- Rejected empty built-in adapter subscriptions before native dispatch.
- Added drift checks for untracked FRB output during release validation.

## 0.2.0

- Added Dart and Flutter Web support through WebAssembly.
- Added `dart run maxt:build_web` for generating application Web assets.
- Added browser relay configuration and explicit credential opt-in.
- **Breaking:** `Timestamp.nanosecondsSinceEpoch` now uses `BigInt` to preserve
  signed 64-bit nanoseconds in JavaScript builds.

## 0.1.0

- Added the common Dart API and built-in adapters for Upbit, Bithumb, Binance,
  and Hyperliquid.
- Added public and private REST methods, live market and account streams, and
  Dart-implemented custom adapters.
- Added Dart Native Assets build hooks for Android, iOS, Linux, macOS, and
  Windows.
