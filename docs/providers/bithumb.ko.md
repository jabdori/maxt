# Bithumb

[English](bithumb.md) | [한국어](bithumb.ko.md)

## 거래 범위와 생성자

현물 전용입니다.

| 생성자 | 기능 |
| --- | --- |
| `BithumbAdapter::new()` | 공개 REST·스트림 |
| `.with_credentials(access_key, secret_key)` | 계좌·주문·비공개 스트림 메서드 |

| 필드 | 값 |
| --- | --- |
| `Market` | `Market::spot(Exchange::Bithumb, "BTC", "KRW")` |
| `MarketInfo::native_symbol` | `KRW-BTC` |

## 공개 REST

| 호출 | 엔드포인트 | 계약 |
| --- | --- | --- |
| `markets(MarketKind::Spot)` | `/v1/market/all?isDetails=true` | 상장된 현물 마켓 |
| `markets(MarketKind::Perpetual)` | — | `Ok(vec![])` |
| `trades(market, limit)` | `/v1/trades/ticks` | `limit in 1..=500`; `None -> 1`; 최신순 |
| `order_book(market, depth)` | `/v1/orderbook` | `depth in 1..=30`; `None -> 30`; `quantity == 0` 제거, 정렬, 로컬 절단 |
| `ticker(market)` | `/v1/ticker` | 마켓 1개 스냅샷 |

`HTTP 2xx + {"error": ...} -> Error::Exchange`. 숫자 `error.name`도 문자열 코드로
보존합니다.

## 캔들

| 계약 | 값 |
| --- | --- |
| 지원 `Interval` | `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` |
| 노출하지 않는 native interval | `10m` |
| 거래소 응답당 최대 개수 | 200 |
| 요청당 거래소 호출 수 | `<= 100` |
| 사전 계산한 캔들 개수 | `<= 20_000` |
| 거래소 `to` | `format_kst(ceil_second(to))`; 배타 경계 |

| 간격 | `open_time` UTC 격자 |
| --- | --- |
| `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1` | UTC 단위 경계 |
| `Hour4` | `03:00`, `07:00`, `11:00`, `15:00`, `19:00`, `23:00` |
| `Day1` | `15:00` |
| `Week1` | 일요일 `15:00` |
| `Month1` | 직전 월 마지막 UTC 날짜 `15:00` |

## 공개 스트림

| 피드 | 계약 |
| --- | --- |
| `Feed::Trades` | 공개 체결 이벤트 |
| `Feed::OrderBook` | 전체 스냅샷; `quantity == 0` 제거; 한쪽당 최대 15단계; 거래소 원본 `timestamp` 단위: µs |
| `Feed::Ticker` | 스냅샷·실시간 갱신 |
| `Feed::Candles(_)` | 연결 전 `Error::Unsupported` |

## 인증 후 기능과 Bithumb 전용 메서드

인증 후 잔고, 미체결 주문, 주문 생성·취소, 계좌 스트림을 사용할 수 있습니다.
`open_orders()`는 `/v1/orders` 1회, 최대 100건입니다.

| 주문 | 필수 `Size` |
| --- | --- |
| 지정가 매수·매도 | `Size::Base` |
| 시장가 매수 | `Size::Quote` |
| 시장가 매도 | `Size::Base` |

| 입력 | 결과 |
| --- | --- |
| `OrderRequest::time_in_force.is_some()` | `Error::InvalidRequest` |
| `OrderRequest::reduce_only == true` | `Error::Unsupported` |
| `cancel_order(...)` | 취소 응답; `status = Cancelled`; 체결 필드 미제공 |

| 메서드 | 계약 |
| --- | --- |
| `market_warnings()` | 상장 마켓당 원본 `NONE` 또는 `CAUTION` 1개 |
| `market_alerts()` | 활성 행만 반환; 마켓·기준당 1행; `ends_at` KST -> UTC |

| 거래소 상태 | 매핑 |
| --- | --- |
| `market_warning == CAUTION` | `MarketStatus::Unknown` |
| `BithumbAlertStep::Caution` | 경보 단계 `주의`; `MarketStatus` 변경 없음 |

## 한도·미지원·공식 링크

| 범위 | 한도 |
| --- | --- |
| 공개 REST | 150/s |
| 비공개 REST | 140/s |
| 주문 REST | 10/s 초과 시 추가 제한 |
| WebSocket 연결 | IP당 10/s; HTTP 429; 반복 초과 시 10분 차단 가능 |

`maxt`는 요청 속도를 제한하지 않습니다. 파생상품, `MarketKind::Perpetual`, 공개
캔들 스트림, `time_in_force`는 미지원입니다.

- [문서 색인](https://apidocs.bithumb.com/llms.txt)
- [요청 제한](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내.md)
- [최근 체결](https://apidocs.bithumb.com/reference/체결-내역-조회.md)
- [캔들](https://apidocs.bithumb.com/reference/분minute-캔들-조회.md)
- [WebSocket](https://apidocs.bithumb.com/reference/기본-정보.md)
- [주문](https://apidocs.bithumb.com/reference/주문-요청.md)

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
