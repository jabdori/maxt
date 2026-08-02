# maxt for TypeScript

[한국어](README.ko.md)

Node.js 22 or later is required.

## Install

```sh
npm install @jabdori/maxt
```

## Binance example

```ts
import { BinanceAdapter, Client, Exchange, Market } from "@jabdori/maxt";

const adapter = BinanceAdapter.spot();
const client = new Client(adapter);
const market = Market.spot(Exchange.Binance, "BTC", "USDT");

const ticker = await client.ticker(market);
const filters = await adapter.spotSymbolFilters(market);

console.log(ticker.lastPrice.toString());
console.log(filters.tickSize?.toString());
```

`Client.ticker()` is part of the common API. `spotSymbolFilters()` is available
only on `BinanceAdapter`.

## Development

```sh
npm ci
npm test
```

Regenerate and verify the shared contract:

```sh
cargo run -p maxt-bindings-codegen --locked
cargo run -p maxt-bindings-codegen --locked -- --check
```

The Node package is prepared for release `0.1.0` but is not published yet.
