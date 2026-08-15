"""Read public Binance Spot candles and prepare a paged history request."""

from __future__ import annotations

import asyncio

from maxt import (
    BinanceAdapter,
    CandleRequest,
    Client,
    Exchange,
    HistoryRequest,
    Interval,
    Market,
)


async def main() -> None:
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
    client = Client(BinanceAdapter.spot())
    candles = await client.candles(CandleRequest(market, Interval.MIN1, limit=5))
    for candle in candles:
        print(f"{candle.open_time}: close={candle.close} volume={candle.volume}")

    # Feed `page.next` into `cursor` only when a previous private response has
    # one. The request itself performs no account or network action.
    history = HistoryRequest(market, limit=100)
    print(f"private history request prepared: {history}")


if __name__ == "__main__":
    asyncio.run(main())
