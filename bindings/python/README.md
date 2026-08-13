# maxt for Python

[English](README.md) | [한국어](README.ko.md)

One async Python API for the same operations, models, errors, and streams as
the Rust contract. Common operations and exchange-specific operations remain
available together. Generated contracts are checked against the compiled
native API.

## Install

```sh
python -m pip install maxt
```

Python has no separate initialization function. Constructing a built-in
adapter loads the native module.

## First read: Binance Spot

The first call below uses Binance `BTC/USDT` public data only. It does not
need credentials and cannot submit an order.

```python
import asyncio

from maxt import BinanceAdapter, Client, Exchange, Market


async def main() -> None:
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
    client = Client(BinanceAdapter.spot())

    ticker = await client.ticker(market)  # common API
    average = await client.adapter.spot_average_price(market)  # Binance-only API

    print(ticker.last_price)
    print(f"{average.minutes}-minute average: {average.price}")


asyncio.run(main())
```

Run the checked-in version with `python examples/binance_public_ticker.py`.
For provider-specific calls, use `client.adapter`; all common market, account,
order, and stream calls stay on `Client`.

## Support

GIL-enabled CPython 3.9 or newer is required. PyPy and free-threaded CPython
are not currently supported. Prebuilt wheels cover glibc 2.17 or newer Linux
(x64 and ARM64), macOS (x64 and ARM64), and Windows (x64). Other platforms
build from the source distribution and require Rust and a native compiler
toolchain.

## Supported exchanges

- Binance Spot and USD-M perpetual futures
- Upbit Spot: Korea, Singapore, Indonesia, and Thailand
- Bithumb Spot
- Hyperliquid Spot and perpetual futures on mainnet and testnet

Binance testnet constructors are not exposed. Hyperliquid HIP-3 perpetual DEXs
and outcome assets are not exposed.

## Package map

| Need | Use |
| --- | --- |
| Public market data and streams | `Client` with an adapter |
| Exchange-only fields or endpoints | `client.adapter` |
| Exact prices and quantities | `decimal.Decimal` values returned by models |
| Timestamps | `Timestamp` as Unix-epoch nanoseconds |
| Native API reference | Generated classes in `maxt.models`, `maxt.adapters`, and `maxt.__init__` |
| Endpoint support and constraints | [generated endpoint reference](../common/generated/api.md) |

`Client` calls are normalized across adapters. Provider methods intentionally
retain exchange-specific fields and appear on the concrete adapter class.

## Authentication boundary

Public calls need no credentials. Signed account, order, and wallet operations
require both credential fields. Hyperliquid also exposes the address-scoped,
unsigned `/info` reads listed below; they require a public `address`, not a
private key. Use `client.supports(feature)` before optional operations when the
adapter or credential state is dynamic.

## Common API

`Client` provides the same method names for every built-in adapter:

- Public REST: `markets()`, `trades()`, `order_book()`, `ticker()`, and
  `candles()`.
- Public streams: `subscribe()` and `subscribe_with()` for trades, order books,
  tickers, and candles. Bithumb does not support candle streams.
- Public funding history: `funding_rates()` on Binance USD-M and Hyperliquid
  perpetual markets.
- Private Spot: `balances()`, `open_orders()`, `place_order()`,
  `cancel_order()`, and `subscribe_account()` on every exchange.
- Private order lookup: `order()`, `order_by_client_id()`, `orders_by_ids()`,
  and `order_history()` on Upbit and Bithumb.
- Private order rules: `order_rules()` on Upbit and Bithumb.
- Private batch cancellation: `cancel_orders()` on Upbit and Bithumb.
- Private wallet lookup and cancellation: `deposit()`, `withdrawal()`, and
  `cancel_withdrawal()` on Upbit and Bithumb. Lookups require an asset and one
  exchange ID or transaction ID; cancellation must be followed by a lookup.
- Private perpetuals: `positions()`, `margin_summary()`, `set_margin()`, and
  `funding_payments()` on Binance USD-M and Hyperliquid.

## Exchange-specific API

Exchange-specific methods remain available through `client.adapter`.

| Adapter | Construction | Additional methods |
| --- | --- | --- |
| `BinanceAdapter` | `BinanceAdapter.spot()` | Public: `aggregate_trades()`, `spot_average_price()`, `spot_symbol_filters()`; authenticated: `spot_order()`, `account_trades()`, `c2c_trade_history()`, `test_order()`, `cancel_all_open_orders()` |
| `BinanceAdapter` | `BinanceAdapter.usd_m_futures()` | Public: `mark_price()`, `mark_prices()`, `open_interest()`, `aggregate_trades()`; authenticated: `account_trades()`, `test_order()`, `cancel_all_open_orders()`, `usd_m_create_listen_key()`, `usd_m_keepalive_listen_key()`, `usd_m_close_listen_key()` |
| `UpbitAdapter` | `UpbitAdapter()` or `UpbitAdapter(region=...)` | `order_books()`, `order_books_at_level()`, `tickers()`, `tickers_by_quote()`, `year_candles()`, `orderbook_instruments()`, `market_events()`; authenticated: `test_order()`, `order_detail()`, `closed_orders()`, `deposit_info()`, `travel_rule_vasps()`, `verify_travel_rule_by_uuid()`, `verify_travel_rule_by_txid()`, `batch_cancel_open_orders()`, `cancel_and_new_order()`; Korea only: `deposit_krw()`, `withdraw_krw()`, `api_keys()`, `list_pockets()`, `list_pocket_api_keys()`, `sub_pocket_balances()`, `universal_transfer()`, `universal_transfers()`, `sub_pocket_transfer()`, `sub_pocket_transfers()` |
| `BithumbAdapter` | `BithumbAdapter()` | `market_warnings()`, `market_alerts()`, `notices()`, `transfer_fees()`; authenticated: `api_keys()`, `withdrawal_addresses()`, `order_detail()`, `order_list()`, `closed_orders()`, `krw_withdrawals()`, `withdraw_krw()`, `krw_deposits()`, `deposit_krw()`, `pending_orders()`, `batch_orders()`, `twap_orders()`, `create_twap_order()`, `cancel_twap_order()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` or `HyperliquidAdapter.testnet()` | Public: `all_mids()`, `asset_context()`; address-scoped, unsigned reads: `basic_open_orders()`, `order_status(reference)`, `historical_orders()`, `user_fills()`, `user_fills_by_time()`, `non_funding_ledger()`, `user_rate_limit()`, `user_role()`, `referral()`, `user_fees()`, `portfolio()`, `sub_accounts()`, `user_vault_equities()` |

`UpbitAdapter.test_order()` validates an order without creating it. The returned
`Order` is a dry-run result: do not query or cancel its `id`, and do not treat
its status as a live order.

`UpbitAdapter.order_detail(request)` is the provider-specific authenticated
`GET /v1/order` read. Supply the expected market plus a UUID and/or identifier;
one identifier is required and Upbit gives UUID priority. It preserves detailed
fills, fees, locked amounts, SMP, and time-in-force raw fields absent from the
common `Order`; reserved identifier characters are safely encoded. Fixture-verified only.

`order_history()` remains the common normalized history API.
`UpbitAdapter.closed_orders(request)` complements it with official closed-order
summary fields, including fees, SMP, `identifier`, and time-in-force, but no
`trades` list. Its optional `market`, `state`, and `states` filters include
mutually exclusive `state` and `states`; the creation-time window is at most seven days,
`limit` is at most 1,000, and ordering can be ascending or descending.
`Timestamp` inputs are passed directly to Upbit as milliseconds, unlike the
common history API's exclusive-end adaptation. The official endpoint does not
state time-boundary inclusion, so this API makes no further boundary claim.
Fixture-verified only; maxt has not performed a live trade or read. See the
[Korea](https://docs.upbit.com/kr/reference/list-closed-orders) and
[Global](https://global-docs.upbit.com/reference/list-closed-orders) references.

`UpbitAdapter.deposit_info(asset, network)` returns the provider's deposit
availability, minimum amount, confirmation, and precision metadata. Upbit may
delay this information by several minutes; it is not a real-time service-status signal.

`UpbitAdapter.travel_rule_vasps()` lists VASPs for Travel Rule verification.
The verification methods are financial writes and are available only in Korea
and Singapore; Indonesia and Thailand fail before a network request. These
paths are fixture-verified only.

`UpbitAdapter.batch_cancel_open_orders(request)` is a financial write.
`UpbitBatchCancelScope.all()` explicitly selects every eligible market; Upbit
still applies the request count (default 20, maximum 300 `wait` orders), and
the result preserves partial failures.

`UpbitAdapter.cancel_and_new_order(request)` is a financial write using the JSON
endpoint. The replacement keeps the original market and side; `post_only` and
SMP cannot be combined. A successful HTTP response may still have no new order
when the previous order fills before cancellation completes. This path is
fixture-verified only.

`UpbitAdapter.deposit_krw(request)` and `withdraw_krw(request)` are Korea-only
financial writes. `UpbitKrwTransferRequest` requires a positive amount and an
`UpbitKrwTwoFactorType` of `KAKAO`, `NAVER`, or `HANA`; the registered account
and second factor stay on Upbit. `api_keys()` is a Korea-only authenticated read
of access-key identifiers and expiry times. All three paths are fixture-verified
only; no live transfer is submitted by maxt.

`list_pockets()`, `list_pocket_api_keys(request)`, and
`sub_pocket_balances(pocket_uuid)` are Korea-only authenticated reads for
pockets, their API keys, and a sub-pocket balance. `universal_transfer(request)`
and `sub_pocket_transfer(request)` are Korea-only financial writes; both request
types require a destination `to` under Upbit's current OpenAPI contract.
`universal_transfers(request)` and `sub_pocket_transfers(request)` list the
corresponding transfer histories. These paths are fixture-verified only.

`BithumbAdapter.batch_orders(request)` accepts 1–20 orders and can return HTTP
200 with per-item failures; inspect every `BithumbBatchOrderOutcome`. Accepted
items preserve `time_in_force` and `stp_type`; rejected items preserve returned
`time_in_force`. This is a fixture-verified financial write only.

`BithumbAdapter.twap_orders(request)` is an authenticated, read-only history
query for Bithumb's KRW markets. `create_twap_order()` and
`cancel_twap_order()` are financial writes; do not call them in a read-only
verification.

`BithumbAdapter.krw_withdrawals()` and `krw_deposits()` read KRW transfer
history. `withdraw_krw()` and `deposit_krw()` are financial writes. Bithumb
requires its registered account and Kakao second-factor flow; maxt neither
accepts nor stores those credentials. These paths are fixture-verified only.

`BithumbAdapter.withdrawal_addresses()` is an authenticated, read-only list of
registered withdrawal allowlist addresses. It is distinct from
`prepare_withdrawal()`: it does not validate a prospective withdrawal or return
a common withdrawal quote. It is fixture-verified only.

`BithumbAdapter.order_detail(request)` retains Bithumb's provider-specific fill,
fee, cancellation, self-trade-prevention, and time-in-force fields; the
normalized common `Order` intentionally does not carry them. The expected
market in the request is checked against the response. This path is
fixture-verified only.

`BithumbAdapter.order_list(request)` is the provider-specific authenticated
`GET /v1/orders` read, separate from common `open_orders()`. It supports an
optional market, either `state` or `states`, UUID/client-ID lists of up to 100
(UUIDs take priority), plus `page >= 1`, `limit` from 1 through 100, and
`order_by`. Its
provider fields are retained rather than reduced to common `Order`. Fixture-verified only.

`order_history()` remains the common normalized history API.
`BithumbAdapter.closed_orders(request)` complements it with Bithumb's official
v2 fee, cancellation, self-trade-prevention, and time-in-force metadata. It
supports an optional `market`, mutually exclusive `state` or `states` (`states[]` query parameter), start/end
times at most seven days apart, `limit` from 1 through 1,000, `order_by`, and an
opaque `next_key` cursor. Times go directly to Bithumb as milliseconds, unlike
the common history API's exclusive-end adaptation; time-boundary inclusion is
not claimed. The page preserves `data`, `has_next`, and `next_key`, plus raw
status/type strings and optional price, creation-time, client-order, and
cancellation fields. Fixture-verified only; maxt has not performed a live
account read or trade. See [closed orders](https://apidocs.bithumb.com/reference/%EC%A2%85%EB%A3%8C-%EC%A3%BC%EB%AC%B8-%EB%AA%A9%EB%A1%9D-%EC%A1%B0%ED%9A%8C.md) and
[authentication](https://apidocs.bithumb.com/docs/%EC%9D%B8%EC%A6%9D-%ED%86%A0%ED%81%B0-%EC%83%9D%EC%84%B1%ED%95%98%EA%B8%B0).

```python
from maxt import (
    BithumbAdapter,
    BithumbTwapOrdersRequest,
    Client,
    Exchange,
    Market,
)

async def read_twap_history() -> None:
    client = Client(BithumbAdapter(access_key=access_key, secret_key=secret_key))
    market = Market.spot(Exchange.BITHUMB, "BTC", "KRW")
    page = await client.adapter.twap_orders(
        BithumbTwapOrdersRequest(market=market, limit=20)
    )
```

The Bithumb TWAP API accepts `progress`, `done`, or `cancel` states and uses a
page size from 1 through 100. Creation uses a 300–43,200 second duration and a
15/20/30/60/120 second interval; buys require `price`, sells require `volume`.

`BinanceAdapter.usd_m_futures()` exposes `mark_price()`, `mark_prices()`, and
`open_interest()` as public, read-only USD-M perpetual market-data calls. These
methods are fixture-verified; they have not been live-read verified.
`aggregate_trades(request)` is a public Spot and USD-M read returning the same
provider aggregate-trade type. Both venues use an inclusive `from_id` cursor or
inclusive time bounds (not both), with `limit` from 1 through 1,000 (`None`
defaults to 500). USD-M only retains the latest 48 hours and requires a time
window shorter than one hour; Spot has no equivalent local limit. This method is
fixture-verified only.
`account_trades(request)` is a signed Spot or USD-M account-trade page with a
1–1,000 limit (default 500) and no safe generic continuation cursor.
`c2c_trade_history(request)` is a signed, read-only Spot/Funding Wallet SAPI
call and is unavailable on `usd_m_futures()`. It requires
`BinanceC2cTradeType.BUY` or `.SELL`, uses a one-based page with at most 100
rows, and permits inclusive timestamp bounds spanning at most 30 days. Its
nullable `code`, `message`, `data`, `total`, and `success` envelope is preserved
instead of being converted to a common cursor. This path is fixture-verified only.
`test_order(BinanceTestOrderRequest(...))` is signed validation that does not
reach the matching engine; `compute_commission_rates` is Spot-only.
`cancel_all_open_orders(market)` is a signed financial write for one market.
These three paths are fixture-verified only.
`HyperliquidAdapter.all_mids()` is also public and read-only. It returns the
default perpetual DEX mids and first-DEX spot mids; Hyperliquid falls back to
the last trade price when a book is empty. This method is fixture-verified and
has not been live-read verified.

`user_rate_limit()`, `user_role()`, `referral()`, `user_fees()`, `portfolio()`,
`sub_accounts()`, and `user_vault_equities()` are public `/info` reads for the
configured Hyperliquid address. They require `address=...`; `private_key` is
optional and these reads do not use a signature. These paths are fixture-verified
only.

`user_fills(aggregate_by_time)` and
`user_fills_by_time(from, to, aggregate_by_time)` are unsigned `POST /info`
reads for the configured public address; no private key or signature is used.
The latter requires `from`, accepts optional `to`, and uses inclusive millisecond
boundaries. Both preserve provider execution, position, fee, order, direction,
and raw fields; they are fixture-verified only.

`basic_open_orders()`, `order_status(reference)`, and `historical_orders()` are
also address-bound, unsigned `POST /info` reads. The first uses Hyperliquid's
compact `openOrders` response and is distinct from common `open_orders()`,
which uses `frontendOpenOrders`. `reference` accepts a numeric `oid` or a
`0x`-prefixed 32-hex-character client order ID; `unknownOid` returns normal
`HyperliquidOrderStatusResponse.UnknownOrder`, while future top-level statuses
retain their status and raw JSON. Historical and found detailed orders retain
trigger, time-in-force, reduce-only, client-ID, status, and raw JSON fields;
`historical_orders()` returns up to the latest 2,000 orders. All three require
a valid configured `address` and fail before network I/O when it is absent or
invalid; no API key, private key, or signature is used. Fixture-verified only.

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

See the [common data and pagination contracts](../../docs/common-api.md) and
[provider limits and data semantics](../../docs/providers.md).

## Documentation and examples

- [Runnable Binance public-ticker example](examples/binance_public_ticker.py)
- [Repository getting started guide](../../docs/getting-started.md)
- [Provider reference](../../docs/providers.md)
- [Generated endpoint coverage reference](../common/generated/api.md)

## License

MIT
