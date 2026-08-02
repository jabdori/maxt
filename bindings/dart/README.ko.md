# maxt Dart / Flutter

[English](README.md)

`maxt`는 Upbit, Bithumb, Binance, Hyperliquid를 같은 네이티브 Dart API로
사용하는 패키지입니다. Dart build hook이 Rust crate를 빌드하므로 미리 빌드된
네이티브 바이너리를 포함하지 않습니다.

## 요구 사항

- Dart 3.10 이상 또는 호환되는 Flutter SDK
- Rustup과 대상 플랫폼의 네이티브 빌드 도구
- Android, iOS, Linux, macOS 또는 Windows
- Dart Web은 지원하지 않습니다.

## 설치

```sh
dart pub add maxt
```

## Binance 공개 API와 전용 API

각 isolate에서 Adapter를 만들기 전에 네이티브 런타임을 한 번 초기화합니다.

```dart
import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();

  final client = Client(BinanceAdapter.spot());
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');

  final ticker = await client.ticker(market);
  final filters = await client.adapter.spotSymbolFilters(market);

  print('$market: ${ticker.lastPrice}');
  print('${filters.symbol} tick size: ${filters.tickSize}');

  await Maxt.dispose();
}
```

`ticker()`는 모든 거래소 Adapter가 공유하는 API입니다.
`spotSymbolFilters()`는 `BinanceAdapter`에서만 제공하며
`client.adapter`를 통해 호출합니다.

인증 정보는 private 계정 및 주문 API를 사용할 때만 필요합니다. 인증된
Adapter를 만들 때는 두 credential 값을 함께 전달하세요.

## 값 계약

| 값 | 계약 |
| --- | --- |
| `Decimal` | `parse()`는 96-bit coefficient와 `0..=28` scale로 정확히 표현 가능한 값만 허용합니다. |
| `Timestamp` | signed 64-bit Unix epoch nanosecond입니다. 작은 단위 getter는 0 방향으로 자릅니다. |
| `Interval` | `month1.seconds`는 `null`이며 `advance()`는 UTC 달력 기준으로 계산합니다. |
| 공통 모델 | `OrderBook`, `Balance`, `Position`, `Page`의 계산 helper는 Rust와 같은 결과를 냅니다. |
| `HyperliquidLedgerKind` | 알려지지 않은 provider 값도 `providerName`으로 보존합니다. |

## 오류

호출 실패는 `InvalidRequestError`, `UnsupportedError`,
`AuthenticationError`, `ExchangeError`, `TransportError`, `DecodeError` 등
구조화된 예외로 반환됩니다.

## 스트림과 종료

스트림 항목은 `StreamEvent` 또는 종료되지 않는 `StreamError`입니다. 네이티브
정리가 끝날 때까지 기다리려면 `await stream.close()`를 호출하거나 구독을
취소하세요. isolate를 종료하기 전에는 `await Maxt.dispose()`를 호출합니다.

## 사용자 정의 Adapter

`AdapterBase`를 확장하고 `exchange`, `features` 및 지원 기능의 메서드를
구현합니다. 구현하지 않은 메서드는 `UnsupportedError`를 반환합니다.
