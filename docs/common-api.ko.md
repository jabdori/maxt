# 공통 API 레퍼런스

[English](common-api.md) | [한국어](common-api.ko.md)

`Client<A>`는 `Adapter` 구현에 공통 계약을 적용합니다. 거래소 전용 메서드는
`Client::adapter()`가 반환한 `A`에서 호출합니다. 런타임에 거래소를 선택하려면
`Client<Box<dyn Adapter>>`를 사용합니다.

## 접근 설정

공개 REST, 시장 스트림, 지원되는 공개 펀딩 이력 호출은 내장 어댑터를 별도 계정 설정
없이 만들어 사용할 수 있습니다. 아래의 비공개 영역은 계정 범위 또는 변경 작업이며,
필요한 설정은 거래소마다 다릅니다.

- Binance, Upbit, Bithumb은 각 거래소의 인증 정보 쌍을 사용합니다.
- Hyperliquid는 계좌 조회에 공개 조회 주소를, 서명 작업에 로컬 signer를 사용합니다.
  `with_wallet(address, private_key)`는 두 설정을 함께 적용합니다.

`Client::new(adapter)` 전에 어댑터를 설정하세요. `supports(feature)`는 현재 설정한
어댑터가 기능을 제공하는지 알려 주지만, 요청 검증, 시장·지역 선택, 거래소 권한은
여전히 별도로 적용됩니다. 생성자는 [거래소 지원](providers.ko.md), 세부 요구 사항은
각 거래소 레퍼런스를 참고하세요.

## API 영역

| 영역 | 메서드 |
| --- | --- |
| Client | `exchange`, `supports`, `adapter`, `into_adapter` |
| 공개 REST | `markets`, `trades`, `order_book`, `ticker`, `candles`, `funding_rates` |
| 공개 스트림 | `subscribe`, `subscribe_with` |
| 비공개 조회 | `balances`, `order_rules`, `asset_networks`, `deposit_addresses`, `deposit_address`, `deposit`, `withdrawal`, `deposits`, `withdrawals`, `open_orders`, `open_orders_on`, `order`, `order_by_client_id`, `orders_by_ids`, `order_history`, `positions`, `positions_on`, `margin_summary`, `funding_payments` |
| 비공개 스트림 | `subscribe_account`, `subscribe_account_with` |
| 비공개 변경 | `create_deposit_address`, `withdraw`, `cancel_withdrawal`, `place_order`, `cancel_order`, `cancel_order_by_client_id`, `cancel_orders`, `set_margin` |

공개 REST와 시장 스트림에는 인증 정보가 필요하지 않습니다. 거래소별
`MarketKind`와 기능 지원 범위는 [거래소 지원](providers.ko.md)을 참고하세요.

## 데이터 계약

| 타입·필드 | 계약 |
| --- | --- |
| `Market` | `{ exchange, kind, base, quote }`; `base`, `quote`는 대문자 |
| `Trade` | REST 결과는 최신순; `taker_side`는 taker 주문 방향 |
| `Candle` | `open_time ASC`; `open_time`은 간격 시작 시각 |
| `OrderBook::bids` | `price DESC` |
| `OrderBook::asks` | `price ASC` |
| `Ticker` | 거래소 요약값; 필드 출처와 집계 구간은 거래소별 계약 |
| `Decimal` | 96-bit 계수(coefficient), scale `0..=28`; 가격·수량·비율·금액에 `f64` 미사용 |
| `Option<T>` | 거래소가 값을 주지 않으면 `None`; 추론 또는 0 대입 없음 |
| `Timestamp` | Unix epoch 기준 UTC nanosecond; `Display`는 millisecond RFC 3339; 정확한 값은 `as_nanos()` |

`timestamp`가 로컬 조회 시각으로 정의된 경우, `maxt`가 응답을 읽은 시각이며 거래소
이벤트 시각이 아닙니다.

`parse_decimal_exact()`는 일반 표기 또는 과학적 표기 값을 `Decimal`로 정확히
표현할 수 있을 때만 반환합니다. 반올림하거나 잘라내지 않고 오류를 반환합니다.

## 공개 REST

| 메서드 | 계약 |
| --- | --- |
| `markets(kind)` | 상장된 `MarketInfo`; 유효한 `kind`에 상장이 없으면 `[]` |
| `trades(market, limit)` | 최근 체결; 최신순 |
| `order_book(market, depth)` | 단일 스냅샷(snapshot); `depth != None`이면 `bids.len() <= depth`, `asks.len() <= depth` |
| `ticker(market)` | 거래소 요약값 1건 |
| `candles(request)` | 과거 캔들; `open_time ASC` |
| `funding_rates(request)` | 공개 무기한 선물 funding rate `Page<FundingRate>` |

`limit`, 호가 `depth`, `timestamp` 출처는 거래소별 계약입니다.

## 캔들

### 간격

`supports(Feature::Candles) == true`는 다음 간격을 보장합니다.

`Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`,
`Month1`

나머지 간격은 거래소별 기능입니다. 거래소 매핑이 없으면
`Error::Unsupported`를 반환하며 다른 간격으로 반올림하지 않습니다.

### `CandleRequest`

결과 정렬은 항상 `open_time ASC`입니다.

| 설정 필드 | 선택 |
| --- | --- |
| `from`, `to`, `limit` | `from <= open_time < to`; 앞 `limit` |
| `from`, `to` | `from <= open_time < to` |
| `from`, `limit` | `from <= open_time`; 앞 `limit` |
| `to`, `limit` | `open_time < to`; 뒤 `limit` |
| `limit` | 뒤 `limit` |
| `from` | `from <= open_time` |
| `to` | `open_time < to`; 거래소 페이지 1개 |
| 없음 | 최신 거래소 페이지 1개 |

| 검증 | 결과 |
| --- | --- |
| `limit == 0` | `Error::InvalidRequest` |
| `from >= to` | `Error::InvalidRequest` |
| 거래소 호출 수 | 요청당 `<= 100` |
| 예상 캔들 수 `> 100 * provider_page_cap` | 네트워크 I/O 전 거절 |

| `Candle::closed` 출처 | 계약 |
| --- | --- |
| REST | `interval_end <= local_read_time` |
| 스트림 | 거래소별 계약; 거래소 레퍼런스 참고 |

## 스트림

### `Subscription`

| 입력 | 계약 |
| --- | --- |
| 구독 집합 | `markets × feeds` |
| 중복 market 또는 feed | 제거; 최초 삽입 순서 유지 |
| `markets.is_empty() || feeds.is_empty()` | `Error::InvalidRequest` |
| 논리 스트림 | 하나 이상의 WebSocket 연결 |

### `StreamConfig`

| 필드 | 기본값 | 계약 |
| --- | ---: | --- |
| `max_reconnect_attempts` | `None` | 유한한 재연결 횟수 제한 없음 |
| `initial_reconnect_delay_ms` | `1_000` | 첫 재연결 대기 시간 |
| `max_reconnect_delay_ms` | `30_000` | 재연결 대기(backoff) 상한 |
| `idle_timeout_ms` | `30_000` | 적용값: `max(idle_timeout_ms, min_idle_timeout)` |
| `buffer_size` | `4_096` | `0 → 1` |
| `overflow` | `Backpressure` | 버퍼가 가득 차면 생산자(producer) 대기 |

| overflow 정책 | 버퍼가 가득 찼을 때 |
| --- | --- |
| `Backpressure` | 소켓 읽기 일시 중지; 의도적인 이벤트 손실 없음 |
| `DropNewest` | 새 데이터와 오류 폐기; 재연결 알림 보존 |

`DropNewest`는 대체 가능한 스냅샷에만 사용합니다. 체결과 확정 캔들 이벤트는
대체할 수 없습니다.

### 상태

| 상태 | 계약 |
| --- | --- |
| `Some(Ok(event))` | 이벤트 |
| `Some(Err(error))` | 비종료 오류 |
| `None` | 스트림 종료 |
| `MarketEvent::Reconnected` | 연결 단절 중 시장 이벤트 유실 |
| `AccountEvent::Reconnected` | 연결 단절 중 계좌 이벤트 유실 |
| 내장 스트림 `Drop` | 스트림 소스(source) 폐기; 내장 연결 작업 전체에 종료 신호 전달 |
| 사용자 스트림 `Drop` | 스트림 소스 폐기; 정리는 생산자 책임 |
| `close().await` | 어댑터의 비동기 정리 완료 후 스트림 소스 폐기 |

재연결 횟수는 내부 연결별로 적용하며 정상 트래픽 이후에도 초기화하지 않습니다.
`AccountEvent::Reconnected` 이후 `balances()`와 `open_orders()`로 상태를 다시
조회합니다.

## 기능 확인

`Client::supports(feature)`는 네트워크 I/O를 수행하지 않습니다. `false`는 필요한
어댑터 설정이 없거나 작업이 구조적으로 미지원이라는 뜻입니다.

| 상태 | 계약 |
| --- | --- |
| 작업 매핑됨; 필수 인증 정보 있음 | `supports(feature) == true`; 인자와 거래소 권한은 별도 검증 |
| 비공개 작업 매핑됨; 인증 정보 없음 | `supports(feature) == false`; 호출 결과 `Error::Auth` |
| 구조적으로 미지원 | `supports(feature) == false`; 호출 결과 `Error::Unsupported` |

## 오류와 재시도

| `Error` | 발생 지점 | `is_retryable()` |
| --- | --- | --- |
| `InvalidRequest` | 네트워크 I/O 전 로컬 검증 | `false` |
| `Unsupported` | 작업 매핑 없음 | `false` |
| `Adapter` | 어댑터 또는 외부 디스패처(dispatcher) 계약 위반 | `false` |
| `Auth` | 로컬에서 인증 요청 생성 불가 | `false` |
| `Exchange` | 거래소 오류 응답 | `kind.is_retryable()` |
| `Transport` | DNS, TLS, socket, timeout | `true` |
| `Decode` | 응답 schema 불일치 | `false` |

`maxt`는 REST 호출을 재시도하거나 속도를 제한하지 않습니다. 거래소 한도와
재시도 대기는 애플리케이션에서 적용합니다. `place_order` 또는 `cancel_order` 후
`Error::Transport`가 발생하면 결과를 알 수 없습니다. 재시도 전에 계좌 또는 주문
상태를 조회하세요.

## 비공개 계좌와 주문

| 타입·메서드 | 계약 |
| --- | --- |
| `open_orders*` | 특정 시점의 스냅샷; 거래소의 모든 페이지 순회는 보장하지 않음 |
| `order_rules(market)` | Upbit·Bithumb의 현재 수수료, 주문 한도, 지원 주문 조합, 호가 자산(quote)·기초 자산(base)의 잔고와 평균 매수가; Bithumb은 매수·매도 가격 단위도 제공 |
| `order(market, order_id)` | 거래소 주문 ID로 주문 1건 조회 |
| `order_by_client_id(market, client_id)` | 주문 생성 시 지정한 ID로 주문 1건 조회 |
| `orders_by_ids(request)` | 거래소 주문 ID 또는 사용자 지정 ID 중 한 종류를 최대 100개 조회; 찾지 못한 ID는 결과에서 빠질 수 있음 |
| `order_history(request)` | 체결 완료 또는 취소 주문을 최신순 `Page<Order>`로 조회 |
| `cancel_orders(request)` | 여러 주문을 비원자적으로 취소; 성공 목록과 실패 목록을 함께 반환하며 거래소별 최대 건수가 다름 |
| `OrderOption::provider_id` | 거래소 원문 값; 새 값은 maxt가 의미를 추가할 때까지 `order_type == None` |
| `OrderRequest::size` | `Size::Base` 또는 `Size::Quote` |
| 주문 정밀도 | `MarketInfo`에 공통 호가 단위(tick size), 수량 단위(lot size), 최소 명목가치(minimum notional) 없음; Bithumb의 활성 매수·매도 가격 단위는 `OrderRules`에서 제공하며 Upbit의 deprecated `price_unit`은 제외 |
| `cancel_order`, `cancel_order_by_client_id` | 유효한 거래소 응답 후 `()` 반환; 체결과의 경합 결과는 주문 조회로 확인 |
| `positions*` | `position.quantity == 0` 행 제거 |
| `MarginSummary` | 거래소 미제공 값은 `None` |
| `FundingPayment::amount < 0` | 계좌가 funding 지급 |

주문 값은 `Decimal`로 구성합니다. 지원 주문 형식과 검증 규칙은 거래소별
계약입니다.

## 자산 입출금

| 메서드 | 계약 |
| --- | --- |
| `asset_networks(asset)` | 자산 하나의 현재 입금·출금 가능 상태, 거래소 네트워크 ID, 수수료, 한도 |
| `deposit_addresses()` | 거래소가 반환한 계정 전체 입금 주소 항목을 조회합니다. 거래소가 `network`, `provider_network`를 주지 않을 수 있고 주소 발급 대기 중에는 `address`가 없을 수 있으므로, 목록 항목이 항상 전송 가능한 목적지를 뜻하지는 않습니다 |
| `deposit_address(request)` | 자산·네트워크 하나의 기존 입금 주소 조회; `address == None`이면 거래소가 아직 주소를 발급하지 않은 상태 |
| `create_deposit_address(request)` | Upbit·Bithumb에서 입금 주소 발급 요청; Upbit의 비동기 발급 중에는 `address == None`을 반환할 수 있음 |
| `deposit(request)`, `withdrawal(request)` | Upbit·Bithumb에서 자산과 거래소 ID 또는 온체인 트랜잭션 ID 하나로 입출금 한 건 조회; 참조값을 생략해 최신 항목을 조회하는 동작은 하지 않음 |
| `deposits(request)`, `withdrawals(request)` | 최신순 입출금 이력 페이지; 거래소 ID와 원본 상태를 보존 |
| `withdraw(request)` | 자동 재시도 없이 출금 한 건 접수; 성공은 거래소가 요청을 접수했다는 뜻이며 목적지 입금 완료를 뜻하지 않음 |
| `cancel_withdrawal(withdrawal_id)` | Upbit·Bithumb에 출금 취소를 한 번 요청; `()`는 거래소가 취소 요청을 접수했다는 뜻뿐이므로 최종 상태는 `withdrawal(request)`로 다시 조회 |

`create_deposit_address()`는 polling 또는 재시도를 수행하지 않습니다. 전송을 준비하기 전에는
자산과 네트워크를 지정한 `deposit_address()`로 비동기 발급 상태를 확인하세요. Upbit·Bithumb의 이 API는 자산과
네트워크만 받으므로, 해당 어댑터는 `DepositAddressRequest::amount`를 네트워크 I/O 전에
거절합니다. endpoint별 지원 및 검증 상태는 생성된 [coverage 레퍼런스](../bindings/common/generated/api.md)를 참고하세요.

`TransferLookupRequest`에는 자산과 `id` 또는 `tx_id` 중 정확히 하나가 필요합니다.
조회와 취소는 전송 오류 뒤 결과를 알 수 없으므로 자동 재시도하지 않습니다.

### `OrderHistoryRequest`

| 필드·상태 | 계약 |
| --- | --- |
| `market` | 선택 시장 필터 |
| `statuses` | `Filled`, `Cancelled` 또는 둘 다; 빈 목록은 둘 다 조회; 다른 상태는 네트워크 요청 전에 거절 |
| `from` | 생성 시각 하한, 포함 |
| `to` | 생성 시각 상한, 미포함; `from`보다 뒤여야 함 |
| 조회 구간 | 두 경계를 모두 지정하면 최대 7일 |
| `cursor` | 거래소가 반환한 불투명 커서; 같은 어댑터에 변경 없이 전달 |
| `limit` | `1..=1_000`; 기본값 `100` |
| 정렬 | 최신순 |

연속 조회 커서를 제공하지 않는 거래소는 `Page::next == None`을 반환하고 입력
`cursor`를 거절합니다.

### `HistoryRequest`

| 필드·상태 | 계약 |
| --- | --- |
| `from` | `from <= item.timestamp` |
| `to` | `item.timestamp < to` |
| `cursor` | 불투명 재개 지점(opaque resume point); `cursor != None` → `from` 무시; 같은 어댑터에 변경 없이 전달 |
| `limit` | 페이지 크기 목표값; 최대값 아님 |
| `limit == 0` | 네트워크 I/O 전 `Error::InvalidRequest` |
| 같은 `timestamp` 그룹이 `limit` 경계를 넘음 | 그룹을 다음 페이지로 미루면 `items.len() < limit`; 첫 그룹만으로 `limit`를 넘으면 `items.len() > limit` |
| 다음 요청 | `request.cursor = page.next` |
| 계속 | `page.next.is_some()`; `items.is_empty()`여도 계속 |
| 종료 | `page.next == None` |

### `MarginRequest`

| 상태 | 계약 |
| --- | --- |
| 로컬 검증 | `leverage.is_some() || margin_mode.is_some()` |
| 거래소 검증 | 한 필드 또는 두 필드를 요구할 수 있음 |
| `set_margin()` | 두 변경의 원자성(atomicity) 또는 롤백(rollback) 보장 없음 |

## 거래소 전용 API

`Client::adapter()`가 반환한 `&A`에서 거래소 전용 일괄 조회(batch), 원본
context, alert, ledger 메서드를 호출합니다. 목록은 거래소별 레퍼런스를 참고하세요.

기록된 endpoint의 매핑과 구현·검증 상태는 생성된
[endpoint 지원 레퍼런스](../bindings/common/generated/api.md)를 참고하세요.

## 외부 어댑터

외부 타입은 `exchange()`와 `supports()`를 구현해 `Adapter`로 사용할 수 있습니다.
지원 메서드만 재정의하며 나머지는 `Error::Unsupported`를 반환합니다. 위 공통
계약을 유지해야 합니다. 새 거래소는 Rust 어댑터와 `Exchange` 항목을 구현하고
바인딩 스키마에 등록한 다음, 배포할 언어의 공개 API를 생성합니다. 생성 대상인
메서드, 모델, 구조 변환을 언어마다 직접 복사하지 않습니다.

[어댑터 체크리스트](../CONTRIBUTING.ko.md#어댑터-체크리스트)
