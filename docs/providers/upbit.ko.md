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
| `order_book(market, depth)` | `/v1/orderbook` | 공통 API의 묶지 않은 호가; `depth: 1..=30`; 각 측 `len() <= depth`; `None → 30` |
| `ticker(market)` | `/v1/ticker` | 시장 스냅샷(snapshot) 1건 |

파생상품 메서드는 `Error::Unsupported`를 반환합니다.

## 캔들

| API | 지원하는 `Interval` | 공통 `Interval`로 제공하지 않는 간격 |
| --- | --- | --- |
| REST | `Sec1`, `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1` | `1y` |
| WebSocket | `Sec1`, `Min1`, `Min3`, `Min5`, `Min10`, `Min15`, `Min30`, `Hour1`, `Hour4` | — |

연간 캔들은 `year_candles(market, to, count)`로 제공합니다. 공통 `Interval`을
확장하지 않고 `UpbitYearCandle`을 반환하는 Upbit 전용 API입니다.

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
지역은 `UpbitAdapter::region()`과 같아야 합니다. 설정 후 잔고, 주문 단건·이력
조회, 주문 생성·취소, 계좌 스트림을 사용할 수 있습니다.

| 공통 호출 | 엔드포인트(endpoint) | 계약 |
| --- | --- | --- |
| `order_rules(market)` | `GET /v1/orders/chance` | 수수료, 지원 주문 방향·유형·유효 조건(TIF), 호가 자산(quote)·기초 자산(base)의 잔고와 평균 매수가, 호가 자산 기준 주문 한도; deprecated 필드는 제외 |
| `open_orders*` | `GET /v1/orders/open` | 한 페이지, 최대 100건 |
| `order(market, order_id)` | `GET /v1/order?uuid=...` | 응답 시장이 요청 시장과 같은지 검증 |
| `order_by_client_id(market, client_id)` | `GET /v1/order?identifier=...` | 응답 시장이 요청 시장과 같은지 검증 |
| `orders_by_ids(request)` | `GET /v1/orders/uuids` | UUID 또는 사용자 지정 ID 중 한 종류를 1~100개 조회; 시장 필터 선택; 최신순 |
| `order_history(request)` | `GET /v1/orders/closed` | `limit: 1..=1_000`; 최대 7일; 최신순; 커서가 없어 `next == None` |
| `cancel_orders(request)` | `DELETE /v1/orders/uuids` | UUID 또는 사용자 지정 ID 중 한 종류를 1~20개 취소; 일부 실패도 결과에 포함 |

| 주문 입력 | 계약 |
| --- | --- |
| 최유리 매수 | `Size::Quote`와 `IOC` 또는 `FOK` |
| 최유리 매도 | `Size::Base`와 `IOC` 또는 `FOK` |
| `client_id` | RFC 3986 비예약 ASCII 문자로 구성한 1–64바이트; `cancel_order_by_client_id`에 사용 가능 |
| 취소 메서드 | 거래소 응답을 검증한 뒤 `()` 반환 |

공통 `Order`는 정규화한 필드만 제공합니다. Upbit 전용 체결·수수료·잠금 수량·자전거래
방지·유효 조건 필드가 필요하면 `order_detail(request)`를 사용하세요.

공통 `order_history(request)`는 정규화한 주문 이력 API로 그대로 유지합니다.
`closed_orders(request)`는 이를 보완하는 Upbit 전용 API로, 수수료·자전거래 방지(SMP)·
`identifier`·유효 조건(time-in-force) 등 공식 종료 주문 요약 필드를 보존하지만 체결 목록
(`trades`)은 반환하지 않습니다. 선택 `market`, `state`와 `states[]`는
서로 함께 쓸 수 없고, 주문 생성 시각 조회 구간은 최대 7일, `limit`은 최대 1,000이며
오래된 순서 또는 최신순 정렬을 선택할 수 있습니다. 입력 `Timestamp`는 공통 이력의 종료
시각 배타(exclusive end) 적응과 달리 Upbit에 밀리초로 직접 전달합니다. 공식 endpoint
문서는 시간 경계의 포함·배제를 명시하지 않으므로 이 API도 경계 의미를 추가로 단정하지
않습니다. fixture로만 검증했으며 maxt는 실제 거래 또는 읽기 요청을 수행하지 않았습니다.

다음 메서드는 `Client::adapter()`가 반환한 어댑터에서 호출합니다.

| 메서드 | 계약 | 요청 한도 그룹 |
| --- | --- | --- |
| `tickers(&[Market])` | `markets.len() >= 1`; 시장당 ticker 1건 | `ticker` |
| `tickers_by_quote(&[String])` | 견적 통화 1개 이상; 대문자로 정규화; 해당 통화의 ticker 스냅샷 전체 반환 | `ticker` |
| `order_books(&[Market], depth)` | `markets.len() >= 1`; `depth: 1..=30` 또는 `None` | `orderbook` |
| `order_books_at_level(&[Market], Decimal, depth)` | Upbit 한국만 지원; `level >= 0`; 0이 아닌 값은 현재 `supported_levels` 메타데이터를 확인한 뒤 요청 | `orderbook` |
| `orderbook_instruments(&[Market])` | `markets.len() >= 1`; 현재 가격 구간의 호가 단위와 지원하는 묶음 단위 반환; 지역 응답에 없으면 묶음 단위는 빈 배열 | `orderbook` |
| `year_candles(market, to, count)` | `count: 1..=200` 또는 `None`; ISO-8601 기준 시각 선택; 오래된 순서; 한국 시작 시각은 지역에 따라 없을 수 있음 | `candle` |
| `market_events()` | 시장별 투자 유의 여부와 기준 | `market` |
| `list_subscriptions(subscription)` | 임시 공개 연결에서 `LIST_SUBSCRIPTIONS`가 반환한 ticket·데이터 유형·시장·선택 호가 묶음 단위를 보존. fixture만 검증 | WebSocket request |
| `test_order(request)` | 주문을 생성하지 않고 검증; 주문 생성 권한 필요; 반환 `Order`는 dry-run 결과이므로 ID를 조회·취소할 수 없고 상태도 실제 활성 주문을 뜻하지 않음 | `order-test` |
| `order_detail(request)` | 인증 필요 거래소 전용 `GET /v1/order`. 요청에는 예상 `market`과 UUID 및/또는 `identifier`를 지정하며 식별자 하나는 필수이고 둘 다 있으면 Upbit가 UUID를 우선합니다. 공통 `Order`가 담지 않는 상세 체결·수수료·잠금 수량·자전거래 방지(SMP)·유효 조건 원본 필드를 보존합니다. `identifier`의 예약 문자는 안전하게 percent-encoding하고 JWT query hash에는 원래 질의 문자열을 사용합니다. fixture만 검증 | `default` |
| `closed_orders(request)` | 인증 필요 거래소 전용 `GET /v1/orders/closed` 종료 주문 요약 목록. 선택 `market`, `state`, `states[]`를 받으며 `state`와 `states[]`는 함께 쓸 수 없음. 주문 생성 시각 조회 구간은 최대 7일, `limit: 1..=1_000`, 오래된 순서 또는 최신순 정렬. Timestamp는 밀리초로 직접 전달. 수수료·SMP·`identifier`·유효 조건을 보존하지만 개별 `trades`는 없음. fixture만 검증했고 실제 거래·읽기 요청은 하지 않음 | `default` |
| `deposit_info(asset, network)` | `View Deposits` 권한 필요; 입금 가능 여부·사유·최소 수량·확인 수·소수 자릿수를 반환. 응답 네트워크는 null일 수 있으며 받은 값을 그대로 보존. 이 메타데이터는 몇 분 지연될 수 있음 | `default` |
| `withdrawal_addresses()` | 인증 필요 `GET /v1/withdraws/coin_addresses` 거래소 전용 조회. 이 계정에 등록한 주소·네트워크·수취인·지갑 메타데이터를 보존. fixture만 검증 | `default` |
| `travel_rule_vasps()` | `View Deposits` 권한 필요. 한국 또는 싱가포르에서 계정주 확인이 가능한 VASP 목록을 조회. 인도네시아·태국은 인증이나 네트워크 요청 전에 `Error::Unsupported` 반환 | `default` |
| `verify_travel_rule_by_uuid(...)`, `verify_travel_rule_by_txid(...)` | 한국 또는 싱가포르 전용 금전성 쓰기. 입금 계정주 확인을 요청하며 동일 입금 건의 재요청 제한은 Upbit가 적용. fixture만 검증했고 maxt가 실제 검증 요청을 보내지는 않음 | `default` |
| `batch_cancel_open_orders(request)` | 주문 생성 권한 필요. `UpbitBatchCancelScope::All`은 전체 마켓 범위를 명시적으로 선택하며, 무제한 주문 취소를 뜻하지 않음. Upbit가 요청 수량을 적용해 기본 20개·최대 300개의 일치하는 `wait` 주문만 취소. 견적 통화와 시장 쌍 범위 중 하나를 선택하고, 제외 시장은 최대 20개 지정. 성공·실패를 모두 결과에 보존. fixture 검증만 했고 maxt가 실제 취소를 실행하지는 않음 | `order-cancel-all` |
| `cancel_and_new_order(request)` | 주문 생성 권한 필요; JSON `POST /v1/orders/cancel_and_new`만 사용. 새 주문은 기존 주문과 같은 시장·매수/매도 방향을 유지하며 주문 유형·수량·가격·유효 조건(TIF)·자전거래 방지(SMP)를 바꿀 수 있음. `post_only`와 SMP는 함께 사용할 수 없음. HTTP 요청이 성공해도 새 주문 생성을 보장하지 않음. 취소 완료 전에 기존 주문이 체결되면 `new_order_uuid`가 없음. fixture 검증만 했고 maxt가 실제 주문을 제출하지는 않음 | `order` |
| `deposit_krw(request)` | Upbit 한국 전용 금전성 `POST /v1/deposits/krw`. `UpbitKrwTransferRequest`에는 양수 금액과 `Kakao`, `Naver`, `Hana` 중 하나의 2차 인증 수단이 필요. 등록 입출금 계좌와 2차 인증은 Upbit가 검증. fixture만 검증했고 maxt가 실제 이체를 실행하지는 않음 | `default` |
| `withdraw_krw(request)` | Upbit 한국 전용 금전성 `POST /v1/withdraws/krw`; 요청 형식은 동일. 출금 권한 API key라도 Upbit 출금 안심차단이 활성화되어 있으면 거절될 수 있음. fixture만 검증했고 maxt가 실제 이체를 실행하지는 않음 | `default` |
| `api_keys()` | Upbit 한국 전용 인증 필요 `GET /v1/api_keys`. access key 식별자와 만료 시각만 반환하며 secret key는 반환하지 않음. fixture만 검증 | `default` |
| `list_pockets()` | Upbit 한국 전용 인증 필요 `GET /v1/pockets`. 설정한 API key가 볼 수 있는 포켓 목록을 반환. fixture만 검증 | `default` |
| `list_pocket_api_keys(request)`, `sub_pocket_balances(pocket_uuid)` | Upbit 한국 전용 인증 필요 읽기. 포켓 API key 그룹과 서브포켓 한 개의 잔고를 조회하며, 해당 Upbit 포켓 관리 권한이 필요. fixture만 검증 | `default` |
| `universal_transfer(request)`, `sub_pocket_transfer(request)` | Upbit 한국 전용 금전성 포켓 이체. 현재 두 요청 계약은 모두 `to`가 필수이고 universal transfer에는 선택 `from`도 지정 가능. fixture만 검증했고 maxt가 실제 이체를 실행하지는 않음 | `default` |
| `universal_transfers(query)`, `sub_pocket_transfers(query)` | Upbit 한국 전용 포켓 이체 이력. UUID·식별자 목록은 각각 최대 20개, 조회 기간은 최대 7일, `limit`은 `1..=100`. fixture만 검증 | `default` |

| 시장 이벤트(market event) | 매핑 |
| --- | --- |
| `warning == true` | `MarketStatus::Unknown` |
| `cautions`가 비어 있지 않음 | `MarketStatus` 변경 없음 |
| `region != UpbitRegion::Korea` | `UpbitMarketEvent::cautions == []` |

`UpbitOrderBookInstrument::tick_size`는 시장 전체에 고정된 값이 아니라 현재 가격
구간의 메타데이터입니다. 주문 가격이 Upbit 가격 구간을 넘으면 다시 조회해야 합니다.

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
- [연간 캔들](https://docs.upbit.com/kr/reference/list-candles-years)
- [견적 통화별 ticker](https://docs.upbit.com/kr/reference/list-quote-tickers)
- [호가 정책](https://docs.upbit.com/kr/reference/list-orderbook-instruments)
- [WebSocket](https://global-docs.upbit.com/reference/websocket-guide)
- [요청 한도](https://global-docs.upbit.com/reference/rate-limits)
- [인증](https://global-docs.upbit.com/reference/auth)
- [테스트 주문](https://global-docs.upbit.com/reference/order-test)
- [조건부 일괄 주문 취소](https://global-docs.upbit.com/reference/batch-cancel-orders)
- [주문 취소 후 재주문](https://global-docs.upbit.com/reference/cancel-and-new-order)
- [입금 가능 정보](https://global-docs.upbit.com/reference/available-deposit-information)
- [등록 출금 주소](https://docs.upbit.com/kr/reference/list-withdrawal-addresses)
- [원화 입금](https://docs.upbit.com/kr/reference/deposit-krw)
- [원화 출금](https://docs.upbit.com/kr/reference/withdraw-krw)
- [API Key 목록](https://docs.upbit.com/kr/reference/list-api-keys)
- [포켓 목록](https://docs.upbit.com/kr/reference/list-pockets)
- [포켓 API Key 목록](https://docs.upbit.com/kr/reference/list-pocket-api-keys)
- [서브포켓 잔고](https://docs.upbit.com/kr/reference/get-sub-pocket-balance)
- [유니버설 포켓 이체](https://docs.upbit.com/kr/reference/universal-transfer)
- [유니버설 포켓 이체 이력](https://docs.upbit.com/kr/reference/list-universal-transfers)
- [서브포켓 이체](https://docs.upbit.com/kr/reference/transfer)
- [서브포켓 이체 이력](https://docs.upbit.com/kr/reference/list-transfers)
- [한국 Travel Rule 지원 거래소](https://docs.upbit.com/kr/reference/list-travelrule-vasps)
- [싱가포르 Travel Rule 지원 거래소](https://global-docs.upbit.com/reference/list-travelrule-vasps)
- [주문 단건 조회와 거래소 전용 상세](https://global-docs.upbit.com/reference/get-order)
- [종료 주문 목록 — 한국](https://docs.upbit.com/kr/reference/list-closed-orders)
- [종료 주문 목록 — Global](https://global-docs.upbit.com/reference/list-closed-orders)

[공통 API](../common-api.ko.md) · [거래소 지원](../providers.ko.md)
