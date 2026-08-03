# maxt Dart / Flutter

[English](README.md) | [한국어](README.ko.md)

네이티브 플랫폼과 Web에서 같은 작업, 모델, 오류, 스트림을 제공하는 Dart 및
Flutter API입니다. 네이티브 빌드는 Dart build hook을 사용하고 Web 빌드는
WebAssembly를 사용합니다.

## 지원 상태

- [x] Android
- [x] iOS
- [x] Linux
- [x] macOS
- [x] Windows
- [x] Dart Web

Dart 3.10 또는 호환 Flutter SDK가 필요합니다. 이 패키지는 미리 빌드된 네이티브
라이브러리를 내려받지 않습니다. Dart 또는 Flutter 애플리케이션을 빌드할 때 build
hook이 포함된 Rust 소스를 컴파일하므로 개발 환경과 CI에 Rustup 및 Android NDK,
Xcode와 같은 대상 플랫폼 도구도 설치해야 합니다. Web 빌드에는 `rust-src`가
설치된 Rust nightly toolchain과 `wasm-pack`도 필요합니다.

## 지원 거래소

- Upbit 현물(Spot): 한국, 싱가포르, 인도네시아, 태국
- Bithumb 현물(Spot)
- Binance 현물(Spot), USD-M 무기한 선물
- Hyperliquid 메인넷·테스트넷 현물(Spot), 무기한 선물

Binance 테스트넷(testnet) 생성자는 제공하지 않습니다. Hyperliquid HIP-3
무기한 선물 DEX와 결과형 자산(outcome asset)은 제공하지 않습니다.

## 공통 API

`Client`는 모든 내장 어댑터에서 같은 메서드 이름을 사용합니다.

- 공개 REST: `markets()`, `trades()`, `orderBook()`, `ticker()`,
  `candles()`
- 공개 스트림: 체결, 호가, 현재가 요약(ticker), 캔들(candle)용
  `subscribe()`, `subscribeWith()`; Bithumb 캔들 스트림은 미지원
- 공개 펀딩 이력(funding history): Binance USD-M, Hyperliquid 무기한 선물의
  `fundingRates()`
- 비공개 현물(Spot): 모든 거래소의 `balances()`, `openOrders()`,
  `placeOrder()`, `cancelOrder()`, `subscribeAccount()`
- 비공개 무기한 선물: Binance USD-M, Hyperliquid의 `positions()`,
  `marginSummary()`, `setMargin()`, `fundingPayments()`

공개 호출에는 인증 정보가 필요하지 않습니다. 비공개 호출에는 인증 필드 두 개를
모두 전달해야 합니다. 어댑터나 인증 상태가 동적으로 바뀌면 선택 기능을 호출하기
전에 `client.supports(feature)`를 확인하세요.

## 거래소 전용 API

거래소 전용 메서드는 `client.adapter`에서 호출합니다.

| 어댑터 | 생성 | 추가 메서드 |
| --- | --- | --- |
| `UpbitAdapter` | `UpbitAdapter()` 또는 `UpbitAdapter.withRegion(...)` | `orderBooks()`, `tickers()`, `marketEvents()` |
| `BithumbAdapter` | `BithumbAdapter()` | `marketWarnings()`, `marketAlerts()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spotSymbolFilters()`; 인증 필요: `spotOrder()` |
| `BinanceAdapter` | `BinanceAdapter.usdMFutures()` | 인증 필요: `usdMCreateListenKey()`, `usdMKeepaliveListenKey()`, `usdMCloseListenKey()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` 또는 `HyperliquidAdapter.testnet()` | `assetContext()`, `nonFundingLedger()` |

## 설치

```sh
dart pub add maxt
```

Web 애플리케이션은 실행하거나 빌드하기 전에 WebAssembly 파일을 `web/pkg`에
생성합니다.

```sh
rustup toolchain install nightly --component rust-src --target wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
dart run maxt:build_web --release
flutter build web
```

`dart run maxt:build_web --release`는 애플리케이션 루트에서 실행합니다. 설치된
`maxt` 패키지의 Rust 소스를 사용하므로 pub.dev에서 설치한 경우에도 같습니다.
배포 과정에서 빌드 산출물을 소스에 포함해야 하는 경우가 아니라면 생성된
`web/pkg` 파일은 커밋하지 않습니다.

브라우저가 생성된 WebAssembly 모듈의 공유 메모리 기능을 활성화할 수 있도록 Web
빌드 응답에 `Cross-Origin-Opener-Policy: same-origin`과
`Cross-Origin-Embedder-Policy: require-corp` 헤더를 설정합니다. 운영 환경에서는
HTTPS를 사용하며, 로컬 개발에는 `http://localhost`를 사용할 수 있습니다.

## 초기화와 Binance 사용

각 isolate에서 어댑터를 만들기 전에 `Maxt.initialize()`를 한 번 호출합니다.
isolate 종료 전에는 `Maxt.dispose()`를 호출합니다. 정리한 isolate는 다시
초기화할 수 없습니다.

```dart
import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();

  final client = Client(BinanceAdapter.spot());
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');

  final ticker = await client.ticker(market);
  final filters = await client.adapter.spotSymbolFilters(market);

  print(ticker.lastPrice);
  print(filters.tickSize);

  await Maxt.dispose();
}
```

`ticker()`는 공통 API입니다. `spotSymbolFilters()`는 Binance Spot 전용이며
`client.adapter`를 통해 호출합니다.

브라우저의 공개 호출은 거래소가 허용하면 HTTP와 WebSocket에 직접 연결합니다.
relay가 필요하면 초기화할 때 주소를 지정합니다.

```dart
await Maxt.initialize(relayUrl: 'https://relay.example');
```

JavaScript와 WebAssembly 메모리는 비밀 저장소가 아니므로 브라우저 인증 정보는
기본적으로 차단됩니다. 브라우저에서 인증 호출을 사용하려면 relay와 명시적 허용을
모두 설정해야 합니다.

```dart
await Maxt.initialize(
  relayUrl: 'https://relay.example',
  allowInsecureBrowserCredentials: true,
);
```

출금 권한이 없고 사용 범위가 제한된 거래소 키를 사용하세요. 인증 정보를
브라우저에 노출하면 안 되는 경우에는 신뢰할 수 있는 백엔드에 보관해야 합니다.

relay는 애플리케이션과 같은 site의 인증·속도 제한이 적용된 TLS ingress 뒤에
배포하세요. relay 자체는 사용자를 인증하지 않으며 Origin 허용 목록도 인증 수단이
아닙니다. [relay 배포·보안 요구사항](../../relay/README.ko.md)을 확인하세요.

## 스트림

```dart
final stream = await client.subscribe(
  Subscription(markets: [market], feeds: [Feed.trades]),
);
try {
  await for (final item in stream) {
    switch (item) {
      case StreamEvent(:final event):
        print(event);
      case StreamError(:final error):
        print(error);
    }
  }
} finally {
  await stream.close();
}
```

`StreamError`는 스트림을 종료하지 않습니다. `close()`는 네이티브 정리 완료를
기다립니다.

## 사용자 정의 어댑터

`AdapterBase`를 확장하고 `exchange`, `features`를 구현한 뒤, 알린 기능의
메서드를 재정의합니다. 인스턴스는 `Client(adapter)`로 감쌉니다. 기본 메서드는
`UnsupportedError`를 반환합니다.

사용자 정의 스트림은 Dart `Stream<StreamItem<T>>`을 `MarketStream` 또는
`AccountStream`으로 감싸 반환합니다. 정리가 필요하면 `onClose`를 전달합니다.

## 계약

- `Decimal`: 96-bit 계수(coefficient), scale `0..=28`의 정확한 값입니다.
- `Timestamp`: `BigInt`로 저장하는 signed 64-bit Unix epoch nanosecond입니다.
- 오류: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthenticationError`, `ExchangeError`, `TransportError`, `DecodeError`.
- 인증 정보: 공개 접근은 두 필드를 모두 생략하고, 비공개 접근은 두 필드를 모두 제공합니다.

[공통 데이터·페이지네이션 계약](../../docs/common-api.ko.md)과
[거래소별 한도·데이터 의미](../../docs/providers.ko.md)를 참고하세요.

## 라이선스

MIT
