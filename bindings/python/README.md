# maxt

`maxt` provides one async Python API for Upbit, Bithumb, Binance, and
Hyperliquid. The Python binding follows the Rust request, result, error, and
stream contracts.

## Install

```bash
pip install maxt
```

For development from a repository checkout:

```bash
mise install
mise exec -- uv sync --project bindings/python --frozen
```

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

`ticker()` is part of the common API. `spot_symbol_filters()` is available only
on `BinanceAdapter`, through `client.adapter`.

## Value contracts

| Value | Contract |
| --- | --- |
| `decimal.Decimal` | Native requests accept only values representable by a 96-bit coefficient and scale `0..=28`; validation never rounds or truncates |
| `Timestamp` | `int` nanoseconds since the Unix epoch |
| `Interval` | `as_secs()` returns `None` for `MONTH1`; `advance()` uses UTC calendar months and returns `None` on overflow |
| Common models | `OrderBook` exposes best prices, spread, and midpoint; `Balance.total()`, `Position.is_flat()`, and `Page.has_more()` match the Rust helpers |
| Common enums | Exchange, feature, market-kind, side, order-status, and exchange-error helpers expose the same classifications as Rust |
| `HyperliquidLedgerKind` | Unknown provider names remain available as the dynamic `OTHER` member's value |

Plain and scientific decimal strings are accepted when exact. Values outside
the native range raise `InvalidRequestError` at the Rust boundary.

## Errors

Calls raise structured exceptions such as `InvalidRequestError`,
`UnsupportedError`, `AuthError`, `ExchangeError`, `TransportError`, and
`DecodeError`. `ExchangeError` preserves the provider code, status, and retry
classification.

## Streams

Stream items are `StreamEvent` or non-terminal `StreamError` values. Use
`async with stream` or `await stream.aclose()` to await native cleanup; garbage
collection is best-effort only.

## Adapters

For a custom adapter, subclass `Adapter`, provide `exchange` and `features`,
and override the methods for every advertised feature. Unimplemented methods
raise `UnsupportedError`.

Provider-specific methods remain on the concrete adapter:

```python
adapter = client.adapter
```

See the provider references for those methods and their contracts.

Project source and provider notes are available at
[github.com/jabdori/maxt](https://github.com/jabdori/maxt).
