# Binance

[English](binance.md) | [한국어](binance.ko.md)

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
| `funding_rates(request)` | `Error::Unsupported` | `/fapi/v1/fundingRate`; `limit: 1..=1000`; `None -> 100` |

Trades are newest-first. Spot order books have no provider timestamp and use
local read time; USD-M order books retain the provider timestamp. Unknown
listing statuses map to `MarketStatus::Unknown`.

## Candles

| Venue | Exposed `Interval` variants |
| --- | --- |
| Spot | `Sec1`, `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| USD-M | Spot list without `Sec1` |

REST and `Feed::Candles` expose the same intervals. Binance native `6h` is not
mapped to `Interval`.

| Constraint | Spot | USD-M |
| --- | ---: | ---: |
| Provider page cap | 1,000 | 1,500 |
| Provider calls per request | `<= 100` | `<= 100` |
| Preflight candle estimate | `<= 100_000` | `<= 150_000` |

## Streams

| Feed | Stream | Contract |
| --- | --- | --- |
| `Feed::Trades` | `{symbol}@trade` | One event per execution; drop `quantity == 0` frames |
| `Feed::OrderBook` | `{symbol}@depth20@100ms` | Full snapshot; 20 levels per side; fixed depth |
| `Feed::Ticker` | `{symbol}@ticker` | Rolling 24-hour summary |
| `Feed::Candles(interval)` | `{symbol}@kline_<interval>` | Preserve Binance `closed` |

Event markets are resolved from the native symbols in `Subscription`; symbols
are not split using a quote-suffix list.

USD-M routes `Trades` and `OrderBook` through `/public/stream`, and `Ticker`
and `Candles` through `/market/stream`. When both are required, the returned
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
| `reduce_only == true` | `Error::Unsupported` | Supported |

Access the following provider-specific methods through `Client::adapter()`:

| Method | Contract |
| --- | --- |
| `spot_symbol_filters(&market)` | Spot `PRICE_FILTER`, `LOT_SIZE`, and `NOTIONAL`; unsupported on USD-M. |
| `spot_order(&market, order_id)` | One Spot order by numeric ID, including completed orders. |
| `usd_m_create_listen_key()` | Create or extend the USD-M account listen key. |
| `usd_m_keepalive_listen_key(&key)` | Extend a USD-M listen key. |
| `usd_m_close_listen_key(&key)` | Close a USD-M listen key. |

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
- [Spot REST limits](https://developers.binance.com/en/docs/products/spot/rest-api)
- [Spot WebSocket streams](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)
- [USD-M REST market data](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [USD-M public streams](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/public)
- [USD-M market streams](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/market)

[Common API](../common-api.md) · [Provider matrix](../providers.md)
