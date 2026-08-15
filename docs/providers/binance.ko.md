# Binance

[English](binance.md) | [한국어](binance.ko.md)

Spot, USD-M, 거래소 전용 공개 조회는 [Binance provider 예제](../examples.md#binance-provider)로 실행할 수 있습니다.

## 거래소와 생성자

`BinanceAdapter` 하나는 Spot 또는 USD-M 무기한 선물에 고정됩니다.

| 생성자 | `MarketKind` | REST | 공개 WebSocket |
| --- | --- | --- | --- |
| `BinanceAdapter::spot()` | `Spot` | `https://api.binance.com` | `wss://stream.binance.com:9443/stream` |
| `BinanceAdapter::usd_m_futures()` | `Perpetual` | `https://fapi.binance.com` | `wss://fstream.binance.com/public/stream`, `/market/stream` |

`BinanceAdapter::default()`는 Spot입니다. `market.exchange != Exchange::Binance` 또는
`market.kind`가 생성자 표의 `MarketKind`와 다르면 네트워크 I/O 전
`Error::InvalidRequest`를 반환합니다.

## REST

| 호출 | Spot | USD-M |
| --- | --- | --- |
| `markets(kind)` | `/api/v3/exchangeInfo`; Spot 상장 | `/fapi/v1/exchangeInfo`; `contractType == PERPETUAL` |
| `trades(market, limit)` | `/api/v3/trades`; `limit: 1..=1000` | `/fapi/v1/trades`; `limit: 1..=1000` |
| `order_book(market, depth)` | `/api/v3/depth`; `depth: 1..=5000`; 각 측 `len() <= depth` | `/fapi/v1/depth`; `depth: {5, 10, 20, 50, 100, 500, 1000}`; 각 측 `len() <= depth` |
| `ticker(market)` | `/api/v3/ticker/24hr`; 최근 24시간 요약 | `/fapi/v1/ticker/24hr`; 최근 24시간 요약 |
| `spot_average_price(market)` | 공개 `/api/v3/avgPrice`; Binance의 평균 구간과 마지막 포함 체결 시각을 담은 현재 Spot 평균가 스냅샷 | `Error::Unsupported` |
| `funding_rates(request)` | `Error::Unsupported` | `/fapi/v1/fundingRate`; `limit: 1..=1000`; `None → 100` |
| `mark_price(market)` | `Error::Unsupported` | `/fapi/v1/premiumIndex`; USD-M 무기한 선물 1건의 mark price 스냅샷 |
| `mark_prices()` | `Error::Unsupported` | `/fapi/v1/premiumIndex`; 지원하는 USD-M 무기한 선물 시장의 현재 스냅샷 |
| `open_interest(market)` | `Error::Unsupported` | `/fapi/v1/openInterest`; USD-M 무기한 선물 1건의 미결제약정(open interest) 스냅샷 |
| `aggregate_trades(request)` | 공개 `/api/v3/aggTrades`; `limit: 1..=1000` (`None → 500`); `from_id`부터 조회하거나 `start_time`~`end_time` 포함 범위를 조회하며, 두 방식은 함께 사용할 수 없음 | 공개 `/fapi/v1/aggTrades`; Spot과 같은 집계 체결 타입과 요청 규칙. Binance는 최근 48시간 선물 체결 이력만 보관하고 `start_time`~`end_time` 포함 범위는 1시간 미만이어야 함 |

체결 결과는 최신순입니다. Spot은 거래소 호가 timestamp가 없어
`OrderBook::timestamp = local_read_time`입니다. USD-M은 거래소 timestamp를
유지합니다. 알 수 없는 상장 상태는 `MarketStatus::Unknown`으로 매핑합니다.

## 캔들

| 거래소 | 지원하는 `Interval` |
| --- | --- |
| Spot | `Sec1`, `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour6`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| USD-M | Spot 목록에서 `Sec1` 제외 |

REST와 `Feed::Candles`가 지원하는 간격은 같습니다.

| 제약 | Spot | USD-M |
| --- | ---: | ---: |
| 거래소 페이지 상한 | 1,000 | 1,500 |
| 요청당 거래소 호출 수 | `<= 100` | `<= 100` |
| 사전 계산 캔들 수 | `<= 100_000` | `<= 150_000` |

## 스트림

| `Feed` | 스트림(stream) | 계약 |
| --- | --- | --- |
| `Feed::Trades` (Spot) | `{symbol}@trade` | 체결당 이벤트 1건 |
| `Feed::Trades` (USD-M) | 미지원 | Binance는 집계 체결만 제공하므로 모든 개별 체결을 뜻하는 이 feed로 반환하지 않음 |
| `Feed::OrderBook` | `{symbol}@depth20@100ms` | 전체 스냅샷(snapshot); 각 측 20개 호가 단계; 고정 depth |
| `Feed::Ticker` | `{symbol}@ticker` | 최근 24시간 요약 |
| `Feed::Candles(interval)` | `{symbol}@kline_<interval>` | Binance `closed` 보존 |

이벤트의 `Market`은 `Subscription`에 등록된 거래소 심볼(native symbol) 매핑으로
결정합니다. quote 접미사 목록으로 symbol을 분리하지 않습니다.

USD-M은 `OrderBook`을 `/public/stream`으로, `Ticker`, `Candles`를
`/market/stream`으로 연결합니다. 둘 다 필요하면 반환된 `MarketStream`이 두 소켓을
병합합니다. 재연결 알림은 소켓별입니다. 한 소켓이 종료되면 논리 스트림을 종료하고
다른 소켓도 폐기합니다.

## 비공개 API와 거래소 전용 API

`.with_credentials(api_key, secret_key)`로 인증 정보를 설정합니다. HMAC-SHA-256만
지원하며 RSA와 Ed25519는 지원하지 않습니다.

| 거래소 | 비공개 기능 |
| --- | --- |
| Spot | 잔고, 미체결 주문, 주문 생성·취소, 계좌 스트림 |
| USD-M | Spot 기능, 포지션, 증거금 요약·설정, funding 지급 이력, reduce-only 주문 |

| 주문 입력 | Spot | USD-M |
| --- | --- | --- |
| `Size::Base` | 모든 주문 | 모든 주문 |
| `Size::Quote` | 시장가만 지원; Limit → `Error::InvalidRequest` | `Error::InvalidRequest` |
| `time_in_force` | `GTC`, `IOC`, `FOK`; `PostOnly → LIMIT_MAKER` | `GTC`, `IOC`, `FOK`; `PostOnly → GTX` |
| `OrderType::Best` | `Size::Base` + `IOC` 또는 `FOK`; `LIMIT + MARKET_PEG` | `Error::Unsupported` |
| `client_id` | `[A-Za-z0-9./:_-]`에 해당하는 1–36자 | 동일 |
| `reduce_only == true` | `Error::Unsupported` | 지원 |

취소 메서드는 Binance 응답을 검증한 뒤 `()`를 반환합니다. 최종 체결 상태가
필요하면 주문 조회 API로 확인하세요.

다음 메서드는 `Client::adapter()`가 반환한 어댑터에서 호출합니다.

| 메서드 | 계약 |
| --- | --- |
| `spot_symbol_filters(&market)` | Spot `PRICE_FILTER`, `LOT_SIZE`, `NOTIONAL`; USD-M은 미지원 |
| `spot_order(&market, order_id)` | 숫자 `order_id`로 Spot 주문 1건 조회; 완료 주문 포함 |
| `spot_exchange_info()` | 공개 Spot `GET /api/v3/exchangeInfo`. 상장 symbol 메타데이터와 원본 filter payload를 보존. fixture만 검증 |
| `spot_account_information()` | 수수료·권한·잔고·원본 거래소 필드를 보존하는 서명된 Spot 계정 조회. fixture만 검증 |
| `spot_cancel_all_open_orders(&market)` | 공통 단위 반환 취소와 달리 Binance의 취소 보고서를 반환하는 서명된 Spot 전체 취소. fixture만 검증 |
| `usd_m_exchange_info()` | 공개 USD-M `GET /fapi/v1/exchangeInfo`. 만기 계약을 포함한 contract 상장 정보와 원본 filter payload를 보존. fixture만 검증 |
| `usd_m_account_information()`, `usd_m_position_information(market)` | 공통 잔고·포지션에 담기지 않는 자산·증거금·leverage·위험 필드를 보존하는 서명된 USD-M 계정·포지션 위험 조회. fixture만 검증 |
| `all_coins_information()`, `api_key_permissions()`, `withdraw_address_list()` | 코인·네트워크 규칙, 설정된 권한, 등록 출금 주소 메타데이터를 읽는 서명된 Wallet 조회. fixture만 검증 |
| `deposit_history(request)`, `withdraw_history(request)` | 공통 이체 이력으로 축소하지 않고 거래소 상태·페이지·시각 필드를 보존하는 서명된 Wallet 이력 조회. fixture만 검증 |
| `questionnaire_requirements()` | 서명된 Wallet Travel Rule 설문 요구사항 조회. 자격 조건은 Binance가 적용. fixture만 검증 |
| `account_trades(request)` | 서명된 계정 체결 페이지: Spot `GET /api/v3/myTrades`, USD-M `GET /fapi/v1/userTrades`. 시장 1개를 지정한 `HistoryRequest`를 사용하며 `limit`은 1~1,000(기본 500). 안전한 공통 재개 커서가 없어 `next == None`. fixture만 검증 |
| `c2c_trade_history(request)` | 서명된 Spot/Funding `GET /sapi/v1/c2c/orderMatch/listUserOrderHistory`. `BUY` 또는 `SELL`이 필수이고, page 기본값은 1, rows 기본값은 100이며 rows는 최대 100, 조회 기간은 최대 30일. 공통 페이지 커서로 억지 변환하지 않고 Binance의 선택 C2C 응답 envelope을 보존. fixture만 검증 |
| `test_order(request)` | 서명된 TRADE 검증: Spot `POST /api/v3/order/test`, USD-M `POST /fapi/v1/order/test`; 매칭 엔진에 주문을 제출하지 않음. `BinanceTestOrderRequest::compute_commission_rates`는 Spot에서만 사용 가능하고 USD-M에서는 요청 전 거절. fixture만 검증 |
| `cancel_all_open_orders(&market)` | 시장 1개의 활성 주문을 취소하는 서명된 금전성 쓰기: Spot `DELETE /api/v3/openOrders`, USD-M `DELETE /fapi/v1/allOpenOrders`. 불완전한 주문 목록 대신 거래소별 문서화된 응답 형태를 검증한 `()`를 반환. fixture만 검증 |
| `usd_m_create_listen_key()` | USD-M account listen key 생성 또는 연장 |
| `usd_m_keepalive_listen_key()` | 설정된 API key가 소유한 활성 USD-M listen key 연장 |
| `usd_m_close_listen_key()` | 설정된 API key가 소유한 활성 USD-M listen key 종료 |

USD-M의 `mark_price`, `mark_prices`, `open_interest`는 공개 읽기 전용
메서드입니다. fixture로 검증했으며 실제 읽기 요청(live read)은 아직 검증하지
않았습니다. `mark_prices()` 결과는 maxt가 지원하는 USD-M 무기한 선물 시장으로
제한됩니다.

`aggregate_trades(request)`는 같은 거래소 전용 집계 체결 타입을 반환하는 공개 읽기
전용 Spot·USD-M 메서드입니다. 두 거래소 모두 포함 `from_id` 커서 또는 포함 시간
범위를 사용하되 함께 사용할 수 없습니다. USD-M은 최근 48시간 선물 체결 이력만
보관하고 시간 범위가 1시간 미만이어야 하지만 Spot에는 같은 로컬 보관·시간 범위
제한이 없습니다. 한 번에 한 페이지를 반환하므로 ID로 순회할 때 마지막 aggregate
ID에 1을 더한 값을 다음 `from_id`로 사용하세요. 이 메서드는 fixture로 검증했으며
실제 읽기 요청(live read)은 아직 검증하지 않았습니다.

`subscribe_account`는 USD-M listen key 수명 주기를 관리합니다. Spot은 서명된
`userDataStream.subscribe.signature` 요청을 사용하며 listen key를 사용하지
않습니다.

주문의 `price`, `size`는 symbol filter에 맞춰 반올림하거나 사전 검증하지 않습니다.
Spot margin, COIN-M, option, portfolio margin, quarterly futures, 스트림 `depth` 설정,
diff-depth 복원, `@aggTrade`, testnet 생성자, RSA·Ed25519 key는 노출하지 않습니다.

## 한도와 공식 링크

Binance는 IP 기준 `REQUEST_WEIGHT`를 부과합니다. 현재 한도는 `exchangeInfo`에서,
사용량은 `X-MBX-USED-WEIGHT-1M`에서 확인하세요. `maxt`는 요청 속도를 제한하거나
해당 header를 소비하지 않습니다. HTTP 429와 418은
`Error::is_rate_limited() == true`입니다.

- [Spot REST 시장 데이터](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market)
- [Spot 계정 체결](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account)
- [Spot 계정·거래소 정보](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account)
- [C2C 거래 이력](https://developers.binance.com/en/docs/catalog/investment-and-services-c2-c/api/rest-api/~#get-c2-ctrade-history)
- [Spot 주문 테스트·일괄 취소](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/trade)
- [Spot REST 한도](https://developers.binance.com/en/docs/products/spot/rest-api)
- [Spot WebSocket stream](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)
- [USD-M REST 시장 데이터](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [USD-M 계정 체결·주문 테스트·일괄 취소](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/trade)
- [USD-M 계정 정보](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/account)
- [USD-M Mark Price](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Mark-Price)
- [USD-M Open Interest](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Open-Interest)
- [USD-M 압축/집계 체결](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data#compressed-aggregate-trades-list)
- [USD-M 공개 stream](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/public)
- [USD-M 시장 stream](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/market)
- [Wallet 계정·자본 API](https://developers.binance.com/en/docs/catalog/core-trading-wallet/api/rest-api/account)

[공통 API](../common-api.ko.md) · [거래소 지원](../providers.ko.md)
