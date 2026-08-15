# Binance

[English](binance.md) | [한국어](binance.ko.md)

Run the [Binance provider examples](../examples.md#binance-provider) for Spot,
USD-M, and provider-specific public reads.

## Venue and constructor

One `BinanceAdapter` is fixed to Spot or USD-M perpetual futures.

| Constructor | `MarketKind` | REST | Public WebSocket |
| --- | --- | --- | --- |
| `BinanceAdapter::spot()` | `Spot` | `https://api.binance.com` | `wss://stream.binance.com:9443/stream` |
| `BinanceAdapter::usd_m_futures()` | `Perpetual` | `https://fapi.binance.com` | `wss://fstream.binance.com/public/stream` and `/market/stream` |

`BinanceAdapter::default()` is Spot. A market with the wrong exchange or
`MarketKind` returns `Error::InvalidRequest` before network I/O.

## REST

| Call | Spot | USD-M |
| --- | --- | --- |
| `markets(kind)` | `/api/v3/exchangeInfo`; Spot listings | `/fapi/v1/exchangeInfo`; `contractType == PERPETUAL` |
| `trades(market, limit)` | `/api/v3/trades`; `limit: 1..=1000` | `/fapi/v1/trades`; `limit: 1..=1000` |
| `order_book(market, depth)` | `/api/v3/depth`; `depth: 1..=5000`; at most `depth` levels per side | `/fapi/v1/depth`; `depth: {5, 10, 20, 50, 100, 500, 1000}`; at most `depth` levels per side |
| `ticker(market)` | `/api/v3/ticker/24hr`; rolling 24-hour summary | `/fapi/v1/ticker/24hr`; rolling 24-hour summary |
| `spot_average_price(market)` | Public `/api/v3/avgPrice`; current Spot average-price snapshot with Binance's averaging window and last included trade time | `Error::Unsupported` |
| `funding_rates(request)` | `Error::Unsupported` | `/fapi/v1/fundingRate`; `limit: 1..=1000`; `None -> 100` |
| `mark_price(market)` | `Error::Unsupported` | `/fapi/v1/premiumIndex`; one USD-M perpetual mark-price snapshot |
| `mark_prices()` | `Error::Unsupported` | `/fapi/v1/premiumIndex`; current snapshots for supported USD-M perpetual markets |
| `open_interest(market)` | `Error::Unsupported` | `/fapi/v1/openInterest`; one USD-M perpetual open-interest snapshot |
| `aggregate_trades(request)` | Public `/api/v3/aggTrades`; `limit: 1..=1000` (`None -> 500`); `from_id` is inclusive and cannot be combined with time bounds | Public `/fapi/v1/aggTrades`; same aggregate-trade type and request rules as Spot; Binance retains only the latest 48 hours of futures trade history, and inclusive `start_time` and `end_time` must span less than one hour |

Trades are newest-first. Spot order books have no provider timestamp and use
local read time; USD-M order books retain the provider timestamp. Unknown
listing statuses map to `MarketStatus::Unknown`.

## Candles

| Venue | Exposed `Interval` variants |
| --- | --- |
| Spot | `Sec1`, `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour6`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| USD-M | Spot list without `Sec1` |

REST and `Feed::Candles` expose the same intervals.

| Constraint | Spot | USD-M |
| --- | ---: | ---: |
| Provider page cap | 1,000 | 1,500 |
| Provider calls per request | `<= 100` | `<= 100` |
| Preflight candle estimate | `<= 100_000` | `<= 150_000` |

## Streams

| Feed | Stream | Contract |
| --- | --- | --- |
| `Feed::Trades` (Spot) | `{symbol}@trade` | One event per execution |
| `Feed::Trades` (USD-M) | Unsupported | Binance exposes only aggregated trades, not every execution required by this feed |
| `Feed::OrderBook` | `{symbol}@depth20@100ms` | Full snapshot; 20 levels per side; fixed depth |
| `Feed::Ticker` | `{symbol}@ticker` | Rolling 24-hour summary |
| `Feed::Candles(interval)` | `{symbol}@kline_<interval>` | Preserve Binance `closed` |

Event markets are resolved from the native symbols in `Subscription`; symbols
are not split using a quote-suffix list.

USD-M routes `OrderBook` through `/public/stream`, and `Ticker` and `Candles`
through `/market/stream`. When both are required, the returned
`MarketStream` merges two sockets. Reconnect notices are per socket. If either
socket terminates, the logical stream terminates and drops the other socket.

## Private and provider-specific APIs

Configure private calls with `.with_credentials(api_key, secret_key)`. The
adapter supports HMAC-SHA-256 keys; RSA and Ed25519 keys are unsupported.

| Venue | Private features |
| --- | --- |
| Spot | Balances, open orders, place/cancel order, account stream |
| USD-M | Spot features plus positions, margin summary/configuration, funding payments, and reduce-only orders |

| Order input | Spot | USD-M |
| --- | --- | --- |
| `Size::Base` | All orders | All orders |
| `Size::Quote` | Market order only; Limit -> `Error::InvalidRequest` | `Error::InvalidRequest` |
| `time_in_force` | `GTC`, `IOC`, `FOK`; `PostOnly -> LIMIT_MAKER` | `GTC`, `IOC`, `FOK`; `PostOnly -> GTX` |
| `OrderType::Best` | `Size::Base` + `IOC` or `FOK`; `LIMIT + MARKET_PEG` | `Error::Unsupported` |
| `client_id` | 1–36 characters matching `[A-Za-z0-9./:_-]` | Same |
| `reduce_only == true` | `Error::Unsupported` | Supported |

Cancel methods return `()` after validating Binance's order response. Use an
order query when the final fill state matters.

Access the following provider-specific methods through `Client::adapter()`:

| Method | Contract |
| --- | --- |
| `spot_symbol_filters(&market)` | Spot `PRICE_FILTER`, `LOT_SIZE`, and `NOTIONAL`; unsupported on USD-M. |
| `spot_order(&market, order_id)` | One Spot order by numeric ID, including completed orders. |
| `spot_exchange_info()` | Public Spot `GET /api/v3/exchangeInfo`; preserves every listed symbol's metadata and raw filter payload. Fixture-verified only. |
| `spot_account_information()` | Signed Spot account read that preserves commissions, permissions, balances, and raw provider fields. Fixture-verified only. |
| `spot_cancel_all_open_orders(&market)` | Signed Spot cancellation that returns Binance's cancellation reports, unlike the common unit-shaped cancellation call. Fixture-verified only. |
| `usd_m_exchange_info()` | Public USD-M `GET /fapi/v1/exchangeInfo`; preserves contract listings, including dated contracts, and raw filter payload. Fixture-verified only. |
| `usd_m_account_information()`, `usd_m_position_information(market)` | Signed USD-M account and position-risk reads that preserve asset, margin, leverage, and risk fields not carried by common balances or positions. Fixture-verified only. |
| `all_coins_information()`, `api_key_permissions()`, `withdraw_address_list()` | Signed Wallet reads for coin/network rules, configured permission flags, and registered withdrawal-address metadata. Fixture-verified only. |
| `deposit_history(request)`, `withdraw_history(request)` | Signed Wallet history reads that preserve provider status, pagination, and time fields instead of reducing them to the common transfer history. Fixture-verified only. |
| `questionnaire_requirements()` | Signed Wallet Travel Rule questionnaire requirement read. Eligibility is enforced by Binance. Fixture-verified only. |
| `account_trades(request)` | Signed account-trade page: Spot `GET /api/v3/myTrades`, USD-M `GET /fapi/v1/userTrades`. A `HistoryRequest` requires one market, accepts a 1–1,000 limit (default 500), and has no safe generic continuation cursor (`next == None`). Fixture-verified only |
| `c2c_trade_history(request)` | Signed Spot/Funding `GET /sapi/v1/c2c/orderMatch/listUserOrderHistory`. Requires `BUY` or `SELL`; page defaults to 1 and rows to 100, with at most 100 rows and a 30-day query window. It preserves Binance's optional C2C response envelope instead of forcing a generic page cursor. Fixture-verified only |
| `test_order(request)` | Signed TRADE validation: Spot `POST /api/v3/order/test`, USD-M `POST /fapi/v1/order/test`; it does not submit an order to the matching engine. `BinanceTestOrderRequest::compute_commission_rates` is available only on Spot and is rejected before a USD-M request. Fixture-verified only |
| `cancel_all_open_orders(&market)` | Signed financial write that cancels active orders for one market: Spot `DELETE /api/v3/openOrders`, USD-M `DELETE /fapi/v1/allOpenOrders`. The deliberately unit-shaped result validates each venue's documented response shape without returning an incomplete order list. Fixture-verified only |
| `usd_m_create_listen_key()` | Create or extend the USD-M account listen key. |
| `usd_m_keepalive_listen_key()` | Extend the active USD-M listen key owned by the configured API key. |
| `usd_m_close_listen_key()` | Close the active USD-M listen key owned by the configured API key. |

The USD-M `mark_price`, `mark_prices`, and `open_interest` calls are public and
read-only. They are fixture-verified; live reads have not been verified. The
`mark_prices()` result is limited to maxt's USD-M perpetual market universe.

`aggregate_trades(request)` is a public, read-only Spot and USD-M endpoint that
returns the same provider aggregate-trade type. Both venues use an inclusive
`from_id` cursor or inclusive time bounds, but not both. USD-M retains only the
latest 48 hours of futures trade history and requires a time window shorter than
one hour; Spot has no equivalent local retention or time-window limit. The
endpoint returns one page, so use the last aggregate ID plus one as the next
`from_id` when walking by cursor. This method is fixture-verified; live reads
have not been verified.

`subscribe_account` manages USD-M listen keys. Spot uses the signed
`userDataStream.subscribe.signature` request and no listen key.

Orders are not rounded or validated against symbol filters. Spot margin,
COIN-M, options, portfolio margin, quarterly futures, configurable stream
depth, diff-depth reconstruction, `@aggTrade`, testnet constructors, and
RSA/Ed25519 keys are not exposed.

## Limits and official links

Binance charges IP-based `REQUEST_WEIGHT`. Read current limits from
`exchangeInfo` and monitor `X-MBX-USED-WEIGHT-1M`. `maxt` does not throttle or
consume that header. HTTP 429 and 418 satisfy
`Error::is_rate_limited() == true`.

- [Spot REST market data](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market)
- [Spot account trades](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account)
- [Spot account and exchange information](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account)
- [C2C trade history](https://developers.binance.com/en/docs/catalog/investment-and-services-c2-c/api/rest-api/~#get-c2-ctrade-history)
- [Spot test and cancel orders](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/trade)
- [Spot REST limits](https://developers.binance.com/en/docs/products/spot/rest-api)
- [Spot WebSocket streams](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)
- [USD-M REST market data](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [USD-M account trades, test, and cancel orders](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/trade)
- [USD-M account information](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/account)
- [USD-M Mark Price](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Mark-Price)
- [USD-M Open Interest](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Open-Interest)
- [USD-M Compressed/Aggregate Trades](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data#compressed-aggregate-trades-list)
- [USD-M public streams](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/public)
- [USD-M market streams](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/market)
- [Wallet account and capital APIs](https://developers.binance.com/en/docs/catalog/core-trading-wallet/api/rest-api/account)

[Common API](../common-api.md) · [Provider support](../providers.md)
