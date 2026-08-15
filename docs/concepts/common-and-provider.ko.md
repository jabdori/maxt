# 개념: 공통 API와 provider API

[English](common-and-provider.md) | [한국어](common-and-provider.ko.md)

`maxt`에는 의도적으로 서로 다른 두 API 계층이 있습니다.

## 공통 API: 이식 가능한 동작

`Client`는 `ticker`, `candles`, `balances`, `openOrders` 같은 작업을 제공합니다.
공통 모델과 공통 스트림·오류 계약을 사용하므로, 기능을 지원하는 어댑터끼리는 작은
변경으로 교체할 수 있습니다.

공통이라는 말이 모든 거래소 endpoint를 하나의 모델에 억지로 넣는다는 뜻은 아닙니다.
정직하게 표현할 수 없는 필드는 추측해서 채우지 않고 남겨 둡니다.

## Provider API: 거래소별 충실도

구체 어댑터(concrete adapter)는 공통 계약 밖에 의미 있는 데이터나 동작이 있을 때
provider 메서드를 노출합니다. Binance mark-price 문맥, Upbit Korea pockets, Bithumb
TWAP, Hyperliquid 주소 단위 Info 응답이 예입니다.

Rust/Python/Dart에서는 `client.adapter`, TypeScript에서는 구체 어댑터 인스턴스를
사용하세요. 이것은 상속이 아니라 합성(composition)입니다. 애플리케이션은 이식 가능한
`Client`를 유지하면서 필요한 부분에서만 거래소 전용 작업을 선택할 수 있습니다.

## 의도적으로 선택하세요

여러 거래소에서 비교 가능한 동작이 필요하면 공통 작업을 선택하세요. 추가 필드,
지역 규칙, 응답 의미가 애플리케이션 판단에 영향을 주면 provider 작업을 선택하세요.
생성된 [API-시나리오 맵](../examples.md#api-to-scenario-map)은 모든 공개 메서드의 실행
작업을 이름으로 보여 주며, 생성된 [바인딩 계약](../../bindings/common/generated/api.md)은
정확한 언어별 이름을 기록합니다.
