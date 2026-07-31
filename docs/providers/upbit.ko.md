# Upbit

[English](upbit.md) | [한국어](upbit.ko.md)

## 거래 범위와 생성자

현물 전용입니다. `UpbitAdapter`는 한 지역에 고정되며 호스트·상장·호가·계좌·인증
정보를 지역 간 공유하지 않습니다.

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

## 공개 REST

| 호출 | 엔드포인트 | 계약 |
| --- | --- | --- |
| `markets(MarketKind::Spot)` | `/v1/market/all?is_details=true` | 상장된 현물 마켓 |
| `markets(MarketKind::Perpetual)` | — | `Ok(vec![])` |
| `trades(market, limit)` | `/v1/trades/ticks` | `limit in 1..=500`; 최신순 |
| `order_book(market, depth)` | `/v1/orderbook` | `depth in 1..=30`; `None -> 30` |
| `ticker(market)` | `/v1/ticker` | 마켓 1개 스냅샷 |

파생상품 메서드는 `Error::Unsupported`를 반환합니다.

## 캔들

| 표면 | 노출하는 `Interval` variant | 노출하지 않는 native interval |
| --- | --- | --- |
| REST | `Sec1`, `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` | `10m`, `1y` |
| WebSocket | `Sec1`, `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4` | `10m` |

| 제약 | 값 |
| --- | ---: |
| 거래소 응답당 최대 개수 | 200 |
| 요청당 거래소 호출 수 | `<= 100` |
| 사전 계산한 캔들 개수 | `<= 20_000` |
| `Sec1` 보존 범위 | 최근 3개월 |

## 공개 스트림

| 피드 | 계약 |
| --- | --- |
| `Feed::Trades` | 체결당 이벤트 1건; `Trade::id = sequential_id` |
| `Feed::OrderBook` | 전체 스냅샷; 한쪽당 30단계; 깊이 고정 |
| `Feed::Ticker` | 전체 스냅샷 |
| `Feed::Candles(interval)` | 형성 봉 갱신과 전이 기반 종결 이벤트 |

| 캔들 프레임 | 이벤트 |
| --- | --- |
| `SNAPSHOT && interval_end <= now` | `closed == true` 1건 |
| `new.open_time == held.open_time` | `held` 교체; `closed == false` 발행 |
| `REALTIME && new.open_time > held.open_time` | `held(closed == true)`, 이후 `new(closed == false)` |
| `new.open_time < held.open_time || new.open_time <= settled.open_time` | 프레임 폐기 |
| 후속 프레임 없음 또는 재연결 | 합성 종결 이벤트 없음 |

## 인증 후 기능과 Upbit 전용 메서드

`.with_credentials(access_key, secret_key)`를 사용합니다. 인증 정보 발급 지역은
`UpbitAdapter::region()`과 같아야 합니다. 인증 후 잔고, 미체결 주문, 주문
생성·취소, 계좌 스트림을 사용할 수 있습니다.

| 메서드 | 계약 | 제한 그룹 |
| --- | --- | --- |
| `tickers(&[Market])` | `markets.len() >= 1`; 마켓당 ticker 1건 | `ticker` |
| `order_books(&[Market], depth)` | `markets.len() >= 1`; `depth in 1..=30` 또는 `None` | `orderbook` |
| `market_events()` | 마켓별 유의 종목 여부와 주의 기준 | `market` |

| 마켓 이벤트 | 매핑 |
| --- | --- |
| `warning == true` | `MarketStatus::Unknown` |
| `cautions` 비어 있지 않음 | `MarketStatus` 변경 없음 |
| `region != UpbitRegion::Korea` | `UpbitMarketEvent::cautions == []` |

## 한도·미지원·공식 링크

| 그룹 | 한도 | 적용 범위 |
| --- | --- | --- |
| `market`, `candle`, `trade`, `ticker`, `orderbook` | 각각 10/s | IP |
| `default` | 30/s | 한국: Pocket; Global: Account |
| `order`, `order-test` | 각각 8/s | 한국: Pocket; Global: Account |
| `order-cancel-all` | 1/2s | 한국: Pocket; Global: Account |
| WebSocket 연결 | 5/s | 미인증: IP; 인증: Pocket 또는 Account |
| WebSocket 메시지 | 5/s, 100/min | 연결 |

`maxt`는 요청 속도를 제한하지 않습니다. `Remaining-Req`를 확인합니다. HTTP 요청
제한 오류는 `Error::is_rate_limited() == true`입니다.

- [지역·엔드포인트](https://global-docs.upbit.com/reference/api-overview)
- [공개 REST](https://global-docs.upbit.com/reference/list-trading-pairs)
- [호가](https://global-docs.upbit.com/reference/list-orderbooks)
- [캔들](https://global-docs.upbit.com/reference/list-candles-minutes)
- [WebSocket](https://global-docs.upbit.com/reference/websocket-guide)
- [요청 제한](https://global-docs.upbit.com/reference/rate-limits)
- [인증](https://global-docs.upbit.com/reference/auth)

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
