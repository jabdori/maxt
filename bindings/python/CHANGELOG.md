# Changelog

## 0.3.1

- Added packaged, runnable task-oriented examples and example navigation.

## 0.3.0

- Added generated provider APIs, models, and native dispatch for the fixed
  Upbit, Bithumb, Binance, and Hyperliquid coverage batch.

## 0.2.2

- Reissued 0.2.1 after fixing generated native conversion code for the release
  compiler.

## 0.2.1

- Reissued 0.2.0 with the same Travel Rule and KRW transfer APIs after GitHub
  Actions skipped the multi-tag push event.

## 0.2.0

- Added Upbit Travel Rule VASP lookup and verification APIs for Korea and Singapore.
- Added Bithumb KRW transfer history and transfer request APIs.
- Generated and checked the matching Python models, adapters, and native dispatch.

## 0.1.1

- Generated public APIs, identifiers, native client methods, adapter dispatch,
  and structural wire conversions from the shared binding schema.
- Preserved omitted optional wire fields and rejected empty native
  subscriptions before dispatch.
- Reused open-enum instances for the same raw value.

## 0.1.0

- Added the common Python API and built-in adapters for Upbit, Bithumb,
  Binance, and Hyperliquid.
- Added public and private REST methods, live market and account streams, and
  Python-implemented custom adapters.
- Added CPython wheels for Linux, macOS, and Windows.
