[English](bithumb.md) | [한국어](bithumb.ko.md)

# Bithumb

현물 전용, 거래소 하나, 캔들 스트림 없음. KRW 페어를 거래하거나 Upbit과 견줄 두
번째 국내 거래소가 필요할 때 고르세요.

```rust
use maxt::{Client, Feature, adapters::BithumbAdapter};

let client = Client::new(BithumbAdapter::new());
assert!(client.supports(Feature::Candles));       // REST로는 가능
assert!(!client.supports(Feature::CandleStream)); // 실시간으로는 불가
```

## 지원 범위

Bithumb의 마켓 코드는 quote 자산을 앞에 써서 `KRW-BTC`가
됩니다. 호출자는 `Market::spot(Exchange::Bithumb, "BTC", "KRW")`를 넘기고 변환은
어댑터가 합니다. Bithumb 화면과 대조할 Bithumb 자체 표기는
`MarketInfo::native_symbol`이 돌려줍니다.

| 호출 | 필요한 것, 또는 동작할 수 없는 이유 |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | 인증 정보 없이 |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | 인증 정보 필요 |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | 언제나 `Error::Unsupported`. Bithumb에는 파생상품 상장이 없습니다 |
| 주문의 `reduce_only` | `Error::Unsupported`. 현물 주문에는 줄일 포지션이 없습니다 |
| `markets(MarketKind::Perpetual)` | 오류가 아니라 빈 목록 |

`Unsupported`가 아닌 것은 [공통 API](../common-api.ko.md)의 설명대로 동작합니다.

## 상한

요청을 만들기 전에 검사합니다.

| 호출 | 허용 범위 | 벗어나면 |
| --- | --- | --- |
| `trades` | `limit` 1~500 | `limit` 필드의 `Error::InvalidRequest` |
| `order_book` | 0보다 큰 `depth`, 곧 가장 좋은 N단계. Bithumb 엔드포인트에는 깊이 파라미터가 없어서 `maxt`가 양쪽을 정렬한 뒤 잘라냅니다 | `depth` 0은 `depth` 필드의 `Error::InvalidRequest` |
| `candles` | `limit`은 제한 없음. Bithumb 응답당 200개이고 최대 100번의 호출까지 페이지를 대신 넘깁니다 | `limit` 0, `to`보다 이르지 않은 `from`, 캔들 20,000개를 넘는 구간은 `Error::InvalidRequest` |
| `candles` 간격 | 1m, 3m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M, 그 밖에는 없음 | `Feature::Candles`를 지목하는 `Error::Unsupported` |
| 주문 식별자 | 영문자, 숫자, `-`, `.`, `_`. 여기서 `maxt`가 직접 만들지 않은 값은 이것뿐이고 식별자 안의 `&`는 이미 서명한 요청에 파라미터를 덧붙입니다 | `order_id` 필드의 `Error::InvalidRequest` |

이 열 간격은 [기준선](../common-api.ko.md#간격)과 정확히 같습니다. Bithumb은
`Interval`이 이름 붙일 수 없는 10분봉도 집계하고 1초 엔드포인트는 아예 발행하지
않습니다. 열 개 밖의 간격은 `maxt`가 엔드포인트를 매핑해 두지 않은 간격으로
거절되며, 실제로도 그렇습니다. 스트림 열이 없는 이유는 어떤 간격에도 캔들
스트림이 없기 때문입니다.

`limit`과 구간 검사는 간격을 찾아보기 전에 돌아갑니다. 매핑되지 않은 간격에
`limit` 0이나 뒤집힌 구간, 페이지 상한을 넘는 범위가 겹치면 `Unsupported`가
아니라 해당 필드를 지목하는 `Error::InvalidRequest`로 보고됩니다. 이 차이로
분기한다면 둘 다 매칭하세요.

`depth`를 Bithumb이 아니라 여기서 적용하므로, Bithumb이 보낸 것보다 많은 단계를
요청하면 오류 없이 더 적은 단계가 나옵니다. `/v1/orderbook`이 몇 단계를
돌려주는지는 Bithumb이 문서로 밝히지 않았습니다. 2026-07-30 수집분은 한 쪽당
30단계로 답했고 이는 소켓이 보내는 양의 두 배입니다. 특정 개수는 계약이 아니라
관찰로 다루고, 깊이를 넘겨짚는 대신 `OrderBook`을 읽으세요.

## 스트림

| 대상 | 동작 |
| --- | --- |
| `Feed::Trades`, `Feed::OrderBook`, `Feed::Ticker` | 제공합니다 |
| `Feed::Candles(_)` | `Error::Unsupported`. 소켓이 열리기도 전에 구독 전체가 실패합니다. 피드 목록에서 조용히 빠지지 않고 체결로 캔들을 대신 합성하지도 않습니다 |
| `Feed::OrderBook` 깊이 | 한 쪽당 15단계이며 Bithumb이 [발행하는](https://apidocs.bithumb.com/reference/호가-orderbook.md) 양이 그만큼이고, 2026-07-30에 40개 마켓에서 받은 모든 프레임이 그 개수였습니다. `Subscription`으로 더 달라거나 덜 달라고 할 수 없습니다 |
| 호가창 이벤트 | 차분이 아니라 전체 스냅숏. 이벤트마다 사본을 덮어쓰세요. 처음의 `SNAPSHOT` 프레임도 뒤따르는 `REALTIME` 프레임도 15단계를 모두 싣습니다 |
| 호가창 이벤트 시계 | 다른 Bithumb 페이로드와 달리 전송 단위가 마이크로초입니다. [주의할 점](#주의할-점)의 시계 항목을 보세요 |
| `subscribe_account` | 마켓 목록이 아니라 계좌 전체, 변경된 자산마다 이벤트 하나. Bithumb은 여러 잔고를 한 프레임에 실어 보냅니다 |

실시간 차트를 만드는 길은 둘입니다. 주기적으로 REST에서 `candles`를 읽거나
`Feed::Trades`를 구독해 직접 집계하세요. 스트림의 15단계와 REST의 30단계는 서로
무관한 두 엔드포인트에서 나온 별개의 수치입니다. 한쪽을 다른 쪽에 맞춰 가늠하지
마세요.

## 요청 할당량

Bithumb은 초당 요청 횟수로 셉니다.

| 그룹 | 한도 |
| --- | --- |
| 공개 REST | 초당 150회 |
| 비공개 REST | 초당 140회 |
| 주문, 비공개 수치 위에 추가 | 초당 10회 |

Bithumb은 REST 수치가 IP 기준인지 계좌 기준인지도, WebSocket 한도가 얼마인지도
밝히지 않습니다. REST 수치는 더 엄격한 해석인 IP 기준으로 다루고, 숫자가
공개되지 않았다는 이유로 소켓에 한도가 없다고 넘겨짚지 마세요. `maxt`는 속도를
조절하지 않습니다. 호출하면 그대로 나가고 너무 빨랐다는 사실은
`Error::is_rate_limited()`로 알게 되며 Bithumb은 계속 밀어붙이는 IP를 일시적으로
차단합니다.

## 주문

Bithumb은 시장가와 지정가의 구분을 주문 유형과 가격·수량 중 무엇이 있는지로 함께
표현합니다. 존재하는 조합은 셋이고 `maxt`는 나머지를 서명 전에 거절합니다.

| 주문 | 크기 | 가격 |
| --- | --- | --- |
| 지정가, 양쪽 모두 | `Size::Base` | 필수 |
| 시장가 매수 | `Size::Quote`, 지출할 금액 | 없음 |
| 시장가 매도 | `Size::Base`, 내놓을 수량 | 없음 |

`Size::Base`로 크기를 잰 시장가 매수나 `Size::Quote`로 크기를 잰 지정가 주문은
`size` 필드의 `Error::InvalidRequest`이고 0 이하의 가격이나 수량도 마찬가지입니다.

어떤 `TimeInForce`든 `time_in_force` 필드의 `Error::InvalidRequest`로
실패합니다. Bithumb의 주문 엔드포인트가 이를 받아들이는지 확인되지 않았고
주문 파라미터의 철자를 잘못 추측하면 주문이 잘못 나갑니다. 이 어댑터로 낸 주문은
해당 유형의 Bithumb 기본 동작을 따릅니다.

## 주문 정밀도와 최소 주문 크기

`maxt`는 둘 다 노출하지 않고 이 어댑터의 어느 것도 그 질문에 답하지 않습니다.
가격과 수량이 0보다 큰지만 확인하고 받은 값을 그대로 보내므로, Bithumb의 호가
단위에서 벗어난 가격이나 최소 주문 금액에 못 미치는 주문은 여기서 나오는
`Error::InvalidRequest`가 아니라 Bithumb의 거절로 돌아옵니다. 첫 주문을 내기
전에 Bithumb의 주문 문서를 읽으세요.

## 주의할 점

| 필드 또는 호출 | 예상할 것 |
| --- | --- |
| `place_order` 상태 | `OrderStatus::Accepted`이며 체결은 아닙니다. Bithumb이 식별자만 응답하기 때문입니다. `open_orders`를 읽거나 계좌 스트림을 지켜보세요 |
| 시장가 매수 뒤의 `Order::remaining_quantity` | 0. 주문 크기가 KRW로 매겨지고 접수 응답에 기준 자산 수량이 없습니다 |
| 취소 뒤의 `Order::side` | Bithumb 응답에 방향이 없으면 `Side::Buy`. 그 값에 의미를 두지 마세요 |
| 투자 유의 표시, 유의 종목 | `MarketStatus::Unknown`. 그런 마켓도 거래는 계속되므로 `Paused`가 아니고 그렇다고 문제없는 상태도 아니므로 `Active`도 아닙니다. 아래 [유의 종목과 경보제](#유의-종목과-경보제)를 보세요 |
| 투자 주의 표시, 주의 종목 | `MarketStatus::Active` 그대로. 마켓 목록에는 실리지 않는 다른 엔드포인트에 있으며 `market_alerts`로 읽습니다 |
| `Trade::id` | Bithumb의 `sequential_id`, 보내온 숫자 그대로. REST에서는 이 값이 체결 밀리초에 1만을 곱한 수라서 같은 밀리초를 공유하는 체결은 식별자도 같습니다. 스트림은 체결마다 다른 번호를 보냅니다. 둘 사이를 잇는 키로 쓰지 마세요 |
| `Feed::OrderBook`의 `OrderBook::timestamp` | 마이크로초로 읽습니다. 그 프레임 하나에 대해 Bithumb이 문서에 적고 실제로 보내는 단위가 그것입니다. 다른 Bithumb 시계는 모두 밀리초입니다 |
| `ticker`의 `Ticker::timestamp`와 `Ticker::last_trade_time` | 아홉 시간을 되돌립니다. `/v1/ticker`는 둘 다 UTC 밀리초라고 문서에 적고도 둘 다 한국 벽시계로 찍습니다. `maxt`는 그 차이를 넘겨짚지 않고 같은 페이로드의 `trade_date`와 `trade_time`에 견주어 잽니다. 그래서 Bithumb이 필드를 고치면 보정도 저절로 사라지고, 제3의 값이 오면 `Error::Decode`입니다 |
| REST에서 `Ticker::timestamp`와 `last_trade_time`의 관계 | 같습니다. `/v1/ticker`는 두 시계를 문서에 적어 두고도 한 숫자를 보냅니다. `Feed::Ticker`는 둘을 따로 보냅니다 |
| `trades` 순서 | 최신 순이며 여기서 정렬합니다. 안정 정렬이므로 같은 밀리초를 공유하는 체결은 Bithumb 자신의 순서를 지킵니다 |
| `candles` 순서 | 오래된 순. Bithumb 자체는 최신 순으로 응답합니다 |
| `candles` 커서 | Bithumb 자체 커서는 시간대 표기가 없는 벽시계 문자열이고 한국 시간으로 읽힙니다. `Timestamp`를 넘기고 UTC로 생각하세요 |
| `Candle::closed` | 시계를 읽어서 정합니다. Bithumb은 형성 중인 캔들을 계속 다시 발행하고 완료 표시를 하지 않기 때문입니다. 캔들 자신의 간격이 끝나면 `true`입니다. 캔들 스트림이 없으므로 그 시계는 언제나 읽는 쪽 기계의 시계입니다 |
| `Month1`의 `Candle::open_time` | 1일이 아니라 직전 UTC 달 마지막 날 15:00 UTC. Bithumb이 월을 한국 시간으로 자르고 `open_time`은 같은 시점을 UTC로 적은 값입니다 |
| `Interval::Hour4` | 03, 07, 11, 15, 19, 23시 UTC입니다. Bithumb이 4시간 창을 한국 시간으로 자르는데 9시간은 4의 배수가 아니기 때문입니다. Upbit, Binance, Hyperliquid는 모두 00, 04시 UTC에 엽니다 |
| `Interval::Day1` | 15:00 UTC, 한국의 자정입니다. 여기서 일봉은 UTC 하루가 아니라 한국의 하루를 담습니다 |
| `Interval::Week1` | 일요일 15:00 UTC, 한국의 월요일 자정입니다. Upbit과 Binance의 주봉은 월요일 0시 UTC에 열립니다 |
| 9시간을 나누어떨어지게 하는 모든 간격 | `Min1`부터 `Hour1`까지는 나머지 셋과 같은 UTC 격자에 놓입니다. 한국이 정수 시간만큼 앞서 있기 때문입니다. 어긋나는 것은 `Hour4`와 하루 이상의 간격뿐입니다 |
| `open_orders` | Bithumb의 대기 주문 상태를 읽습니다. 공통 API에서 "미체결"이 뜻하는 바가 그것입니다 |
| 인증 정보 없음 | 요청을 만들기 전에 `Error::Unsupported`가 아니라 `Error::Auth` |
| Bithumb이 거부한 인증 정보 | `Error::Auth`가 아니라 Bithumb 자신의 이름을 담은 `Error::Exchange`. Bithumb이 공개한 표에는 HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_jwt`, `NotAllowIP`, `out_of_scope`가 있습니다. 뒤의 둘은 Upbit과 다릅니다. Upbit은 `no_authorization_ip`로 쓰고 `out_of_scope`를 403에 둡니다. 한 거래소에 맞춘 규칙이 다른 거래소에서 틀리는 이유입니다. **측정이 아니라 문서 기준입니다.** 이 크레이트 검증에 쓸 Bithumb 키가 없었습니다 |

`Month1`은 한국 달력으로 확정됩니다. Bithumb이 월을 자르는 자리가 거기이기
때문입니다. Bithumb 자신의 `KRW-BTC` `/v1/candles/months` 응답과 맞춰 읽으세요.

| 한국 기준 월 | `open_time` | `closed`가 `true`가 되는 시점 |
| --- | --- | --- |
| 2026년 3월 | `2026-02-28T15:00Z` | `2026-03-31T15:00Z` |
| 2026년 4월 | `2026-03-31T15:00Z` | `2026-04-30T15:00Z` |

첫 줄의 `open_time`에서 UTC로 한 달을 더하면 3월 28일이고, 이는 봉이 멈추기
사흘 전입니다. 그래서 이 걸음은 한국 시간에서 일어납니다. 열두 달 중 다섯 달이
그렇게 어긋납니다. 다른 간격은 모두 고정 길이라 영향이 없고, Upbit의 월봉은 1일
0시 UTC에 열리므로 두 한국 거래 시장은 같은 달을 같은 시각으로 적지 않습니다.

`Auth`와 `Unsupported`는 바꿔 쓸 수 없습니다. 앞의 것은 키를 넣으면 해결되지만
뒤의 것은 `maxt`가 그곳에 매핑한 엔드포인트가 없어서 해결되지 않습니다. 예외를
잡기보다 분기하고 싶다면 `client.supports(...)`를 먼저 물어보세요.

## 유의 종목과 경보제

**Bithumb은 두 가지 표시를 서로 다른 엔드포인트로 발행하며 둘은 같은 뜻이
아닙니다.** `MarketStatus`에는 둘 사이에 값이 하나뿐이라 앞의 것만 거기에
닿습니다.

| 표시 | 무엇인가 | `MarketStatus` | 어디서 읽는가 |
| --- | --- | --- | --- |
| 유의 종목 | 사람이 지정하고 공지하며 그동안에도 마켓은 계속 거래됩니다. `/v1/market/all?isDetails=true`가 `market_warning`으로 알려 줍니다 | `Unknown` | `MarketInfo::status` 또는 `market_warnings` |
| 주의 종목 | Bithumb의 경보제입니다. 공표된 기준에 따라 자동으로 올라가고 내려가며 기준마다 한 줄씩, 각각 경보 단계와 종료 시점을 함께 실어 보냅니다 | `Active` | `market_alerts`로만 |

**`market_warning`의 값은 `CAUTION`으로 적히지만 뜻은 유의입니다.** Bithumb은 이
필드를 유의 종목 여부라고 문서에 적고 주의 종목은 다른 곳을 보라고 안내하므로,
오해를 부르는 것은 철자뿐입니다. 이 enum이 갖는 다른 값은 `NONE` 하나입니다.
이 표시가 Upbit이 유의 종목이라 부르는 것과 같은 개념이라서, 두 한국 거래 시장
어댑터는 같은 개념을 `Unknown`에 둡니다.

2026-07-30 기준 Bithumb에 상장된 486개 마켓 가운데 15개가 유의 종목이었고 18개가
경보를 하나 이상 달고 있었으며 둘 다인 것이 2개였습니다. 경보까지 `Unknown`으로
보고하면 그 15개가 묻히고, 경보 쪽 목록은 하루 동안 계속 바뀌는 반면 유의 종목
쪽은 그렇지 않습니다.

**경보 단계에는 순서가 있고 `BithumbAlertStep`은 그 순서대로 비교됩니다.**
그래서 "가장 약한 단계보다 위"를 거르는 조건은
`step >= BithumbAlertStep::Warning`입니다.

| 단계 | Bithumb의 표기 | 무엇인가 |
| --- | --- | --- |
| `BithumbAlertStep::Caution` | `CAUTION`, 주의 | 가장 먼저 올라가는 단계 |
| `BithumbAlertStep::Warning` | `WARNING`, 경고 | 가운데 단계이며 가장 드뭅니다 |
| `BithumbAlertStep::Danger` | `DANGER`, 위험 | Bithumb이 문서에 적은 가장 무거운 단계이자 실제로 가장 흔한 단계 |
| `BithumbAlertStep::Unknown` | 그 밖의 값 | `Danger`보다 위에 놓입니다. Bithumb이 나중에 단계를 늘려도 문턱값이 그것을 그냥 통과시키지 않고 드러내도록 |

`BithumbAlertStep::Caution`은 주의, 경보의 가장 약한 단계입니다.
`market_warnings`가 돌려주는 `CAUTION`은 유의입니다. 겹치는 것은 철자이고 표시
자체는 다릅니다.

`BithumbMarketAlert::kind`는 Bithumb의 경보 유형을 그 표기 그대로 담습니다.

- `PRICE_SUDDEN_FLUCTUATION`, 가격 급등락
- `PRICE_DIFFERENCE_HIGH`, 글로벌 시세 차이
- `SPECIFIC_ACCOUNT_HIGH_TRANSACTION`, 소수계정 거래 집중
- `TRADING_VOLUME_SUDDEN_FLUCTUATION`, 거래량 급등
- `DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION`, 입금량 급등

목록이 비어 있는지가 아니라 실제로 대응하는 유형을 짚어 분기하세요. `kind`가
enum이 아니라 문자열인 것은 거래소가 늘리는 쪽이 기준 목록이기 때문입니다.

## Bithumb 전용 호출

`Client::adapter()`를 통해 호출합니다.

| 메서드 | 돌려주는 것 | 공통이 될 수 없는 이유 |
| --- | --- | --- |
| `market_warnings()` | 상장된 모든 마켓과 그 유의 종목 표시를 Bithumb이 쓰는 표기 그대로, 표시가 없으면 `"NONE"` | `MarketStatus`에는 "거래 중이지만 표시됨"에 해당하는 값이 없습니다 |
| `market_alerts()` | 올라가 있는 경보마다 `BithumbMarketAlert` 하나씩, 마켓과 경보 유형과 단계와 종료 시점 | `MarketStatus`에는 단계도 유형도 종료 시점도 없고, 주의 종목은 `Active`를 움직이지 않습니다 |

Bithumb은 투자자가 조심하기를 바라는 마켓에 표시를 붙이면서도 거래는 그대로 열어
둡니다. `market_warnings()`는 모든 마켓이 돌아오므로 표시를 기준으로 거르는
경우에는 마켓 목록 역할도 겸합니다. `market_alerts()`는 그렇지 않습니다. 경보가
없는 마켓은 아예 빠지고, 여러 기준에 걸린 마켓은 기준 수만큼 나옵니다.

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
공개 시세에는 아무것도 필요하지 않습니다.

```rust
use maxt::{Client, adapters::BithumbAdapter};

fn client() -> Client<BithumbAdapter> {
    let access_key = std::env::var("BITHUMB_ACCESS_KEY").expect("BITHUMB_ACCESS_KEY");
    let secret = std::env::var("BITHUMB_SECRET_KEY").expect("BITHUMB_SECRET_KEY");
    Client::new(BithumbAdapter::new().with_credentials(access_key, secret))
}
```

프라이빗 호출은 모두 secret key로 HS256 서명한 JWT를 함께 보내며 토큰에는 access
key와 새로 만든 nonce, 밀리초 단위 타임스탬프가 담깁니다. 파라미터가 있는 호출은
그 SHA-512 해시까지 담으므로 질의가 변조되면 서명이 무효가 됩니다. secret key는
프로세스 안에서 서명할 뿐 밖으로 나가지 않습니다. 프라이빗 WebSocket은 프레임이
아니라 최초 handshake에서 인증하고 토큰은 handshake마다 새로 발급되므로, 오래
붙어 있는 스트림도 나이 먹은 토큰을 다시 쓰지 않습니다.

**토큰이 타임스탬프를 주장하는 만큼, 시계가 어긋나면 멀쩡한 인증 정보도
깨집니다.** 잘 쓰던 키가 실패하기 시작하면 장비의 시계부터 확인하세요.

키는 소스에 두지 말고 프로그램이 실제로 쓰는 만큼만 권한을 주세요.

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
