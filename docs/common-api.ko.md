# 공통 API 레퍼런스

[English](common-api.md) | [한국어](common-api.ko.md)

`Client<A>`는 `Adapter` 위에 하나의 공통 계약을 제공합니다. 여러 제공자에서 같은
동작이 필요할 때는 공통 메서드를 사용하고, 한 제공자만 지원하는 기능은
`Client::adapter()`로 접근하세요. 실행 중에 제공자를 선택해야 한다면
`Client<Box<dyn Adapter>>`를 사용할 수 있으며, 어댑터 호출마다 동적 디스패치(dynamic dispatch)가
한 번 발생합니다.

## API 범위

인증 정보는 어댑터를 `Client`로 감싸기 전에 설정합니다.

| 접근 방식 | 공통 메서드 |
| --- | --- |
| 공개 REST | `markets`, `trades`, `order_book`, `ticker`, `candles`, `funding_rates` |
| 공개 스트림 | `subscribe`, `subscribe_with` |
| 비공개 조회 | `balances`, `open_orders`, `open_orders_on`, `positions`, `positions_on`, `margin_summary`, `funding_payments` |
| 비공개 스트림 | `subscribe_account`, `subscribe_account_with` |
| 비공개 변경 | `place_order`, `cancel_order`, `set_margin` |

여기서 공개란 계좌 인증 정보가 필요 없다는 뜻이며, 모든 제공자나 마켓 종류가 해당
메서드를 지원한다는 뜻은 아닙니다. `Client::supports`와 관련
[제공자 페이지](providers.ko.md)를 확인하세요.

## 공통 계약

| 항목 | 계약 |
| --- | --- |
| 마켓 식별 | `Market`은 거래소, 현물 또는 무기한 선물 종류, 기초 자산(base asset), 호가 자산(quote asset)을 포함합니다. 자산 코드는 대문자로 정규화됩니다. |
| 체결 | REST 결과는 최신순입니다. `Trade::taker_side`는 유동성을 가져간 쪽의 방향입니다. |
| 캔들 | 결과는 오래된 순입니다. `open_time`은 구간이 시작된 시각입니다. |
| 호가창 | 매수 호가는 높은 가격부터, 매도 호가는 낮은 가격부터 정렬되므로 양쪽 모두 최우선 호가부터 나옵니다. |
| 숫자 | 가격, 수량, 비율, 금액은 `f64`가 아닌 정확한 `maxt::Decimal`을 사용합니다. |
| 없는 값 | 제공자가 공개하지 않은 필드는 추론하거나 0으로 채우지 않고 `None`으로 둡니다. |
| 시간 | `Timestamp`는 Unix epoch 이후 UTC 나노초입니다. `Display`는 밀리초 정밀도의 RFC 3339 형식으로 출력합니다. 정확한 정밀도가 필요하면 `as_nanos()`를 사용하세요. |
| 대체 시각 | 거래소가 페이로드 시각을 제공하지 않으면 관련 필드 문서에 `maxt`가 로컬 수신 시각을 사용한다고 명시합니다. 이는 거래소 이벤트 시각이 아닙니다. |

티커 필드가 모두 같은 시점이나 집계 구간을 나타내는 것은 아닙니다. 특히
Hyperliquid는 자산 컨텍스트에서 최근 체결가를 제공하지 않습니다. REST와 스트림
티커는 `midPx`를 `Ticker::last_price`로 매핑하고, 값이 없으면 `markPx`를
사용하며, `last_trade_time`은 `None`입니다. 이 필드를 Hyperliquid 체결가로
해석하지 마세요. [Hyperliquid 페이지](providers/hyperliquid.ko.md)를 참고하세요.

## 공개 시장 데이터

| 메서드 | 결과와 주요 경계 |
| --- | --- |
| `markets(kind)` | 해당 종류의 상장 마켓입니다. 지원하지 않는 마켓 종류는 빈 목록일 수 있습니다. |
| `trades(market, limit)` | 최근 체결을 최신순으로 반환합니다. 호출당 상한은 제공자마다 다릅니다. |
| `order_book(market, depth)` | 스냅샷(snapshot) 하나를 반환합니다. `depth`는 한쪽당 호가 단계 수이며 허용값은 제공자마다 다릅니다. |
| `ticker(market)` | 집계 구간과 일부 필드가 제공자마다 다른 티커 요약입니다. |
| `candles(request)` | 과거 캔들을 오래된 순으로 반환하며 내부 페이지 이동 횟수에 상한이 있습니다. |
| `funding_rates(request)` | 공개 무기한 선물 펀딩 비율 이력을 한 번에 `Page` 하나씩 반환합니다. |

정확한 체결 개수, 호가창 깊이, 타임스탬프 출처, 캔들 시작 경계는
[Upbit](providers/upbit.ko.md), [Bithumb](providers/bithumb.ko.md),
[Binance](providers/binance.ko.md),
[Hyperliquid](providers/hyperliquid.ko.md) 페이지에 나와 있습니다.

### 간격

`Feature::Candles`를 지원할 때 이식 가능한 기준 간격은 `Min1`, `Min3`,
`Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`,
`Month1`입니다. 추가 간격과 캔들 시작 경계는 제공자마다 다릅니다. 지원하지 않는
간격은 가까운 간격으로 반올림하지 않고 `Error::Unsupported`를 반환합니다.

### 캔들 범위와 확정

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{CandleRequest, Client, Exchange, Interval, Market, Timestamp};

# async fn read() -> maxt::Result<()> {
let client = Client::new(UpbitAdapter::new());
let request = CandleRequest::new(
    Market::spot(Exchange::Upbit, "BTC", "KRW"),
    Interval::Min1,
)
.from(Timestamp::from_millis(1_700_000_000_000))
.to(Timestamp::from_millis(1_700_007_200_000))
.limit(120);

let candles = client.candles(&request).await?;
assert!(candles.windows(2).all(|pair| pair[0].open_time <= pair[1].open_time));
# Ok(())
# }
```

- `from`은 캔들 시작 시각을 기준으로 포함됩니다.
- `to`는 캔들 시작 시각을 기준으로 제외됩니다.
- `from`이 있으면 `limit`은 그 경계부터 조건에 맞는 가장 오래된 캔들을 선택합니다.
- `from`이 없으면 `limit`은 가장 최신 캔들을 선택합니다.
- `limit`을 지정하면 1 이상이어야 합니다. `from >= to`는 잘못된 요청입니다.
- `maxt`의 내부 페이지 이동은 제공자 호출 100번까지입니다. 더 긴 이력은 범위를
  나눈 여러 요청으로 읽으세요.

`Candle::closed`는 해당 간격이 끝났다는 뜻입니다. REST 응답에는 아직 만들어지는
중인 최신 캔들이 포함될 수 있습니다. Binance 스트림은 제공자의 확정 여부를 그대로
전달합니다. Upbit와 Hyperliquid는 다음 구간이 처음 도착할 때 이전 캔들을 확정하여
내보냅니다. 따라서 마지막 구간이나 재연결로 중단된 구간은 확정 이벤트가 나오지 않을
수 있습니다. Bithumb은 캔들 스트림을 제공하지 않습니다. 위의 제공자 페이지는 이
내용을 중복하지 않고 각 제공자의 간격과 스트림 제약을 설명합니다.

## 스트림

`Subscription`은 하나의 논리적 스트림이며 WebSocket 하나를 보장하지 않습니다.
요청한 모든 피드를 요청한 모든 마켓에 적용하고, 마켓 또는 피드 집합이 비어 있으면
거절하며, 중복을 제거하고 삽입 순서를 유지합니다. Binance USD-M은 구독 하나를
여러 소켓으로 나눈 뒤 이벤트를 반환하는 `MarketStream`으로 합칠 수 있습니다.

`StreamConfig::default()`의 값은 다음과 같습니다.

| 필드 | 기본값 | 의미 |
| --- | ---: | --- |
| `max_reconnect_attempts` | `None` | 재연결 시도 횟수에 유한한 상한을 두지 않습니다. |
| `initial_reconnect_delay_ms` | `1_000` | 첫 재연결 대기 시간입니다. |
| `max_reconnect_delay_ms` | `30_000` | 재연결 백오프 대기 시간의 상한입니다. |
| `idle_timeout_ms` | `30_000` | 요청하는 최소 무활동 제한 시간입니다. 어댑터가 더 긴 값으로 올릴 수 있습니다. |
| `buffer_size` | `4_096` | 소비자 이벤트 큐입니다. 0을 요청하면 용량 1로 구현됩니다. |
| `overflow` | `Backpressure` | 이벤트를 버리지 않고 소비자가 따라올 때까지 기다립니다. |

`Overflow::Backpressure`는 소비자가 따라올 때까지 소켓 읽기를 멈춥니다. 이벤트를
버리지는 않지만, 너무 오래 멈추면 제공자가 연결을 닫을 수 있습니다.
`Overflow::DropNewest`는 큐가 가득 찬 동안 새 데이터와 오류 항목을 조용히
버립니다. 티커나 전체 호가창 스냅샷처럼 나중 이벤트가 놓친 값을 완전히 대체할 때만
적합합니다. 고유한 체결과 캔들의 유일한 확정 이벤트에는 사용할 수 없습니다.

재연결 알림은 `DropNewest`에서도 버리지 않습니다. 큐에 자리가 생길 때까지
보관했다가 이후 데이터보다 먼저 전달합니다. 이 알림은 연결이 끊긴 동안의 이벤트를
놓쳤다는 뜻입니다. 다음 스냅샷으로 호가창을 다시 만들고, 계좌 스트림이 재연결된
뒤에는 REST로 잔고와 미체결 주문을 다시 읽으세요.

재연결 횟수는 모든 재연결을 세며 각 내부 연결에 따로 적용됩니다. 연결이 한동안
정상이었다고 횟수가 초기화되지 않으므로, 제공자가 소켓을 주기적으로 교체하면 유한한
상한에 도달해 스트림이 끝날 수 있습니다. 스트림의 `Err` 항목은 문제 보고일 뿐
그 자체로 스트림을 끝내지 않습니다. `None`만 종료를 뜻하며, 스트림을 드롭하면
내부 연결을 모두 닫습니다. 분할된 Binance USD-M 구독에서 소켓 하나가 종료되면
일부 피드만 남는 상태를 막기 위해 논리 스트림 전체를 끝내고 나머지 소켓도 닫습니다.

## 기능 확인

`Feature::needs_credentials()`는 계좌에 접근하는 기능을 식별합니다.
`Client::supports(feature)`는 로컬에서 판단하며 네트워크를 호출하지 않습니다.

### `true`여도 호출할 때 다시 확인해야 합니다

`true`는 현재 구성된 어댑터에 해당 기능을 수행하는 작업이 매핑되어 있다는
뜻입니다. 특정 마켓, 간격, 주문 형식, 인증 정보, 거래소 권한까지 받아들여진다는
보장은 아닙니다.

`false`가 나오는 원인은 실질적으로 서로 다른 두 가지입니다.

1. 제공자에 해당 기능이 구조적으로 없습니다. 호출하면 `Error::Unsupported`를
   반환하며 인증 정보를 추가해도 달라지지 않습니다.
2. 함께 제공되는 어댑터에 비공개 엔드포인트가 매핑되어 있지만 인증 정보가 없습니다.
   호출하면 `Error::Auth`를 반환하며 인증 정보를 설정하면 `supports`가 `true`로
   바뀔 수 있습니다.

기능 지원 여부를 확인했더라도 호출의 `Result`를 반드시 처리하세요.

## 오류와 재시도 안전성

| 오류 | 의미 | 같은 요청을 그대로 재시도해도 되는가? |
| --- | --- | --- |
| `InvalidRequest` | 전송 전에 로컬 필드 검증이 요청을 거절했습니다. | 아니요. |
| `Unsupported` | 이 제공자나 마켓 종류에 매핑된 작업이 없습니다. | 아니요. |
| `Auth` | 로컬에서 인증된 요청을 만들 수 없습니다. | 아니요. |
| `Exchange` | 거래소가 요청을 거절했거나 처리에 실패했습니다. 거래소의 코드와 메시지를 유지합니다. | `is_retryable()`을 확인하세요. |
| `Transport` | DNS, TLS, 소켓, 제한 시간 문제로 결과를 확인하지 못했습니다. | 가능할 수 있지만 아래 내용을 확인하세요. |
| `Decode` | 응답 구조를 해석할 수 없습니다. | 아니요. 문제를 보고하세요. |

`maxt`는 REST 호출을 자동으로 재시도하지 않습니다. 애플리케이션이
`Error::is_retryable`을 바탕으로 자동 재시도 정책을 만든다면, 백오프와 제공자별
요청 한도(rate limit)를 적용해 조회와 그 밖의 멱등 요청에만 사용하세요. 주문이나
취소를 보낸 뒤 전송 오류가 발생해도 거래소가 변경 요청을 거절했다는 뜻은 아닙니다.
다시 변경 요청을 보내기 전에 주문이나 계좌 상태를 조회하세요. 그렇지 않으면 의도한
작업이 중복될 수 있습니다.

## 비공개 계좌와 거래

비공개 경로는 오프라인 픽스처와 모의 서버 테스트로 검증하지만 2026-07-31 실시간
적합성 검사에는 포함하지 않았습니다. 읽기 전용 또는 테스트넷 인증 정보로 시작하고,
인증과 주문 제약은 제공자 페이지를 따르세요.

`open_orders`와 `open_orders_on`은 스냅샷이며 구독이 아닙니다. 또한 모든 제공자의
이력 페이지를 끝까지 읽었다는 보장도 아닙니다. 특히 현재 Bithumb 어댑터는
`/v1/orders` 요청을 한 번만 보내고 해당 엔드포인트의 한 페이지인 최대 100개
주문을 반환합니다. 이보다 강한 보장을 전제로 정합성 복구를 구현하지 마세요.

`OrderRequest`는 `Size`로 기초 자산과 호가 자산 기준 수량을 구분합니다. 시장가
주문, 호가 자산 기준 수량, 포지션 감소 전용(reduce-only), 주문 유효 조건
(`TimeInForce`)의 지원 범위는 제공자마다 다릅니다. 주문 취소와 체결은 경합하므로
반환된 주문을 확인하고, 체결 여부가 중요하면 계좌 상태를 다시 맞추세요.

### 주문 정밀도와 최소 크기

공통 `MarketInfo`는 호가 단위, 주문 수량 단위, 최소 주문 금액을 노출하지
않습니다. 일부 어댑터는 전송 전에 제공자 정밀도를 검증하지만 최소값은 여전히
거래소에서만 적용하는 규칙일 수 있습니다. 제공자별 주문 절을 확인하고 `f64`로
반올림하지 마세요. 값은 `maxt::Decimal`로 만드세요.

### 파생상품과 이력 페이지

`positions`와 `positions_on`은 수량이 0인 행을 제외합니다. 제공자가 값을 공개하지
않으면 `MarginSummary` 필드는 `None`으로 남습니다. 펀딩 비율 이력은 공개
데이터이고, 펀딩 지급 이력은 서명이 필요한 비공개 데이터입니다. 음수 금액은 해당
계좌가 펀딩 금액을 지급했다는 뜻입니다.

`Page<T>`에서는 `Page::next`를 다음 요청의 커서로 다시 전달하고, 값이 `None`일
때만 중단하세요. 커서가 있으면 `items` 목록이 짧거나 비어 있어도 마지막 페이지라는
뜻이 아닙니다.

`MarginRequest`는 레버리지, 마진 모드 또는 둘 다 표현할 수 있지만 제공자마다
요구사항이 다릅니다. 두 필드를 모두 받는 경우에도 `set_margin`의 원자성은 보장되지
않습니다. 한 제공자 작업이 성공한 뒤 다른 작업이 실패할 수 있습니다. 롤백됐다고
가정하지 말고 결과 설정을 다시 읽으세요.

## 제공자별 메서드

제공자 전용 일괄 처리, 경보, 네이티브 컨텍스트, 원장 호출은 구체적인 어댑터에
남아 있습니다.

```rust
use maxt::adapters::UpbitAdapter;
use maxt::Client;

let client = Client::new(UpbitAdapter::new());
let upbit: &UpbitAdapter = client.adapter();
let _ = upbit.region();
```

이 방식은 유용한 거래소 기능을 숨기지 않으면서 공통 계약의 이식성을 유지합니다.

## 요청 한도

`maxt`는 REST 호출 속도를 제한하거나 애플리케이션을 대신해 제공자의 요청 예산을
배분하지 않습니다. 각 제공자 페이지의 요청 한도 절을 확인하고, 애플리케이션에서
제한을 한곳에 모아 관리하며 `Error::is_rate_limited()`가 참이면 백오프하세요.

## 외부 어댑터

공개 `Adapter` 트레이트는 크레이트 외부에서도 구현할 수 있습니다. `exchange`와
`supports`를 구현하고, 지원하는 작업만 재정의하며, 모든 공통 정렬·검증·없는 값·
Decimal·타임스탬프 계약을 지키세요. 선택적 메서드는 기본적으로
`Error::Unsupported`를 반환합니다. 실제 거래소를 새로 추가하려면 여전히
`maxt`에 `Exchange` 열거형 항목(variant)이 필요합니다. [어댑터 체크리스트](../CONTRIBUTING.ko.md#어댑터-체크리스트)를
참고하세요.
