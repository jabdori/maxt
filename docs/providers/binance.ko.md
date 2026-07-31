# Binance

[English](binance.md) | [한국어](binance.ko.md)

## 거래 범위와 생성자

`BinanceAdapter`는 현물 또는 USD-M 무기한 선물 중 하나에 고정됩니다.

| 생성자 | `MarketKind` | REST | 공개 시장 스트림 |
| --- | --- | --- | --- |
| `BinanceAdapter::spot()` | `Spot` | `https://api.binance.com` | `wss://stream.binance.com:9443/stream` |
| `BinanceAdapter::usd_m_futures()` | `Perpetual` | `https://fapi.binance.com` | `wss://fstream.binance.com/public/stream`, `wss://fstream.binance.com/market/stream` |

`BinanceAdapter::default()`는 현물 어댑터를 생성합니다. 다른 거래소나
`MarketKind`의 `Market`은 네트워크 요청 전에 `Error::InvalidRequest`를 반환합니다.

## 공개 REST

| 호출 | 현물 | USD-M |
| --- | --- | --- |
| `markets(kind)` | `/api/v3/exchangeInfo`; 현물 상장 | `/fapi/v1/exchangeInfo`; `contractType == PERPETUAL` |
| `trades(market, limit)` | `/api/v3/trades`; `limit in 1..=1000` | `/fapi/v1/trades`; `limit in 1..=1000` |
| `order_book(market, depth)` | `/api/v3/depth`; `depth in 1..=5000` | `/fapi/v1/depth`; `depth in {5, 10, 20, 50, 100, 500, 1000}` |
| `ticker(market)` | `/api/v3/ticker/24hr`; 24시간 이동 요약 | `/fapi/v1/ticker/24hr`; 24시간 이동 요약 |
| `funding_rates(request)` | `Error::Unsupported` | `/fapi/v1/fundingRate`; `limit in 1..=1000`; `None -> 100` |

체결 결과는 최신순입니다. 현물 호가는 조회 시각을 `timestamp`로 사용하고,
USD-M 호가는 Binance가 제공한 시각을 유지합니다. 알 수 없는 상장 상태는
`MarketStatus::Unknown`입니다.

## 캔들

| 시장 | 노출하는 `Interval` variant |
| --- | --- |
| 현물 | `Sec1`, `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| USD-M | 현물 목록에서 `Sec1` 제외 |

REST와 `Feed::Candles`의 지원 간격은 같습니다. Binance native `6h`는
`Interval`에 매핑하지 않습니다.

| 제약 | 현물 | USD-M |
| --- | ---: | ---: |
| 거래소 응답당 최대 개수 | 1,000 | 1,500 |
| 요청당 거래소 호출 수 | `<= 100` | `<= 100` |
| 사전 계산한 캔들 개수 | `<= 100_000` | `<= 150_000` |

## 공개 스트림

| 피드 | 스트림 | 계약 |
| --- | --- | --- |
| `Feed::Trades` | `{symbol}@trade` | 체결 1건당 이벤트 1건; 수량 `== 0` 프레임은 버림 |
| `Feed::OrderBook` | `{symbol}@depth20@100ms` | 각 측 20단계 전체 스냅샷; 100ms; 깊이 변경 미지원 |
| `Feed::Ticker` | `{symbol}@ticker` | 24시간 이동 요약 |
| `Feed::Candles(interval)` | `{symbol}@kline_<interval>` | Binance `closed` 값을 유지 |

마켓은 `Subscription`의 native symbol로 조회합니다. USD-M은 `Trades`와
`OrderBook`에 `/public/stream`, `Ticker`와 `Candles`에 `/market/stream`을
사용합니다. 두 소켓을 사용하면 `MarketEvent::Reconnected`는 소켓별로
발행됩니다. 한 소켓이 종료되면 다른 소켓도 닫고 `MarketStream`을 종료합니다.

## 인증 후 기능과 Binance 전용 메서드

`with_credentials(api_key, secret_key)`는 HMAC-SHA-256 키만 지원합니다.

| 시장 | 인증 후 기능 |
| --- | --- |
| 현물 | 잔고, 미체결 주문, 주문 생성·취소, 계정 스트림 |
| USD-M | 현물 기능 + 포지션, 증거금 요약·설정, 펀딩 지급 이력, reduce-only 주문 |

| 주문 입력 | 현물 | USD-M |
| --- | --- | --- |
| `Size::Base` | 모든 주문 | 모든 주문 |
| `Size::Quote` | 시장가 주문만 지원; 지정가 -> `Error::InvalidRequest` | `Error::InvalidRequest` |
| `time_in_force` | `GTC`, `IOC`, `FOK`; `PostOnly -> LIMIT_MAKER` | `GTC`, `IOC`, `FOK`; `PostOnly -> GTX` |
| `reduce_only == true` | `Error::Unsupported` | 지원 |

| 메서드 | 계약 |
| --- | --- |
| `spot_symbol_filters(&market)` | 현물 `PRICE_FILTER`, `LOT_SIZE`, `NOTIONAL`; USD-M 미지원 |
| `spot_order(&market, order_id)` | 숫자 `order_id`로 현물 주문 1건 조회; 종료 주문 포함 |
| `usd_m_create_listen_key()` | USD-M listen key 생성·연장 |
| `usd_m_keepalive_listen_key(&key)` | USD-M listen key 연장 |
| `usd_m_close_listen_key(&key)` | USD-M listen key 종료 |

`subscribe_account`는 USD-M listen key 수명 주기를 관리합니다. 현물 계정
스트림은 서명된 `userDataStream.subscribe.signature`를 사용합니다.

주문 값은 symbol filter에 맞춰 반올림하거나 사전 검증하지 않습니다.

## 한도·미지원·공식 링크

Binance는 IP별 `REQUEST_WEIGHT`를 제한합니다. 호출별 weight와 상한은
`exchangeInfo.rateLimits`, 사용량은 `X-MBX-USED-WEIGHT-1M`에서 확인합니다.
`maxt`는 요청 속도를 제한하지 않습니다. HTTP 429와 418은
`Error::is_rate_limited() == true`입니다.

현물 마진, COIN-M, 옵션, 포트폴리오 마진, 분기물, configurable stream depth,
diff-depth 재구성, `@aggTrade`, 테스트넷 생성자, RSA·Ed25519 키는
노출하지 않습니다.

- [현물 REST 시장 데이터](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market)
- [현물 REST 한도·인증](https://developers.binance.com/en/docs/products/spot/rest-api)
- [현물 WebSocket](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)
- [USD-M REST 시장 데이터](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [USD-M public stream](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/public)
- [USD-M market stream](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/market)

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
