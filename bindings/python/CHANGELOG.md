# Changelog

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
