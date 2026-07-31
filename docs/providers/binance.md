[English](binance.md) | [한국어](binance.ko.md)

# Binance

`BinanceAdapter` exposes Binance Spot and USD-margined perpetual futures. One
adapter is fixed to one venue; public REST and market streams need no
credentials.

## Constructors and venues

```rust
use maxt::{Client, adapters::BinanceAdapter};

let spot = Client::new(BinanceAdapter::spot());
let usd_m = Client::new(BinanceAdapter::usd_m_futures());
```

| Constructor | `Market` / `MarketKind` | REST host | Public market-stream host |
| --- | --- | --- | --- |
| `BinanceAdapter::spot()` | `Market::spot` / `MarketKind::Spot` | `https://api.binance.com` | `wss://stream.binance.com:9443/stream` |
| `BinanceAdapter::usd_m_futures()` | `Market::perpetual` / `MarketKind::Perpetual` | `https://fapi.binance.com` | `wss://fstream.binance.com/public/stream` and `/market/stream` |

`BinanceAdapter::default()` is spot. A market from another exchange or with the
wrong kind is rejected as `Error::InvalidRequest` before network I/O.

## Public REST

| Common call | Spot | USD-M futures |
| --- | --- | --- |
| `markets(kind)` | `/api/v3/exchangeInfo`; spot listings | `/fapi/v1/exchangeInfo`; exact `PERPETUAL` contracts only |
| `trades(market, limit)` | `/api/v3/trades`; optional `limit` is `1..=1000` | `/fapi/v1/trades`; optional `limit` is `1..=1000` |
| `order_book(market, depth)` | `/api/v3/depth`; optional depth is any integer in `1..=5000` | `/fapi/v1/depth`; optional depth is one of `5, 10, 20, 50, 100, 500, 1000` |
| `ticker(market)` | `/api/v3/ticker/24hr` | `/fapi/v1/ticker/24hr` |
| `candles(request)` | `/api/v3/klines`; 1,000 candles per exchange call | `/fapi/v1/klines`; 1,500 candles per exchange call |
| `funding_rates(request)` | Unsupported | Public `/fapi/v1/fundingRate`; page `limit` is `1..=1000`, default 100 |

`maxt` pages candle requests larger than one exchange response. Trades are
returned newest first. Spot depth has no exchange timestamp, so its read time
is used; USD-M depth keeps Binance's timestamp.

An unrecognised listing status, including a future status such as
`CANCEL_ONLY`, is `MarketStatus::Unknown` rather than guessed.

## Candle intervals

- Spot: `1s`, `1m`, `3m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `8h`,
  `12h`, `1d`, `3d`, `1w`, `1M`.
- USD-M: the same list without `1s`; one minute is the shortest interval.
- Binance also offers `6h`, but the current `maxt::Interval` API does not expose
  it. REST candles and `Feed::Candles` use the same exposed list.

## Public streams

| Feed | Binance stream name | Delivery |
| --- | --- | --- |
| `Feed::Trades` | `{symbol}@trade` | One event per fill; a zero-quantity USD-M frame is dropped |
| `Feed::OrderBook` | `{symbol}@depth20@100ms` | Full snapshots with 20 levels per side at 100 ms; depth is not configurable |
| `Feed::Ticker` | `{symbol}@ticker` | Rolling 24-hour ticker |
| `Feed::Candles(interval)` | `{symbol}@kline_<interval>` | Forming updates and Binance's closed flag |

Public frames are resolved through the markets supplied in the subscription,
keyed by their lowercase native symbols. The decoder does not guess a base and
quote split from a fixed quote suffix list. This preserves markets such as
`ADAEUR`, `USDTUSD`, `BTCU`, and UTF-8 symbols.

USD-M splits feeds across two sockets when necessary:

| Entry point | Feeds |
| --- | --- |
| `wss://fstream.binance.com/public/stream` | `Trades`, `OrderBook` |
| `wss://fstream.binance.com/market/stream` | `Ticker`, `Candles` |

The sockets are merged into one logical `MarketStream`. Each socket emits its
own `MarketEvent::Reconnected` notice. If either socket terminates, the logical
stream terminates and drops the other socket instead of silently losing half
of the subscription. Spot uses one market-data socket.

## Request weight

Binance limits IP-based `REQUEST_WEIGHT`, not only request count. On the
2026-07-31 verification date, `exchangeInfo` advertised 6,000 per minute for
Spot and 2,400 for USD-M. Treat the live `rateLimits` entry and
`X-MBX-USED-WEIGHT-1M` response header as authoritative. `maxt` does not
throttle or consume that header; HTTP 429 and 418 classify as rate limited.

## Credentials and Binance-only methods

Use `.with_credentials(api_key, secret_key)` for account, order, and private
stream calls. The adapter implements HMAC-SHA-256 API-key/secret signing only.
Binance also supports RSA and Ed25519 keys, but `maxt` does not.

| Method | Contract |
| --- | --- |
| `spot_symbol_filters(&market)` | Spot `PRICE_FILTER`, `LOT_SIZE`, and `NOTIONAL` values; unsupported on USD-M |
| `spot_order(&market, order_id)` | Looks up one spot order by Binance's numeric id, including filled or cancelled orders |
| `usd_m_create_listen_key()` | Creates or extends the account's USD-M listen key |
| `usd_m_keepalive_listen_key(&key)` | Extends the current USD-M listen key |
| `usd_m_close_listen_key(&key)` | Closes the USD-M listen key |

`subscribe_account` manages the USD-M listen-key lifecycle. Spot account
streams instead send a signed `userDataStream.subscribe.signature` request to
`wss://ws-api.binance.com:443/ws-api/v3`; Spot has no listen key.

## Limitations

| Area | Current boundary |
| --- | --- |
| USD-M listings | Only exact `contractType == PERPETUAL` is exposed. Current `TRADIFI_PERPETUAL`, `CURRENT_QUARTER`, and `NEXT_QUARTER` listings are omitted |
| Other Binance products | Spot margin, COIN-M futures, options, and portfolio-margin APIs are not exposed |
| Candle intervals | Binance's `6h` interval is not represented |
| Order rules | `spot_symbol_filters` has no USD-M equivalent in `maxt`; orders are not rounded or pre-validated against filters |
| Stream variants | Fixed partial depth is exposed; configurable depths, diff-depth reconstruction, and `@aggTrade` are not |
| Hosts and credentials | No testnet constructor; RSA and Ed25519 credentials are unsupported |

## Verification scope

On 2026-07-31, representative BTC/USDT Spot and USD-M public REST and stream
smoke checks passed. Private live calls were not verified.

## Example

```text
cargo run --example public_rest -- binance BTC USDT
```

## Official documentation

- [Spot REST market data](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market)
- [Spot REST security and limits](https://developers.binance.com/en/docs/products/spot/rest-api)
- [Spot WebSocket market streams](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)
- [USD-M REST market data](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [USD-M public WebSocket streams](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/public)
- [USD-M market WebSocket streams](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/ws-streams/market)
- [USD-M general information](https://developers.binance.com/en/docs/products/derivatives-trading-usds-futures/general-info)

---

[The common API](../common-api.md) · [Choosing an exchange](../providers.md)
