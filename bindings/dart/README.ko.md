# maxt Dart / Flutter

[English](README.md) | [한국어](README.ko.md)

Rust 계약과 같은 작업, 모델, 오류, 스트림을 제공하는 네이티브 Dart API입니다.
Dart build hook이 대상 플랫폼용 Rust crate를 컴파일하며 생성 계약과 네이티브
API의 정합성을 검사합니다.

## 지원 상태

- [x] Android
- [x] iOS
- [x] Linux
- [x] macOS
- [x] Windows
- [ ] Dart Web

Dart 3.10 또는 호환 Flutter SDK, Rustup, 대상 빌드 도구가 필요합니다.

## 설치

```sh
dart pub add maxt
```

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
- `Timestamp`: signed 64-bit Unix epoch nanosecond입니다.
- 오류: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthenticationError`, `ExchangeError`, `TransportError`, `DecodeError`.
- 인증 정보: 공개 접근은 두 필드를 모두 생략하고, 비공개 접근은 두 필드를 모두 제공합니다.

[공통 API](../../docs/common-api.ko.md)와 [거래소 지원](../../docs/providers.ko.md)을 참고하세요.

## 라이선스

MIT
