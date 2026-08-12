# maxt Python

[English](README.md) | [한국어](README.ko.md)

Rust 계약과 같은 작업, 모델, 오류, 스트림을 제공하는 비동기 Python API입니다.
공통 기능과 거래소 전용 기능을 함께 사용할 수 있습니다. 생성 계약은 컴파일된
네이티브 API와 정합성을 검사합니다.

## 설치

GIL을 사용하는 CPython 3.9 이상이 필요합니다. PyPy와 free-threaded CPython은
0.1.0에서 지원하지 않습니다. 미리 빌드된 wheel은 glibc 2.17 이상
Linux(x64, ARM64),
macOS(x64, ARM64), Windows(x64)를 지원합니다. 다른 플랫폼은 source
distribution에서 빌드하므로 Rust와 네이티브 컴파일 도구가 필요합니다.

```sh
python -m pip install maxt
```

Python에는 별도 초기화 함수가 없습니다. 내장 어댑터를 생성할 때 네이티브
모듈을 불러옵니다.

## 지원 거래소

- Upbit 현물(Spot): 한국, 싱가포르, 인도네시아, 태국
- Bithumb 현물(Spot)
- Binance 현물(Spot), USD-M 무기한 선물
- Hyperliquid 메인넷·테스트넷 현물(Spot), 무기한 선물

Binance 테스트넷(testnet) 생성자는 제공하지 않습니다. Hyperliquid HIP-3
무기한 선물 DEX와 결과형 자산(outcome asset)은 제공하지 않습니다.

## 공통 API

`Client`는 모든 내장 어댑터에서 같은 메서드 이름을 사용합니다.

- 공개 REST: `markets()`, `trades()`, `order_book()`, `ticker()`,
  `candles()`
- 공개 스트림: 체결, 호가, 현재가 요약(ticker), 캔들(candle)용
  `subscribe()`, `subscribe_with()`; Bithumb 캔들 스트림은 미지원
- 공개 펀딩 이력(funding history): Binance USD-M, Hyperliquid 무기한 선물의
  `funding_rates()`
- 비공개 현물(Spot): 모든 거래소의 `balances()`, `open_orders()`,
  `place_order()`, `cancel_order()`, `subscribe_account()`
- 비공개 주문 조회: Upbit, Bithumb의 `order()`, `order_by_client_id()`,
  `orders_by_ids()`, `order_history()`
- 비공개 주문 가능 정보: Upbit, Bithumb의 `order_rules()`
- 비공개 다건 취소: Upbit, Bithumb의 `cancel_orders()`
- 비공개 입출금 조회·취소: Upbit, Bithumb의 `deposit()`, `withdrawal()`,
  `cancel_withdrawal()`; 조회에는 자산과 거래소 ID 또는 온체인 트랜잭션 ID 하나가 필요하며,
  취소 후에는 반드시 다시 조회해 최종 상태를 확인
- 비공개 무기한 선물: Binance USD-M, Hyperliquid의 `positions()`,
  `margin_summary()`, `set_margin()`, `funding_payments()`

공개 호출에는 인증 정보가 필요하지 않습니다. 비공개 호출에는 인증 필드 두 개를
모두 전달해야 합니다. 어댑터나 인증 상태가 동적으로 바뀌면 선택 기능을 호출하기
전에 `client.supports(feature)`를 확인하세요.

## 거래소 전용 API

거래소 전용 메서드는 `client.adapter`에서 호출합니다.

| 어댑터 | 생성 | 추가 메서드 |
| --- | --- | --- |
| `UpbitAdapter` | `UpbitAdapter()` 또는 `UpbitAdapter(region=...)` | `order_books()`, `order_books_at_level()`, `tickers()`, `tickers_by_quote()`, `year_candles()`, `orderbook_instruments()`, `market_events()`; 인증 필요: `test_order()`, `deposit_info()`, `batch_cancel_open_orders()`, `cancel_and_new_order()` |
| `BithumbAdapter` | `BithumbAdapter()` | `market_warnings()`, `market_alerts()`, `notices()`, `transfer_fees()`; 인증 필요: `api_keys()`, `pending_orders()`, `batch_orders()`, `twap_orders()`, `create_twap_order()`, `cancel_twap_order()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spot_symbol_filters()`; 인증 필요: `spot_order()` |
| `BinanceAdapter` | `BinanceAdapter.usd_m_futures()` | 공개: `mark_price()`, `mark_prices()`, `open_interest()`, `aggregate_trades()`; 인증 필요: `usd_m_create_listen_key()`, `usd_m_keepalive_listen_key()`, `usd_m_close_listen_key()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` 또는 `HyperliquidAdapter.testnet()` | 공개: `all_mids()`; `asset_context()`, `non_funding_ledger()` |

`UpbitAdapter.test_order()`는 주문을 생성하지 않고 검증합니다. 반환 `Order`는
dry-run 결과이므로 `id`를 조회·취소에 사용하면 안 되며 상태도 실제 활성 주문을 뜻하지 않습니다.

`UpbitAdapter.deposit_info(asset, network)`는 거래소가 제공하는 입금 가능 여부, 최소
수량, 확인 수, 소수 자릿수 메타데이터를 반환합니다. Upbit 응답은 몇 분 지연될 수 있어
실시간 서비스 상태로 사용하면 안 됩니다.

`UpbitAdapter.batch_cancel_open_orders(request)`는 금전성 쓰기 요청입니다.
`UpbitBatchCancelScope.all()`은 모든 대상 마켓 범위를 명시적으로 선택하며, Upbit는
요청 수량을 적용해 기본 20개·최대 300개의 일치하는 `wait` 주문만 취소합니다. 일부
실패도 결과에 보존합니다.

`UpbitAdapter.cancel_and_new_order(request)`는 JSON endpoint를 사용하는 금전성
쓰기입니다. 새 주문은 기존 주문의 시장과 매수/매도 방향을 유지하며
`post_only`와 SMP를 함께 사용할 수 없습니다. HTTP 요청이 성공해도 취소 완료 전에
기존 주문이 체결되면 새 주문이 없을 수 있습니다. 이 경로는 fixture 검증만 했습니다.

`BithumbAdapter.batch_orders(request)`는 1~20건을 받고 항목별 실패가 있어도 HTTP
200을 반환할 수 있으므로 `BithumbBatchOrderOutcome`를 모두 확인해야 합니다. 성공
항목은 `time_in_force`와 `stp_type`을, 실패 항목은 반환된 `time_in_force`를 보존합니다.
이 메서드는 fixture로만 검증한 금전성 쓰기입니다.

`BithumbAdapter.twap_orders(request)`는 Bithumb KRW 마켓의 인증된 읽기 전용
주문 이력 조회입니다. `create_twap_order()`와 `cancel_twap_order()`는 금전성
쓰기이므로 읽기 전용 검증에서 호출하지 마세요.

```python
from maxt import (
    BithumbAdapter,
    BithumbTwapOrdersRequest,
    Client,
    Exchange,
    Market,
)

async def read_twap_history() -> None:
    client = Client(BithumbAdapter(access_key=access_key, secret_key=secret_key))
    market = Market.spot(Exchange.BITHUMB, "BTC", "KRW")
    page = await client.adapter.twap_orders(
        BithumbTwapOrdersRequest(market=market, limit=20)
    )
```

Bithumb TWAP API는 `progress`, `done`, `cancel` 상태와 1~100개 페이지 크기를
지원합니다. 생성 시 주문 시간은 300~43,200초, 간격은 15/20/30/60/120초이며,
매수에는 `price`, 매도에는 `volume`이 필요합니다.

`BinanceAdapter.usd_m_futures()`는 USD-M 무기한 선물의 공개 읽기 전용
시세 데이터 메서드 `mark_price()`, `mark_prices()`, `open_interest()`를
제공합니다. 이 메서드들은 fixture로 검증했으며 실제 읽기 요청(live read)은
아직 검증하지 않았습니다. `aggregate_trades(request)`도 공개 USD-M 읽기입니다.
`from_id`부터 조회하거나 `start_time`~`end_time` 범위를 조회하며, 두 방식은 함께 사용할 수 없습니다. 시간 간격은 1시간 미만이고 `limit`은
1~1,000(`None → 500`)입니다. Binance는 최근 48시간만 보관하며 이 메서드도
fixture 검증만 했습니다.
`HyperliquidAdapter.all_mids()`는 공개 읽기 전용이며,
기본 무기한 선물 DEX와 첫 번째 DEX의 Spot mid 가격을 반환합니다. 호가가 비어
있으면 Hyperliquid가 마지막 체결 가격을 대체값으로 사용합니다. 이 메서드도
fixture로 검증했으며 실제 읽기 요청은 아직 검증하지 않았습니다.

## Binance 공통 API와 거래소 전용 API

```python
import asyncio

from maxt import BinanceAdapter, Client, Exchange, Market


async def main() -> None:
    client = Client(BinanceAdapter.spot())
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")

    ticker = await client.ticker(market)
    filters = await client.adapter.spot_symbol_filters(market)

    print(ticker.last_price)
    print(filters.tick_size)


asyncio.run(main())
```

`ticker()`는 공통 API입니다. `spot_symbol_filters()`는 Binance Spot 전용이며
`client.adapter`를 통해 호출합니다.

## 스트림

```python
from maxt import Feed, StreamError, StreamEvent, Subscription

subscription = Subscription((market,), (Feed.TRADES,))
async with await client.subscribe(subscription) as stream:
    async for item in stream:
        if isinstance(item, StreamEvent):
            print(item.event)
        elif isinstance(item, StreamError):
            print(item.error)
```

`StreamError`는 반복을 종료하지 않습니다. `async with` 또는
`await stream.aclose()`로 네이티브 정리 완료를 기다립니다.

## 사용자 정의 어댑터

`Adapter`를 상속하고 `exchange`, `features`를 구현한 뒤, 알린 기능의 메서드를
재정의합니다. 인스턴스는 `Client(adapter)`로 감쌉니다. 기본 메서드는
`UnsupportedError`를 발생시킵니다.

사용자 정의 스트림은 비동기 반복자를 `MarketStream` 또는 `AccountStream`으로
감싸 반환합니다. `StreamEvent`, `StreamError`를 내보내고, 정리가 필요하면
반복자의 `aclose()`를 구현합니다.

## 계약

- `decimal.Decimal`: 96-bit 계수(coefficient), scale `0..=28`; 네이티브 경계에서 반올림하지 않습니다.
- `Timestamp`: Unix epoch 기준 nanosecond `int`입니다.
- 오류: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthError`, `ExchangeError`, `TransportError`, `DecodeError`.
- `ExchangeError`: 거래소 코드, HTTP status, 재시도 분류를 보존합니다.

[공통 데이터·페이지네이션 계약](../../docs/common-api.ko.md)과
[거래소별 한도·데이터 의미](../../docs/providers.ko.md)를 참고하세요.

## 라이선스

MIT
