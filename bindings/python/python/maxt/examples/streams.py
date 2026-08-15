"""Print three public Binance trade-stream items, then close the stream."""

from __future__ import annotations

import asyncio

from maxt import BinanceAdapter, Client, Exchange, Feed, Market, Subscription


async def main() -> None:
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
    client = Client(BinanceAdapter.spot())
    subscription = Subscription((market,), (Feed.TRADES,))

    async with await client.subscribe(subscription) as stream:
        for _ in range(3):
            print(await anext(stream))


if __name__ == "__main__":
    asyncio.run(main())
