# Upbit

[English](upbit.md) | [한국어](upbit.ko.md)

## 거래소와 생성자

현물 전용입니다. `UpbitAdapter`는 한 지역에 고정됩니다. host, 상장, 호가, 계좌,
인증 정보는 지역 간 공유하지 않습니다.

| 지역 | 생성자 | REST | WebSocket |
| --- | --- | --- | --- |
| 한국 | `UpbitAdapter::new()` 또는 `with_region(UpbitRegion::Korea)` | `https://api.upbit.com` | `wss://api.upbit.com/websocket/v1` |
| 싱가포르 | `with_region(UpbitRegion::Singapore)` | `https://sg-api.upbit.com` | `wss://sg-api.upbit.com/websocket/v1` |
| 인도네시아 | `with_region(UpbitRegion::Indonesia)` | `https://id-api.upbit.com` | `wss://id-api.upbit.com/websocket/v1` |
| 태국 | `with_region(UpbitRegion::Thailand)` | `https://th-api.upbit.com` | `wss://th-api.upbit.com/websocket/v1` |

| 필드 | 값 |
| --- | --- |
| `Market` | `Market::spot(Exchange::Upbit, "BTC", "KRW")` |
| `MarketInfo::native_symbol` | `KRW-BTC` |
| `base`, `quote` | `[A-Z0-9]+` |

## REST

| 호출 | 엔드포인트(endpoint) | 계약 |
| --- | --- | --- |
| `markets(MarketKind::Spot)` | `/v1/market/all?is_details=true` | 상장된 Spot 시장 |
| `markets(MarketKind::Perpetual)` | — | `Ok(vec![])` |
| `trades(market, limit)` | `/v1/trades/ticks` | `limit: 1..=500`; 최신순 |
| `order_book(market, depth)` | `/v1/orderbook` | `depth: 1..=30`; 각 측 `len() <= depth`; `None → 30` |
| `ticker(market)` | `/v1/ticker` | 시장 스냅샷(snapshot) 1건 |

파생상품 메서드는 `Error::Unsupported`를 반환합니다.

## 캔들

| API | 지원하는 `Interval` | 매핑하지 않는 거래소 간격 |
| --- | --- | --- |
| REST | `Sec1`, `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` | `1y` |
| WebSocket | `Sec1`, `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4` | — |

| 제약 | 값 |
| --- | ---: |
| 거래소 페이지 상한 | 200 |
| 요청당 거래소 호출 수 | `<= 100` |
| 사전 계산 캔들 수 | `<= 20_000` |
| `Sec1` 보존 기간 | 최근 3개월 |

## 스트림

| `Feed` | 계약 |
| --- | --- |
| `Feed::Trades` | 체결당 이벤트 1건; `Trade::id = sequential_id` |
| `Feed::OrderBook` | 전체 스냅샷; 각 측 30개 호가 단계; 고정 depth |
| `Feed::Ticker` | 전체 스냅샷 |
| `Feed::Candles(interval)` | 형성 중 캔들 갱신; `open_time` 전환 시 확정 이벤트 |

| 수신 캔들의 `open_time` | 결과 |
| --- | --- |
| `SNAPSHOT && interval_end <= now` | `closed == true` 1건 발행 |
| 현재 캔들과 같음 | 현재 캔들 교체; `closed == false` 발행 |
| 현재 캔들보다 큼; `REALTIME` | 이전 캔들 `closed == true` 발행 → 수신 캔들 `closed == false` 발행 |
| 현재 캔들보다 작음 또는 마지막 확정 캔들 이하 | 프레임(frame) 폐기 |
| 후속 프레임 없음 또는 재연결 | 합성 확정 이벤트 없음 |

## 비공개 API와 거래소 전용 API

`.with_credentials(access_key, secret_key)`로 인증 정보를 설정합니다. 인증 정보 발급
지역은 `UpbitAdapter::region()`과 같아야 합니다. 설정 후 잔고, 미체결 주문, 주문
생성·취소, 계좌 스트림을 사용할 수 있습니다.

| 주문 입력 | 계약 |
| --- | --- |
| 최유리 매수 | `Size::Quote`와 `IOC` 또는 `FOK` |
| 최유리 매도 | `Size::Base`와 `IOC` 또는 `FOK` |
| `client_id` | RFC 3986 비예약 ASCII 문자로 구성한 1–64바이트; `cancel_order_by_client_id`에 사용 가능 |
| 취소 메서드 | 거래소 응답을 검증한 뒤 `()` 반환 |

다음 메서드는 `Client::adapter()`가 반환한 어댑터에서 호출합니다.

| 메서드 | 계약 | 요청 한도 그룹 |
| --- | --- | --- |
| `tickers(&[Market])` | `markets.len() >= 1`; 시장당 ticker 1건 | `ticker` |
| `order_books(&[Market], depth)` | `markets.len() >= 1`; `depth: 1..=30` 또는 `None` | `orderbook` |
| `market_events()` | 시장별 투자 유의 여부와 기준 | `market` |

| 시장 이벤트(market event) | 매핑 |
| --- | --- |
| `warning == true` | `MarketStatus::Unknown` |
| `cautions`가 비어 있지 않음 | `MarketStatus` 변경 없음 |
| `region != UpbitRegion::Korea` | `UpbitMarketEvent::cautions == []` |

## 한도와 공식 링크

| 그룹 | 한도 | 범위 |
| --- | --- | --- |
| `market`, `candle`, `trade`, `ticker`, `orderbook` | 각각 10/s | IP |
| `default` | 30/s | 한국: Pocket; Global: Account |
| `order`, `order-test` | 각각 8/s | 한국: Pocket; Global: Account |
| `order-cancel-all` | 1/2s | 한국: Pocket; Global: Account |
| WebSocket 연결 | 5/s | 미인증: IP; 인증: Pocket 또는 Account |
| WebSocket 메시지 | 5/s, 100/min | 연결 |

`maxt`는 요청 속도를 제한하지 않습니다. `Remaining-Req`를 확인하세요. HTTP 요청
한도 오류는 `Error::is_rate_limited() == true`입니다.

- [지역과 엔드포인트](https://global-docs.upbit.com/reference/api-overview)
- [공개 REST](https://global-docs.upbit.com/reference/list-trading-pairs)
- [호가](https://global-docs.upbit.com/reference/list-orderbooks)
- [캔들](https://global-docs.upbit.com/reference/list-candles-minutes)
- [WebSocket](https://global-docs.upbit.com/reference/websocket-guide)
- [요청 한도](https://global-docs.upbit.com/reference/rate-limits)
- [인증](https://global-docs.upbit.com/reference/auth)

[공통 API](../common-api.ko.md) · [거래소 지원](../providers.ko.md)
