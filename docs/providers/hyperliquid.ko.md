[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

# Hyperliquid

체인에서 정산하는 거래소 한 곳에 현물과 무기한 선물 마켓이 함께 있습니다.
`HyperliquidAdapter` 하나가 둘 다 담당하고 구분은 `Market.kind`가 담습니다.

```rust
use maxt::{Client, Feature, adapters::HyperliquidAdapter};

let client = Client::new(HyperliquidAdapter::new());
let testnet = Client::new(HyperliquidAdapter::testnet());

assert!(client.supports(Feature::TradeStream)); // 실시간: 모든 체결
assert!(client.supports(Feature::Trades));      // REST: 최근 10건
```

## 거래 시장

| 항목 | 현물 | 무기한 |
| --- | --- | --- |
| `Market` 생성자 | `Market::spot`, 토큰 표를 거쳐 해석 | `Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC")`, USDC로 정산하는 코인 이름 그 자체 |
| 자체 심볼 | `@107`, 또는 `PURR/USDC` 같은 옛 슬래시 표기. 둘 다 해석됩니다 | 코인 이름 |
| 페어 해석 | 어댑터가 첫 호출에서 읽어 수명 내내 들고 있는 토큰 표가 결정합니다. 그 뒤에 상장된 마켓은 어댑터를 새로 만들어야 잡힙니다. 페어는 `markets(MarketKind::Spot)`으로 찾으세요. Hyperliquid 화면과 대조할 때는 `MarketInfo::native_symbol`을 씁니다 | 필요 없음 |
| 잔고 | `balances()` | `margin_summary()`, USDC 기준 |
| `funding_rates`, `funding_payments` | 해당 기능을 지목하는 `Error::Unsupported` | 지원 |
| `set_margin` | `Feature::MarginConfig`를 지목하는 `Error::Unsupported` | 지원 |
| 주문의 `reduce_only` | `Feature::ReduceOnlyOrders`를 지목하는 `Error::Unsupported` | 지원 |
| `positions_on(&market)` | 오류가 아니라 `Ok(vec![])`. 무기한 선물 계좌를 읽어 거른 결과입니다. 포지션이 빈 무기한 마켓과 반환값이 같으므로 `Market::kind`로 분기하세요 | 포지션이 있으면 그 포지션 |
| 상장되지 않은 마켓에 `positions_on` | `market`을 지목하는 `Error::InvalidRequest` | `market`을 지목하는 `Error::InvalidRequest` |

## 상한

요청을 만들기 전에 검사합니다.

| 호출 | 허용 범위 | 범위 밖 |
| --- | --- | --- |
| `trades` | `limit` 10까지, 비워두면 10건 전부. `recentTrades`는 개수를 받지 않아 10건이 이 엔드포인트의 전부입니다. 그보다 넓은 공백은 `Feed::Trades`로만 메웁니다 | 10을 넘으면 `limit` 필드의 `Error::InvalidRequest` |
| `order_book` | `depth` 1~20 | `Error::InvalidRequest` |
| `candles` | `limit`은 제한 없음. 호출당 5,000개이고 최대 100번의 호출까지 페이지를 대신 넘깁니다 | 캔들 500,000개를 넘는 구간은 `Error::InvalidRequest` |
| 캔들 간격 | 열네 개. [기준선](../common-api.ko.md#간격) 열 개에 `Hour2`, `Hour8`, `Hour12`, `Day3`를 더한 것 | `Interval::Sec1`은 `candles`와 `Feed::Candles` 양쪽 모두에서 `Error::Unsupported` |
| `Sec1`에 잘못된 `limit`이나 구간이 겹칠 때 | `limit`과 구간을 먼저 검사합니다 | `Unsupported`가 아니라 해당 필드의 `Error::InvalidRequest` |
| 주문 가격 | 해당 자산의 가격 소수 자릿수까지, 소수부가 있으면 유효숫자 5자리까지 | 해당 필드를 지목하는 `Error::InvalidRequest` |
| 주문 수량 | 해당 자산 자신의 소수 자릿수까지 | 해당 필드를 지목하는 `Error::InvalidRequest` |
| `funding_rates`, `funding_payments`, `non_funding_ledger` | 페이지당 500건 | `Page::next`가 `None`이 될 때까지 따라가세요 |

## 주문 정밀도와 최소 주문 크기

두 규칙 모두 서명 전에 강제하며 허용 자릿수는 오류 메시지에 담깁니다. 두
자릿수 모두 `HyperliquidAssetContext`에서 나오므로 주문을 처음부터 맞춰 만들 수
있습니다.

```rust
use maxt::{Client, Exchange, Market, adapters::HyperliquidAdapter};

async fn precision() -> maxt::Result<()> {
    let client = Client::new(HyperliquidAdapter::new());
    let market = Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC");
    let context = client.adapter().asset_context(&market).await?;

    println!(
        "sizes take {} decimals, prices {}",
        context.size_decimals, context.price_decimals
    );
    Ok(())
}
```

| 필드 | 값 |
| --- | --- |
| `price_decimals` | 현물 8, 무기한 선물 6에서 해당 자산의 수량 자릿수를 뺀 값. 유효숫자 5자리 규칙은 이 값으로 표현되지 않습니다. 소수부가 있는 모든 가격에 그 위로 겹쳐 적용됩니다 |
| `size_decimals` | 해당 자산 자신의 소수 자릿수 |
| 최소 주문 금액 | Hyperliquid 자신의 규칙이고 `maxt`는 검사하지 않습니다 |

## 스트림

| 대상 | 동작 |
| --- | --- |
| `Feed::OrderBook` | Hyperliquid의 `fast` 플래그 없는 `l2Book`. 한 쪽당 20단계이며 REST와 같은 깊이, 변경 불가 |
| 호가창 이벤트 | 차분이 아니라 전체 스냅숏. 사본을 덮어쓰세요 |
| `nSigFigs`, `mantissa` | Hyperliquid의 가격 집계. 공통 API에는 이를 담을 필드가 없습니다 |
| `Candle::closed` | 한 구간당 한 번만 `true`입니다. Hyperliquid가 다음 구간을 열 때 보냅니다. 아래를 보세요 |
| 한 구간당 캔들 이벤트 | `closed`가 false인 이벤트가 여러 번, 그다음 `true`인 이벤트가 정확히 한 번. 확정 이벤트는 그 구간 마지막 프레임의 값을 그대로 옮깁니다 |
| 재연결 | 붙들고 있던 구간을 버립니다. 그래서 `MarketEvent::Reconnected`가 끊은 구간은 `closed` 이벤트를 받지 못합니다 |
| `subscribe_account` | `balances()`와 마찬가지로 현물 잔고 |
| keepalive | 15초마다 `{"method":"ping"}` |

### 스트림에서의 `Candle::closed`

**Hyperliquid는 한 구간이 자기 종료 시각에 이르기 약 2초 전에 그 구간 발행을
멈춥니다. 그래서 그 구간의 마지막 프레임이 도착하는 시점에도 페이로드의 `T`는
아직 미래입니다.**

| `candle` `1m` `BTC` 프레임, 2026-07-30 | `t` | `T` | 수신 시각 |
| --- | --- | --- | --- |
| 07:45 구간이 받은 마지막 프레임 | 07:45:00.000 | 07:45:59.999 | 07:45:57.557 |
| 다음 구간의 첫 프레임 | 07:46:00.000 | 07:46:59.999 | 07:46:01.416 |

210초 동안 다섯 구간을 읽어 보면 한 구간의 마지막 프레임은 자기 `T`보다 1.7초,
2.1초, 2.4초 앞서 도착했고 한 번만 0.3초 뒤에 도착했습니다. 그래서 `maxt`는
마켓과 간격마다 가장 최근 프레임을 붙들고 있다가 더 뒤 구간을 여는 프레임이 오면
`closed`를 표시해 내보냅니다. 확정 이벤트는 한 프레임 늦게 도착합니다. 뒤따르는
구간이 없는 구간은 끝내 확정되지 않습니다.

## 요청 할당량

Hyperliquid가
[공개한 한도](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)는
IP 기준과 주소 기준입니다. 주소 기준 예산은 나머지 세 거래소에 대응물이 없습니다.

| 예산 | 허용량 |
| --- | --- |
| IP 기준 | 모든 REST 요청을 합쳐 분당 가중치 1,200. 가중치는 호출 단위가 아니라 엔드포인트 단위입니다 |
| 주소 기준 | 주소가 만들어진 뒤 누적으로 거래한 USDC 1당 *action* 1회, 그 위에 시작 여유분 10,000이 얹힙니다. 다 쓰면 10초에 한 번으로 조여집니다. 취소에는 누적 허용량을 더 크게 줍니다 |
| WebSocket | 연결 10개, 분당 새 연결 30개, 구독 1,000개, 분당 송신 메시지 2,000개, 처리 중인 post 메시지 100개 |

| 가중치 | 부과 대상 |
| --- | --- |
| 2 | `l2Book`, `allMids`, `clearinghouseState`, `orderStatus`, `spotClearinghouseState`, `exchangeStatus` |
| 20 | 문서에 있는 나머지 info 요청 전부 |
| 60 | `userRole` |
| 반환 항목 20개마다 1 | 페이지를 넘기는 엔드포인트, 자체 가중치에 더해서 |
| 캔들 60개마다 1 | 캔들 스냅숏, 자체 가중치에 더해서 |

주소별 예산은 조회가 아니라 action에서 차감됩니다. Hyperliquid 문서가 그대로
적어 두었습니다. 주소 기준 한도는 "info 요청이 아니라 action에만 적용됩니다".
조회는 모두 `POST /info`로, 상태를 바꾸는 동작은 모두 `POST /exchange`로
갑니다.

| 주소별 예산에서 차감 | 차감되지 않음 |
| --- | --- |
| `place_order`, `cancel_order`, `set_margin` | `balances`, `positions`, `positions_on`, `open_orders`, `open_orders_on`, `margin_summary`, `funding_rates`, `funding_payments`, `non_funding_ledger`, 그리고 모든 공개 조회 |

묶음 주문은 IP 예산에서 요청 하나로, 주소별 예산에서 `n`회로 셉니다. `maxt`는
action 하나에 주문 하나를 보냅니다. `maxt`는 이 중 어느 것에도 맞춰
[속도를 조절하지 않으며](../common-api.ko.md#호출-한도), 자체 keepalive가 연결
하나당 분당 송신 메시지 2,000개 중 4개를 씁니다.

## 주의할 점

| 필드 또는 호출 | 동작 |
| --- | --- |
| HTTP 200 | 거절을 담을 수 있습니다. 주문, 취소, 레버리지 변경의 실패가 성공 상태 코드의 본문에 담깁니다. 이미 `ok`라고 답한 응답 안에 개별 주문의 거절이 들어 있기도 합니다. 어댑터는 두 곳을 모두 읽어 `Error::Exchange`로 올립니다 |
| `Ticker::timestamp` | `maxt`가 읽은 시각. asset context에는 시계가 없습니다 |
| `Ticker::last_trade_time` | 같은 이유로 `None` |
| `Ticker::high`, `low` | `None`. asset context에는 세션 고가와 저가가 없습니다 |
| `Interval::Month1` | 달력상의 한 달이 아니라 유닉스 에포크부터 세는 고정 격자 위의 30일 구간입니다. 시작 시각은 2026-05-07, 2026-06-06, 2026-07-06으로 이어지고 모두 00:00 UTC이며 모든 마켓이 같은 경계를 씁니다. 6월 구간은 없고 7월 1일에 닫히는 구간도 없습니다. `maxt`는 Hyperliquid 자신의 `open_time`을 그대로 전합니다 |
| `Interval::Week1` | 같은 에포크부터 세는 7일 구간이라 Upbit이나 Binance처럼 월요일이 아니라 **목요일** 00:00 UTC에 열립니다. 1970년 1월 1일이 목요일이었습니다 |
| `Interval::Day3` | 같은 에포크부터 세는 3일 구간이고 00:00 UTC에 열립니다. Binance의 `Day3`는 이보다 하루 앞섭니다 |
| `Interval::Day1`, `Hour12` 및 그보다 짧은 간격 | Upbit, Binance와 같은 UTC 격자. 에포크부터 세는 간격은 `Day3`, `Week1`, `Month1` 셋뿐입니다 |
| `candleSnapshot`의 `endTime` | Hyperliquid의 시기마다 뜻이 다릅니다. 대략 2023년 중반 이전 구간은 `endTime`이 그 구간의 종료 시각에 이르러야 응답에 들어옵니다. 그보다 최근 구간은 `endTime`이 시작 시각에 이르는 순간 들어옵니다. `maxt`는 한 간격 더 뒤까지 요청한 뒤 응답을 잘라냅니다. `from`과 `limit`을 함께 주면 그 경계의 어느 쪽에서도 요청한 개수가 그대로 돌아옵니다 |
| `balances()` | 현물만. 무기한 선물의 증거금은 `margin_summary()`가 USDC 기준으로 보고합니다 |
| `MarginSummary::available_balance` | 여유를 좌우하는 값이자 새 포지션의 크기를 재는 기준. `margin_balance`는 이미 걸어 둔 증거금이고 `equity`에는 미실현 손익이 들어갑니다 |
| 주문 타입 | 지정가만, `Size::Base`만, 가격은 필수. 시장가 주문이 없으니 호가창을 관통하는 가격의 immediate-or-cancel 지정가 주문을 보내세요. fill-or-kill도 없습니다 |
| 주문 접수 응답 | 체결됐는지 호가창에 남았는지만 말하고 얼마나 체결됐는지는 말하지 않습니다. 체결은 전량 체결로, 대기는 미체결로 읽힙니다 |
| 취소 응답 | 결과만 담습니다. 반환된 `Order`의 방향, 수량, 가격은 자리를 채운 값이니 `open_orders`로 주문을 다시 읽으세요 |
| `open_orders_on`, `positions_on` | 로컬에서 거릅니다. Hyperliquid는 두 질문 모두 계좌 전체로 답하고 마켓 인자를 받지 않습니다 |
| 닫힌 포지션 | 크기 0으로 보고되는 것이 아니라 `clearinghouseState`에 아예 오르지 않습니다 |
| `set_margin` | 한 번의 동작이라 레버리지와 마진 모드를 둘 다 주어야 하고 하나만 주면 `Error::InvalidRequest`입니다. 레버리지는 해당 자산의 상한 이내의 정수이며 일부 자산은 isolated 마진만 받습니다 |
| 빌더가 배포한 무기한 마켓 | 이름에 콜론이 들어가는 마켓입니다. 별도 universe에 자체 asset 번호 체계를 쓰고 `markets()`에는 오르지 않습니다 |
| 지갑 형식 | `with_wallet`이 아니라 지갑이 필요한 첫 호출이 검사합니다. `with_wallet` 자체는 실패하지 않습니다 |
| 지갑 없음 | `Error::Auth`이고 `Client::supports`는 모든 계좌 기능에 `false`로 답합니다 |
| Hyperliquid가 거부한 서명 | `Error::Auth`가 아니라 `Error::Exchange`. HTTP 200에 `{"status":"err","response":"User or API Wallet 0x… does not exist."}`로 답합니다. 그 주소는 틀린 서명에서 복원된 값이라 요청마다 달라집니다. 분기할 코드 자체가 없습니다. **측정이 아니라 문서 기준입니다** |

## Hyperliquid 전용 호출

`Client::adapter()`를 통해 호출합니다.

| 메서드 | 반환값 |
| --- | --- |
| `non_funding_ledger(from, to, cursor, limit)` | 입금, 출금, 지갑과 서브계정 사이의 이체, vault 입출금, 청산. 금액은 부호 없는 크기이고 방향은 항목의 `kind`가 말합니다. 청산에는 단일 금액 자체가 없습니다. 페이지 처리는 `funding_payments`와 같습니다. `Page::next`가 `None`이 될 때까지 `cursor`로 되돌려 주세요 |
| `asset_context(&market)` | mark·mid·oracle 가격, 미결제약정, 그리고 지금 쌓이고 있는 펀딩 비율. 이 비율은 `FundingRate`가 보고하는 값과 다릅니다. 그쪽은 이미 부과된 펀딩의 기록입니다 |

```rust
use maxt::{Client, Exchange, Market, adapters::HyperliquidAdapter};

async fn accruing_funding() -> maxt::Result<()> {
    let client = Client::new(HyperliquidAdapter::new());
    let context = client
        .adapter()
        .asset_context(&Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"))
        .await?;

    if let Some(rate) = context.funding_rate {
        println!("hourly funding is running at {rate}");
    }
    Ok(())
}
```

## 인증 정보

API key가 아니라 지갑 주소와 16진수 private key입니다.

```rust
use maxt::{Client, adapters::HyperliquidAdapter};

fn client() -> Client<HyperliquidAdapter> {
    let address = std::env::var("HYPERLIQUID_ADDRESS").expect("HYPERLIQUID_ADDRESS");
    let key = std::env::var("HYPERLIQUID_PRIVATE_KEY").expect("HYPERLIQUID_PRIVATE_KEY");

    Client::new(HyperliquidAdapter::new().with_wallet(address, key))
}
```

| 항목 | 규칙 |
| --- | --- |
| 키의 쓰임 | 서명, 그것뿐입니다. 비공개 요청마다 로컬에서 인코딩하고 해시해 EIP-712 다이제스트 위에 secp256k1로 서명합니다. 전송되는 값은 동작, nonce, 서명이고 조회에는 계좌 주소가 더해집니다. 키 자체는 전송되지 않습니다 |
| `Debug` | 키를 가리므로 어댑터에 `{:?}`를 써도 로그에 남지 않습니다 |
| 주소 | `0x` 접두사가 붙은 20바이트 16진수이며 보내기 전에 소문자로 바꿉니다. Hyperliquid가 `user` 필드를 글자 그대로 대조해서 체크섬 주소는 오류 없이 빈 계좌로 읽힙니다 |
| private key | `0x` 접두사가 있든 없든 32바이트 16진수. 그 밖의 값이면 지갑이 필요한 첫 호출에서 `Error::Auth`입니다 |
| 어느 키를 쓸까 | 계좌 자체의 키보다 승인된 API wallet 키를 쓰세요. 이 어댑터가 보내는 동작은 둘 다 서명하지만 API wallet 키로는 출금할 수 없습니다. 어느 쪽이든 전달한 주소가 실제로 다뤄지는 계좌입니다 |
| 공개 시세 | 지갑이 필요 없습니다 |

## 예제

`cargo run --example public_rest -- hyperliquid HYPE USDC`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Hyperliquid 공식 문서

- [Rate limits and user limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Info endpoint: perpetuals](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Info endpoint: spot](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [Exchange endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)

---

[공통 API](../common-api.ko.md) · [거래소 고르기](../providers.ko.md)
