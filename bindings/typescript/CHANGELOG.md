# Changelog

## 0.3.0

- Added generated provider APIs, models, native dispatch, and provider stream
  support for the fixed exchange coverage batch.

## 0.2.2

- Reissued 0.2.1 after restoring npm lockfile metadata required by `npm ci`.

## 0.2.1

- Reissued 0.2.0 with the same Travel Rule and KRW transfer APIs after GitHub
  Actions skipped the multi-tag push event.

## 0.2.0

- Added Upbit Travel Rule VASP lookup and verification APIs for Korea and Singapore.
- Added Bithumb KRW transfer history and transfer request APIs.
- Generated the matching Node.js and browser WebAssembly API surfaces.

## 0.1.1

- Generated public APIs, identifiers, contracts, and structural codecs from
  the shared binding schema.
- Rejected malformed or unsafe unsigned wire integers.
- Reused known open-identifier instances during decoding.

## 0.1.0

- Added one TypeScript API for Node.js and browser WebAssembly.
- Added built-in adapters for Upbit, Bithumb, Binance, and Hyperliquid,
  including common and exchange-specific methods.
- Added live market and account streams, custom adapters, native Node.js
  packages, and browser relay configuration.
