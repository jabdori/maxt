[English](bithumb.md) | [한국어](bithumb.ko.md)

# Bithumb

빗썸(Bithumb) 어댑터(adapter)는 현물(spot) 전용입니다. 공개 REST 시세와 공개 체결·호가창·현재가 스트림(stream)을 제공하지만, 파생상품과 캔들 스트림은 제공하지 않습니다.

## 생성과 지원 범위

```rust
use maxt::{Client, adapters::BithumbAdapter};

let public = Client::new(BithumbAdapter::new());
let access_key = "your access key";
let secret_key = "your secret key";
let authenticated = Client::new(
    BithumbAdapter::new().with_credentials(access_key, secret_key),
);
```

`BithumbAdapter::new()`에는 인증 정보(credentials)가 필요 없습니다. `with_credentials(access_key, secret_key)`는 계좌·주문·비공개 스트림 호출에 필요한 액세스 키와 시크릿 키를 추가하며, 현물 전용 범위는 바꾸지 않습니다.

마켓은 `Market::spot(Exchange::Bithumb, "BTC", "KRW")`로 만듭니다. 빗썸의 자체 마켓 코드는 `KRW-BTC`입니다.

## 공개 REST

| 호출 | 빗썸 동작과 한도 |
| --- | --- |
| `markets(MarketKind::Spot)` | 상장된 현물 마켓을 반환하며, 다른 마켓 종류에는 빈 목록을 반환합니다 |
| `trades` | `limit`은 `1..=500`이어야 하며, 생략하면 빗썸 기본값 1을 사용합니다 |
| `order_book` | `depth`는 0보다 커야 합니다. `maxt`는 단일 마켓을 요청하고, 수량이 0인 슬롯을 제거한 뒤 양쪽을 최우선 호가 순으로 정렬합니다. 한쪽당 유효한 호가를 최대 30단계까지 반환하고 `depth`만큼 더 잘라냅니다 |
| `ticker` | 요청한 마켓의 현재가 스냅샷(snapshot) 하나를 반환합니다 |
| `candles` | 빗썸은 지원하는 모든 캔들 응답을 최대 200개로 제한합니다. `maxt`는 최대 100회까지 페이지를 넘기므로 요청 하나에서 최대 20,000개를 모을 수 있고, 더 큰 `limit`이나 시간 구간은 거절합니다 |

지원하는 캔들 간격(interval)은 `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1`입니다.

## 공개 스트림

| 피드(feed) | 동작 |
| --- | --- |
| `Feed::Trades` | 공개 체결 이벤트를 제공합니다 |
| `Feed::OrderBook` | 수량이 0인 슬롯을 제거한 뒤 한쪽당 유효한 호가를 최대 15단계까지 담은 전체 스냅샷을 제공합니다. 전송 시각은 마이크로초 단위입니다 |
| `Feed::Ticker` | 공개 현재가 스냅샷과 실시간 갱신을 제공합니다 |
| `Feed::Candles(_)` | 소켓을 열기 전에 `Error::Unsupported`를 반환합니다. 빗썸은 공개 캔들 스트림을 발행하지 않으며 `maxt`도 이를 합성하지 않습니다 |

## 캔들 범위와 기준선

캔들 요청의 시작 시각(`CandleRequest::from`)은 포함하고 종료 시각(`CandleRequest::to`)은 포함하지 않습니다. 빗썸의 `to` 파라미터도 한국 표준시(KST) 벽시계 값이며 해당 시각을 제외합니다. `maxt`는 호출자의 UTC `Timestamp`를 변환하면서 1초 미만 종료 시각의 배타성도 보존합니다. 결과는 오래된 순으로 반환합니다.

| 간격 | UTC 기준 캔들 시작 시각 |
| --- | --- |
| `Min1`부터 `Hour1`까지 | 일반 UTC 단위 경계 |
| `Hour4` | 03:00, 07:00, 11:00, 15:00, 19:00, 23:00 |
| `Day1` | 15:00, 다음 날짜 00:00 KST |
| `Week1` | 일요일 15:00, 월요일 00:00 KST |
| `Month1` | 직전 달의 마지막 UTC 날짜 15:00, 해당 월 1일 00:00 KST |

## 빗썸 전용 마켓 표시

다음 메서드는 `Client::adapter()`를 통해 호출하며, 서로 다른 빗썸 표시를 설명합니다.

| 메서드 | 의미 |
| --- | --- |
| `market_warnings()` | `/v1/market/all?isDetails=true`를 읽어 상장된 모든 마켓과 원본 `market_warning` 값(`NONE` 또는 `CAUTION`)을 반환합니다. 여기서 `CAUTION`은 유의 종목을 뜻하며, 거래는 계속되고 `MarketStatus::Unknown`으로 매핑됩니다 |
| `market_alerts()` | 경보제 엔드포인트를 읽어 현재 활성화된 경보만 마켓·기준별 한 행으로 반환합니다. 경보 기준, 단계, KST 종료 시각을 UTC로 바꾼 값이 들어갑니다. 경보가 없는 마켓은 빠지며 경보는 `MarketStatus`를 바꾸지 않습니다 |

따라서 `CAUTION`은 문맥에 따라 다릅니다. `market_warning` 값이면 유의 종목이고, `BithumbAlertStep::Caution`이면 경보제에서 가장 낮은 주의 단계입니다.

## 인증 정보와 현재 비공개 기능 한계

인증 정보는 잔고, 대기 주문, 주문 생성·취소, 비공개 계좌 스트림을 활성화합니다. 인증 정보가 없으면 비공개 요청을 만들기 전에 `Error::Auth`를 반환합니다.

현재 빗썸 API는 조건에 맞는 주문에서 IOC, FOK, Post-Only 같은 주문 처리 조건(time in force, TIF)을 지원합니다. 현재 `maxt` 빗썸 어댑터는 이 기능을 노출하지 않으며, `OrderRequest::time_in_force`를 지정하면 `Error::InvalidRequest`로 거절합니다.

## 요청 한도

| 공식 적용 범위 | 한도 |
| --- | --- |
| 공개 REST API | 초당 최대 150회 |
| 비공개 REST API | 초당 최대 140회 |
| 주문 관련 REST API | 초당 10회를 초과하면 적용될 수 있는 추가 제한 |
| WebSocket 연결 요청 | 공개·비공개 모두 IP당 초당 최대 10회입니다. 초과 요청에는 HTTP 429가 반환되고, 초과가 지속되면 WebSocket 사용이 10분 동안 차단될 수 있습니다 |

빗썸은 과도한 트래픽이 발생하면 사전 공지 없이 REST 한도를 낮출 수 있습니다.

## 오류

로컬 범위와 요청 형태 검증 실패는 `Error::InvalidRequest`입니다. 빗썸의 `{"error": ...}` 오류 응답 구조(error envelope)는 HTTP 상태가 2xx여도 `Error::Exchange`로 처리하며, 숫자인 `error.name`도 문자열 오류 코드로 보존합니다. 2xx가 아닌 빗썸 오류도 `Error::Exchange`입니다.

## 검증

2026-07-31에 BTC/KRW 대표 공개 스모크 테스트(smoke test)로 마켓 조회, 현재가, 호가창, 최근 체결, 캔들, 공개 Trades·OrderBook·Ticker 스트림을 확인했습니다. 비공개 실계좌와 주문 동작은 검증하지 않았습니다.

공개 REST 예제는 다음 명령으로 실행합니다.

```text
cargo run --example public_rest -- bithumb BTC KRW
```

## 공식 문서

| 주제 | 현재 빗썸 문서 |
| --- | --- |
| 문서 목록과 한도 | [문서 목록](https://apidocs.bithumb.com/llms.txt) · [API 요청 수 제한](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내.md) |
| 공개 REST | [마켓 목록](https://apidocs.bithumb.com/reference/거래-대상-목록-조회.md) · [경보제](https://apidocs.bithumb.com/reference/경보제-조회.md) · [최근 체결](https://apidocs.bithumb.com/reference/체결-내역-조회.md) · [현재가](https://apidocs.bithumb.com/reference/현재가-조회.md) · [호가창](https://apidocs.bithumb.com/reference/호가-조회.md) |
| 캔들 | [분](https://apidocs.bithumb.com/reference/분minute-캔들-조회.md) · [일](https://apidocs.bithumb.com/reference/일day-캔들-조회.md) · [주](https://apidocs.bithumb.com/reference/주week-캔들-조회.md) · [월](https://apidocs.bithumb.com/reference/월month-캔들-조회.md) |
| 공개 WebSocket | [기본 정보와 연결 한도](https://apidocs.bithumb.com/reference/기본-정보.md) · [현재가](https://apidocs.bithumb.com/reference/현재가-ticker.md) · [체결](https://apidocs.bithumb.com/reference/체결-trade.md) · [호가창](https://apidocs.bithumb.com/reference/호가-orderbook.md) |
| 주문 처리 조건 | [주문 요청](https://apidocs.bithumb.com/reference/주문-요청.md) |

---

[공통 API](../common-api.ko.md) · [거래소 고르기](../providers.ko.md)
