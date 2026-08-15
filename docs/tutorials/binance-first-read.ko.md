# 튜토리얼: 첫 Binance 가격 읽기

[English](binance-first-read.md) | [한국어](binance-first-read.ko.md)

이 튜토리얼은 아직 거래소 계정을 설정하지 않은 개발자를 위한 것입니다. 목표는
Binance 현물 `BTC/USDT`의 공개 가격을 안전하게 한 번 읽는 것입니다. API 키, 주문,
전송, 릴레이(relay)는 사용하지 않습니다.

## 언어 선택

체크인된 각 파일은 같은 첫 조회를 실행 가능한 형태로 제공합니다.

| 언어 | 설치 | 실행 |
| --- | --- | --- |
| Rust | `Cargo.toml`에 `maxt = "0.3.2"`와 Tokio 추가 | `cargo run --example binance_first_read` |
| Python | `python -m pip install maxt` | `python -m maxt.examples.binance_public_ticker` |
| Dart / Flutter | `dart pub add maxt` | `dart run example/main.dart` |
| TypeScript / Node.js | `npm install @jabdori/maxt` | `node examples/binance-public-ticker.mjs` |

Rust, Dart, TypeScript 명령은 저장소 또는 패키지 체크아웃(checkout)에서 실행하세요.
배포되는 패키지에도 소스가 포함되므로 애플리케이션으로 복사해 사용할 수 있습니다.
Python 모듈 명령은 패키지 설치 후 바로 실행할 수 있습니다.

## 프로그램이 하는 일

모든 버전은 Binance 현물 어댑터(adapter)를 만들고 `Client`로 감싼 뒤 `BTC/USDT`
시장 식별자를 만듭니다.

1. `client.ticker(...)`는 공통 API입니다. ticker를 지원하는 모든 어댑터에서 같은
   `Client` 호출을 사용합니다.
2. `client.adapter.spotAveragePrice(...)`는 Binance 전용 API입니다. 결과가 Binance의
   계약에 속하므로 구체 어댑터에 남아 있습니다.
3. 예제는 두 값을 출력한 뒤 종료합니다. 인증 정보나 금융 요청은 만들지 않습니다.

소스는 짧지만 언어별 실제 초기화, 정밀도, 오류 경로를 사용합니다.

## 작업별 다음 단계

- 캔들 조회 또는 페이지네이션 이해: [캔들과 이력](../examples.md#candles-and-history)
- 공개 시장 업데이트 받기: [스트림](../examples.md#streams)
- 주문을 만들지 않고 설정된 계좌 읽기: [계좌와 안전성](../examples.md#account-and-assets)
- 거래소 전용 API 사용: [Binance](../examples.md#binance-provider), [Upbit](../examples.md#upbit-provider), [Bithumb](../examples.md#bithumb-provider), [Hyperliquid](../examples.md#hyperliquid-provider)

거래소 전용 호출을 다른 거래소로 옮기기 전에는 [공통 API와 provider API](../concepts/common-and-provider.ko.md)를 읽어주세요.
