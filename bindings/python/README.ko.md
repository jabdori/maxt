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

[공통 API](../../docs/common-api.ko.md)와 [거래소 지원](../../docs/providers.ko.md)을 참고하세요.

## 라이선스

MIT
