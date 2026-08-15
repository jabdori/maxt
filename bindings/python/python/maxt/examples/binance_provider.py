"""Use public Binance provider-specific Spot and USD-M reads."""

from __future__ import annotations

import asyncio

from maxt import BinanceAdapter, Exchange, Market


async def main() -> None:
    spot_market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
    spot = BinanceAdapter.spot()
    average = await spot.spot_average_price(spot_market)
    filters = await spot.spot_symbol_filters(spot_market)
    exchange = await spot.spot_exchange_info()
    print(f"{average.minutes}-minute average={average.price}; tick={filters.tick_size}")
    print(f"Spot symbols: {len(exchange.symbols)}")

    futures = BinanceAdapter.usd_m_futures()
    metadata = await futures.usd_m_exchange_info()
    print(f"USD-M symbols: {len(metadata.symbols)}")


if __name__ == "__main__":
    asyncio.run(main())
