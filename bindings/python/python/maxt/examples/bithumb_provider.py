"""Read public Bithumb market warnings, notices, and transfer fees."""

from __future__ import annotations

import asyncio

from maxt import BithumbAdapter


async def main() -> None:
    adapter = BithumbAdapter()
    warnings = await adapter.market_warnings()
    notices = await adapter.notices(5)
    fees = await adapter.transfer_fees("BTC")
    print(f"{len(warnings)} warning rows, {len(notices)} notices, {len(fees)} BTC fee rows")


if __name__ == "__main__":
    asyncio.run(main())
