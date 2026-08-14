# ADR 0002: Upbit 연결 제어는 어댑터 내부에 둔다

[English](0002-upbit-connection-scoped-operations.md) | [한국어](0002-upbit-connection-scoped-operations.ko.md)

- 상태(Status): 승인됨(Accepted)
- 결정일(Date): 2026-08-14

## 배경

Upbit의 `LIST_SUBSCRIPTIONS`는 이미 열려 있는 하나의 WebSocket 연결에서 수행하는
작업(operation)입니다. 두 번째 socket을 열어 같은 구독을 다시 만들면 요청 형식은
검증할 수 있지만, 호출자가 사용 중인 연결이 실제로 받는 데이터를 답할 수는 없습니다.

`MarketStream`은 의도적으로 수신 전용이며 거래소가 달라도 같은 방식으로 쓰는 공통
스트림입니다. 이 한 거래소 작업 때문에 모든 어댑터와 바인딩에 양방향 제어 추상화를
공개하면, 실제 사용처 없는 공통 API만 넓어집니다.

## 결정

1. `UpbitAdapter::list_subscriptions(subscription)`를 공개 거래소 전용 작업으로
   유지합니다.
2. 활성 Upbit 시장 연결마다 변경되지 않는 `Subscription` 선택자(selector)를 등록합니다.
   어댑터는 일치하는 세션의 내부 송신 handle로 `LIST_SUBSCRIPTIONS`를 보내고, 응답은
   시장 이벤트로 해석하기 전에 해당 호출에 전달합니다. 반환된 `MarketStream`은 이 응답을
   전달하는 연결 처리자(dispatcher)이므로, 응답을 기다리는 동안 계속 실행해야 합니다.
3. 일치하는 활성 연결은 정확히 하나여야 합니다. 없거나 둘 이상이면 로컬
   `InvalidRequest` 오류를 반환하며, SDK가 어느 socket을 뜻하는지 추측하지 않습니다.
4. 내부 socket 송신 handle은 crate 내부에만 둡니다. 공통 스트림의 공개 API로 만들지
   않고, 응답 해석도 Upbit 전용으로 유지합니다.
5. 응답을 기다리는 중 세션이 재연결되면 이전 socket에 보낸 요청이므로 호출을 실패로
   끝냅니다.

## 결과

- Rust, Python, Dart, TypeScript의
  `list_subscriptions(subscription)` 시그니처(signature)는 유지됩니다.
- 이 작업은 Upbit 공식 API와 같은 연결 범위를 가집니다.
- 동일한 구독을 가진 활성 연결이 여러 개인 경우에도 임의 선택 대신 명시적인 모호성
  오류가 발생합니다.
- 다른 거래소가 더 풍부한 연결 제어를 실제로 필요로 하면 먼저 거래소 전용 스트림 API를
  검토합니다. 이 작업 하나만으로 범용 변경 가능한 stream 추상화를 만들지 않습니다.

## 기각한 대안

### 임시 socket 열기

임시 socket의 구독만 반환하며 호출자가 사용 중인 연결을 조회하지 못합니다.

### `MarketStream`에 공개 제어 메서드 추가

모든 시장 stream이 거래소 전용 operation frame과 응답 상관관계를 지원한다는 잘못된
계약이 됩니다.

### 가장 최근의 일치 연결 선택

같은 구독 연결이 여러 개일 때 결과가 타이밍에 따라 달라지고, 잘못된 세션을 조회할 수
있습니다.
