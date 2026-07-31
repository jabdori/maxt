[English](providers.md) | [한국어](providers.ko.md)

# Choosing an exchange

## Which one for which job

| Need | Adapter |
| --- | --- |
| Korean won markets | [Upbit](providers/upbit.md) or [Bithumb](providers/bithumb.md), both spot-only |
| Global spot | [Binance](providers/binance.md), `BinanceAdapter::spot()` |
| Perpetual futures | [Binance USD-M](providers/binance.md), `BinanceAdapter::usd_m_futures()`, or [Hyperliquid](providers/hyperliquid.md) |
| Wallet signing instead of an API key | [Hyperliquid](providers/hyperliquid.md) only |
| A test network | [Hyperliquid](providers/hyperliquid.md), `HyperliquidAdapter::testnet()`. The other three have no test environment here |

Public market data needs no credentials on any of the four.

## Constructors and credentials

| Adapter | Built with | Credentials |
| --- | --- | --- |
| Upbit | `UpbitAdapter::new()`, or `::with_region(..)` for Singapore, Indonesia, Thailand | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance | `BinanceAdapter::spot()` or `::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid | `HyperliquidAdapter::new()`, or `::testnet()` | `with_wallet(address, private_key)`, and prefer an [approved API wallet key](providers/hyperliquid.md#credentials) |

Until credentials are supplied, `Client::supports` answers `false` for every
account feature.

## The differences that change a design

| Adapter | Difference |
| --- | --- |
| Upbit | No derivatives listed: positions, margin, funding rates, funding payments, leverage configuration and reduce-only orders are `Error::Unsupported`. [Four separate exchanges](providers/upbit.md#what-is-supported) besides, one per adapter. Korea, Singapore, Indonesia and Thailand have separate listings, separate order books and separate credentials |
| Bithumb | No derivatives listed, exactly as on Upbit. Also [no candle stream](providers/bithumb.md#streams): a subscription containing `Feed::Candles(_)` fails as a whole with `Error::Unsupported` before a socket is opened |
| Binance | [Two venue configurations](providers/binance.md#venues), spot and USD-M futures, fixed at construction. Separate hosts, separate balances, separate listings, and `BTCUSDT` exists on both at different prices |
| Hyperliquid | One configuration for both spot and perpetual markets; the distinction rides on `Market::kind`. The derivatives features read as supported on the adapter and refuse per market: funding, positions and reduce-only on a Hyperliquid *spot* market are `Error::Unsupported` |

## What no longer differs

| Subject | Everywhere |
| --- | --- |
| Candles | Oldest-first. `CandleRequest::from` is honoured on all four, and `limit` is honoured past the per-response cap by paging, up to a hundred pages |
| Candle intervals | `supports(Feature::Candles) == true` means `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1` and `Month1` all work over REST. Beyond those it is per-exchange, and the stream carries a different set again |
| Recent trades | Newest-first on every adapter that offers them |

`Client::supports` answers per feature, not per argument, so a `true` can still
refuse at the call:
[the common API](common-api.md#a-true-still-has-to-be-checked-at-the-call).

## The provider pages

- [Upbit](providers/upbit.md) ([한국어](providers/upbit.ko.md))
- [Bithumb](providers/bithumb.md) ([한국어](providers/bithumb.ko.md))
- [Binance](providers/binance.md) ([한국어](providers/binance.ko.md))
- [Hyperliquid](providers/hyperliquid.md) ([한국어](providers/hyperliquid.ko.md))
