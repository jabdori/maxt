# maxt TypeScript

[English](README.md) | [한국어](README.ko.md)

네이티브 Node.js 코드 또는 브라우저 WebAssembly(WASM)를 사용하는 TypeScript
API입니다. 두 backend가 같은 생성 모델, 오류, 어댑터, 스트림 계약을 사용하며
생성 검사가 컴파일된 backend API와 정합성을 유지합니다.

## 지원 상태

- [x] Node.js 22 이상
- [x] Browser WebAssembly

Node.js 패키지는 ESM만 지원합니다. 미리 빌드된 네이티브 모듈은 glibc Linux
(x64, ARM64), macOS(x64, ARM64), Windows(x64)를 지원합니다. Alpine을 포함한
musl Linux는 0.1.0에서 지원하지 않습니다. 브라우저는 Chromium, Firefox,
WebKit에서 검사합니다.

## 설치

```sh
npm install @jabdori/maxt
```

## 지원 거래소

- Upbit 현물(Spot): 한국, 싱가포르, 인도네시아, 태국
- Bithumb 현물(Spot)
- Binance 현물(Spot), USD-M 무기한 선물
- Hyperliquid 메인넷·테스트넷 현물(Spot), 무기한 선물

Binance 테스트넷(testnet) 생성자는 제공하지 않습니다. Hyperliquid HIP-3
무기한 선물 DEX와 결과형 자산(outcome asset)은 제공하지 않습니다.

## 공통 API

`Client`는 Node.js와 브라우저 WebAssembly(WASM)에서 같은 메서드 이름을
사용합니다.

- 공개 REST: `markets()`, `trades()`, `orderBook()`, `ticker()`,
  `candles()`
- 공개 스트림: 체결, 호가, 현재가 요약(ticker), 캔들(candle)용
  `subscribe()`, `subscribeWith()`; Bithumb 캔들 스트림은 미지원
- 공개 펀딩 이력(funding history): Binance USD-M, Hyperliquid 무기한 선물의
  `fundingRates()`
- 비공개 현물(Spot): 모든 거래소의 `balances()`, `openOrders()`,
  `placeOrder()`, `cancelOrder()`, `subscribeAccount()`
- 비공개 주문 조회: Upbit, Bithumb의 `order()`, `orderByClientId()`,
  `ordersByIds()`, `orderHistory()`
- 비공개 주문 가능 정보: Upbit, Bithumb의 `orderRules()`
- 비공개 다건 취소: Upbit, Bithumb의 `cancelOrders()`
- 비공개 입출금 조회·취소: Upbit, Bithumb의 `deposit()`, `withdrawal()`,
  `cancelWithdrawal()`; 조회에는 자산과 거래소 ID 또는 온체인 트랜잭션 ID 하나가 필요하며,
  취소 후에는 반드시 다시 조회해 최종 상태를 확인
- 비공개 무기한 선물: Binance USD-M, Hyperliquid의 `positions()`,
  `marginSummary()`, `setMargin()`, `fundingPayments()`

공개 호출에는 인증 정보가 필요하지 않습니다. 비공개 호출에는 인증 필드 두 개를
모두 전달해야 합니다. 브라우저 비공개 호출에는 릴레이(relay)와
`allowInsecureBrowserCredentials: true`도 필요합니다. 어댑터나 인증 상태가
동적으로 바뀌면 선택 기능을 호출하기 전에 `client.supports(feature)`를
확인하세요.

## 거래소 전용 API

거래소 전용 메서드는 두 백엔드(backend) 모두 `client.adapter`에서 호출합니다.

| 어댑터 | 생성 | 추가 메서드 |
| --- | --- | --- |
| `UpbitAdapter` | `new UpbitAdapter()` 또는 `UpbitAdapter.withRegion(...)` | `orderBooks()`, `orderBooksAtLevel()`, `tickers()`, `tickersByQuote()`, `yearCandles()`, `orderbookInstruments()`, `marketEvents()`; 인증 필요: `testOrder()` |
| `BithumbAdapter` | `new BithumbAdapter()` | `marketWarnings()`, `marketAlerts()`, `notices()`, `transferFees()`; 인증 필요: `apiKeys()`, `pendingOrders()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spotSymbolFilters()`; 인증 필요: `spotOrder()` |
| `BinanceAdapter` | `BinanceAdapter.usdMFutures()` | 인증 필요: `usdMCreateListenKey()`, `usdMKeepaliveListenKey()`, `usdMCloseListenKey()` |
| `HyperliquidAdapter` | `new HyperliquidAdapter()` 또는 `HyperliquidAdapter.testnet()` | `assetContext()`, `nonFundingLedger()` |

`UpbitAdapter.testOrder()`는 주문을 생성하지 않고 검증합니다. 반환 `Order`는
dry-run 결과이므로 `id`를 조회·취소에 사용하면 안 되며 상태도 실제 활성 주문을 뜻하지 않습니다.

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

릴레이는 애플리케이션과 같은 site의 인증·속도 제한이 적용된 TLS ingress 뒤에
배포하세요. 릴레이 자체는 사용자를 인증하지 않으며 Origin 허용 목록은 인증
수단이 아닙니다.

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

[릴레이](../../relay/README.ko.md),
[공통 데이터·페이지네이션 계약](../../docs/common-api.ko.md),
[거래소별 한도·데이터 의미](../../docs/providers.ko.md)를 참고하세요.

## 라이선스

MIT
