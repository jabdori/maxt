# maxt TypeScript

[English](README.md) | [한국어](README.ko.md)

네이티브 Node.js 코드 또는 브라우저 WebAssembly(WASM)를 사용하는 TypeScript
API입니다. 두 backend가 같은 생성 모델, 오류, 어댑터, 스트림 계약을 사용하며
생성 검사가 컴파일된 backend API와 정합성을 유지합니다.

## 설치

```sh
npm install @jabdori/maxt
```

## 첫 읽기: Node.js의 Binance 현물

서버 애플리케이션에는 Node.js 진입점을 사용합니다. 아래 코드는 `BTC/USDT`
공개 데이터만 읽으며 인증 정보나 주문 제출이 필요하지 않습니다.

```ts
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const client = new Client(BinanceAdapter.spot());

const ticker = await client.ticker(market); // 공통 API
const average = await client.adapter.spotAveragePrice(market); // Binance 전용 API

console.log(ticker.lastPrice.toString());
console.log(`${average.minutes}분 평균: ${average.price}`);
```

네이티브 모듈을 빌드한 뒤 `node examples/binance-public-ticker.mjs`로 저장소
예제를 실행할 수 있습니다. 공통 호출은 `Client`, 거래소 전용 호출은
`client.adapter`에 둡니다.

## 지원 상태

- [x] Node.js 22 이상
- [x] Browser WebAssembly

Node.js 패키지는 ESM만 지원합니다. 미리 빌드된 네이티브 모듈은 glibc Linux
(x64, ARM64), macOS(x64, ARM64), Windows(x64)를 지원합니다. Alpine을 포함한
musl Linux는 현재 지원하지 않습니다. 브라우저는 Chromium, Firefox,
WebKit에서 검사합니다.

## 지원 거래소

- Binance 현물(Spot), USD-M 무기한 선물
- Upbit 현물(Spot): 한국, 싱가포르, 인도네시아, 태국
- Bithumb 현물(Spot)
- Hyperliquid 메인넷·테스트넷 현물(Spot), 무기한 선물

Binance 테스트넷(testnet) 생성자는 제공하지 않습니다. Hyperliquid HIP-3
무기한 선물 DEX와 결과형 자산(outcome asset)은 제공하지 않습니다.

## 패키지 지도

| 필요 | 사용할 API |
| --- | --- |
| Node.js 애플리케이션 | `@jabdori/maxt/node` |
| 브라우저 WebAssembly 애플리케이션 | `@jabdori/maxt/browser` |
| 공개 시장 데이터·스트림 | 어댑터를 넣은 `Client` |
| 거래소 전용 필드·endpoint | `client.adapter` |
| 정확한 가격·수량 | JavaScript `number`가 아닌 `Decimal` |
| 64-bit ID·시각 | `bigint` 기반 모델 |
| endpoint 지원·제약 | [생성 endpoint reference](../common/generated/api.md) |

브라우저의 비공개 호출에는 신뢰하는 relay와 명시적 인증 정보 허용이 추가로
필요합니다. 공개 데이터도 브라우저 CORS와 네트워크 정책이 허용할 때만 직접
호출할 수 있습니다.

## 인증 경계

공개 호출에는 인증 정보가 필요하지 않습니다. 서명하는 계정·주문·지갑 호출에는
인증 필드 두 개가 모두 필요합니다. 아래에 나열한 Hyperliquid 주소별 서명 없는
`/info` 조회는 공개 `address`만 필요하고 개인 키는 필요하지 않습니다. 인증 정보를
사용하는 브라우저 호출에는 릴레이(relay)와
`allowInsecureBrowserCredentials: true`도 필요합니다. 어댑터나 인증 상태가
동적으로 바뀌면 선택 기능을 호출하기 전에 `client.supports(feature)`를
확인하세요.

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

## 거래소 전용 API

거래소 전용 메서드는 두 백엔드(backend) 모두 `client.adapter`에서 호출합니다.

| 어댑터 | 생성 | 추가 메서드 |
| --- | --- | --- |
| `BinanceAdapter` | `BinanceAdapter.spot()` | 공개: `aggregateTrades()`, `spotAveragePrice()`, `spotSymbolFilters()`, `spotExchangeInfo()`; 인증 필요: `spotOrder()`, `spotAccountInformation()`, `spotCancelAllOpenOrders()`, `accountTrades()`, `c2cTradeHistory()`, `testOrder()`, `cancelAllOpenOrders()`; Wallet: `allCoinsInformation()`, `apiKeyPermissions()`, `depositHistory()`, `questionnaireRequirements()`, `withdrawAddressList()`, `withdrawHistory()` |
| `BinanceAdapter` | `BinanceAdapter.usdMFutures()` | 공개: `markPrice()`, `markPrices()`, `openInterest()`, `aggregateTrades()`, `usdMExchangeInfo()`; 인증 필요: `usdMAccountInformation()`, `usdMPositionInformation()`, `accountTrades()`, `testOrder()`, `cancelAllOpenOrders()`, `usdMCreateListenKey()`, `usdMKeepaliveListenKey()`, `usdMCloseListenKey()` |
| `UpbitAdapter` | `new UpbitAdapter()` 또는 `UpbitAdapter.withRegion(...)` | `orderBooks()`, `orderBooksAtLevel()`, `tickers()`, `tickersByQuote()`, `yearCandles()`, `orderbookInstruments()`, `marketEvents()`; 인증 필요: `testOrder()`, `orderDetail()`, `closedOrders()`, `depositInfo()`, `withdrawalAddresses()`, `travelRuleVasps()`, `verifyTravelRuleByUuid()`, `verifyTravelRuleByTxid()`, `batchCancelOpenOrders()`, `cancelAndNewOrder()`; 한국 전용: `depositKrw()`, `withdrawKrw()`, `apiKeys()`, `listPockets()`, `listPocketApiKeys()`, `subPocketBalances()`, `universalTransfer()`, `universalTransfers()`, `subPocketTransfer()`, `subPocketTransfers()` |
| `BithumbAdapter` | `new BithumbAdapter()` | `marketWarnings()`, `marketAlerts()`, `notices()`, `transferFees()`; 인증 필요: `apiKeys()`, `withdrawalAddresses()`, `orderDetail()`, `orderList()`, `closedOrders()`, `krwWithdrawals()`, `withdrawKrw()`, `krwDeposits()`, `depositKrw()`, `pendingOrders()`, `batchOrders()`, `twapOrders()`, `createTwapOrder()`, `cancelTwapOrder()` |
| `HyperliquidAdapter` | `new HyperliquidAdapter()` 또는 `HyperliquidAdapter.testnet()` | 공개: `allMids()`, `assetContext()`, `candleSnapshot()`, `l2Book()`, `recentTrades()`, `fundingHistory()`, `spotMeta()`, `spotMetaAndAssetContexts()`; 주소별 서명 없는 읽기: `userFunding()`, `spotClearinghouseState()`, `basicOpenOrders()`, `orderStatus(reference)`, `historicalOrders()`, `userFills()`, `userFillsByTime()`, `nonFundingLedger()`, `userRateLimit()`, `userRole()`, `referral()`, `userFees()`, `portfolio()`, `subAccounts()`, `userVaultEquities()` |

`UpbitAdapter.testOrder()`는 주문을 생성하지 않고 검증합니다. 반환 `Order`는
dry-run 결과이므로 `id`를 조회·취소에 사용하면 안 되며 상태도 실제 활성 주문을 뜻하지 않습니다.

`UpbitAdapter.orderDetail(request)`는 거래소 전용 인증 필요 `GET /v1/order`
조회입니다. 예상 시장과 UUID 및/또는 identifier를 지정하며 식별자 하나는 필수이고
둘 다 있으면 UUID가 우선합니다. 공통 `Order`에 없는 상세 체결·수수료·잠금 수량·SMP·유효
조건 원본 필드를 보존하며 예약 문자가 든 identifier도 안전하게 인코딩합니다. fixture만 검증했습니다.

공통 `orderHistory()`는 정규화한 주문 이력 API로 그대로 유지합니다.
`UpbitAdapter.closedOrders(request)`는 이를 보완하는 종료 주문 요약 API로 수수료·SMP·
`identifier`·유효 조건 같은 공식 필드를 보존하지만 `trades` 목록은 반환하지 않습니다.
선택 `market`, `state`, `states`를 받으며 `state`와 `states`는 함께 쓸 수 없고, 주문 생성 시각
조회 구간은 최대 7일, `limit`은 최대 1,000이며 오름차순 또는 내림차순을 고릅니다.
입력 `Timestamp`는 공통 이력의 종료 시각 배타(exclusive end) 적응과 달리 Upbit에
밀리초로 직접 전달합니다. 공식 endpoint 문서가 시간 경계의 포함·배제를 명시하지 않아
이 API도 경계 의미를 추가로 단정하지 않습니다. fixture로만 검증했으며 maxt는 실제 거래
또는 읽기 요청을 수행하지 않았습니다. [한국](https://docs.upbit.com/kr/reference/list-closed-orders)·
[Global](https://global-docs.upbit.com/reference/list-closed-orders) 문서를 확인하세요.

`UpbitAdapter.depositInfo(asset, network)`는 거래소가 제공하는 입금 가능 여부, 최소
수량, 확인 수, 소수 자릿수 메타데이터를 반환합니다. Upbit 응답은 몇 분 지연될 수 있어
실시간 서비스 상태로 사용하면 안 됩니다.

`UpbitAdapter.travelRuleVasps()`는 Travel Rule 확인에 사용할 수 있는 VASP 목록을
조회합니다. 검증 메서드는 금전성 쓰기이며 한국과 싱가포르에서만 사용할 수 있습니다.
인도네시아와 태국에서는 네트워크 요청 전에 실패합니다. 이 경로는 fixture로만 검증했습니다.

`UpbitAdapter.batchCancelOpenOrders(request)`는 금전성 쓰기 요청입니다.
`UpbitBatchCancelScope.all()`은 모든 대상 마켓 범위를 명시적으로 선택하며, Upbit는
요청 수량을 적용해 기본 20개·최대 300개의 일치하는 `wait` 주문만 취소합니다. 일부
실패도 결과에 보존합니다.

`UpbitAdapter.cancelAndNewOrder(request)`는 JSON endpoint를 사용하는 금전성
쓰기입니다. 새 주문은 기존 주문의 시장과 매수/매도 방향을 유지하며
`postOnly`와 SMP를 함께 사용할 수 없습니다. HTTP 요청이 성공해도 취소 완료 전에
기존 주문이 체결되면 새 주문이 없을 수 있습니다. 이 경로는 fixture 검증만 했습니다.

`UpbitAdapter.depositKrw(request)`, `withdrawKrw(request)`는 한국 전용 금전성
쓰기입니다. `UpbitKrwTransferRequest`에는 양수 금액과 `UpbitKrwTwoFactorType.Kakao`,
`.Naver`, `.Hana` 중 하나가 필요하며, 등록 계좌와 2차 인증은 Upbit에서
처리합니다. `apiKeys()`는 access key 식별자와 만료 시각을 읽는 한국 전용 인증 API입니다.
세 경로 모두 fixture만 검증했고 maxt는 실제 이체를 실행하지 않습니다.

`listPockets()`, `listPocketApiKeys(request)`, `subPocketBalances(pocketUuid)`는
각각 포켓, 포켓 API 키, 하위 포켓 잔고를 읽는 한국 전용 인증 조회입니다.
`universalTransfer(request)`와 `subPocketTransfer(request)`는 한국 전용 금전성
쓰기이고, 현행 Upbit OpenAPI 계약에 따라 두 요청 모델 모두 목적지 `to`가 필수입니다.
`universalTransfers(request)`와 `subPocketTransfers(request)`는 해당 이체 이력을
조회합니다. 이 경로들은 fixture만 검증했습니다.

`BithumbAdapter.batchOrders(request)`는 1~20건을 받고 항목별 실패가 있어도 HTTP
200을 반환할 수 있으므로 `BithumbBatchOrderOutcome`를 모두 확인해야 합니다. 성공
항목은 `timeInForce`와 `stpType`을, 실패 항목은 반환된 `timeInForce`를 보존합니다.
이 메서드는 fixture로만 검증한 금전성 쓰기입니다.

`BithumbAdapter.twapOrders(request)`는 Bithumb KRW 마켓의 인증된 읽기 전용
주문 이력 조회입니다. `createTwapOrder()`와 `cancelTwapOrder()`는 금전성
쓰기이므로 읽기 전용 검증에서 호출하지 마세요.

`BithumbAdapter.krwWithdrawals()`와 `krwDeposits()`는 원화 입출금 이력을
조회합니다. `withdrawKrw()`와 `depositKrw()`는 금전성 쓰기입니다. Bithumb의
등록 계좌와 카카오 2차 인증 절차가 필요하며, maxt는 계좌나 인증 수단을 받거나 저장하지
않습니다. 이 경로는 fixture로만 검증했습니다.

`BithumbAdapter.withdrawalAddresses()`는 등록된 출금 허용 주소를 읽는 인증 필요
읽기 전용 API입니다. `prepareWithdrawal()`와 달리 예정 출금의 가능 여부를 검증하거나
공통 출금 견적을 반환하지 않습니다. fixture만 검증했습니다.

`BithumbAdapter.orderDetail(request)`는 Bithumb 전용 체결, 수수료, 취소,
자전거래 방지(self-trade prevention), 시간 유효 조건(time in force) 필드를 보존합니다.
정규화된 공통 `Order`에는 이 필드를 의도적으로 담지 않습니다. 요청의 예상 마켓은
응답과 대조합니다. 이 경로는 fixture만 검증했습니다.

`BithumbAdapter.orderList(request)`는 공통 `openOrders()`와 별개인 거래소 전용
인증 필요 `GET /v1/orders` 조회입니다. 시장은 선택이고 `state` 또는 `states` 중 하나만,
UUID·client ID 목록은 각각 최대 100개(둘 다 있으면 UUID 우선)이고, `page >= 1`,
`limit`은 1~100, `orderBy`를 지정합니다. 공통 `Order`로 축소하지 않고 거래소 필드를 보존합니다. fixture만 검증했습니다.

공통 `orderHistory()`는 정규화한 주문 이력 API로 그대로 유지합니다.
`BithumbAdapter.closedOrders(request)`는 이를 보완하는 Bithumb 공식 v2 종료 주문
목록 API로 수수료·취소·자전거래 방지·시간 유효 조건 메타데이터를 보존합니다. 선택
`market`, 상호배타인 `state` 또는 `states` (`states[]` 쿼리 파라미터), 최대 7일 간격의 시작·종료 시각, 1~1,000의
`limit`, `orderBy`, 불투명 `next_key` 커서를 지원합니다. 거래소 시각은 밀리초로 직접
전달하므로 공통 이력의 종료 시각 배타(exclusive end) 적응과 다르며, 시간 경계 포함성은
단정하지 않습니다. 페이지의 `data`·`has_next`·`next_key`, 원본 상태·유형 문자열,
선택 가격·생성 시각·client 주문·취소 필드를 보존합니다. fixture만 검증했고 실제 계좌
조회나 주문은 실행하지 않았습니다. [종료 주문 목록](https://apidocs.bithumb.com/reference/%EC%A2%85%EB%A3%8C-%EC%A3%BC%EB%AC%B8-%EB%AA%A9%EB%A1%9D-%EC%A1%B0%ED%9A%8C.md)과
[인증 토큰 생성](https://apidocs.bithumb.com/docs/%EC%9D%B8%EC%A6%9D-%ED%86%A0%ED%81%B0-%EC%83%9D%EC%84%B1%ED%95%98%EA%B8%B0)을 확인하세요.

```ts
const adapter = new BithumbAdapter({ accessKey, secretKey });
const market = Market.spot(Exchange.Bithumb, "BTC", "KRW");
const page = await adapter.twapOrders(
  new BithumbTwapOrdersRequest(market, [], null, null, 20, null),
);
```

Bithumb TWAP API는 `progress`, `done`, `cancel` 상태와 1~100개 페이지 크기를
지원합니다. 생성 시 주문 시간은 300~43,200초, 간격은 15/20/30/60/120초이며,
매수에는 `price`, 매도에는 `volume`이 필요합니다.

`BinanceAdapter.usdMFutures()`는 USD-M 무기한 선물의 공개 읽기 전용
시세 데이터 메서드 `markPrice()`, `markPrices()`, `openInterest()`를
제공합니다. 이 메서드들은 fixture로 검증했으며 실제 읽기 요청(live read)은
아직 검증하지 않았습니다. `aggregateTrades(request)`는 같은 거래소 전용 집계 체결
타입을 반환하는 공개 Spot·USD-M 읽기입니다. 두 거래소 모두 `fromId`부터 조회하거나
`startTime`~`endTime` 포함 범위를 조회하며, 두 방식은 함께 사용할 수 없습니다.
`limit`은 1~1,000(`null → 500`)입니다. USD-M은 최근 48시간만 보관하고 시간 범위가
1시간 미만이어야 하지만 Spot에는 같은 로컬 제한이 없습니다. 이 메서드도 fixture만
검증했습니다.
`accountTrades(request)`는 Spot·USD-M의 서명된 계정 체결 페이지이며 `limit`은
1~1,000(기본 500)이고 안전한 공통 재개 커서가 없습니다.
`c2cTradeHistory(request)`는 서명된 읽기 전용 Spot/Funding Wallet SAPI 호출이며
`usdMFutures()`에서는 사용할 수 없습니다. `BinanceC2cTradeType.Buy` 또는 `.Sell`이
필수이고, 페이지 번호는 1부터 시작하며 행 수는 최대 100개입니다. 시작·종료 시각을
함께 지정하면 포함 범위는 최대 30일입니다. nullable `code`, `message`, `data`,
`total`, `success` envelope은 공통 커서로 변환하지 않고 보존합니다. 이 경로는 fixture만
검증했습니다.
`testOrder(new BinanceTestOrderRequest(...))`는 매칭 엔진으로 보내지 않는 서명된
검증이며 `computeCommissionRates`는 Spot 전용입니다.
`cancelAllOpenOrders(market)`는 시장 1개의 활성 주문을 취소하는 서명된 금전성
쓰기입니다. 세 경로 모두 fixture만 검증했습니다.
`HyperliquidAdapter.allMids()`는 공개 읽기 전용이며,
기본 무기한 선물 DEX와 첫 번째 DEX의 Spot mid 가격을 반환합니다. 호가가 비어
있으면 Hyperliquid가 마지막 체결 가격을 대체값으로 사용합니다. 이 메서드도
fixture로 검증했으며 실제 읽기 요청은 아직 검증하지 않았습니다.

`userRateLimit()`, `userRole()`, `referral()`, `userFees()`, `portfolio()`,
`subAccounts()`, `userVaultEquities()`는 설정한 Hyperliquid 주소의 공개 `/info`
조회입니다. `address` 설정이 필요하고 `privateKey`는 선택 사항이며 서명은 하지
않습니다. 이 경로들은 fixture만 검증했습니다.

`userFills(aggregateByTime)`, `userFillsByTime(from, to, aggregateByTime)`는
설정한 공개 주소의 서명 없는 `POST /info` 조회이며 개인 키나 서명을 사용하지 않습니다.
후자는 `from`이 필수이고 `to`는 선택이며 밀리초 경계 양쪽을 포함합니다. 두 메서드는 체결·포지션·수수료·주문·방향·원본 거래소 필드를 보존하며 fixture만 검증했습니다.

`basicOpenOrders()`, `orderStatus(reference)`, `historicalOrders()`도 주소에
묶인 서명 없는 `POST /info` 조회입니다. 첫 메서드는 Hyperliquid의 간결한
`openOrders` 응답을 사용하며, `frontendOpenOrders`를 사용하는 공통
`openOrders()`와 구분됩니다. `reference`에는 숫자 `oid` 또는 `0x` 접두사가 있는
32자리 16진수 클라이언트 주문 ID를 지정합니다. `unknownOid`는 오류가 아닌 일반
`{ kind: "unknown_order" }` 결과이며 이후 추가되는 최상위 상태는 상태와 원본 JSON을
보존합니다. 이력과 조회된 상세 주문에는 trigger, 시간 유효 조건(time in force),
reduce-only, client ID, 상태, 원본 JSON 필드를 보존하고, `historicalOrders()`는 최근
주문 최대 2,000건을 반환합니다. 세 메서드 모두 유효한 `address` 설정이 필요하며
주소가 없거나 유효하지 않으면 네트워크 I/O 전에 실패합니다. API 키(API key), 개인 키
(private key), 서명은 사용하지 않습니다. fixture만 검증했습니다.

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

## 문서와 예제

- [실행 가능한 Binance 공개 시세 예제](examples/binance-public-ticker.mjs)
- [저장소 시작하기](../../docs/getting-started.ko.md)
- [거래소 reference](../../docs/providers.ko.md)
- [생성 endpoint 지원 reference](../common/generated/api.md)

## 라이선스

MIT
