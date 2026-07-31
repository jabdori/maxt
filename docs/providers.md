# Choosing a provider

[English](providers.md) | [한국어](providers.ko.md)

Public market data needs no credentials on any shipped adapter.

## Choose by use case

| Need | Adapter |
| --- | --- |
| Korean won spot markets | [Upbit](providers/upbit.md) or [Bithumb](providers/bithumb.md) |
| Global spot markets | [Binance Spot](providers/binance.md) |
| USD-margined perpetuals | [Binance USD-M](providers/binance.md) or [Hyperliquid](providers/hyperliquid.md) |
| Wallet signing instead of API keys | [Hyperliquid](providers/hyperliquid.md) |
| A wired test network | `HyperliquidAdapter::testnet()`; Binance testnet hosts are not exposed by this crate |

## Constructors and credentials

| Provider | Public constructor | Enable private features |
| --- | --- | --- |
| Upbit | `UpbitAdapter::new()` for Korea; `with_region(...)` for Singapore, Indonesia, or Thailand | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance Spot | `BinanceAdapter::spot()` | `with_credentials(api_key, secret_key)` |
| Binance USD-M | `BinanceAdapter::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid mainnet/testnet | `HyperliquidAdapter::new()` or `::testnet()` | `with_wallet(address, private_key)` |

Missing credentials make private capabilities return `false` from
`Client::supports`; their calls return `Error::Auth`. A structurally unavailable
capability also returns `false`, but its call returns `Error::Unsupported`. See
[capability checks](common-api.md#capability-checks).

## Differences that affect design

| Provider | Design boundary |
| --- | --- |
| Upbit | Spot only. Four regions have separate hosts, listings, books, and credentials; Korea is the default. |
| Bithumb | Spot only, with no candle stream. The current `open_orders` implementation reads one page and returns at most 100 orders. |
| Binance | Spot and USD-M are separate adapter configurations; the wrong `MarketKind` is invalid. One logical USD-M subscription may use and merge multiple sockets. |
| Hyperliquid | One adapter handles spot and perpetual markets. Funding, margin configuration, and reduce-only orders reject spot arguments; `positions_on(spot)` returns an empty list. `Ticker::last_price` is `midPx`, falling back to `markPx`, rather than a last trade. |

Across providers, REST trades are newest-first, candles are oldest-first, book
sides are best-first, numbers use `maxt::Decimal`, and unpublished fields remain
`None`. Read the [common API reference](common-api.md) for candle ranges,
stream reconnects, retry safety, and private-call boundaries.

Provider-only batching, market alerts, native contexts, and ledger calls remain
on the concrete adapter. Access them through `Client::adapter()`; they are not
added to the portable common surface.

## Live verification scope

On 2026-07-31, public REST and supported public streams were checked live on
Upbit Korea `BTC/KRW`, Bithumb `BTC/KRW`, Binance Spot `BTC/USDT`, Binance USD-M
`BTC/USDT` perpetual, and Hyperliquid mainnet `BTC/USDC` perpetual. The check
used no credentials. Private account and trading paths were not live-verified.

## Provider references

- [Upbit](providers/upbit.md) ([한국어](providers/upbit.ko.md))
- [Bithumb](providers/bithumb.md) ([한국어](providers/bithumb.ko.md))
- [Binance](providers/binance.md) ([한국어](providers/binance.ko.md))
- [Hyperliquid](providers/hyperliquid.md) ([한국어](providers/hyperliquid.ko.md))
