# maxt for Python

[English](README.md) | [한국어](README.ko.md)

One async Python API for the same operations, models, errors, and streams as
the Rust contract. Common operations and exchange-specific operations remain
available together. Generated contracts are checked against the compiled
native API.

## Install

GIL-enabled CPython 3.9 or newer is required. PyPy and free-threaded CPython
are not supported in 0.1.0. Prebuilt wheels cover glibc 2.17 or newer Linux
(x64 and ARM64),
macOS (x64 and ARM64), and Windows (x64). Other platforms build from the source
distribution and require Rust and a native compiler toolchain.

```sh
python -m pip install maxt
```

Python has no separate initialization function. Constructing a built-in
adapter loads the native module.

## Binance common and exchange-specific APIs

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

`ticker()` is common. `spot_symbol_filters()` is Binance Spot-specific and is
available through `client.adapter`.

## Streams

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

`StreamError` does not terminate iteration. Use `async with` or
`await stream.aclose()` to await native cleanup.

## Custom adapters

Subclass `Adapter`, implement `exchange` and `features`, then override every
advertised operation. Wrap the instance with `Client(adapter)`. Default
methods raise `UnsupportedError`.

For custom streams, return `MarketStream` or `AccountStream` over an async
iterator. Emit `StreamEvent` and `StreamError`; implement the iterator's
`aclose()` when cleanup is required.

## Contracts

- `decimal.Decimal`: exact 96-bit coefficient, scale `0..=28`; no rounding at the native boundary.
- `Timestamp`: Unix epoch nanoseconds as `int`.
- Errors: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthError`, `ExchangeError`, `TransportError`, `DecodeError`.
- `ExchangeError`: preserves provider code, HTTP status, and retry classification.

See the [common API](../../docs/common-api.md) and [provider support](../../docs/providers.md).

## License

MIT
