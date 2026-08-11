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
| `UpbitAdapter` | `UpbitAdapter()` 또는 `UpbitAdapter(region=...)` | `order_books()`, `order_books_at_level()`, `tickers()`, `tickers_by_quote()`, `year_candles()`, `orderbook_instruments()`, `market_events()` |
| `BithumbAdapter` | `BithumbAdapter()` | `market_warnings()`, `market_alerts()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spot_symbol_filters()`; 인증 필요: `spot_order()` |
| `BinanceAdapter` | `BinanceAdapter.usd_m_futures()` | 인증 필요: `usd_m_create_listen_key()`, `usd_m_keepalive_listen_key()`, `usd_m_close_listen_key()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` 또는 `HyperliquidAdapter.testnet()` | `asset_context()`, `non_funding_ledger()` |

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
