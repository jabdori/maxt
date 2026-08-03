# Changelog

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
