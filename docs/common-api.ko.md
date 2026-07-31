# 공통 API 레퍼런스

[English](common-api.md) | [한국어](common-api.ko.md)

`Client<A>`는 `Adapter` 구현을 공통 계약으로 노출합니다. 거래소 전용 메서드는
`Client::adapter()`로 호출합니다. 런타임 제공자 선택에는
`Client<Box<dyn Adapter>>`를 사용합니다.

## 메서드

인증 정보는 `Client::new(adapter)` 전에 어댑터에 설정합니다.

| 영역 | 메서드 |
| --- | --- |
| 클라이언트 | `exchange`, `supports`, `adapter`, `into_adapter` |
| 공개 REST | `markets`, `trades`, `order_book`, `ticker`, `candles`, `funding_rates` |
| 공개 스트림 | `subscribe`, `subscribe_with` |
| 비공개 조회 | `balances`, `open_orders`, `open_orders_on`, `positions`, `positions_on`, `margin_summary`, `funding_payments` |
| 비공개 스트림 | `subscribe_account`, `subscribe_account_with` |
| 비공개 변경 | `place_order`, `cancel_order`, `set_margin` |

공개 REST와 시장 스트림에는 인증 정보가 필요하지 않습니다. 제공자별
`MarketKind` 지원 범위는 [제공자 선택](providers.ko.md)을 참고하세요.

## 공통 타입

| 타입·필드 | 계약 |
| --- | --- |
| `Market` | `{ exchange, kind, base, quote }`; `base`·`quote`는 대문자 |
| `Trade` | REST 결과는 최신순; `taker_side`는 유동성 제거 주문의 방향 |
| `Candle` | `open_time ASC`; `open_time`은 구간 시작 시각 |
| `OrderBook::bids` | `price DESC` |
| `OrderBook::asks` | `price ASC` |
| `Ticker` | 제공자 요약값; 필드 출처·집계 구간은 제공자별 계약 |
| `Decimal` | 가격·수량·비율·금액의 정확한 십진수; `f64` 미사용 |
| `Option<T>` | 제공자 미제공 값은 `None`; 추론·0 보정 없음 |
| `Timestamp` | Unix epoch 기준 UTC 나노초; `Display`는 밀리초 RFC 3339; 전체 값은 `as_nanos()` |

로컬 조회 시각으로 문서화된 `timestamp`는 `maxt`가 응답을 읽은 시각이며 거래소
이벤트 시각이 아닙니다.

## 공개 REST

| 메서드 | 계약 |
| --- | --- |
| `markets(kind)` | 상장된 `MarketInfo`; 유효한 `kind`에 상장이 없으면 `[]` |
| `trades(market, limit)` | 최근 체결, 최신순 |
| `order_book(market, depth)` | 단일 스냅샷; `levels_per_side = depth` |
| `ticker(market)` | 단일 제공자 요약값 |
| `candles(request)` | 과거 캔들, `open_time ASC` |
| `funding_rates(request)` | 공개 무기한 선물 펀딩 비율 `Page<FundingRate>` |

요청 상한, 호가 깊이, 타임스탬프 출처는 제공자별 계약입니다.

## 캔들

### 간격

`supports(Feature::Candles) == true`가 보장하는 간격:

`Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`,
`Week1`, `Month1`

공통 집합 밖의 간격은 제공자별 기능입니다. 매핑이 없으면
`Error::Unsupported`이며 다른 간격으로 반올림하지 않습니다.

### `CandleRequest`

| 요청 | 선택 |
| --- | --- |
| `from + to + limit` | `from <= open_time < to`; 가장 이른 `limit`개; `open_time ASC` |
| `from + to` | `from <= open_time < to`; 전체; `open_time ASC` |
| `from + limit` | `open_time >= from`; 가장 이른 `limit`개; `open_time ASC` |
| `to + limit` | `open_time < to`; 가장 최근 `limit`개; `open_time ASC` |
| `limit` | 가장 최근 `limit`개; `open_time ASC` |
| `from` | `open_time >= from`; 전체; `open_time ASC` |
| `to` | `open_time < to`인 제공자 페이지 1개 |
| 범위 없음 | 최신 제공자 페이지 1개 |

| 검증 | 결과 |
| --- | --- |
| `limit == 0` | `Error::InvalidRequest` |
| `from >= to` | `Error::InvalidRequest` |
| 제공자 호출 | 요청당 최대 100회 |
| 예상 캔들 수 `> 100 * provider_page_cap` | 네트워크 호출 전 거절 |

| `Candle::closed` 출처 | 계약 |
| --- | --- |
| REST | `interval_end <= local_read_time` |
| 스트림 | 제공자별 계약 |

## 스트림

### `Subscription`

| 입력 | 계약 |
| --- | --- |
| 구독 집합 | `markets × feeds` |
| 중복 마켓·피드 | 제거; 최초 삽입 순서 유지 |
| `markets.is_empty() || feeds.is_empty()` | `Error::InvalidRequest` |
| 논리 스트림 | 하나 이상의 WebSocket |

### `StreamConfig`

| 필드 | 기본값 | 계약 |
| --- | ---: | --- |
| `max_reconnect_attempts` | `None` | 재연결 횟수 제한 없음 |
| `initial_reconnect_delay_ms` | `1_000` | 첫 재연결 대기 |
| `max_reconnect_delay_ms` | `30_000` | 백오프 상한 |
| `idle_timeout_ms` | `30_000` | `max(config, provider_minimum)` |
| `buffer_size` | `4_096` | `0 -> 1` |
| `overflow` | `Backpressure` | 버퍼가 가득 차면 생산자 대기 |

| 오버플로 정책 | 버퍼가 가득 찼을 때 |
| --- | --- |
| `Backpressure` | 소켓 읽기 대기; 의도적 이벤트 손실 없음 |
| `DropNewest` | 새 데이터·오류 폐기; 재연결 알림 보존 |

`DropNewest`는 대체 가능한 스냅샷에만 사용합니다. 체결과 캔들 확정 이벤트는
대체할 수 없습니다.

### 상태

| 상태 | 계약 |
| --- | --- |
| `Some(Ok(event))` | 이벤트 |
| `Some(Err(error))` | 비종료 오류 |
| `None` | 스트림 종료 |
| `MarketEvent::Reconnected` | 연결 단절 구간의 시장 이벤트 유실 |
| `AccountEvent::Reconnected` | 연결 단절 구간의 계좌 이벤트 유실 |
| 내장 스트림 `Drop` | 소스 폐기; 내장 연결 작업 전체에 종료 신호 전달 |
| 사용자 스트림 `Drop` | 소스 폐기; 정리는 생산자 책임 |
| `close().await` | 어댑터의 비동기 정리 완료 대기 후 소스 폐기 |

재연결 횟수는 내부 연결별로 적용하며 정상 트래픽 이후에도 초기화하지 않습니다.
`AccountEvent::Reconnected` 이후 `balances()`와 `open_orders()`로 상태를 다시
읽습니다.

## 기능 지원

`Client::supports(feature)`는 네트워크를 호출하지 않습니다.

| 상태 | 계약 |
| --- | --- |
| 작업 매핑·필수 인증 정보 있음 | `supports(feature) == true`; 인자 검증·거래소 권한은 별도 |
| 비공개 작업 매핑·인증 정보 없음 | `supports(feature) == false`; 호출 결과 `Error::Auth` |
| 구조적 미지원 | `supports(feature) == false`; 호출 결과 `Error::Unsupported` |

## 오류와 재시도

| `Error` | 발생 지점 | `is_retryable()` |
| --- | --- | --- |
| `InvalidRequest` | 네트워크 호출 전 로컬 검증 | `false` |
| `Unsupported` | 작업 매핑 없음 | `false` |
| `Adapter` | 어댑터 또는 외부 디스패처 계약 위반 | `false` |
| `Auth` | 로컬에서 인증 요청 생성 불가 | `false` |
| `Exchange` | 거래소 오류 응답 | `kind.is_retryable()` |
| `Transport` | DNS·TLS·소켓·시간 초과 | `true` |
| `Decode` | 응답 스키마 불일치 | `false` |

`maxt`는 REST 호출을 재시도하거나 속도를 제한하지 않습니다. 제공자별 한도와
백오프는 애플리케이션에서 적용합니다. `place_order` 또는 `cancel_order` 후
`Error::Transport`가 발생하면 처리 결과를 알 수 없으므로 계좌·주문 상태를 조회한
뒤 재시도합니다.

## 비공개 계좌와 주문

| 타입·메서드 | 계약 |
| --- | --- |
| `open_orders*` | 시점 스냅샷; 전체 제공자 페이지 순회는 보장하지 않음 |
| `OrderRequest::size` | `Size::Base` 또는 `Size::Quote` |
| 주문 정밀도 | `MarketInfo`에 공통 호가 단위·수량 단위·최소 주문 금액 없음 |
| `cancel_order` | 체결과 경합 가능; 반환된 `Order`는 거래소 응답이며 최종 체결 상태가 없을 수 있음 |
| `positions*` | `position.quantity == 0` 행 제거 |
| `MarginSummary` | 제공자 미제공 값은 `None` |
| `FundingPayment::amount < 0` | 계좌가 펀딩 금액 지급 |

주문 값은 `Decimal`로 구성합니다. 지원 주문 형식과 검증 규칙은 제공자별
계약입니다.

### `HistoryRequest`

| 입력·상태 | 계약 |
| --- | --- |
| `from + to` | `from <= item.timestamp < to` |
| `limit` | 페이지 크기 목표; 동일 타임스탬프 분할 방지를 위해 초과 가능 |
| 다음 요청 | `request.cursor = page.next` |
| 계속 | `page.next.is_some()` |
| 종료 | `page.next == None` |
| `items.is_empty()` | `page.next.is_some()`이면 종료 조건 아님 |

### `MarginRequest`

| 상태 | 계약 |
| --- | --- |
| 로컬 검증 | `leverage.is_some() || margin_mode.is_some()` |
| 제공자 검증 | 한 필드 또는 두 필드 요구 가능 |
| `set_margin()` | 두 변경의 원자성·롤백 보장 없음 |

## 제공자 전용 API

`Client::adapter(&self) -> &A`는 공통화하지 않은 일괄 조회·네이티브 컨텍스트·경보·
원장 메서드를 노출합니다. 목록은 제공자별 레퍼런스를 참고하세요.

## 외부 어댑터

외부 타입은 `exchange()`와 `supports()`를 구현해 `Adapter`로 사용할 수 있습니다.
지원 메서드만 재정의하며 나머지는 `Error::Unsupported`를 반환합니다. 위 공통
계약을 유지합니다. 새 거래소에는 `maxt`의 새 `Exchange` 항목이 필요합니다.

[어댑터 체크리스트](../CONTRIBUTING.ko.md#내장-어댑터-체크리스트)
