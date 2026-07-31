[English](bithumb.md) | [한국어](bithumb.ko.md)

# Bithumb

현물 전용, 거래소 하나, 캔들 스트림 없음.

```rust
use maxt::{Client, Feature, adapters::BithumbAdapter};

let client = Client::new(BithumbAdapter::new());
assert!(client.supports(Feature::Candles));       // REST로는 가능
assert!(!client.supports(Feature::CandleStream)); // 실시간으로는 불가
```

## 지원 범위

마켓 코드는 quote 자산을 앞에 씁니다. `KRW-BTC`처럼요.
`Market::spot(Exchange::Bithumb, "BTC", "KRW")`를 넘기면 됩니다. Bithumb 자체
표기는 `MarketInfo::native_symbol`이 돌려줍니다.

| 호출 | 요건 |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | 없음 |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | 인증 정보 |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | 언제나 `Error::Unsupported`. Bithumb에는 파생상품 상장이 없습니다 |
| 주문의 `reduce_only` | `Error::Unsupported` |
| `markets(MarketKind::Perpetual)` | 오류가 아니라 빈 목록 |

`Unsupported`가 아닌 호출은 [공통 API](../common-api.ko.md) 설명대로 동작합니다.

## 상한

요청을 만들기 전에 검사합니다.

| 호출 | 허용 범위 | 범위 밖 |
| --- | --- | --- |
| `trades` | `limit` 1~500 | `limit` 필드의 `Error::InvalidRequest` |
| `order_book` | 0보다 큰 `depth`, 곧 가장 좋은 N단계. Bithumb 엔드포인트에는 깊이 파라미터가 없어서 `maxt`가 양쪽을 정렬한 뒤 잘라냅니다 | `depth` 0은 `depth` 필드의 `Error::InvalidRequest` |
| Bithumb이 보낸 것보다 큰 `order_book` `depth` | 오류 없이 더 적은 단계. `/v1/orderbook`에는 단계 수가 문서로 밝혀져 있지 않습니다. 2026-07-30 수집분은 한 쪽당 30단계였습니다. `OrderBook`을 읽으세요 | 없음 |
| `candles` | `limit`은 제한 없음. Bithumb 응답당 200개이고 최대 100번의 호출까지 페이지를 대신 넘깁니다 | `limit` 0, `to`보다 이르지 않은 `from`, 캔들 20,000개를 넘는 구간은 `Error::InvalidRequest` |
| `candles` 간격 | [기준선](../common-api.ko.md#간격) 열 개뿐입니다. Bithumb에는 1초 엔드포인트가 없습니다 | `Feature::Candles`를 지목하는 `Error::Unsupported` |
| 매핑되지 않은 간격에 잘못된 `limit`이나 구간이 겹칠 때 | `limit`과 구간을 먼저 검사합니다 | `Unsupported`가 아니라 해당 필드의 `Error::InvalidRequest` |
| 주문 식별자 | 영문자, 숫자, `-`, `.`, `_` | `order_id` 필드의 `Error::InvalidRequest` |

## 스트림

| 대상 | 동작 |
| --- | --- |
| `Feed::Trades`, `Feed::OrderBook`, `Feed::Ticker` | 제공합니다 |
| `Feed::Candles(_)` | `Error::Unsupported`. 소켓이 열리기 전에 구독 전체가 실패합니다. 체결로 캔들을 대신 합성하지 않습니다 |
| `Feed::OrderBook` 깊이 | 한 쪽당 15단계. Bithumb이 [발행하는](https://apidocs.bithumb.com/reference/호가-orderbook.md) 양이 그만큼입니다. 2026-07-30에 40개 마켓에서 받은 모든 프레임이 그랬습니다. `Subscription`으로 늘리거나 줄일 수 없습니다. REST의 30단계와는 무관합니다 |
| 호가창 이벤트 | 차분이 아니라 전체 스냅숏. 이벤트마다 사본을 덮어쓰세요. 첫 `SNAPSHOT` 프레임도 뒤따르는 `REALTIME` 프레임도 15단계를 모두 싣습니다 |
| 호가창 이벤트 시계 | 다른 Bithumb 페이로드와 달리 마이크로초 단위로 옵니다 |
| `subscribe_account` | 마켓 목록이 아니라 계좌 전체, 변경된 자산마다 이벤트 하나. 한 프레임에 여러 잔고가 실려 오기도 합니다 |
| 실시간 차트 | REST `candles`를 읽거나 `Feed::Trades`를 직접 집계하세요 |

## 요청 할당량

| 그룹 | 한도 |
| --- | --- |
| 공개 REST | 초당 150회 |
| 비공개 REST | 초당 140회 |
| 주문, 비공개 수치 위에 추가 | 초당 10회 |
| REST 수치의 적용 범위 | Bithumb이 밝히지 않았습니다. IP 기준으로 다루세요 |
| WebSocket | Bithumb이 밝히지 않았습니다. 그렇다고 한도가 없는 것은 아닙니다 |

`maxt`는 [속도를 조절하지 않습니다](../common-api.ko.md#호출-한도). 너무
빨랐다는 사실은 `Error::is_rate_limited()`로 알게 됩니다. Bithumb은 계속
밀어붙이는 IP를 일시적으로 차단합니다.

## 주문

| 주문 | 크기 | 가격 |
| --- | --- | --- |
| 지정가, 양쪽 모두 | `Size::Base` | 필수 |
| 시장가 매수 | `Size::Quote`, 지출할 금액 | 없음 |
| 시장가 매도 | `Size::Base`, 내놓을 수량 | 없음 |

이 밖의 조합은 `size` 필드의 `Error::InvalidRequest`이고 0 이하의 가격이나
수량도 마찬가지입니다. `TimeInForce`는 무엇을 넣든 `time_in_force` 필드의
`Error::InvalidRequest`입니다. 주문은 해당 유형의 Bithumb 기본 동작을 따릅니다.

## 주문 정밀도와 최소 주문 크기

[노출하지 않습니다](../common-api.ko.md#주문-정밀도와-최소-주문-크기). 가격과
수량이 0보다 큰지만 확인하고 받은 값을 그대로 보내므로 Bithumb의 호가 단위에서
벗어난 가격이나 최소 주문 금액에 못 미치는 주문은 여기서 나오는
`Error::InvalidRequest`가 아니라 Bithumb의 거절로 돌아옵니다.

## 주의할 점

| 필드 또는 호출 | 동작 |
| --- | --- |
| `place_order` 상태 | 체결이 아니라 `OrderStatus::Accepted`. Bithumb이 식별자만 응답합니다. `open_orders`를 읽거나 계좌 스트림을 지켜보세요 |
| 시장가 매수 뒤의 `Order::remaining_quantity` | 0. 접수 응답에 기준 자산 수량이 없습니다 |
| 취소 뒤의 `Order::side` | Bithumb 응답에 방향이 없으면 `Side::Buy`. 아무 뜻도 없는 값입니다 |
| 유의 종목 | `MarketStatus::Unknown`. 거래는 계속됩니다. [유의 종목과 경보제](#유의-종목과-경보제)를 보세요 |
| 주의 종목 | `MarketStatus::Active` 그대로. 마켓 목록에는 실리지 않고 `market_alerts`로 읽습니다 |
| `Trade::id` | Bithumb의 `sequential_id`를 보내온 그대로. REST에서는 체결 밀리초에 1만을 곱한 값이라 같은 밀리초의 체결끼리 식별자가 겹칩니다. 스트림은 체결마다 다른 번호를 보냅니다. 둘을 잇는 키로 쓰지 마세요 |
| `Feed::OrderBook`의 `OrderBook::timestamp` | 마이크로초. Bithumb이 그 프레임 하나에 대해 문서에 적고 실제로 보내는 단위입니다. 다른 Bithumb 시계는 모두 밀리초입니다 |
| `ticker`의 `Ticker::timestamp`와 `Ticker::last_trade_time` | 9시간을 빼서 보정합니다. `/v1/ticker`는 둘 다 UTC 밀리초라고 문서에 적고도 한국 벽시계로 찍습니다. `maxt`는 같은 페이로드의 `trade_date`, `trade_time`에 견주어 차이를 재므로 Bithumb이 필드를 고치면 보정도 저절로 사라집니다. 제3의 값이 오면 `Error::Decode`입니다 |
| REST에서 `Ticker::timestamp`와 `last_trade_time` | 같습니다. `/v1/ticker`는 두 시계에 한 숫자를 보냅니다. `Feed::Ticker`는 둘을 따로 보냅니다 |
| `trades` 순서 | 최신 순이며 여기서 정렬합니다. 안정 정렬이라 같은 밀리초의 체결은 Bithumb 순서를 지킵니다 |
| `candles` 순서 | 오래된 순. Bithumb 자체는 최신 순으로 응답합니다 |
| `candles` 커서 | Bithumb 커서는 시간대 표기가 없는 벽시계 문자열이고 한국 시간으로 읽힙니다. `Timestamp`를 넘기고 UTC로 생각하세요 |
| `Candle::closed` | 캔들 자신의 간격이 끝나면 `true`입니다. 판정 기준은 읽는 쪽 기계의 시계입니다. Bithumb은 형성 중인 캔들을 계속 다시 발행하고 완료 표시를 하지 않습니다 |
| `Month1`의 `Candle::open_time` | 1일이 아니라 직전 UTC 달 마지막 날 15:00 UTC |
| `Interval::Hour4` | 03, 07, 11, 15, 19, 23시 UTC. Bithumb이 4시간 구간을 한국 시간으로 자르는데 9시간은 4의 배수가 아니기 때문입니다. Upbit, Binance, Hyperliquid는 00, 04시 UTC에 엽니다 |
| `Interval::Day1` | 15:00 UTC, 한국의 자정. 일봉이 담는 하루는 한국의 하루입니다 |
| `Interval::Week1` | 일요일 15:00 UTC, 한국의 월요일 자정. Upbit과 Binance의 주봉은 월요일 0시 UTC에 열립니다 |
| `Min1`부터 `Hour1`까지 | 다른 거래소와 같은 UTC 격자. 어긋나는 간격은 `Hour4`와 하루 이상뿐입니다 |
| `open_orders` | Bithumb의 대기 주문 상태 |
| 인증 정보 없음 | 요청을 만들기 전에 `Error::Unsupported`가 아니라 `Error::Auth` |
| Bithumb이 거부한 인증 정보 | `Error::Auth`가 아니라 Bithumb 이름을 담은 `Error::Exchange`. HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_jwt`, `NotAllowIP`, `out_of_scope`입니다. 뒤의 둘을 Upbit은 `no_authorization_ip`로 쓰고 `out_of_scope`를 403에 둡니다. 한 거래소에 맞춘 규칙은 다른 거래소에서 틀립니다. **측정이 아니라 문서 기준입니다** |

Bithumb은 `Month1`을 한국 달력으로 자릅니다. `KRW-BTC`의
`/v1/candles/months`와 맞춰 읽으세요.

| 한국 기준 월 | `open_time` | `closed` 시점 |
| --- | --- | --- |
| 2026년 3월 | `2026-02-28T15:00Z` | `2026-03-31T15:00Z` |
| 2026년 4월 | `2026-03-31T15:00Z` | `2026-04-30T15:00Z` |

열두 달 중 다섯 달이 UTC 기준 월과 어긋납니다. 다른 간격은 모두 길이가 고정이라
영향이 없습니다. Upbit의 월봉은 1일 0시 UTC에 열립니다.

## 유의 종목과 경보제

표시가 둘, 엔드포인트가 둘, 뜻도 둘입니다.

| 표시 | 읽는 곳 | `MarketStatus` | 성격 |
| --- | --- | --- | --- |
| 유의 종목 | `market_warnings()` 또는 `MarketInfo::status` | `Unknown`. Upbit의 유의 종목도 여기로 들어갑니다 | `/v1/market/all?isDetails=true`의 `market_warning` 필드. 사람이 지정해 공지하며 그동안에도 거래는 계속됩니다 |
| 주의 종목 | `market_alerts()`에서만 | `Active` | Bithumb의 경보제. 공표된 기준에 따라 자동으로 오르내리고 기준마다 한 줄씩, 경보 단계와 종료 시점을 함께 싣습니다 |

`CAUTION`이라는 문자열이 양쪽에 다 나오는데 뜻은 서로 다릅니다.

| `CAUTION` | 읽는 곳 | 뜻 |
| --- | --- | --- |
| `market_warning` 필드 | `market_warnings()` | 유의. 이 필드가 갖는 다른 값은 `NONE`뿐입니다 |
| `BithumbMarketAlert::step` | `market_alerts()` | 주의, 경보의 가장 약한 단계 |

2026-07-30 기준 상장 마켓 486개 가운데 15개가 유의 종목, 18개가 경보 하나 이상,
2개가 둘 다였습니다. 경보 쪽 목록은 하루 사이에도 바뀌지만 유의 종목 쪽은 그렇지
않습니다.

`BithumbAlertStep`은 단계 순서대로 비교하므로 "가장 약한 단계보다 위"를 거르는
조건은 `step >= BithumbAlertStep::Warning`입니다.

| 단계 | Bithumb의 표기 | 순위 |
| --- | --- | --- |
| `BithumbAlertStep::Caution` | `CAUTION`, 주의 | 가장 먼저 올라갑니다 |
| `BithumbAlertStep::Warning` | `WARNING`, 경고 | 가운데 단계이자 가장 드뭅니다 |
| `BithumbAlertStep::Danger` | `DANGER`, 위험 | Bithumb이 문서에 적은 가장 무거운 단계이자 실제로 가장 흔합니다 |
| `BithumbAlertStep::Unknown` | 그 밖의 값 | `Danger`보다 위. Bithumb이 나중에 단계를 늘려도 문턱값에 걸립니다 |

`BithumbMarketAlert::kind`는 Bithumb의 경보 기준을 문자열 그대로 담습니다.

- `PRICE_SUDDEN_FLUCTUATION`, 가격 급등락
- `PRICE_DIFFERENCE_HIGH`, 글로벌 시세 차이
- `SPECIFIC_ACCOUNT_HIGH_TRANSACTION`, 소수계정 거래 집중
- `TRADING_VOLUME_SUDDEN_FLUCTUATION`, 거래량 급등
- `DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION`, 입금량 급등

## Bithumb 전용 호출

`Client::adapter()`를 통해 호출합니다.

| 메서드 | 반환값 |
| --- | --- |
| `market_warnings()` | 상장된 모든 마켓과 그 유의 종목 표시를 Bithumb 표기 그대로, 표시가 없으면 `"NONE"`. 마켓 목록 역할도 겸합니다 |
| `market_alerts()` | 올라가 있는 경보마다 `BithumbMarketAlert` 하나씩. 마켓, 기준, 단계, 종료 시점이 들어갑니다. 경보가 없는 마켓은 빠집니다. 여러 기준에 걸린 마켓은 기준 수만큼 나옵니다 |

```rust
use maxt::{Client, adapters::{BithumbAdapter, BithumbAlertStep}};

async fn flagged() -> maxt::Result<()> {
    let client = Client::new(BithumbAdapter::new());
    for (market, label) in client.adapter().market_warnings().await? {
        println!("{market}: {label}");
    }
    for (market, alert) in client.adapter().market_alerts().await? {
        if alert.step >= BithumbAlertStep::Warning {
            println!("{market}: {} until {:?}", alert.kind, alert.ends_at);
        }
    }
    Ok(())
}
```

## 인증 정보

함께 발급되는 access key와 secret key입니다. 이것이 `Feature::Balances`,
`Feature::OpenOrders`, `Feature::Trading`, `Feature::AccountStream`을 엽니다.

```rust
use maxt::{Client, adapters::BithumbAdapter};

fn client() -> Client<BithumbAdapter> {
    let access_key = std::env::var("BITHUMB_ACCESS_KEY").expect("BITHUMB_ACCESS_KEY");
    let secret = std::env::var("BITHUMB_SECRET_KEY").expect("BITHUMB_SECRET_KEY");
    Client::new(BithumbAdapter::new().with_credentials(access_key, secret))
}
```

| 항목 | 규칙 |
| --- | --- |
| 서명 | 비공개 호출마다 JWT 하나. secret key로 HS256 서명하고 access key와 새 nonce, 밀리초 타임스탬프를 담습니다 |
| 파라미터 | SHA-512 해시로 함께 담기므로 질의가 변조되면 서명이 무효가 됩니다 |
| secret key | 프로세스 안에서 서명만 하고 밖으로 나가지 않습니다 |
| 프라이빗 WebSocket | 프레임이 아니라 최초 handshake에서 인증합니다. 토큰은 handshake마다 새로 발급됩니다 |
| 시계 오차 | 토큰에 타임스탬프가 들어가므로 멀쩡한 인증 정보도 깨집니다. 잘 쓰던 키가 실패하기 시작하면 장비 시계부터 확인하세요 |

## 예제

`cargo run --example public_rest`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Bithumb 공식 문서

| 주제 | 문서 |
| --- | --- |
| 할당량 | [호출 한도](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내) |
| 공개 REST | [마켓 목록](https://apidocs.bithumb.com/reference/거래-대상-목록-조회.md) · [경보제](https://apidocs.bithumb.com/reference/경보제-조회.md) · [티커](https://apidocs.bithumb.com/reference/현재가-조회.md) · [호가창](https://apidocs.bithumb.com/reference/호가-조회.md) · [체결](https://apidocs.bithumb.com/reference/체결-내역-조회.md) · [분 캔들](https://apidocs.bithumb.com/reference/분minute-캔들-조회.md) |
| 비공개 REST | [계좌](https://apidocs.bithumb.com/reference/전체-자산-조회.md) · [미체결 주문](https://apidocs.bithumb.com/reference/대기-주문-목록-조회.md) · [주문 취소](https://apidocs.bithumb.com/reference/주문-취소-접수.md) |
| WebSocket | [체결](https://apidocs.bithumb.com/reference/체결-trade.md) · [호가창](https://apidocs.bithumb.com/reference/호가-orderbook.md) · [내 주문](https://apidocs.bithumb.com/reference/내-주문-및-체결-myorder.md) · [내 자산](https://apidocs.bithumb.com/reference/내-자산-myasset.md) |

---

[공통 API](../common-api.ko.md) · [거래소 고르기](../providers.ko.md)
