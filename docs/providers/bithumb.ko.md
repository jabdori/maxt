# Bithumb

[English](bithumb.md) | [한국어](bithumb.ko.md)

## 거래소와 생성자

현물 전용입니다.

| 생성자 | 기능 |
| --- | --- |
| `BithumbAdapter::new()` | 공개 REST와 스트림 |
| `.with_credentials(access_key, secret_key)` | 계좌, 주문, 비공개 스트림 메서드 |

| 필드 | 값 |
| --- | --- |
| `Market` | `Market::spot(Exchange::Bithumb, "BTC", "KRW")` |
| `MarketInfo::native_symbol` | `KRW-BTC` |

## REST

| 호출 | 엔드포인트(endpoint) | 계약 |
| --- | --- | --- |
| `markets(MarketKind::Spot)` | `/v1/market/all?isDetails=true` | 상장된 Spot 시장 |
| `markets(MarketKind::Perpetual)` | — | `Ok(vec![])` |
| `trades(market, limit)` | `/v1/trades/ticks` | `limit: 1..=500`; `None → 1`; 최신순 |
| `order_book(market, depth)` | `/v1/orderbook` | `depth: 1..=30`; 각 측 `len() <= depth`; `None → 30`; `quantity == 0` 제거 → 정렬 → 로컬 절단 |
| `ticker(market)` | `/v1/ticker` | 시장 스냅샷(snapshot) 1건 |

`HTTP 2xx + {"error": ...} → Error::Exchange`. 숫자인 `error.name`도 문자열
code로 보존합니다.

## 캔들

| 계약 | 값 |
| --- | --- |
| 지원하는 `Interval` | `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` |
| 거래소 페이지 상한 | 200 |
| 요청당 거래소 호출 수 | `<= 100` |
| 사전 계산 캔들 수 | `<= 20_000` |
| 거래소 요청값 | `to → format_kst(ceil_second(to))`; 결과 조건 `open_time < to` |

| 간격 | UTC `open_time` grid |
| --- | --- |
| `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1` | UTC 단위 경계 |
| `Hour4` | `03:00`, `07:00`, `11:00`, `15:00`, `19:00`, `23:00` |
| `Day1` | `15:00` |
| `Week1` | 일요일 `15:00` |
| `Month1` | 이전 달의 마지막 UTC 날짜 `15:00` |

## 스트림

| `Feed` | 계약 |
| --- | --- |
| `Feed::Trades` | 공개 체결 이벤트 |
| `Feed::OrderBook` | 전체 스냅샷; `quantity == 0` 제거; 각 측 최대 15개 호가 단계; 거래소 `timestamp` 단위 µs |
| `Feed::Ticker` | 스냅샷과 실시간 갱신 |
| `Feed::Candles(_)` | 연결 전 `Error::Unsupported` |

## 비공개 API와 거래소 전용 API

인증 정보 설정 후 잔고, 주문 단건·이력 조회, 주문 생성·취소, 계좌 스트림을 사용할
수 있습니다.

| 공통 호출 | 엔드포인트(endpoint) | 계약 |
| --- | --- | --- |
| `open_orders*` | `GET /v1/orders` | 한 페이지, 최대 100건 |
| `order(market, order_id)` | `GET /v1/order?uuid=...` | 응답 시장이 요청 시장과 같은지 검증 |
| `order_by_client_id(market, client_id)` | `GET /v1/order?client_order_id=...` | 응답 시장이 요청 시장과 같은지 검증 |
| `orders_by_ids(request)` | `POST /v2/orders/search` | 주문 ID 또는 사용자 지정 ID 중 한 종류를 1~100개 조회; 찾지 못한 ID는 제외하고 중복 ID는 한 건으로 처리 |
| `order_history(request)` | `GET /v2/orders/history` | `limit: 1..=1_000`; 최대 7일; 최신순; `next_key`를 불투명 `Page::next` 커서로 반환 |
| `cancel_orders(request)` | `POST /v2/orders/cancel` | 주문 ID 또는 사용자 지정 ID 중 한 종류를 1~30개 취소; 항목별 실패 코드와 메시지를 보존 |

| 주문 | 필수 `Size` |
| --- | --- |
| 지정가 매수 또는 매도 | `Size::Base` |
| 시장가 매수 | `Size::Quote` |
| 시장가 매도 | `Size::Base` |
| 최유리 매수 | `Size::Quote` |
| 최유리 매도 | `Size::Base` |

| 입력 | 결과 |
| --- | --- |
| 지정가 + `IOC`, `FOK`, `PostOnly` | KRW 마켓만 지원 |
| 최유리 + `IOC` 또는 `FOK` | KRW 마켓만 지원; 체결 조건 필수 |
| `client_id` | 영문 대·소문자, 숫자, `-`, `_`로 구성한 1–36자 |
| `OrderRequest::reduce_only == true` | `Error::Unsupported` |
| `cancel_order(...)`, `cancel_order_by_client_id(...)` | 취소 응답을 검증한 뒤 `()` 반환 |

공통 `Order`는 정규화한 필드만 제공합니다. Bithumb 전용 취소·자전거래 방지 필드와
상세 `trades` 배열은 아직 노출하지 않습니다.

다음 메서드는 `Client::adapter()`가 반환한 어댑터에서 호출합니다.

| 메서드 | 계약 |
| --- | --- |
| `market_warnings()` | 상장 시장마다 원본 `NONE` 또는 `CAUTION` 1건 |
| `market_alerts()` | 활성 행만 반환; 시장·기준당 1행; `ends_at`은 KST에서 UTC로 변환 |

| 거래소 상태 | 매핑 |
| --- | --- |
| `market_warning == CAUTION` | `MarketStatus::Unknown` |
| `BithumbAlertStep::Caution` | 경보(alert) 단계 `주의`; `MarketStatus` 변경 없음 |

## 한도와 공식 링크

| 범위 | 한도 |
| --- | --- |
| 공개 REST | 150/s |
| 비공개 REST | 140/s |
| 주문 REST | 10/s 초과 시 추가 제한 |
| WebSocket 연결 | IP당 10/s; HTTP 429; 반복 초과 시 최대 10분 차단 가능 |

`maxt`는 요청 속도를 제한하지 않습니다. 파생상품, `MarketKind::Perpetual`, 공개
캔들 스트림은 지원하지 않습니다.

- [문서 색인](https://apidocs.bithumb.com/llms.txt)
- [요청 한도](https://apidocs.bithumb.com/docs/api-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9C-%EC%95%88%EB%82%B4.md)
- [최근 체결](https://apidocs.bithumb.com/reference/%EC%B2%B4%EA%B2%B0-%EB%82%B4%EC%97%AD-%EC%A1%B0%ED%9A%8C.md)
- [캔들](https://apidocs.bithumb.com/reference/%EB%B6%84minute-%EC%BA%94%EB%93%A4-%EC%A1%B0%ED%9A%8C.md)
- [WebSocket](https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4.md)
- [주문](https://apidocs.bithumb.com/reference/%EC%A3%BC%EB%AC%B8-%EC%9A%94%EC%B2%AD.md)
- [주문 단건 조회](https://apidocs.bithumb.com/reference/%EA%B0%9C%EB%B3%84-%EC%A3%BC%EB%AC%B8-%EC%A1%B0%ED%9A%8C)
- [종료 주문 목록](https://apidocs.bithumb.com/reference/%EC%A2%85%EB%A3%8C-%EC%A3%BC%EB%AC%B8-%EB%AA%A9%EB%A1%9D-%EC%A1%B0%ED%9A%8C)

[공통 API](../common-api.ko.md) · [거래소 지원](../providers.ko.md)
