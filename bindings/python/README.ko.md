# maxt Python

[English](README.md)

`maxt`는 Upbit, Bithumb, Binance, Hyperliquid를 같은 비동기 API로 사용하는
Python 패키지입니다. 요청, 결과, 오류, 스트림의 의미는 Rust API와 같습니다.

## 설치

```bash
pip install maxt
```

저장소에서 개발할 때는 다음 명령을 사용합니다.

```bash
mise install
mise exec -- uv sync --project bindings/python --frozen
```

## Binance 공개 API와 전용 API

```python
import asyncio

from maxt import BinanceAdapter, Client, Exchange, Market


async def main() -> None:
    client = Client(BinanceAdapter.spot())
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")

    ticker = await client.ticker(market)
    filters = await client.adapter.spot_symbol_filters(market)

    print(f"{market}: {ticker.last_price}")
    print(f"{filters.symbol} tick size: {filters.tick_size}")


asyncio.run(main())
```

`ticker()`는 모든 거래소 Adapter가 공유하는 API입니다.
`spot_symbol_filters()`는 `BinanceAdapter`에서만 제공하며
`client.adapter`를 통해 호출합니다.

## 값 계약

| 값 | 계약 |
| --- | --- |
| `decimal.Decimal` | 96-bit coefficient와 `0..=28` scale로 정확히 표현 가능한 값만 허용합니다. 검증 과정에서 반올림하거나 자르지 않습니다. |
| `Timestamp` | Unix epoch 기준 nanosecond `int`입니다. |
| `Interval` | `MONTH1.as_secs()`는 `None`이며 `advance()`는 UTC 달력 기준으로 계산합니다. |
| 공통 모델 | `OrderBook`, `Balance`, `Position`, `Page`의 계산 helper는 Rust와 같은 결과를 냅니다. |
| `HyperliquidLedgerKind` | 알려지지 않은 provider 값도 `OTHER` 값으로 보존합니다. |

## 오류

호출 실패는 `InvalidRequestError`, `UnsupportedError`, `AuthError`,
`ExchangeError`, `TransportError`, `DecodeError` 등 구조화된 예외로 반환됩니다.
`ExchangeError`는 거래소 오류 코드, HTTP status, 재시도 분류를 보존합니다.

## 스트림

스트림 항목은 `StreamEvent` 또는 종료되지 않는 `StreamError`입니다. 네이티브
작업이 끝날 때까지 기다리려면 `async with stream` 또는
`await stream.aclose()`를 사용하세요.

## 사용자 정의 Adapter

`Adapter`를 상속하고 `exchange`, `features` 및 지원 기능의 메서드를
구현합니다. 구현하지 않은 메서드는 `UnsupportedError`를 반환합니다.
