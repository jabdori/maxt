# maxt TypeScript

[English](README.md)

Node.js 22 이상이 필요합니다.

## 설치

```sh
npm install @jabdori/maxt
```

## Binance 예제

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

`Client.ticker()`는 공통 API입니다. `spotSymbolFilters()`는
`BinanceAdapter`에서만 사용할 수 있습니다.

## 개발

```sh
npm ci
npm test
```

공통 계약 생성 및 검사:

```sh
cargo run -p maxt-bindings-codegen --locked
cargo run -p maxt-bindings-codegen --locked -- --check
```

Node 패키지는 `0.1.0` 배포 준비 상태이며 아직 npm에 배포하지 않았습니다.
