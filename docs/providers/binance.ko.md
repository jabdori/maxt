[English](binance.md) | [한국어](binance.ko.md)

# Binance

`BinanceAdapter`는 Binance 현물과 USD 마진 무기한 선물을 제공합니다. 어댑터
하나는 생성할 때 선택한 거래 시장 한 곳만 사용하며, 공개 REST와 시장 스트림에는
인증 정보가 필요하지 않습니다.

## 생성자와 거래 시장

```rust
use maxt::{Client, adapters::BinanceAdapter};

let spot = Client::new(BinanceAdapter::spot());
let usd_m = Client::new(BinanceAdapter::usd_m_futures());
```

| 생성자 | `Market` / `MarketKind` | REST 호스트 | 공개 시장 스트림 호스트 |
| --- | --- | --- | --- |
| `BinanceAdapter::spot()` | `Market::spot` / `MarketKind::Spot` | `https://api.binance.com` | `wss://stream.binance.com:9443/stream` |
| `BinanceAdapter::usd_m_futures()` | `Market::perpetual` / `MarketKind::Perpetual` | `https://fapi.binance.com` | `wss://fstream.binance.com/public/stream`, `/market/stream` |

`BinanceAdapter::default()`는 현물입니다. 다른 거래소의 마켓이나 종류가 맞지 않는
마켓은 네트워크 요청 전에 `Error::InvalidRequest`로 거절합니다.

## 공개 REST

| 공통 호출 | 현물 | USD-M 선물 |
| --- | --- | --- |
| `markets(kind)` | `/api/v3/exchangeInfo`; 현물 목록 | `/fapi/v1/exchangeInfo`; 정확히 `PERPETUAL`인 계약만 |
| `trades(market, limit)` | `/api/v3/trades`; 지정한 `limit`은 `1..=1000` | `/fapi/v1/trades`; 지정한 `limit`은 `1..=1000` |
| `order_book(market, depth)` | `/api/v3/depth`; 지정한 깊이는 `1..=5000`의 모든 정수 | `/fapi/v1/depth`; 지정한 깊이는 `5, 10, 20, 50, 100, 500, 1000` 중 하나 |
| `ticker(market)` | `/api/v3/ticker/24hr` | `/fapi/v1/ticker/24hr` |
| `candles(request)` | `/api/v3/klines`; 거래소 호출당 1,000개 | `/fapi/v1/klines`; 거래소 호출당 1,500개 |
| `funding_rates(request)` | 미지원 | 공개 `/fapi/v1/fundingRate`; 페이지 `limit`은 `1..=1000`, 기본값 100 |

`maxt`는 거래소 응답 한 번보다 큰 캔들 요청을 자동으로 페이지 처리합니다. 체결
목록은 최신순입니다. 현물 호가창에는 거래소 시각이 없어 읽은 시각을 사용하고,
USD-M 호가창은 Binance가 보낸 시각을 유지합니다.

앞으로 추가될 `CANCEL_ONLY` 같은 미등록 상장 상태는 추측하지 않고
`MarketStatus::Unknown`으로 변환합니다.

## 캔들 간격

- 현물: `1s`, `1m`, `3m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `8h`,
  `12h`, `1d`, `3d`, `1w`, `1M`.
- USD-M: 위 목록에서 `1s`를 뺀 나머지이며, 가장 짧은 간격은 1분입니다.
- Binance는 `6h`도 제공하지만 현재 `maxt::Interval` API에는 없습니다. REST
  캔들과 `Feed::Candles`는 같은 목록을 노출합니다.

## 공개 스트림

| 피드 | Binance 스트림 이름 | 전달 방식 |
| --- | --- | --- |
| `Feed::Trades` | `{symbol}@trade` | 체결마다 이벤트 하나. 수량이 0인 USD-M 프레임은 버립니다 |
| `Feed::OrderBook` | `{symbol}@depth20@100ms` | 한쪽당 20단계 전체 스냅샷을 100ms 주기로 전달. 깊이는 변경할 수 없습니다 |
| `Feed::Ticker` | `{symbol}@ticker` | 최근 24시간 이동 구간 티커 |
| `Feed::Candles(interval)` | `{symbol}@kline_<interval>` | 미완성 갱신과 Binance의 확정 여부 |

공개 프레임의 마켓은 구독할 때 전달한 마켓과 소문자 네이티브 심볼(native
symbol)의 대응표로 찾습니다. 고정된 호가 자산 접미사 목록으로 기준 자산과 호가
자산을 추측하지 않습니다. 따라서 `ADAEUR`, `USDTUSD`, `BTCU`, UTF-8 심볼도
구독한 마켓 그대로 유지됩니다.

USD-M은 필요하면 피드를 소켓 두 개로 나눕니다.

| 진입점 | 피드 |
| --- | --- |
| `wss://fstream.binance.com/public/stream` | `Trades`, `OrderBook` |
| `wss://fstream.binance.com/market/stream` | `Ticker`, `Candles` |

두 소켓은 논리 시장 스트림(`MarketStream`) 하나로 합칩니다. 재연결 알림
(`MarketEvent::Reconnected`)은 소켓마다 따로 전달합니다. 어느 한 소켓이라도
종료되면 구독의 절반만 조용히 사라지지 않도록 다른 소켓도 버리고 논리 스트림
전체를 종료합니다. 현물 시장 데이터는 소켓 하나를 사용합니다.

## 요청 가중치

Binance는 단순 요청 수뿐 아니라 IP 기준 요청 가중치(`REQUEST_WEIGHT`)를
제한합니다. 2026-07-31 검증 시점의 `exchangeInfo`는 현물 분당 6,000, USD-M
분당 2,400을 알렸습니다. 실제 `rateLimits` 항목과
`X-MBX-USED-WEIGHT-1M` 응답 헤더를 기준으로 삼으세요. `maxt`는 자체적으로 속도를
조절하거나 이 헤더를 읽지 않으며, HTTP 429와 418은 요청 제한 오류로 분류합니다.

## 인증 정보와 Binance 전용 메서드

계정·주문·비공개 스트림 호출에는 `.with_credentials(api_key, secret_key)`를
사용합니다. 어댑터는 API 키와 시크릿 키(secret key)를 이용한 HMAC-SHA-256 서명만
구현합니다. Binance 자체는 RSA와 Ed25519 키도 지원하지만 `maxt`는 지원하지
않습니다.

| 메서드 | 계약 |
| --- | --- |
| `spot_symbol_filters(&market)` | 현물의 `PRICE_FILTER`, `LOT_SIZE`, `NOTIONAL` 값. USD-M에서는 미지원 |
| `spot_order(&market, order_id)` | Binance가 발급한 숫자 식별자로 현물 주문 하나를 조회. 체결 완료·취소 주문도 포함 |
| `usd_m_create_listen_key()` | 계정의 USD-M 리슨 키(listen key)를 생성하거나 연장 |
| `usd_m_keepalive_listen_key(&key)` | 현재 USD-M 리슨 키를 연장 |
| `usd_m_close_listen_key(&key)` | USD-M 리슨 키를 종료 |

`subscribe_account`는 USD-M 리슨 키의 수명 주기를 관리합니다. 현물 계정 스트림은
리슨 키 대신 `wss://ws-api.binance.com:443/ws-api/v3`에 서명된
`userDataStream.subscribe.signature` 요청을 보냅니다.

## 제한사항

| 영역 | 현재 경계 |
| --- | --- |
| USD-M 상장 목록 | 정확히 `contractType == PERPETUAL`인 항목만 노출. 현재 `TRADIFI_PERPETUAL`, `CURRENT_QUARTER`, `NEXT_QUARTER` 항목은 제외 |
| 다른 Binance 상품 | 현물 마진, COIN-M 선물, 옵션, 포트폴리오 마진 API는 미지원 |
| 캔들 간격 | Binance의 `6h` 간격은 표현할 수 없음 |
| 주문 규칙 | `spot_symbol_filters`에 대응하는 USD-M 메서드가 없고, 필터에 맞춘 반올림이나 주문 사전 검증도 하지 않음 |
| 스트림 변형 | 고정 부분 호가창만 노출. 깊이 변경, 차분 호가창 재구성, `@aggTrade`는 미지원 |
| 호스트와 인증 | 테스트넷 생성자가 없으며 RSA·Ed25519 인증 정보를 지원하지 않음 |

## 검증 범위

2026-07-31에 대표 BTC/USDT 현물·USD-M 마켓으로 공개 REST와 스트림 스모크
테스트(smoke test)를 통과했습니다. 비공개 실시간 호출은 검증하지 않았습니다.

## 예제

```text
cargo run --example public_rest -- binance BTC USDT
```

## 공식 문서

- [현물 REST 시장 데이터](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market)
- [현물 REST 보안과 요청 한도](https://developers.binance.com/en/docs/products/spot/rest-api)
- [현물 WebSocket 시장 스트림](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)
- [USD-M REST 시장 데이터](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [USD-M 공개 WebSocket 스트림](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/public)
- [USD-M 시장 WebSocket 스트림](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/market)
- [USD-M 일반 정보](https://developers.binance.com/en/docs/products/derivatives-trading-usds-futures/general-info)

---

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
