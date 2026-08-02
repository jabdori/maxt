# maxt TypeScript

[English](README.md) | [한국어](README.ko.md)

네이티브 Node.js 코드 또는 브라우저 WebAssembly(WASM)를 사용하는 TypeScript
API입니다. 두 backend가 같은 생성 모델, 오류, 어댑터, 스트림 계약을 사용하며
생성 검사가 컴파일된 backend API와 정합성을 유지합니다.

## 지원 상태

- [x] Node.js 22 이상
- [x] Browser WebAssembly

## 설치

```sh
npm install @jabdori/maxt
```

## Node.js

Node.js 진입점을 사용합니다. 같은 옵션의 `initialize()`는 여러 번 호출해도 됩니다.

```ts
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const client = new Client(BinanceAdapter.spot());
const market = Market.spot(Exchange.Binance, "BTC", "USDT");

const ticker = await client.ticker(market);
const filters = await client.adapter.spotSymbolFilters(market);

console.log(ticker.lastPrice.toString());
console.log(filters.tickSize?.toString());
```

`ticker()`는 공통 API입니다. `spotSymbolFilters()`는 Binance Spot 전용이며
`client.adapter`를 통해 호출합니다.

## Browser WebAssembly

브라우저 진입점을 사용하고 어댑터 생성 전에 `initialize()` 완료를 기다립니다.
기본값은 패키지의 WASM 파일이며 `wasmUrl`로 바꿀 수 있습니다.

```ts
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/browser";

await initialize();

const client = new Client(BinanceAdapter.spot());
const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker = await client.ticker(market);
```

`relayUrl`이 없으면 공개 HTTP와 WebSocket은 브라우저에서 거래소로 직접
연결하며 브라우저 CORS와 네트워크 정책의 적용을 받습니다. 공개 작업에는
릴레이가 필요하지 않습니다.

인증 정보가 있는 어댑터는 명시적 허용과 릴레이 origin이 모두 필요합니다.

```ts
await initialize({
  relayUrl: "https://relay.example",
  allowInsecureBrowserCredentials: true,
});

const adapter = BinanceAdapter.spot({ apiKey, secretKey });
```

`relayUrl`은 인증 정보, path, query, fragment가 없는 `http` 또는 `https`
origin이어야 합니다. 설정 후 Browser WASM의 HTTP 요청은 릴레이를 사용합니다.
WebSocket은 거래소가 handshake header를 요구할 때 릴레이를 사용하며, 공개
WebSocket은 직접 연결합니다.

경고: `allowInsecureBrowserCredentials`는 브라우저 인증 정보를 안전하게 만들지
않습니다. 원본 인증 정보는 JavaScript/WASM 메모리에 있으며 XSS, 확장 프로그램,
source map, log, 손상된 runtime에 노출될 수 있습니다. 릴레이 메모리에는 인증
header와 서명된 payload가 들어옵니다. 신뢰하는 TLS 릴레이에 인증과 속도 제한을
적용하고 권한을 최소화한 인증 정보를 사용하세요.

## 스트림

```ts
import { Feed, StreamError, Subscription } from "@jabdori/maxt/node";

const stream = await client.subscribe(new Subscription([market], [Feed.Trades]));
try {
  for await (const item of stream) {
    if (item instanceof StreamError) console.error(item.error);
    else console.log(item.event);
  }
} finally {
  await stream.close();
}
```

`StreamError`는 반복을 종료하지 않습니다. `close()`는 backend 정리 완료를
기다립니다.

## 사용자 정의 어댑터

`Adapter`를 확장하고 `exchange`, `features`를 설정한 뒤, 알린 기능의 메서드를
재정의합니다. 인스턴스는 `new Client(adapter)`로 감쌉니다. 기본 메서드는
`UnsupportedError`로 reject됩니다.

사용자 정의 스트림은 `StreamEvent`, `StreamError`의 `AsyncIterable`을
`MarketStream` 또는 `AccountStream`으로 감싸 반환합니다. 정리가 필요하면 close
callback을 전달합니다. 브라우저 사용자 정의 어댑터도 Node.js와 같은 생성 bridge를
사용하며 먼저 브라우저 초기화를 완료해야 합니다.

[릴레이](../../relay/README.ko.md), [공통 API](../../docs/common-api.ko.md),
[거래소 지원](../../docs/providers.ko.md)을 참고하세요.

## 라이선스

MIT
