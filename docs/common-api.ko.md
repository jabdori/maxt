# 공통 API

[English](common-api.md) | [한국어](common-api.ko.md)

`Client`는 어댑터를 감싸고 지원 거래소들이 공통으로 갖춘 기능을 모두 드러냅니다.
밑에 어떤 거래소가 있든 호출의 뜻은 같습니다. 한 거래소에만 있고 나머지에는 없는
기능은 그 거래소의 어댑터에 그대로 남습니다.

## 시장 데이터

인증 정보가 필요한 호출은 여기에 없습니다.

| 호출 | 답하는 것 |
| --- | --- |
| `markets(kind)` | 거래소가 상장한 해당 종류의 모든 마켓 |
| `ticker(&market)` | 최근 24시간 요약 |
| `order_book(&market, depth)` | 호가창 스냅숏 |
| `trades(&market, limit)` | 최근 체결, 최신 순 |
| `candles(&CandleRequest)` | 과거 캔들, 오래된 순 |

| 경우 | 일어나는 일 |
| --- | --- |
| 거래소가 상장하지 않은 종류의 `markets` | 에러가 아니라 빈 목록 |
| 거래소가 제공하지 않는 `depth` | `Error::InvalidRequest`. 가까운 깊이로 반올림하지 않습니다. 제공하는 값의 집합은 각 거래소 페이지에 있습니다 |
| 범위를 벗어난 `trades`의 `limit` | 같은 규칙으로 `Error::InvalidRequest`. 상한은 한 번의 호출이 담는 양이고 편차가 큽니다. Binance 1,000, Upbit과 Bithumb 500, Hyperliquid 10이며 Hyperliquid의 엔드포인트는 개수를 아예 받지 않습니다. 거래소가 주는 만큼 받으려면 `limit`을 비워 두세요 |
| `trades` 정렬 | Binance는 체결 식별자 내림차순이라 여러 체결이 같은 밀리초에 몰려도 정확합니다. Upbit, Bithumb, Hyperliquid는 타임스탬프로 정렬합니다 |
| `MarketInfo::native_symbol` 읽기 | 거래소 자신이 그 마켓을 부르는 이름. 거래소 화면이나 공식 문서와 대조할 때 씁니다 |

### 상장 항목이 담는 것

`markets`는 맨 `Market`이 아니라 `MarketInfo`를 돌려줍니다.

| 필드 | 담는 것 |
| --- | --- |
| `market` | 다른 모든 호출이 받는 그 정체성 |
| `native_symbol` | 거래소 자신의 심볼 그대로: `KRW-BTC`, `BTCUSDT`, `BTC` |
| `status` | 거래소가 이 마켓에서 주문을 받고 있는지 |
| `korean_name`, `english_name` | 거래소가 발행한다면 그 자산의 이름. Binance와 Hyperliquid는 어느 쪽도 발행하지 않으므로 그곳에서는 둘 다 `None`입니다 |

| `MarketStatus` | 뜻 |
| --- | --- |
| `Active` | 상장되어 거래 중 |
| `Paused` | 멈췄지만 상장은 그대로 남아 있고 페어는 돌아옵니다 |
| `Delisted` | 사라졌습니다 |
| `Unknown` | 거래소의 답이 나머지 셋 중 어디에도 대응되지 않습니다 |

`MarketStatus`는 `#[non_exhaustive]`이므로 `_` 갈래를 두고 매칭하세요.

`Unknown`은 거래 불가와 같은 말이 아니고 한국 거래소에서는 보통 거래 불가가
아닙니다. Upbit과 Bithumb은 거래를 그대로 열어 둔 채 유의 종목으로 지정합니다.
`MarketStatus`에는 "거래 중이지만 표시가 붙음"을 뜻하는 값이 없어서 그런 마켓이
여기서는 `Unknown`으로 읽힙니다. 각 거래소가 붙인 표시는
`BithumbAdapter::market_warnings`와 `UpbitAdapter::market_events`에 그대로
남습니다. 두 거래소 모두 `MarketStatus`에 닿지 않는 더 약한 지정을 하나씩 더
공시하며, `UpbitAdapter::market_events`와 `BithumbAdapter::market_alerts`가 각각
그것을 읽을 수 있는 유일한 곳입니다. `Unknown`은 거절이 아니라
"거래소에 물어보라"는 뜻으로 다루세요.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::{Client, MarketKind, MarketStatus};

async fn tradable_krw_markets(client: &Client<UpbitAdapter>) -> maxt::Result<usize> {
    let listed = client.markets(MarketKind::Spot).await?;

    let tradable = listed
        .iter()
        // 상장은 지금 거래된다는 약속이 아닙니다.
        .filter(|info| info.market.quote == "KRW" && info.status == MarketStatus::Active)
        .count();

    for info in listed.iter().take(3) {
        // 거래소가 이름을 발행하지 않는 곳에서는 `english_name`이 `None`입니다.
        let name = info.english_name.as_deref().unwrap_or(&info.native_symbol);
        println!("{} ({name}) is {} upstream", info.market, info.native_symbol);
    }

    Ok(tradable)
}
```

## 캔들

네 거래소 중 둘에는 시작 시각 파라미터가 없고 넷 모두 한 응답의 개수를
제한합니다. `maxt`가 그 차이를 안에서 맞추므로 계약은 어디서나 하나이고 거래소가
어떤 순서로 답했든 결과는 언제나 오래된 순입니다.

| 요청 | 돌아오는 것 |
| --- | --- |
| `from` | 모든 거래소에서 지켜집니다. 어떤 어댑터도 `Error::Unsupported`로 보고하지 않습니다 |
| `limit` | 응답당 상한을 넘어서도 페이지를 넘겨 가며 지켜집니다. 그 상한은 호출이 아니라 HTTP 응답 하나의 성질입니다 |
| `from`과 `limit`을 함께 | `from` 시점 이후로 가장 오래된 `limit`개. 백필이 요구하는 형태입니다 |
| `limit`만 | 가장 최신 그만큼 |
| `limit`이 `0` | `limit` 필드의 `Error::InvalidRequest`. 기본 한 페이지를 원하면 비워 두세요 |
| `from`이 `to`와 같거나 그보다 뒤 | `from` 필드의 `Error::InvalidRequest` |

### `Candle::closed`

네 거래소 모두 캔들의 창이 끝나면 `true`입니다. 응답의 마지막 캔들만 `false`가
될 수 있고, 그것이 아직 형성 중인 캔들입니다.

| 창의 끝을 어디서 얻는가 | 거래소 |
| --- | --- |
| 페이로드에 실린 거래소 자신의 마감 시각 | Binance, Hyperliquid |
| 거래소가 자르는 시간대에서 `open_time`으로부터 간격만큼 한 칸 나아간 지점. Upbit은 UTC, Bithumb은 KST | REST의 Upbit과 Bithumb |
| 다음 창을 여는 프레임. 한 창의 어떤 프레임도 그 창의 끝에 닿거나 그 뒤의 시각으로 찍히지 않기 때문입니다 | Upbit과 Hyperliquid 캔들 스트림 |
| 프레임에 실린 거래소 자신의 플래그 | Binance 캔들 스트림 |

넷 다 같은 질문에 답하므로 소비자는 `closed`만 보고 확정하면 되고 거래소별로
분기할 필요가 없습니다.

스트림 둘은 프레임 하나만으로 답할 수 없어서 `maxt`가 전환 시점에서 답을
얻습니다. Upbit은 다음 창이 열리는 순간 이전 창의 발행을 멈추고, Hyperliquid는
그 창 자신의 마감 시각보다 약 2초 먼저 멈춥니다. 둘 다 창의 끝에 닿거나 그 뒤의
시각으로 프레임을 찍는 일이 없으므로, 페이로드로도 프레임 하나의 시계로도 봉을
확정할 수 없습니다. 두 곳 모두 구독마다 창의 마지막 프레임을 들고 있다가
거래소가 다음 창을 열면 `closed`를 세워 내보냅니다. 그래서 거래소가 발행을
멈추기 전에 창이 끝났다고 말하는 일이 없고, 구독을 끊으면 마지막 창은 확정되지
않은 채 남습니다. Binance는 자기 플래그로 직접 알려주고, Bithumb에는 캔들
스트림이 없습니다. 이 확정 이벤트는 그 창의 `open_time`을 다시 실어 보내므로,
`open_time`을 키로 쓰는 소비자는 덧붙이지 않고 덮어씁니다. 재연결 시에는 들고
있던 것을 길이를 알 수 없는 공백 너머로 확정하지 않고 버리므로,
`MarketEvent::Reconnected`가 끊고 지나간 창은 확정 이벤트를 받지 못합니다. 창마다
확정 이벤트가 하나뿐이라는 사실은 `Feed::Candles`에 `Overflow::DropNewest`가
맞지 않는 이유이기도 합니다. [스트림 설정](#스트림-설정)을 보세요.

**간격은 길이의 이름이지 격자의 이름이 아닙니다.** 그 길이의 캔들이 어디서
열리는지는 거래소가 정하고, `Hour4`, `Day1`, `Day3`, `Week1`, `Month1`에서 네
곳의 답이 갈립니다. 전체 표는 `Interval` 자신의 문서에 있고 네 곳을 실제로 읽어
만든 것입니다. 요약하면 이렇습니다. Bithumb은 모든 창을 한국 시간으로 자르므로
9시간을 나누어떨어지게 하는 간격에서는 나머지 셋과 일치하고 그렇지 않은
간격에서는 어긋나며, 그것이 `Hour4`와 하루 이상의 모든 간격입니다. Hyperliquid는
`Day3`, `Week1`, `Month1`을 달력이 아니라 Unix 에포크에서부터 재므로 주는
목요일에 열리고 월은 30일 구간입니다. 그래도 `closed`는 네 곳 모두 같은 질문에
답합니다. 다른 것은 `open_time`뿐이고, 두 거래소를 `open_time`으로 조인하는 것은
그 표가 양쪽에 대해 같은 말을 할 때만 안전합니다.

```rust
use maxt::{Candle, Decimal};

// 형성 중인 캔들은 확정된 캔들이 아닙니다. 확정된 것 중 마지막을 취합니다.
fn last_settled_close(candles: &[Candle]) -> Option<Decimal> {
    candles.iter().rev().find(|candle| candle.closed).map(|candle| candle.close)
}
```

### 페이지 조회는 100회에서 멈춥니다

이 거래소들은 하나같이 한 응답씩 뒤로 거슬러 페이지를 넘기므로 넓은 구간은
페이지마다 순차 왕복 한 번을 쓰고 이를 빠르게 만드는 방법은 없습니다. `maxt`는
최대 100페이지를 걸어가므로 한 번의 호출로 모을 수 있는 최대치는 거래소의 응답당
상한에 100을 곱한 값입니다. 한 번에 200개인 Upbit이라면 캔들 2만 개쯤 됩니다.
그보다 더 요구하면 **첫 호출 전에** `Error::InvalidRequest`가 되고, 구간을 넓힌
필드의 이름과 한계치를 함께 알려 줍니다.

| 요청 | 거절되는 필드 |
| --- | --- |
| `limit` 없이, 100페이지를 넘겨야 할 만큼 과거의 `from` | `from` |
| 응답당 상한의 100배를 넘는 `limit` | `limit` |

미리 거절하는 것이 핵심입니다. 절반쯤 가다 호출 한도에 걸려 중단된 순회는 이미 그
호출들을 다 써 버린 뒤입니다. 한계치보다 넓게 읽으려면 `limit`을 정해 두고 구간을
직접 옮기세요. 순회와 한계치는 `src/adapters/candles.rs`에 있습니다.

## 간격

| 간격 집합 | 성립하는 것 |
| --- | --- |
| 기준선 | `client.supports(Feature::Candles) == true`는 모든 거래소에서 열 개를 보장합니다. `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1`입니다. 이 열 개를 기준으로 쓰면 같은 코드가 네 거래소에서 모두 캔들을 읽습니다. **네 곳에서 같은 격자를 읽는다는 뜻은 아닙니다.** `Hour4`, `Day1`, `Week1`, `Month1`에서는 여는 시각이 거래소마다 다르고, 어디가 다른지는 `Interval`이 문서로 남깁니다 |
| 그 밖 | 거래소마다 다릅니다. `Hour2`, `Hour8`, `Hour12`, `Day3`은 Binance와 Hyperliquid에만 있습니다. `Sec1`은 Upbit과 Binance 현물에 있고 Binance USD-M은 제공하지 않습니다. 거래소 페이지를 확인하세요 |
| 거래소가 제공하지 않는 간격 | `Feature::Candles`를 지목하는 `Error::Unsupported`입니다. 어느 `Feature`도 두 집합을 가르지 않으므로 플래그로는 어느 쪽인지 알 수 없습니다 |

**간격에 붙은 `Unsupported`는 `maxt`가 그곳에 엔드포인트를 대응시켜 두지 않았다는
뜻입니다.** 거래소가 그 간격을 한 번도 집계하지 않았다는 주장이 아닙니다. Upbit과
Bithumb은 둘 다 10분봉을 제공하고 Upbit은 연봉까지 제공하지만 `Interval`이 어느
쪽도 이름 붙이지 못해 `maxt`가 요청할 방법이 없습니다. `maxt`가 닿는 간격을
거래소가 발행한다면 `maxt`는 그 간격에 닿습니다. 기준선은 어댑터가 아니라 네
거래소 자신의 문서에서 읽어 낸 것이고
[`tests/unsupported_is_honest.rs`](../tests/unsupported_is_honest.rs)의
`BASELINE_INTERVALS`가 바로 그 기준으로 검증합니다.

## 계좌

여기 있는 호출은 모두 인증 정보가 필요합니다.

| 호출 | 답하는 것 |
| --- | --- |
| `balances()` | 계좌가 보유한 모든 자산의 사용 가능분과 잠긴 분. 거래소가 상장한 모든 자산을 대부분 0으로 채워 돌려주기도 하니 필요한 것만 걸러 쓰세요 |
| `open_orders()` | 모든 마켓의 미체결 주문 |
| `open_orders_on(&market)` | 한 마켓의 미체결 주문 |
| `subscribe_account()` | 잔고와 주문 변경의 실시간 스트림 |
| `Balance::total()` | 사용 가능분과 잠긴 분의 합 |

주문 크기는 `Balance::available`을 기준으로 잡으세요. `locked`는 걸려 있는 주문에
이미 약속된 몫이고 그것까지 쓰려 들면 거래소가 거절합니다.

## 주문

두 호출 모두 인증 정보가 필요합니다.

| 호출 또는 타입 | 하는 일 |
| --- | --- |
| `place_order(&OrderRequest)` | 거래소 자신의 식별자를 담은 `Order`를 돌려줍니다 |
| `cancel_order(&market, order_id)` | 그 식별자를 도로 받습니다 |
| `OrderRequest::market(market, side, size)` | 가격이 없는 시장가 주문을 만듭니다 |
| `OrderRequest::limit(market, side, size, price)` | 가격이 언제나 있는 지정가 주문을 만듭니다 |
| `Size::Base`, `Size::Quote` | 크기가 어느 자산 기준인지 밝힙니다. 시장가 매수는 보통 가격을 매기는 쪽인 quote 자산 기준, 시장가 매도는 기준 자산 기준이기 때문입니다 |
| `.time_in_force(..)` | `TimeInForce::PostOnly`는 호가창에 걸릴 때만 주문을 냅니다 |
| `.reduce_only()` | 이미 있는 포지션을 닫는 쪽으로만 주문을 제한합니다. 파생상품 전용입니다 |
| `OrderStatus::is_live()` | 아직 체결될 수 있는 상태, 곧 접수됨·미체결·부분 체결에서 참 |

두 생성자 모두 크기를 맨 숫자가 아니라 `Size`로 받으므로 원으로 잰 시장가 매수를
비트코인으로 잰 것과 혼동할 수 없습니다.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Decimal, Exchange, Market, OrderRequest, OrderStatus, Side, Size, TimeInForce};

async fn buy_a_little_bitcoin(client: &Client<UpbitAdapter>) -> maxt::Result<()> {
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    // 가격은 이 문서가 아니라 호가창에서 가져옵니다. 문서에 적힌 숫자는 시세가
    // 그 위에 있는 동안에만 안전하고, 시세가 내려온 날 곧바로 테이커 체결이
    // 되며, 그렇게 바뀌었다고 알려 주는 것은 아무것도 없습니다. 거래소가 돌려준
    // 가장 깊은 매수 호가는 다른 어떤 매수 호가보다 낮다는 것이 구조로
    // 보장되므로, 시세가 어떻든 그 자리의 매수는 매도 호가를 건드리지 못합니다.
    // 게다가 이미 이 거래 시장이 받는 가격이고, 직접 지어낸 숫자는 그렇지
    // 않을 수 있습니다.
    let book = client.order_book(&market, None).await?;
    let Some(deepest_bid) = book.bids.last() else {
        return Ok(());
    };

    let order = client
        .place_order(
            &OrderRequest::limit(
                market.clone(),
                Side::Buy,
                // 0.001 BTC. 여기서 `Size::Quote`였다면 0.001 KRW라는 뜻입니다.
                Size::Base(Decimal::new(1, 3)),
                deepest_bid.price,
            )
            // 위에서 읽은 뒤 이 호출 사이에 호가창이 움직였다면 테이커로
            // 체결되지 않고 그대로 거절됩니다.
            .time_in_force(TimeInForce::PostOnly),
        )
        .await?;

    // 접수는 체결이 아닙니다. "아직 체결될 수 있는가"를 묻는 것이 `is_live`입니다.
    if order.status.is_live() {
        // 취소에 쓰는 것은 반환된 주문의 식별자입니다. 취소는 호가창과 경합하므로
        // 넣은 주문이 아니라 돌아온 주문을 믿으세요.
        let cancelled = client.cancel_order(&market, &order.id).await?;
        assert_eq!(cancelled.id, order.id);
        if cancelled.status == OrderStatus::Filled {
            println!("filled before the cancel landed");
        }
    }

    Ok(())
}
```

모든 거래소가 모든 조합을 받지는 않습니다. Hyperliquid에는 시장가 주문 종류가
아예 없고 quote 자산 기준 크기도 보편적이지 않습니다. 어떤 형태를 받는지는 각
거래소 페이지에 적혀 있습니다.

**문서에 적어 둔 지정가는 어느 것도 계속 안전하지 않습니다.** 어떤 숫자가
호가창에 걸릴지 테이커로 체결될지는 실행한 그날의 시세에 달린 사실이고, 문서는
그날을 알 수 없습니다. 그러니 이렇게 하세요.

| 도착하자마자 체결되지 않는 주문을 내려면 | 방법 |
| --- | --- |
| 가격을 정할 때 | 조금 전에 읽은 호가창을 기준으로, 매수는 최우선 매수 호가 이하로, 매도는 최우선 매도 호가 이상으로 잡으세요 |
| 보호할 때 | `TimeInForce::PostOnly`를 쓰세요. 그 사이 호가창이 움직였다면 거래소가 체결하지 않고 거절합니다 |
| 하지 말아야 할 것 | 이 문서를 포함해 어떤 문서에서도 가격을 베껴 오지 마세요. 여기 있는 숫자는 실행할 때 거래소에서 읽어 온 값이거나 수량이고, 가격인 적은 없습니다 |

## 주문 정밀도와 최소 주문 크기

호가 단위, 수량 단위, 최소 주문 금액이 주문의 접수 여부를 가릅니다. 공통 API는
이 셋을 싣지 않습니다. 표현하는 방식이 거래소마다 다르고 하나로 눌러 편 타입은
대부분의 거래소를 두고 사실이 아닌 말을 하게 됩니다. 어댑터 둘이 자기 거래소의
답을 그대로 내놓고 둘 다 `Client::adapter`로 닿습니다.

| 거래소 구성 | 규칙이 있는 곳 |
| --- | --- |
| Binance 현물 | `BinanceAdapter::spot_symbol_filters(&market)`이 `BinanceSymbolFilters`를 돌려줍니다: `tick_size`, `min_price`, `max_price`, `step_size`, `min_quantity`, `max_quantity`, `min_notional`. **알려 주기만 하고 강제하지는 않습니다.** `place_order`는 이 값을 읽지 않습니다 |
| Hyperliquid | `HyperliquidAdapter::asset_context(&market)`의 `size_decimals`와 `price_decimals`가 각각 소수 몇 자리까지 담을 수 있는지 말합니다. 자릿수뿐이고 최소 주문 금액은 없습니다. `maxt`가 서명 전에 둘 다 검사하므로 자산이 허용하는 것보다 잘게 쪼갠 주문은 로컬에서 거절됩니다 |
| Binance USD-M | 노출하지 않습니다. USD-M 어댑터에서 `spot_symbol_filters`는 `Error::Unsupported`를 보고합니다. USD-M 상장 정보에는 다른 필터 집합이 붙습니다 |
| Upbit, Bithumb | 아예 노출하지 않습니다 |

**규칙을 노출하는 것과 규칙에 맞춰 검사하는 것은 다릅니다.** 다섯 거래소 구성
중 로컬에서 검사하는 곳은 Hyperliquid 하나입니다
(`src/adapters/hyperliquid/rest.rs`, 서명 전에 두 규칙 모두). Binance 현물을
포함한 나머지 넷에서는 가격이나 크기가 너무 잘게 쪼개진 주문을 거래소가 거절하고
나서야 알게 되고, 그 거절은 거래소 자신의 코드와 메시지를 담은
`Error::Exchange`로 도착합니다. `spot_symbol_filters`를 읽고 그 값에 맞춰
반올림하는 일은 호출자의 몫입니다. `maxt`는 다섯 구성 어디에서도 주문을 규격에
맞춰 반올림하지 않습니다.

## 파생상품

무기한 선물 마켓에서만 의미가 있습니다. 현물 전용 어댑터에서는 아래 모두가
`Error::Unsupported`이고 모든 구성에서 그러한지를
[`tests/unsupported_is_honest.rs`](../tests/unsupported_is_honest.rs)가
검사합니다. Hyperliquid 어댑터 하나는 두 종류를 함께 다루므로 그곳에서는 같은
호출들이 지원되는 것으로 읽히고 마켓별로 거절합니다. 현물 마켓을 건네면 그때도
`Error::Unsupported`입니다.

| 호출 | 답하는 것 | 인증 정보 |
| --- | --- | --- |
| `positions()`, `positions_on(&market)` | 보유 포지션. 크기가 0인 포지션은 포지션이 아니며, 거래소가 무엇을 게시하든 어느 어댑터도 그런 것을 보고하지 않습니다 | 필요 |
| `margin_summary()` | 계좌 전체의 마진 상태 | 필요 |
| `funding_rates(&HistoryRequest)` | 마켓의 펀딩 비율 이력. 계좌가 아니라 마켓의 성질입니다 | 불필요 |
| `funding_payments(&HistoryRequest)` | 한 계좌가 실제로 물린 금액 | 필요 |
| `set_margin(&MarginRequest)` | 레버리지, 마진 모드, 또는 둘 다를 설정합니다. 최소 하나는 있어야 하고 Hyperliquid는 둘 다 요구합니다. 그곳에서 하나만 주면 `Error::InvalidRequest`입니다 | 필요 |

### 어느 값으로 크기를 잡을지

`MarginSummary`가 싣는 선택적 값은 셋이고 셋은 서로 바꿔 쓸 수 없습니다.

| 필드 | 무엇인가 |
| --- | --- |
| `equity` | 잔고에 미실현 손익을 더한 값 |
| `margin_balance` | 열려 있는 포지션과 주문에 **이미 걸어 둔** 금액 |
| `available_balance` | **새로 열 수 있는** 여윳돈 |

새 주문의 크기는 `available_balance`를 기준으로 잡으세요. `margin_balance`는 이미
묶인 돈이고 그 값으로 크기를 잡은 주문은 계좌가 이미 써 버린 자금을 기준으로 잰
것입니다. 셋 다 `Option`입니다. 모든 거래소가 셋을 다 발행하지는 않기 때문입니다.
없는 값을 0으로 읽는 크기 규칙은 아무것도 열지 못하고 무제한으로 읽는 규칙은
지나치게 많이 엽니다. `None`은 "거래소가 말하지 않았다"로 읽고 거기서 멈추세요.

### 포지션

| 필드 | 이렇게 읽으세요 |
| --- | --- |
| `quantity` | 부호가 없습니다. 방향은 `side`에 있고 포지션이 비어 있으면 `None`입니다 |
| `is_flat()` | 크기를 판별합니다. 거래소는 이미 들고 있지도 않은 마켓의 빈 포지션을 보고하기도 하므로 그런 항목은 건너뛰세요. 하나하나가 열린 위험은 아닙니다 |
| 발행되지 않는 모든 필드 | `None`이고 `None`은 0이 아닙니다 |
| Binance의 `leverage`, `margin_mode` | 언제나 `None`입니다. `maxt`가 읽는 포지션 엔드포인트가 두 필드를 더는 발행하지 않습니다(`src/adapters/binance/private.rs`). 그곳의 `None`은 포지션에 마진이 어떻게 걸렸는지를 두고 아무 말도 하지 않습니다. `leverage`를 `1`로 채우면 20배로 연 포지션을 레버리지 없는 포지션으로 보고해 위험을 스무 배 낮춰 말하게 됩니다. 같은 이유로 `maxt`는 계정 스트림에서 `ACCOUNT_CONFIG_UPDATE`도 구독하지 않습니다. 이 이벤트가 싣는 레버리지 변화를 `maxt`가 보고할 곳이 없습니다 |

### 파생상품 읽기 예제

```rust
use maxt::adapters::BinanceAdapter;
use maxt::{
    Client, Decimal, Exchange, HistoryRequest, MarginMode, MarginRequest, Market,
};

async fn size_a_perpetual_order() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::usd_m_futures().with_credentials("key", "secret"));
    let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
    let leverage = Decimal::from(5);

    // 요청은 하나지만 Binance 호출은 둘입니다. `POST /fapi/v1/leverage`가
    // 먼저 나가고 `POST /fapi/v1/marginType`이 그다음입니다. Binance는 이 짝을
    // 원자적으로 만들 방법을 주지 않으므로 그 사이에 계좌는 새 레버리지와
    // 예전 모드에 놓입니다. 포지션이 열린 상태에서 `marginType`을 바꾸려 하면
    // Binance가 흔히 거절하는데, 그렇게 두 번째 호출이 실패하면 계좌는 그
    // 상태로 남습니다. 요청한 레버리지를 믿고 크기를 잡기 전에 상태를 다시
    // 읽으세요.
    client
        .set_margin(
            &MarginRequest::new(market.clone())
                .leverage(leverage)
                .margin_mode(MarginMode::Isolated),
        )
        .await?;

    // 새 포지션을 받쳐 주는 것은 `available_balance`입니다. `margin_balance`는
    // 이미 있는 포지션을 받치고 있는 금액입니다.
    let margin = client.margin_summary().await?;
    match margin.available_balance {
        Some(free) => println!("{} {} free, {} of notional at 5x", free, margin.asset, free * leverage),
        None => println!("{} publishes no free-margin figure", client.exchange()),
    }

    for position in client.positions_on(&market).await? {
        // 빈 포지션도 거래소가 보고할 수 있는 포지션입니다.
        if position.is_flat() {
            continue;
        }
        // `None`은 "발행하지 않음"이라서 0으로 더하면 노출이 실제보다 작아집니다.
        println!(
            "{:?} {} at {:?}, notional {:?}",
            position.side, position.quantity, position.entry_price, position.notional
        );
    }

    // 공개: 계좌의 청구서가 아니라 마켓의 비율입니다.
    let rates = client
        .funding_rates(&HistoryRequest::new(market.clone()).limit(100))
        .await?;
    let mean: Decimal = rates.items.iter().map(|rate| rate.rate).sum::<Decimal>()
        / Decimal::from(rates.items.len().max(1));
    println!("{mean} mean rate over {} observations", rates.items.len());

    // 비공개: 부호가 있고 음수는 계좌가 지급한 금액입니다.
    let paid = client
        .funding_payments(&HistoryRequest::new(market).limit(100))
        .await?;
    let net: Decimal = paid.items.iter().map(|payment| payment.amount).sum();
    println!("{net} net funding, more pages: {}", paid.has_more());

    Ok(())
}
```

## 구독

| 호출 | 답하는 것 | 인증 정보 |
| --- | --- | --- |
| `subscribe(&Subscription)` | 시장 데이터 스트림 | 불필요 |
| `subscribe_with(&Subscription, &StreamConfig)` | 연결을 조율한 같은 스트림 | 불필요 |
| `subscribe_account()` | 잔고와 주문 스트림 | 필요 |
| `subscribe_account_with(&StreamConfig)` | 연결을 조율한 같은 스트림 | 필요 |

`Subscription`은 `Subscription::new()`으로 만들고 원하는 만큼 `.market(..)`과
`.feed(..)`을 붙이거나, 순회 가능한 값에서 여러 마켓을 한꺼번에 넣는
`.markets_iter(..)`을 씁니다. 같은 마켓이나 피드를 두 번 넣어도 비용도 변화도
없습니다. 넣은 순서는 그대로 남고 거래소에 묻는 순서가 바로 그 순서입니다.

| 경우 | 일어나는 일 |
| --- | --- |
| 거기 적은 마켓과 피드 | 각각 몇 개를 넣든 연결 하나. 곱집합입니다. 마켓 셋과 피드 셋이면 소켓 하나 위에 스트림 아홉 개입니다 |
| 두 간격의 `Feed::Candles` | 교체가 아니라 피드 둘 |
| 마켓이 없거나 피드가 없음 | `Error::InvalidRequest` |
| 거래소가 발행하지 않는 피드 | 소켓을 열기도 전에 구독 전체가 `Error::Unsupported`. 목록에서 조용히 빠지지 않습니다 |
| 스트림 드롭 | 연결이 닫힙니다 |

```rust
use maxt::{Exchange, Feed, Interval, Market, Subscription};

fn majors() -> Subscription {
    let subscription = Subscription::new()
        .markets_iter(["BTC", "ETH", "XRP"].map(|base| Market::spot(Exchange::Upbit, base, "KRW")))
        .feed(Feed::Trades)
        .feed(Feed::OrderBook)
        .feed(Feed::Candles(Interval::Min1))
        // 간격이 다르면 다른 피드입니다.
        .feed(Feed::Candles(Interval::Hour1))
        // 같은 것을 다시 넣어도 비용도 변화도 없습니다.
        .feed(Feed::Trades);

    assert_eq!(subscription.markets().len(), 3);
    assert_eq!(subscription.feeds().len(), 4);
    // 거래소에 묻는 순서가 넣은 순서입니다.
    assert_eq!(subscription.feeds()[0], Feed::Trades);
    subscription
}
```

## 스트림 설정

`StreamConfig`는 실시간 연결이 나빠졌을 때의 동작을 정합니다. 필드가 모두 공개된
평범한 구조체이므로 달라질 필드만 지정하고 나머지는 `..StreamConfig::default()`로
물려받으세요.

| 필드 | 기본값 | 설명 |
| --- | --- | --- |
| `buffer_size` | 이벤트 4096개 | 뒤처진 소비자가 쌓아 둘 수 있는 이벤트 수 |
| `overflow` | `Overflow::Backpressure` | 아무것도 잃지 않습니다 |
| `max_reconnect_attempts` | `None` | 무한히 재시도합니다. `Some(n)`은 결과가 무엇이었든 재연결이 `n`번 일어나면 포기합니다 |
| `initial_reconnect_delay_ms` | 1 000 | 첫 백오프. 실패할 때마다 두 배가 되며 1 ms를 하한으로 둡니다 |
| `max_reconnect_delay_ms` | 30 000 | 두 배로 늘어나다 멈추는 상한. 1 ms를 하한으로 둡니다 |
| `idle_timeout_ms` | 30 000 | 어댑터가 올릴 수 있는 하한, [하트비트](#하트비트) 참고 |

백오프는 `initial_reconnect_delay_ms`에서 연속 실패 한 번마다 두 배가 되고
`max_reconnect_delay_ms`에서 멈춥니다. 1초, 2초, 4초, 8초를 거쳐 30초까지
갑니다. 두 필드 중 어느 쪽의 0도 1밀리초로 읽습니다. 0에서는 두 배로 늘려도
0을 벗어나지 못하고, 상한이 0이면 모든 지연이 0으로 눌리기 때문입니다.

백오프를 첫 지연으로 되돌리는 것은 핸드셰이크의 성공이 아니라 거래소가 무언가를
보낸 연결입니다. **`max_reconnect_attempts`를 되돌리는 것은 없습니다.** 실패가
아니라 재연결을 셉니다.

| 이런 재연결은 | `max_reconnect_attempts`에 셉니다 | 백오프를 되돌립니다 |
| --- | --- | --- |
| 아예 열리지 않았다 | 셉니다 | 되돌리지 않습니다 |
| 소켓은 열렸지만 거래소가 아무것도 보내지 않았다 | 셉니다 | 되돌리지 않습니다 |
| 열린 소켓으로 프레임이 하나라도 왔다 | 셉니다 | 되돌립니다 |

연결은 프레임을 날것으로 읽을 뿐 해석하지 않습니다. 그래서 거래소가 구독을
거절한 응답도 데이터와 똑같은 프레임입니다. 잘못된 심볼, 없어진 스트림 이름,
비공개 스트림에서 폐기된 인증 정보, 게이트웨이가 돌려준 HTML 에러가 모두 같은
모습으로 도착하고, 해석은 한 층 위 어댑터가 합니다. 프레임 하나로 되돌아가는
예산은 모든 연결에 거절로 답하는 거래소를 전혀 묶지 못합니다.

이 규칙이 치르는 대가입니다.

| 상황 | 일어나는 일 |
| --- | --- |
| 거래소가 제 일정대로 멀쩡한 소켓을 갈아 치웁니다 | 그 재연결도 예산을 씁니다. 그래서 유한한 `Some(n)`은 결국 멀쩡한 스트림도 끝냅니다. 거래소의 소켓 관리를 견디는 장치가 아니라, 프로세스 감독자에게 실패를 보여 주는 장치입니다 |
| 거래소가 모든 연결에 거절로 답하는데 기본값 `None`입니다 | 거절도 프레임이라 백오프를 되돌리므로, 프로세스가 사는 내내 `initial_reconnect_delay_ms` 간격으로 계속 재연결합니다. 그 응답을 읽는 것은 소비자의 몫이고, 묶는 것은 `max_reconnect_attempts`입니다 |
| 서버가 연결은 받아 주고 아무 말도 하지 않습니다 | 상한까지 백오프하고, 재연결 세 번이 연달아 살아 있는 연결을 남기지 못하면 전송 에러로 보고하며, 닿지 않는 서버와 똑같이 묶입니다 |

**어떤 지연 필드도 무엇이 세이는지를 정하지 않습니다.**
`max_reconnect_delay_ms`를 올리면 재시도가 더 느긋해질 뿐 그 밖에는 아무것도
달라지지 않습니다.

| `Overflow` | 동작 | 알맞은 곳 |
| --- | --- | --- |
| `Backpressure`(기본값) | 소비자가 따라올 때까지 소켓에서 읽지 않습니다 | 이벤트를 하나도 잃으면 안 되는 경우. 너무 오래 멈춰 있으면 거래소가 연결을 끊기도 합니다 |
| `DropNewest` | 버퍼가 가득한 동안 도착하는 이벤트를 버립니다 | 티커와 호가창 스냅숏처럼 같은 피드의 다음 이벤트가 버려진 이벤트의 내용을 통째로 다시 말해 주는 경우 |
| 가장 오래된 것을 버리는 정책은 없음 | 보내는 쪽에서는 가득 찬 큐의 앞쪽을 축출하지 못합니다. 최신 우선 의미가 필요하면 스트림을 촘촘한 루프로 비우면서 마지막에 본 이벤트만 남기세요 | 없습니다. 그런 정책 자체가 없습니다 |

피드에 던져야 할 질문은 이벤트 하나가 그 자체로 중요한가가 아니라, 같은 피드의
뒤 이벤트가 그것을 다시 말해 주는가입니다.

| 피드 | `DropNewest`가 치르는 대가 |
| --- | --- |
| `Feed::Ticker`, `Feed::OrderBook` | 낡은 값뿐이고 그 외에는 없습니다. 이벤트 하나하나가 완결된 현재 값이고 다음 이벤트가 그것을 다시 말합니다 |
| `Feed::Trades` | 조용히 모자란 합계입니다. 체결 하나하나가 뒤에 반복되지 않는 별개의 사실입니다 |
| `Feed::Candles` | 봉 자체입니다. 창 하나에는 `Candle::closed`가 선 [확정 이벤트가 하나뿐](#candleclosed)이고 그 창 자신의 최종 수치를 싣습니다. 그 뒤의 것은 모두 다음 창에 속합니다. 다시 말해 주는 이벤트가 없으므로 저장한 시계열은 봉 하나가 빈 채로 남습니다 |

**버려진 이벤트는 흔적을 남기지 않고 에러도 같은 방식으로 버려집니다.**
`DropNewest`는 계수기도, 스트림에 실리는 이벤트도, 로그 한 줄도 없이 조용히
버리고, 딱 하나를 빼면 연결이 전달하는 모든 것에 적용됩니다. 보고할 만한 재연결
실패도, `max_reconnect_attempts`가 소진될 때 나는 전송 에러도 오버플로 정책을
거칩니다. 그래서 버퍼가 가득 찬 `DropNewest` 스트림은 아무것도 보고하지 않은 채
끝나 `None`만이 끝났다는 유일한 신호로 남기도 합니다. 기다려 주지 말라고 요청한
소비자는 자기 스트림을 끝내는 실패 앞에서도 기다림을 받지
못합니다(`src/transport/ws.rs`). `buffer_size`는 그 일이 가정으로만 남을 만큼
잡으세요.

**예외는 재연결 소식입니다.** `MarketEvent::Reconnected`와
`AccountEvent::Reconnected`는 버퍼가 가득하면 버려지지 않고 붙들려 있다가,
자리가 나는 첫 이벤트보다 앞서 전달됩니다. 기다리지는 않습니다. 자리가 날
때까지 연결은 계속 읽고 데이터는 계속 버립니다. 공백이 있었다는 사실을 끝내
듣지 못한 소비자는 그 공백이 무효로 만든 호가창과 잔고를 계속 믿게 되고 뒤에
그것을 바로잡아 줄 이벤트도 없습니다. 이 소식이 주변 데이터와 다른 이유가
그것입니다.

## 재연결

`MarketEvent::Reconnected`는 연결이 끊겼다가 다시 붙었고 구독도 복구됐다는
뜻입니다. 그 사이에 거래소가 발행한 것은 모두 놓쳤고 `maxt`는 그 빈 구간을
추측하지 않습니다.

| 피드 | 빈 구간의 대가 |
| --- | --- |
| `Feed::OrderBook` | 없습니다. 모든 어댑터의 모든 호가창 이벤트는 차분이 아니라 양쪽으로 여러 단계를 담은 완결된 스냅숏입니다. 따라갈 순번도, 다시 쌓을 로컬 호가창도, 재동기화 단계도 없습니다. 이벤트마다 사본을 덮어쓰면 재연결의 비용은 놓친 메시지 하나입니다. 피드가 몇 단계를 싣는지는 각 거래소 페이지에 있습니다 |
| `Feed::Trades` | 진짜 구멍이 생깁니다. 체결이 곧 그 순서이고 끊긴 사이에 발행된 것은 다시 오지 않습니다. `trades`로 REST에서 메우고, 거래소가 채워 주는 곳에서는 겹치는 구간을 `Trade::id`로 중복 제거하세요. Binance에서는 정확히 맞습니다. 두 거래 시장 모두 `@trade`와 `/trades`가 같은 체결을 같은 식별자로 부릅니다. Hyperliquid는 최근 10건만 주므로 그보다 넓은 공백은 그곳에서 구멍으로 남습니다 |

구독 복구에는 거래소가 요구하는 경우의 재인증도 들어갑니다. 그래서 비공개
스트림은 호출자가 다시 구독하지 않아도 재연결을 넘깁니다. 각 거래소가 소켓을
어떻게 인증하는지, 토큰을 왜 핸드셰이크마다 새로 서명하는지는
[Upbit](providers/upbit.ko.md#인증-정보)과
[Bithumb](providers/bithumb.ko.md#인증-정보)에 있습니다.

`AccountEvent::Reconnected`는 둘 중 어느 쪽보다도 무겁습니다. 빈 구간에 체결이
있었을 수 있어서 로컬에 들고 있던 잔고와 미체결 주문은 이제 추측입니다. 둘 다
REST로 다시 읽으세요. 두 소식 모두 `overflow`가 무엇이든 가득 찬 버퍼에 잃히지
않으므로, 다시 읽으라는 지시는 나머지를 전부 버리고 있는 소비자에게도
닿습니다.

## 스트림 종료

스트림을 끝내는 것은 `None`뿐입니다. `Err` 항목은 스트림이 지나쳐 가며 보고하는
내용입니다. 읽지 못한 프레임이나, 일시적이라고 보기 어려워진 재연결은 구독이
계속 도는 가운데 에러로 전달되므로 첫 `Err`에서 루프를 빠져나가는 소비자는 곧
회복될 스트림을 버리는 셈입니다. `None`은
`StreamConfig::max_reconnect_attempts`가 소진되었을 때, 또는 스트림이 드롭된 뒤에
옵니다. `MarketStream`과 `AccountStream` 모두 이렇게 동작합니다.

```rust
use futures_util::StreamExt;
use maxt::adapters::UpbitAdapter;
use maxt::{Client, MarketEvent, Subscription};

async fn watch(client: &Client<UpbitAdapter>, subscription: &Subscription) -> maxt::Result<()> {
    let mut stream = client.subscribe(subscription).await?;

    // `while let Some(..)`을 씁니다. 항목에 `?`를 붙이면 스트림이 지나쳐 갈
    // 보고 하나에 루프가 끝나 버립니다.
    while let Some(item) = stream.next().await {
        match item {
            Ok(MarketEvent::Trade(trade)) => println!("{} {}", trade.price, trade.quantity),
            Ok(other) => println!("{other:?}"),
            Err(error) => eprintln!("reported, still subscribed: {error}"),
        }
    }

    Ok(()) // 스트림이 `None`이라고 말했습니다
}
```

## 하트비트

`idle_timeout_ms`는 그 시간 동안 아무 말도 없는 소켓을 닫고 다시 잇습니다. 재는
것은 **수신 쪽 침묵뿐**입니다. 거래소에서 프레임이 도착하면 타이머가 다시 걸리고
그 프레임이 소비자에게 닿으면 한 번 더 걸립니다. 그래서
`Overflow::Backpressure`를 붙들 만큼 느린 소비자가 있어도 연결이 끊기지 않고
`maxt`가 스스로 내보내는 keepalive도 기한을 밀어내지 못합니다. 답을 멈춘 소켓은
쓰기만으로 살려 둘 수 없습니다.

각 어댑터는 자기 거래소의 keepalive를 일정 간격으로 보내 유휴 타이머가 지켜보는
수신 트래픽을 만들고, 자기 거래소의 속도가 맞출 수 있는 하한까지
`idle_timeout_ms`를 올립니다. 하한보다 길게 요청하면 요청한 값을 받고 짧게
요청하면 하한을 받습니다.

| 거래소 | keepalive 간격 | 보내는 프레임 | 유휴 하한 | 왜 그 프레임인가 |
| --- | --- | --- | --- | --- |
| Upbit | 15초 | 텍스트 `PING` | 60초 | 모든 텍스트 프레임을 명령으로 읽고 이것을 처리하므로, keepalive가 Upbit 자신의 타이머를 재설정하면서 수신 트래픽으로도 돌아옵니다 |
| Bithumb | 15초 | 텍스트 `PING` | 60초 | 같습니다 |
| Hyperliquid | 15초 | `{"method":"ping"}` | 60초 | 같습니다 |
| Binance | 60초 | 프로토콜 ping | 240초 | 알 수 없는 텍스트 프레임에 에러로 답하므로 keepalive가 API 아래에 있습니다. 서버 ping을 3분마다 보내고 pong이 10분 동안 오지 않을 때에만 끊으므로 그곳에서 3분의 침묵은 건강한 소켓입니다. 하한이 있는 이유가 이것입니다. 30초 기본값은 멀쩡한 연결을 다시 붙게 만듭니다 |

## `Feature`와 `Client::supports`

`client.supports(feature)`는 인증 정보를 넣었는지까지 포함해 지금 구성된 어댑터를
기준으로 요청 없이 로컬에서 답합니다. 그 답에 따라 프로그램이 다르게 움직여야 할
때 물어보세요. 왕복도 호출 한도 토큰도 쓰지 않습니다. 어차피 프로그램이 끝날
상황이라면 `Error::Unsupported`를 잡으세요.

| `Feature` | 여는 것 | 인증 정보 | 파생상품 전용 |
| --- | --- | --- | --- |
| `Markets` | `markets` | 불필요 | 아님 |
| `Trades` | `trades` | 불필요 | 아님 |
| `OrderBook` | `order_book` | 불필요 | 아님 |
| `Ticker` | `ticker` | 불필요 | 아님 |
| `Candles` | `candles` | 불필요 | 아님 |
| `TradeStream` | `subscribe`의 `Feed::Trades` | 불필요 | 아님 |
| `OrderBookStream` | `Feed::OrderBook` | 불필요 | 아님 |
| `TickerStream` | `Feed::Ticker` | 불필요 | 아님 |
| `CandleStream` | `Feed::Candles(_)` | 불필요 | 아님 |
| `Balances` | `balances` | 필요 | 아님 |
| `OpenOrders` | `open_orders`, `open_orders_on` | 필요 | 아님 |
| `AccountStream` | `subscribe_account`, `subscribe_account_with` | 필요 | 아님 |
| `Trading` | `place_order`, `cancel_order` | 필요 | 아님 |
| `Positions` | `positions`, `positions_on` | 필요 | 맞음 |
| `Margin` | `margin_summary` | 필요 | 맞음 |
| `FundingRates` | `funding_rates` | 불필요 | 맞음 |
| `FundingPayments` | `funding_payments` | 필요 | 맞음 |
| `MarginConfig` | `set_margin` | 필요 | 맞음 |
| `ReduceOnlyOrders` | `place_order`에 붙는 `OrderRequest::reduce_only` | 필요 | 맞음 |

`ReduceOnlyOrders`는 호출이 아니라 필드를 엽니다. `.reduce_only()`로 만든 요청의
`place_order`에는 이것이 필요하고 그 필드가 없는 같은 호출에는 `Trading`만
있으면 됩니다.

`Feature::needs_credentials()`와 `Feature::is_derivatives_only()`는 const이고
어댑터 없이도 답합니다. `Feature`는 `#[non_exhaustive]`이므로 `_` 갈래를 두고
매칭하세요.

### `true`도 호출 지점에서 다시 확인해야 합니다

**`supports`는 기능에 답하고 여러분이 건넨 인자에는 답하지 않습니다.**

| 답 | 그 뒤의 테스트가 어디까지 가는지 |
| --- | --- |
| `false` | 믿을 만합니다. [`tests/unsupported_is_honest.rs`](../tests/unsupported_is_honest.rs)는 기능과 어댑터 구성의 곱집합 전체를 돌면서, 거절한 기능이 전송 에러도 인증 에러도 성공도 아닌 바로 그 기능을 지목하는 `Unsupported`로 거절하는지 단언합니다. 어댑터가 통째로 거절하는 기능이 여기에 해당하고 `false`가 뜻하는 바가 그것입니다 |
| `true` | 보이는 것보다 좁습니다. 같은 테스트가 주장된 기능은 결코 `Unsupported`로 답하지 않는지도 확인하지만 호출은 대표 인자 하나로만 해 볼 수 있습니다. 그래서 어떤 인자에는 있고 어떤 인자에는 없는 기능도 여전히 `true`로 읽힙니다 |

실제로 배포된 사례가 셋입니다.

| `true`로 읽히지만 | 여전히 `Unsupported`로 거절 |
| --- | --- |
| `Feature::Candles` | 거래소의 REST 간격 집합 밖에서. 예를 들어 Binance USD-M의 `Interval::Sec1` |
| `Feature::CandleStream` | 거래소가 REST로는 제공하지만 스트림으로는 보내지 않는 간격에서. Upbit의 `Day1`, `Week1`, `Month1` `Feed::Candles`가 그렇고 전용 테스트 `a_candle_interval_upbit_does_not_stream_is_refused`가 이를 단언합니다 |
| Hyperliquid의 `Feature::FundingRates`, `Feature::ReduceOnlyOrders`를 비롯한 파생상품 쪽 전부 | 건넨 마켓이 현물일 때. Hyperliquid 어댑터 하나가 두 종류를 함께 상장하므로 기능은 어댑터가 싣지 그 위의 모든 마켓이 싣지 않습니다 |

그래서 `supports(Feature::CandleStream)`을 읽고 Upbit 일봉 캔들을 구독으로 보내는
라우터는 `subscribe`에서 죽는 구독을 만듭니다. 기능으로는 거래소를 고르되 호출
지점에서는 그래도 `Error::Unsupported`를 처리하세요. REST 간격 집합과 스트림 간격
집합은 거래소마다 다르고 각 거래소 페이지에 적혀 있으며 어떤 `Feature`도 둘을
가르지 않습니다.

## `Error`

| 배리언트 | 뜻 | 같은 요청을 재시도? |
| --- | --- | --- |
| `InvalidRequest { field, detail }` | 요청이 잘못됐고 프로세스를 떠나기 전에 거절됨 | 안 됨 |
| `Unsupported { feature, exchange, detail }` | `maxt`가 그곳에 매핑한 엔드포인트가 없음. 키를 넣어도 달라지지 않음 | 안 됨 |
| `Auth { detail }` | `maxt`가 인증 정보를 담은 요청을 만들지 못해 아무것도 보내지 않음: 없거나, 형식이 틀렸거나, 서명에 쓸 수 없음 | 안 됨 |
| `Exchange { exchange, code, message, status, kind }` | 거래소가 답했고 거절함. 거래소 자신의 코드와 메시지를 그대로 보존. *거래소가* 거부한 인증 정보는 `Auth`가 아니라 여기 | `kind`에 따라 다름 |
| `Transport { detail }` | 연결 실패: DNS, TLS, 소켓, 타임아웃 | 됨 |
| `Decode { detail }` | 거래소가 답했으나 `maxt`가 읽지 못한 응답 | 안 됨. 버그로 신고하세요 |

| 판별 | 참인 경우 |
| --- | --- |
| `is_retryable()` | 위 표의 마지막 열을 한 호출로 접은 것 |
| `is_rate_limited()` | 일시적인 연결 문제보다 더 긴 대기를 요구하는, 거래소 자신의 "요청이 너무 많다"는 판정 |

```rust
use maxt::{Client, Error, Market, adapters::HyperliquidAdapter};

async fn print_last_price(client: &Client<HyperliquidAdapter>, market: &Market) {
    match client.ticker(market).await {
        Ok(ticker) => println!("{}", ticker.last_price),
        Err(error) if error.is_rate_limited() => println!("slow down: {error}"),
        Err(error) if error.is_retryable() => println!("try again behind a backoff: {error}"),
        Err(Error::Unsupported { feature, exchange, .. }) => {
            println!("{exchange} does not offer {feature}; nothing to retry")
        }
        Err(error) => println!("give up: {error}"),
    }
}
```

인증 정보가 없을 때는 모든 어댑터에서 `Error::Unsupported`가 아니라
`Error::Auth`입니다. `Auth`는 엔드포인트는 있는데 키가 없다는 뜻이라 키를 넣으면
해결되고, `Unsupported`는 `maxt`가 그곳에 매핑한 엔드포인트가 없다는 뜻이라 키를
넣어도 달라지지 않습니다. 인증 정보
없이 만든 어댑터는 `client.supports(Feature::Balances)`에 `false`를 답하고 호출
자체도 네트워크에 닿기 전에 `Auth`로 실패합니다.

### 거래소가 거부한 인증 정보

`Auth`가 그어지는 선은 인증 정보가 아니라 프로세스 경계입니다. `maxt`가 보냈고
거래소가 읽고 거부한 키는 그 거래소 자신의 코드를 달고 `Error::Exchange`로
돌아옵니다.

| 한 일 | 돌아오는 것 |
| --- | --- |
| 인증 정보 없이 어댑터를 만듦 | 아무것도 보내기 전에 `Error::Auth` |
| `maxt`가 서명에 쓸 수 없는 키를 넣음 | 아무것도 보내기 전에 `Error::Auth` |
| 틀렸거나 폐기된 키를 넣음 | 거래소의 코드를 담은 `Error::Exchange` |

`maxt`는 셋째 줄을 둘째 줄로 접지 않습니다. 거부된 인증 정보를 어떻게 표기하는지
부터 네 거래소가 서로 다르기 때문입니다.

| 거래소 | 거부된 인증 정보 | 근거 |
| --- | --- | --- |
| Binance | 서명이 틀리면 HTTP 400 `-1022`, 키가 틀리면 HTTP 401 `-2015`, 키가 없으면 HTTP 401 `-2014` | 2026-07-31 측정 |
| Upbit | HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_access_key`, `nonce_used`, `no_authorization_ip`, `no_authorization_token`, HTTP 403 `out_of_scope` | Upbit가 공개한 표 |
| Bithumb | HTTP 401 `jwt_verification`, `invalid_query_payload`, `expired_jwt`, `NotAllowIP`, `out_of_scope` | Bithumb이 공개한 표 |
| Hyperliquid | HTTP 200에 `status: "err"`, 요청마다 주소가 바뀌는 영어 문장, 코드는 아예 없음 | Hyperliquid가 공개한 서명 안내 |

HTTP 상태만 보는 규칙이라면 Binance의 틀린 서명은 거절로, 틀린 키는 인증 실패로
갈라 놓고 Hyperliquid에서는 한 번도 걸리지 않습니다. 코드로 쓰는 규칙이라면 목록이
넷이고 그중 셋은 여기서 확인할 수 없으며, 어느 거래소가 코드 이름을 바꾸는 순간
조용히 낡습니다. 그래서 코드를 그대로 전달하고 분기는 호출자에게 맡깁니다. 각
거래소 문서의 **뜻밖의 것들**에 그 거래소의 목록이 있습니다.

### `is_retryable()`과 수신 창을 벗어난 시각

`is_retryable()`이 묻는 것은 *똑같은* 요청을 다시 보내면 성공할 수 있느냐입니다.
서명된 요청은 서명할 때의 시각을 그대로 지니므로, 거래소가 수신 창을 벗어났다고
거부한 시각은 시계 오차가 일시적인 조건인데도 여기서 `false`입니다.

| 원인 | 해결 |
| --- | --- |
| 이 기계의 시계가 틀림 | 시계를 맞추세요. 맞출 때까지 다시 만든 요청도 매번 실패하고, 재시도 루프는 그 사실을 배우는 데 예산을 다 씁니다 |
| 요청이 가는 길에 지연됨 | 요청을 다시 만들어 한 번 보내세요. `maxt`는 요청을 만들 때 시계를 읽으므로 새 요청은 새 시각을 지닙니다 |

둘 다 재시도 루프가 아니고, `is_retryable()`이 허가하려는 것이 바로 그 루프입니다.

## `Decimal`

| 항목 | 뜻 |
| --- | --- |
| 모든 가격, 수량, 금액 | `rust_decimal::Decimal`이고, 호출자가 실수로 다른 버전을 쓰는 일이 없도록 `maxt::Decimal`로 다시 내보냅니다 |
| `Decimal`이 정확히 담지 못하는 숫자 | 반올림된 값이 아니라 `Error::Decode`입니다. 자릿수가 넘치는 평범한 경우와 거래소가 작은 값에 쓰는 지수 표기가 똑같이 여기에 들어갑니다. `1e-30`은 조용한 0이 아니라 `Error::Decode`이고 자릿수가 넘치는 `1.234…e5`도 `Error::Decode`입니다. `Decimal::from_scientific`이었다면 둘 다 반올림했을 값입니다 |
| 그 규칙이 있는 곳 | 모든 어댑터가 거쳐 가는 하나의 공용 리더이므로 네 거래소에서 똑같이 성립합니다(`src/adapters/decimal.rs`) |

`f64`는 대부분의 십진 가격을 정확히 표현하지 못합니다. 체결을 더하고, 지정가를
호가창의 한 단계와 비교하고, 잔고를 주문 크기와 견주는 일을 이진 부동소수점으로
하면 오차가 쌓이고 그 오차는 거래소가 거절하는 주문이나 대사가 맞지 않는
포지션으로 드러납니다. 부동소수점이 정말로 필요한 계산이라면 들어오는 길목이
아니라 경계에서 변환하세요.

## `Timestamp`

| 항목 | 뜻 |
| --- | --- |
| 단위 | Unix epoch 기준 나노초, UTC |
| 단위를 하나로 둔 이유 | 거래소들은 초, 밀리초, 마이크로초, 나노초를 제각각 발행합니다. 한 해상도로 모아야 서로 다른 거래소의 이벤트를 시간순으로 놓습니다 |
| `Display` | UTC 밀리초 정밀도의 RFC 3339. `Timestamp::from_millis(1_700_000_000_000)`은 `2023-11-14T22:13:20.000Z`로 찍힙니다 |
| 왕복 | 밀리초 미만은 찍히지 않으므로 정확한 값이 중요할 때는 `as_nanos`를 쓰세요 |
| `chrono`나 `time`의 타입이 아닌 이유 | `maxt`는 날짜·시간 라이브러리를 강요하지 않습니다. `as_nanos`나 `into_system_time`으로 경계에서 변환하세요 |
| 값을 지어내야만 채울 수 있는 필드 | 대신 `None`. Binance와 Hyperliquid에서 `Ticker::last_trade_time`이 `None`인 것은 둘 다 마지막 가격이 언제 체결됐는지 말하지 않기 때문입니다 |

어떤 응답은 거래소 시계 대신 읽은 시각을 담습니다. 거래소가 시계를 함께
발행하지 않기 때문입니다. Binance 현물 호가창과 Hyperliquid 티커가 그렇고 각
필드의 문서에 그렇게 적혀 있습니다. 그런 타임스탬프는 데이터 나이의 상한으로
다루세요. 이 값으로 신선도를 재면 실제보다 짧게 나옵니다.

## `Market`과 `MarketKind`

`Market`은 거래소, 종류, 기준 자산, quote 자산으로 이루어지고 자산 이름은
대문자로 바뀝니다.

| 항목 | 하는 일 |
| --- | --- |
| `Market::spot(exchange, base, quote)` | 현물 마켓 |
| `Market::perpetual(exchange, base, quote)` | 무기한 선물 마켓 |
| `Market::new(exchange, kind, base, quote)` | 종류를 명시한 마켓. 종류 자체가 실행 시점 값일 때 씁니다 |
| `Display` | `binance:BTC/USDT`, 무기한 선물은 `binance:BTC/USDT:perp` |
| `Hash`, `Ord` | 별도 래퍼 없이 맵 키로 씁니다 |
| `MarketKind::is_derivative()` | `Client`의 파생상품 절반이 의미를 갖는 마켓 |

앞의 둘을 먼저 쓰세요. `Market::new`는 종류가 소스가 아니라 설정이나 명령줄에서
올 때 쓰는 생성자입니다. 타입 있는 어댑터 메서드를 잃어 가면서까지
`Client<Box<dyn Adapter>>`를 쓸 만한 상황도 바로 그 상황입니다.

```rust
use maxt::{Exchange, Market, MarketKind};

fn market_from_config(kind: &str) -> Option<Market> {
    let kind = match kind {
        "spot" => MarketKind::Spot,
        "perp" => MarketKind::Perpetual,
        _ => return None,
    };
    // 소문자로 넣어도 대문자로 나옵니다. 자산 이름은 생성자가 정규화합니다.
    Some(Market::new(Exchange::Binance, kind, "btc", "usdt"))
}

fn check() {
    let perp = market_from_config("perp").expect("a perpetual");
    assert_eq!(perp.to_string(), "binance:BTC/USDT:perp");
    // 현물 종류를 넘긴 `new`는 `spot`이 만드는 것과 똑같이 만듭니다.
    assert_eq!(
        market_from_config("spot"),
        Some(Market::spot(Exchange::Binance, "BTC", "USDT"))
    );
    assert_eq!(market_from_config("futures"), None);
}
```

같은 페어의 현물과 무기한 선물은 플래그 하나가 다른 한 마켓이 아니라 서로 다른
마켓입니다. 가격이 다르고 호가창이 다르며 Binance에서는 호스트도 잔고도
다릅니다. 종류를 정체성의 일부로 둔 덕분에 무기한 선물 수량을 현물 가격과 견주는
일이 막힙니다.

## `Page`와 `Cursor`

펀딩 비율과 펀딩 지급 이력은 페이지 단위로 옵니다.

| 항목 | 하는 일 |
| --- | --- |
| `Page::next` | 더 있으면 `Some(cursor)`, 이력의 끝에서는 `None` |
| `Page::has_more()` | 같은 질문에 `bool`로 답합니다 |
| `Cursor` | 불투명합니다. 거래소가 만들고 그 거래소만 읽으니 그대로 돌려주기만 하고 파싱하지 마세요 |
| `Cursor::as_str()` | 내용물. 실행과 실행 사이에 위치를 저장할 때 씁니다 |
| `Cursor::new(string)` | 돌아오는 길. 저장해 둔 문자열을 다시 커서로 감쌉니다 |
| `HistoryRequest::cursor(cursor)` | 그 커서에서 순회를 이어 갑니다 |
| `HistoryRequest::limit` | 총량이 아니라 페이지 크기. 어댑터마다 상한을 두고 기본값을 두기도 하며 각각은 거래소 페이지에 있습니다 |

| 페이지 길이 | 예상할 것 |
| --- | --- |
| 짧거나 빈 페이지 | 신호가 아닙니다. 페이지는 다른 이유로도 짧아지므로 순회를 끝내는 것은 커서가 없다는 사실뿐입니다. 짧은 페이지에서 멈추면 이력이 잘립니다 |
| `limit`이 비어 있을 때 | Binance는 100을 기본값으로 씁니다. Hyperliquid는 개수를 아예 받지 않으므로 한 번에 500개를 읽고 여러분의 `limit`에 맞춰 잘라 냅니다 |
| **`limit`보다 길 때** | Hyperliquid에서 가능합니다. 다음 커서는 남긴 마지막 항목보다 1밀리초 뒤에서 이어지므로, 같은 밀리초를 공유하는 항목 묶음 안에서 자르면 그 묶음의 나머지가 버려집니다. 대신 묶음이 시작하는 자리까지 물러나 자르고, 묶음이 페이지 맨 앞까지 닿아 물러날 곳이 없으면 묶음 전체를 남깁니다(`src/adapters/hyperliquid/rest.rs`). 호출자가 버리면 되는 항목 몇 개가 커서가 이미 지나쳐 버린 항목보다 낫습니다 |

버퍼 크기는 `limit`이 아니라 페이지가 돌려준 개수에 맞춰 잡으세요.

```rust
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Cursor, Exchange, HistoryRequest, Market, Timestamp};

/// `start`부터 현재까지의 모든 펀딩 비율.
///
/// 이 순회를 전체 이력으로 만드는 것이 `from`입니다. 커서는 *앞으로* 걸어가므로
/// `from`이 비어 있으면 Binance는 가장 최근 페이지를 주고, 첫 커서는 이미 이력의
/// 끝을 넘어간 자리를 가리키며, 순회는 왕복 두 번 뒤에 그 한 페이지만 읽고
/// 끝납니다.
async fn funding_rates_since(start: Timestamp) -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::usd_m_futures());
    let mut request = HistoryRequest::new(Market::perpetual(Exchange::Binance, "BTC", "USDT"))
        .from(start)
        .limit(100);

    loop {
        let page = client.funding_rates(&request).await?;
        for rate in &page.items {
            println!("{} {}", rate.timestamp, rate.rate);
        }

        let Some(cursor) = page.next else { break };
        request = request.cursor(cursor);
    }

    Ok(())
}

/// 프로세스를 두 번 실행해 같은 구간을 이어서 걷습니다.
///
/// 위치를 꺼내는 것이 `as_str`, 되돌리는 것이 `Cursor::new`입니다. 문자열만
/// 저장하고 그 생김새는 저장하지 마세요. 어느 거래소의 커서는 타임스탬프이고
/// 어느 거래소의 커서는 주문 식별자이며 둘 다 약속은 아닙니다.
async fn resume(client: &Client<BinanceAdapter>, saved: Option<String>) -> maxt::Result<Option<String>> {
    let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
    let mut request = HistoryRequest::new(market).limit(100);
    if let Some(saved) = saved {
        request = request.cursor(Cursor::new(saved));
    }

    let page = client.funding_payments(&request).await?;
    println!("{} entries, more to come: {}", page.items.len(), page.has_more());

    Ok(page.next.map(|cursor| cursor.as_str().to_string()))
}
```

## 호출 한도

`maxt`는 속도를 조절하지 않습니다. 클라이언트 쪽 제한기도, 요청 큐도, 자동
백오프도 없습니다. 호출하면 그대로 나가고 속도 조절은 여러분의 몫입니다.

| 거래소 | 공개된 한도 | 출처 |
| --- | --- | --- |
| Upbit | 공개 시세는 IP당 초당 10회, 비공개 조회는 계정당 초당 30회, 주문은 초당 8회 | [호출 한도](https://global-docs.upbit.com/reference/rate-limits) |
| Bithumb | 공개 초당 150회, 비공개 초당 140회, 주문은 그 위에 초당 10회로 추가 제한 | [API 요청 수 제한 안내](https://apidocs.bithumb.com/docs/api-요청-수-제한-안내) |
| Binance | 요청 수가 아니라 IP당 구간당 요청 가중치 예산이고 엔드포인트마다 고유한 가중치가 붙습니다. 문서는 고정된 상한을 밝히지 않습니다. 지금 값은 현물 `/api/v3/exchangeInfo`, USD-M `/fapi/v1/exchangeInfo`의 `rateLimits` 배열에 있습니다 | [Spot REST limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits), [USD-M REST limits](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info) |
| Hyperliquid | IP당 분당 가중치 1,200. 여기에 주소가 만들어진 뒤 거래한 USDC 1당 요청 1회의 주소별 예산이 더해지며 시작값은 10,000입니다. **주소별 예산은 action만 셉니다.** `maxt`의 호출 중에서는 `place_order`, `cancel_order`, `set_margin`만 여기서 차감되고 나머지는 차감되지 않습니다 | [호출 한도](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits) |

| 직접 속도를 조절할 때 | 알아 둘 것 |
| --- | --- |
| 가중치는 요청 수가 아님 | Binance와 Hyperliquid의 값은 가중치이므로 프로그램이 실제로 쓸 수 있는 양은 어떤 엔드포인트를 부르는지에 달려 있습니다. 두 거래소 모두 엔드포인트마다 가중치를 공개합니다 |
| 누적값 읽기 | Binance는 응답마다 `X-MBX-USED-WEIGHT-(intervalNum)(intervalLetter)` 헤더로 돌려주고 1분 제한기의 헤더는 `X-MBX-USED-WEIGHT-1M`입니다. `maxt`는 이 값을 읽지 않으므로 예산에 맞춰 조절하려는 프로그램은 직접 읽습니다 |
| Bithumb의 기준 | 밝히지 않았습니다. 더 엄격한 해석인 IP 기준으로 다루세요 |
| 네 거래소에 통하는 백오프 | `is_rate_limited()`이면 잠들었다가 재시도하되 대기 시간을 상한까지 매번 두 배로 늘리세요. Binance는 `429`를 계속 무시하는 IP를 차단합니다 |
| 묶어 부를 수 있는 곳에서는 묶기 | Upbit에 티커 서른 개를 한 요청으로 물으면 초당 열 번 중 서른 번이 아니라 한 번을 씁니다. [`Client::adapter`](#clientadapter)로 하는 일입니다. 그 마켓 목록의 길이에 `maxt`도 상한을 두지 않고 Upbit도 상한을 공시하지 않으므로, URL이 버거울 만큼 긴 목록은 Upbit이나 그 앞단이 거절하고 `Error::Exchange`로 도착합니다. 쓸 수 있는 상한은 직접 찾으세요. [Upbit](providers/upbit.ko.md#upbit-전용-호출)을 보세요 |

## `Client::adapter`

`Client::adapter()`는 어댑터를 그대로 돌려주고 그 거래소가 공통 API 밖에 둔
타입 있는 메서드들을 함께 넘겨줍니다. 이것이 탈출구이고
`Client::into_adapter()`는 어댑터 자체로 되돌려 줍니다.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Market};

// 서른 번이 아니라 한 번의 요청. Upbit은 여러 마켓을 한 번에 답하고 할당량은
// 요청 수로 셉니다.
async fn tickers(client: &Client<UpbitAdapter>, markets: &[Market]) -> maxt::Result<()> {
    println!("{} tickers", client.adapter().tickers(markets).await?.len());
    Ok(())
}
```

둘 다 구체 어댑터 타입이 있어야 합니다. `Client<Box<dyn Adapter>>`는
`&Box<dyn Adapter>`를 돌려주고 거기에는 공통 API밖에 없습니다. 실행 시점에
거래소를 고르는 대가입니다.

각 거래소가 무엇을 자기 어댑터에 남겨 두었고 왜 그랬는지는 해당 거래소 페이지에
있습니다. [Upbit](providers/upbit.ko.md), [Bithumb](providers/bithumb.ko.md),
[Binance](providers/binance.ko.md),
[Hyperliquid](providers/hyperliquid.ko.md).
