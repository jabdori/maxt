# Tutorial: read your first Binance price

[English](binance-first-read.md) | [한국어](binance-first-read.ko.md)

This tutorial is for a developer who has not configured an exchange account.
Its goal is one safe outcome: read the public Binance Spot `BTC/USDT` price.
No API key, order, transfer, or relay is involved.

## Choose a language

Each checked-in file is an executable version of the same first read.

| Language | Install | Run |
| --- | --- | --- |
| Rust | Add `maxt = "0.3.2"` and Tokio to `Cargo.toml` | `cargo run --example binance_first_read` |
| Python | `python -m pip install maxt` | `python -m maxt.examples.binance_public_ticker` |
| Dart / Flutter | `dart pub add maxt` | `dart run example/main.dart` |
| TypeScript / Node.js | `npm install @jabdori/maxt` | `node examples/binance-public-ticker.mjs` |

Run the Rust, Dart, and TypeScript commands from a repository or package
checkout; their published archives include the sources to read or copy into an
application. The Python module command runs after package installation.

## What the program does

Every version constructs a Binance Spot adapter, wraps it in `Client`, and
creates the market identity `BTC/USDT`.

1. `client.ticker(...)` is a common API: the same client call is available on
   every adapter that supports tickers.
2. `client.adapter.spotAveragePrice(...)` is a Binance-specific API: it stays
   on the concrete adapter because its result belongs to Binance.
3. The example prints both values and exits. It never creates credentials or a
   financial request.

The source is deliberately short, but it uses the production initialization,
precision, and error paths for its language.

## Continue by task

- Read candles or understand pagination: [candles and history](../examples.md#candles-and-history)
- Receive public market updates: [streams](../examples.md#streams)
- Read a configured account without placing an order: [account and safety](../examples.md#account-and-assets)
- Use a provider-specific API: [Binance](../examples.md#binance-provider), [Upbit](../examples.md#upbit-provider), [Bithumb](../examples.md#bithumb-provider), or [Hyperliquid](../examples.md#hyperliquid-provider)

See [common versus provider APIs](../concepts/common-and-provider.md) before
porting a provider call to another exchange.
