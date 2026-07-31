[English](upbit.md) | [한국어](upbit.ko.md)

# Upbit

현물만 다루고 지역별 거래소가 넷입니다. 어댑터 하나가 생성할 때 고른 한 곳을
맡습니다.

```rust
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let korea = Client::new(UpbitAdapter::new());
let singapore = Client::new(UpbitAdapter::with_region(UpbitRegion::Singapore));
```

## 지원 범위

기본값은 `UpbitRegion::Korea`이고 `Singapore`, `Indonesia`, `Thailand`가
나머지입니다. 넷은 서로의 복제본이 아니라 별개의 거래소라서 상장도 인증 정보도
지역을 넘어가지 않습니다. 향하는 지역은 `UpbitAdapter::region()`이 알려 줍니다.

Upbit의 마켓 코드는 quote 자산을 앞에 써서 `KRW-BTC`가 됩니다.
`Market::spot(Exchange::Upbit, "BTC", "KRW")`를 넘기세요. Upbit 자체 표기는
`MarketInfo::native_symbol`이 돌려줍니다.

| 호출 | 조건 |
| --- | --- |
| `markets`, `trades`, `order_book`, `ticker`, `candles`, `subscribe`, `subscribe_with` | 인증 정보 없이 |
| `balances`, `open_orders`, `open_orders_on`, `place_order`, `cancel_order`, `subscribe_account`, `subscribe_account_with` | 인증 정보 필요 |
| `positions`, `positions_on`, `margin_summary`, `funding_rates`, `funding_payments`, `set_margin` | 언제나 `Error::Unsupported`. Upbit에는 파생상품 상장이 없습니다 |
| 주문의 `reduce_only` | `Error::Unsupported`. 현물 주문에는 줄일 포지션이 없습니다 |
| `markets(MarketKind::Perpetual)` | 오류가 아니라 빈 목록 |

## 상한

요청을 만들기 전에 검사합니다.

| 호출 | 허용 범위 | 오류 |
| --- | --- | --- |
| `trades` | `limit` 1~500 | `limit` 필드의 `Error::InvalidRequest` |
| `order_book` | 한 쪽당 `depth` 1~30 | `depth` 필드의 `Error::InvalidRequest` |
| `candles` | `limit`은 제한 없음. Upbit 응답당 200개이고 최대 100번의 호출까지 페이지를 대신 넘깁니다 | `limit` 0, `to`보다 이르지 않은 `from`, 캔들 20,000개를 넘는 구간은 `Error::InvalidRequest` |
| 자산 코드 | 대문자 ASCII 알파벳과 숫자 | `Error::InvalidRequest` |

| 간격 | REST `candles` | `Feed::Candles` |
| --- | --- | --- |
| 1s | 있음 | 있음 |
| 1m, 3m, 5m, 15m, 30m, 1h, 4h | 있음 | 있음 |
| 1d, 1w, 1M | 있음 | 없음 |

REST의 열한 간격은 [기준선](../common-api.ko.md#간격) 열 개에 `Sec1`을 더한
것입니다. Upbit이 집계하는 10분봉과 연봉은 `Interval`에 이름이 없습니다.

이력에서는 `Sec1`만 예외라 약 3개월치뿐이고 나머지 간격은 전 기간을 줍니다
([Upbit 문서](https://global-docs.upbit.com/reference/list-candles-seconds)).

열에 없는 간격은 `Feature::Candles`나 `Feature::CandleStream`을 지목하는
`Error::Unsupported`입니다. **단 요청이 나머지 면에서 올바를 때만 그렇습니다.**
위 표의 `limit`·구간·페이지 검사가 먼저 돌면서 자기 필드를 지목하는
`Error::InvalidRequest`를 냅니다. 둘 다 매칭하세요.

## 스트림

| 대상 | 동작 |
| --- | --- |
| `Feed::OrderBook` | **한 쪽당 30단계**, Upbit 자신의 [기본값](https://global-docs.upbit.com/reference/websocket-orderbook)입니다. `Subscription`으로 좁힐 수 없습니다 |
| 호가창 이벤트 | 차분이 아니라 전체 스냅숏. 이벤트마다 사본을 덮어쓰세요 |
| 원하는 깊이 | REST의 `Client::order_book`, 1~30단계 |
| `Candle::closed` | 한 구간당 한 번만 `true`입니다. Upbit이 다음 구간을 열 때 보냅니다 |
| 구간당 캔들 이벤트 | `closed`가 false인 이벤트 여러 개, 그다음 true인 이벤트 정확히 하나. 마감된 봉은 그 구간이 받은 마지막 프레임의 수치를 담습니다 |
| `subscribe_account` | 마켓 목록이 아니라 계좌 전체, 변경된 자산마다 이벤트 하나 |

### 스트림의 `Candle::closed`

Upbit은 다음 구간이 열리는 즉시 이전 구간 발행을 멈춥니다. 그래서 프레임 자신의
`timestamp`가 `open_time + interval`까지 가는 일이 없고 `closed`도 시계가 아니라
다음 구간으로 넘어가는 시점을 보고 정합니다.

| 2026-07-30 `candle.1m` `KRW-BTC` 프레임 | `candle_date_time_utc` | 프레임 `timestamp` | 구간 종료 |
| --- | --- | --- | --- |
| 07:46이 받은 마지막 프레임 | 07:46:00 | 07:46:58.309 | 07:47:00.000 |
| 다음 구간의 첫 프레임 | 07:47:00 | 07:47:00.706 | 07:48:00.000 |

| 상황 | 발행 |
| --- | --- |
| 구간이 진행 중일 때 | 형성 중인 봉, 갱신마다 다시 발행되며 `closed`는 false |
| 다음 구간이 열릴 때 | 마감된 봉이 먼저, 그다음 새로 형성 중인 봉 |
| 재연결 뒤 | 들고 있던 봉은 마감이 아니라 폐기됩니다. 처음으로 마감되는 구간은 다음에 열리는 구간입니다 |
| 이미 지나간 구간의 프레임이 뒤늦게 올 때 | 아무것도 오지 않습니다 |
| 구독을 끊을 때의 마지막 구간 | 끝내 마감되지 않습니다 |

REST에서는 읽는 쪽 기계의 시계로 `closed`를 정합니다. `Month1`도 예외 없이
달력이 마감해서 6월 봉은 7월 1일에 마감됩니다.

## 요청 할당량

| 그룹 | 한도 | 단위 |
| --- | --- | --- |
| 공개 시세: `markets`, `candles`, `trades`, `ticker`, `order_book` | 초당 10회 | IP |
| 거래소 기본: `balances`, `open_orders`, `cancel_order` | 초당 30회 | 계좌 |
| 주문 접수: `place_order` | 초당 8회 | 계좌 |
| 새 WebSocket 연결 | 초당 5회 | 인증 전이면 IP, 인증했으면 계좌 |
| 한 WebSocket으로 보내는 프레임 | 초당 5회, 분당 100회 | 연결 |

취소 후 재주문 루프의 한계는 `cancel_order`가 아니라 `place_order`가 정합니다.

`maxt`는 속도를 조절하지 않습니다. 속도 조절은 여러분의 몫이고 너무 빨랐다는
사실은 `Error::is_rate_limited()`로 알게 됩니다.

## 주문

| 주문 | 크기 | 가격 |
| --- | --- | --- |
| 지정가, 양쪽 모두 | `Size::Base` | 필수 |
| 시장가 매수 | `Size::Quote`, 지출할 금액 | 없음 |
| 시장가 매도 | `Size::Base`, 내놓을 수량 | 없음 |

그 밖의 조합은 `size` 필드의 `Error::InvalidRequest`이고 0 이하의 가격이나
수량도 마찬가지입니다. `maxt`가 서명 전에 거절합니다.

| 지정가 주문의 `TimeInForce` | 전송값 |
| --- | --- |
| `GoodTilCancelled` | 아무것도 보내지 않으며 이것이 기본값입니다 |
| `ImmediateOrCancel` | `ioc` |
| `FillOrKill` | `fok` |
| `PostOnly` | `post_only` |

시장가 주문에서는 `ImmediateOrCancel`만 받아들이고 아무것도 보내지 않습니다.
그 밖의 값은 `time_in_force`에서 실패합니다.

## 주문 정밀도와 최소 주문 크기

`maxt`는 가격과 수량이 0보다 큰지만 확인합니다. 그래서 Upbit의 호가 단위에서
벗어난 가격이나 최소 주문 금액에 못 미치는 주문은 여기가 아니라 Upbit이
거절합니다. Upbit은 호가 단위를 가격 구간별 표로, 최소 주문 금액을 quote
자산별로 공시하는데 `maxt`의 어떤 타입도 두 값을 담지 않습니다. 첫 주문 전에
Upbit에서 직접 읽으세요.

## 주의할 점

| 필드 또는 호출 | 동작 |
| --- | --- |
| `Trade::id` | Upbit의 `sequential_id`, 보내온 숫자 그대로이며 스트림과 REST가 같은 값을 줍니다. 이 값으로 중복을 제거하세요 |
| `Trade::id`로 정렬 | 하지 마세요. Upbit은 이 값을 유일성 판단 근거로 [설명하고](https://global-docs.upbit.com/reference/today-trades-history) 체결 순서는 보장하지 않습니다. `Trade::timestamp`로 정렬하세요 |
| `trades` 순서 | 최신 순이며 여기서 정렬합니다. 안정 정렬이므로 같은 밀리초를 공유하는 체결은 Upbit 자신의 순서를 지킵니다 |
| `candles` 순서 | 오래된 순. Upbit 자체는 최신 순으로 응답합니다 |
| 투자 유의 종목 | `MarketStatus::Unknown`. 그런 마켓도 거래는 그대로 열려 있습니다. 아래 [유의 종목과 주의 종목](#유의-종목과-주의-종목)을 보세요 |
| 투자 주의 종목 | `MarketStatus::Active` 그대로. 아래 [유의 종목과 주의 종목](#유의-종목과-주의-종목)을 보세요 |
| `open_orders` | 한 페이지에 100건씩 모든 페이지를 순회하며 호가창에 올라간 주문과 발동을 기다리는 주문을 함께 요청합니다 |
| `cancel_order` | 마켓과 주문 식별자를 받습니다. Upbit은 식별자만으로 취소합니다 |
| 인증 정보 없음 | `supports(Feature::Balances)`가 `false`인데도 `Error::Unsupported`가 아니라 `Error::Auth`. 둘 다 매칭하세요 |
| Upbit이 거부한 인증 정보 | `Error::Auth`가 아니라 Upbit 자신의 이름을 담은 `Error::Exchange`. HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_access_key`, `nonce_used`, `no_authorization_ip`, `no_authorization_token`과 HTTP 403 `out_of_scope`입니다. **측정이 아니라 문서 기준입니다** |

## 유의 종목과 주의 종목

Upbit의 두 지정을 통틀어 `MarketStatus`에는 값이 하나뿐이라 첫 번째만
반영됩니다.

| 지정 | 내용 | `MarketStatus` | 읽는 곳 |
| --- | --- | --- | --- |
| 유의 종목 | Upbit이 직접 지정하고 공지합니다. 프로젝트에 원인 해소를 요청하고 해소되지 않으면 거래 지원을 종료할 수 있습니다 | `Unknown` | `MarketInfo::status` 또는 `market_events` |
| 주의 종목 | 공시된 기준에 따라 자동으로 지정되고 해제되며 기준마다 플래그가 하나씩 붙습니다 | `Active` | `market_events`에서만 |

2026-07-30 기준 Upbit이 올린 800개 마켓 가운데 11개가 유의 종목이었고 190개가
주의 기준을 하나 이상 달고 있었으며 그중 175개는 `GLOBAL_PRICE_DIFFERENCES`
하나뿐이었습니다. 주의 종목 수는 하루 사이에도 움직이지만 유의 종목 수는
그렇지 않습니다. Bithumb의 `market_warning`은 여기의 유의 종목과 같은 지정이고
주의 종목은 [별도 엔드포인트](bithumb.ko.md#유의-종목과-경보제)에 있습니다.

**네 지역이 같은 필드를 보내지 않습니다.** 한국은 두 지정을 모두 담은
`market_event`를, 나머지 셋은 유의 종목만 실리는 예전 필드 `market_warning`을
보냅니다. `MarketInfo::status`는 그래도 네 지역에서 같은 뜻이고
`UpbitMarketEvent::cautions`는 한국 밖에서 비어 있습니다.

## Upbit 전용 호출

`Client::adapter()`로 호출합니다. 호출 하나가 여러 마켓에 한꺼번에 답합니다.

| 메서드 | 반환 | 비용 |
| --- | --- | --- |
| `tickers(&[Market])` | 마켓마다 티커 하나 | 초당 열 번의 공개 요청 중 한 번 |
| `order_books(&[Market], Option<u32>)` | 마켓마다 호가창 하나, 깊이는 한 쪽당 30단계까지 | 마찬가지로 한 번 |
| `market_events()` | 마켓마다 `UpbitMarketEvent` 하나. 유의 종목 플래그와 주의 기준 이름들 | 마찬가지로 한 번 |

`Client::ticker`와 `Client::order_book`은 앞의 두 메서드에 마켓 하나만 넘긴 것입니다.

**묶음이 어디서 끊기는지는 양쪽 모두 문서에 없습니다.** 두 메서드는 마켓 코드
개수에 상한을 두지 않고 Upbit도 이를 공시하지 않으므로 한계는 Upbit이나 그 앞단
프록시가 받아 주는 URL 길이입니다. 넘으면 `Error::InvalidRequest`가 아니라
`Error::Exchange`입니다. 서른 개는 넉넉히 안쪽이고 수백 개는 쓰기 전에 직접
시험해 보세요.

```rust
use maxt::{Client, Exchange, Market, adapters::UpbitAdapter};

async fn breadth(client: &Client<UpbitAdapter>) -> maxt::Result<()> {
    let markets = [Market::spot(Exchange::Upbit, "BTC", "KRW")];
    let _tickers = client.adapter().tickers(&markets).await?;
    let _books = client.adapter().order_books(&markets, Some(5)).await?;
    Ok(())
}
```

## 인증 정보

어댑터와 같은 지역에서 함께 발급되는 access key와 secret key입니다. 이 키가
`Feature::Balances`, `Feature::OpenOrders`, `Feature::Trading`,
`Feature::AccountStream`을 엽니다.

```rust
use maxt::{Client, adapters::UpbitAdapter};

fn client() -> Client<UpbitAdapter> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    Client::new(UpbitAdapter::new().with_credentials(access_key, secret))
}
```

secret key는 프로세스 밖으로 나가지 않습니다. 키는 소스에 두지 말고 프로그램이
실제로 쓰는 만큼만 권한을 주세요. 주문을 내지 않는다면 읽기 전용 키면
충분합니다.

## 예제

`cargo run --example public_rest`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Upbit 공식 문서

| 주제 | 문서 |
| --- | --- |
| 할당량 | [호출 한도](https://global-docs.upbit.com/reference/rate-limits) |
| 공개 REST | [마켓 목록](https://global-docs.upbit.com/reference/list-trading-pairs.md) · [티커](https://global-docs.upbit.com/reference/list-tickers.md) · [호가창](https://global-docs.upbit.com/reference/list-orderbooks.md) · [체결](https://global-docs.upbit.com/reference/list-pair-trades.md) · [분 캔들](https://global-docs.upbit.com/reference/list-candles-minutes.md) · [초 캔들](https://global-docs.upbit.com/reference/list-candles-seconds) |
| 비공개 REST | [계좌](https://global-docs.upbit.com/reference/get-balance.md) · [미체결 주문](https://global-docs.upbit.com/reference/list-open-orders.md) |
| WebSocket | [체결](https://global-docs.upbit.com/reference/websocket-trade.md) · [호가창](https://global-docs.upbit.com/reference/websocket-orderbook) · [캔들](https://global-docs.upbit.com/reference/websocket-candle.md) · [내 주문](https://global-docs.upbit.com/reference/websocket-myorder.md) · [내 자산](https://global-docs.upbit.com/reference/websocket-myasset.md) |

---

[공통 API](../common-api.ko.md) · [거래소 고르기](../providers.ko.md)
