# 가이드: 시장 데이터와 스트림

[English](market-data-and-streams.md) | [한국어](market-data-and-streams.ko.md)

애플리케이션에서 공개 가격, 캔들, 호가, 실시간 시장 피드를 사용할 때 이 가이드를
사용하세요.

## 먼저 스냅샷을 읽으세요

이식 가능한 스냅샷 호출에는 `Client`를 사용합니다.

- 시장 목록: `markets`
- 최신 가격: `ticker`
- 최우선 호가 또는 깊이 스냅샷: `orderBook`
- 최근 개별 체결: `trades`
- 오래된 순서의 캔들 구간: `candles`

[공개 시장 데이터 예제](../examples.md#market-data)는 시장 목록, ticker, 호가,
체결을 보여 줍니다. [캔들 예제](../examples.md#candles-and-history)는
`CandleRequest`를 사용합니다. 하한 시간은 포함하고 상한 시간은 제외합니다.

## 스냅샷 다음에 스트림을 여세요

하나 이상의 시장과 feed를 이름으로 지정하는 `Subscription`으로 구독하세요. 스트림
이전에 읽은 스냅샷은 애플리케이션의 초기 상태가 됩니다. 재연결 이벤트는 이벤트
공백이 있을 수 있다는 뜻이므로, 스트림을 다시 동기화된 것으로 취급하기 전에 새
스냅샷을 읽으세요.

실행 가능한 [스트림 예제](../examples.md#streams)는 몇 개 이벤트를 읽고 닫습니다.
운영 애플리케이션은 보통 스트림을 계속 유지하고, 항목별 스트림 오류를 노출하며,
재연결 뒤 로컬 상태를 다시 만드는 정책을 정합니다.

## 맞는 계약을 선택하세요

공통 스트림은 의도적으로 정규화되어 있습니다. 거래소 전용 이벤트에 공통 이벤트가
담지 않는 중요한 필드가 있다면 provider의 `subscribeDetailed` 메서드를 사용하세요.
이는 단순한 업그레이드 토글이 아니라 다른 계약입니다. 지원되는 상세 스트림은
[provider 문서](../providers.ko.md)에 기록되어 있습니다.

## 지원하지 않는 제품을 추정하지 마세요

`Client.supports(feature)`는 현재 설정된 어댑터가 공통 기능을 지원하는지 알려 줍니다.
그렇다고 모든 거래소의 비슷한 이름의 endpoint가 같은 데이터 모델을 제공한다는
뜻은 아닙니다. 현재 공개 표면은 생성된 [API-시나리오 맵](../examples.md#api-to-scenario-map)과
[endpoint reference](../../bindings/common/generated/api.md)로 확인하세요.
