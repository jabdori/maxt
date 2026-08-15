"""Binance Spot public ticker example.

Run this after installing the package. It reads public data only and never
accepts credentials or submits an order.
"""

from __future__ import annotations

import asyncio

from maxt import BinanceAdapter, Client, Exchange, Market


async def main() -> None:
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
    client = Client(BinanceAdapter.spot())

    ticker = await client.ticker(market)
    average = await client.adapter.spot_average_price(market)

    print(f"{market}: last={ticker.last_price}")
    print(f"Binance {average.minutes}-minute average={average.price}")


if __name__ == "__main__":
    asyncio.run(main())
