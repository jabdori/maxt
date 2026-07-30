[English](providers.md) | [한국어](providers.ko.md)

# Choosing an exchange

Four adapters. This page picks one. Each exchange's own page carries the
per-call limits, the order shapes it accepts, and the things it does that the
common API has no room for.

## Which one for which job

| You need | Adapter |
| --- | --- |
| Korean won markets | [Upbit](providers/upbit.md) or [Bithumb](providers/bithumb.md), both spot-only |
| Global spot | [Binance](providers/binance.md), `BinanceAdapter::spot()` |
| Perpetual futures | [Binance USD-M](providers/binance.md), `BinanceAdapter::usd_m_futures()`, or [Hyperliquid](providers/hyperliquid.md) |
| Wallet signing instead of an API key | [Hyperliquid](providers/hyperliquid.md) only |
| A test network | [Hyperliquid](providers/hyperliquid.md), `HyperliquidAdapter::testnet()`. The other three have no test environment here. |

Public market data works on all four with no account at all. The derivatives
half of the API, meaning positions, margin, funding, and leverage, is worked
through end to end in
[the common API](common-api.md#a-worked-derivatives-read); on Upbit and Bithumb
every one of those calls is `Error::Unsupported`.

## Constructors and credentials

| Adapter | Built with | Credentials |
| --- | --- | --- |
| Upbit | `UpbitAdapter::new()`, or `::with_region(..)` for Singapore, Indonesia, Thailand | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance | `BinanceAdapter::spot()` or `::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid | `HyperliquidAdapter::new()`, or `::testnet()` | `with_wallet(address, private_key)` |

`maxt` does not flatten those into one credential type, because they are not one
thing. Upbit and Bithumb sign with a key pair. Binance sends an API key and
signs with a secret. Hyperliquid signs each request locally with a wallet key
and sends no key at all, and there an approved API wallet key is the better
choice: it signs the same actions but cannot withdraw.

Until credentials are supplied, `Client::supports` answers `false` for every
account feature.

## The differences that change a design

| Adapter | What it changes |
| --- | --- |
| Upbit, Bithumb | No derivatives listed. Positions, margin, funding rates, funding payments, leverage configuration, and reduce-only orders are `Error::Unsupported`. |
| Upbit | Four separate exchanges. Korea, Singapore, Indonesia, and Thailand have separate listings, separate order books, and separate credentials. One adapter talks to exactly one region, chosen with `UpbitAdapter::with_region`, and a credential issued for one region does not work on another. |
| Bithumb | No candle stream. A subscription containing `Feed::Candles(_)` fails as a whole with `Error::Unsupported` before a socket is opened. It is not dropped from the feed list, and no candles are synthesized from trades on your behalf. Poll candles over REST or aggregate `Feed::Trades` yourself. |
| Binance | Two adapters. Spot and USD-M futures are separate APIs with separate hosts, separate balances, and separate listings, and `BTCUSDT` exists on both at different prices. The venue is fixed at construction, and a market of the wrong kind is refused before anything reaches the network. |
| Hyperliquid | One adapter for both spot and perpetual markets; the distinction rides on `Market::kind`. That also means the derivatives features read as supported on the adapter and refuse per market: funding, positions, and reduce-only on a Hyperliquid *spot* market are `Error::Unsupported`. |
| Hyperliquid | `trades` reads the last ten and no more. `recentTrades` takes no count, so ten is the whole window and a wider gap cannot be read back over REST. `Feed::Trades` is where a continuous record comes from. |

## What no longer differs

Three things used to be per-exchange and are now the same everywhere. The
details are in [the common API](common-api.md).

| What | The same everywhere |
| --- | --- |
| Candles | One contract. Oldest-first, `CandleRequest::from` honoured on all four, and `limit` honoured past the per-response cap by paging, up to a hundred pages. No adapter reports `from` as `Error::Unsupported`, and none makes you walk a cursor yourself. |
| Candle intervals | Ten are guaranteed. `supports(Feature::Candles) == true` means `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, and `Month1` all work over REST. Beyond those it is per-exchange, and the stream carries a different set again. |
| Recent trades | Newest-first on every adapter that offers them. |

The rest is per-exchange: per-call caps, the intervals each exchange adds beyond
the baseline and the different set its stream carries, the order shapes it
accepts, which timestamps are the exchange's own, and how deep each live order
book feed goes. Read the page for the adapter you picked before writing code
against it. Each states its gaps up front.

One thing no page can settle for you: `Client::supports` answers per feature,
not per argument. `Feature::CandleStream` is `true` on Upbit and
`Feed::Candles(Interval::Day1)` is still `Error::Unsupported` there, because
Upbit streams no daily candle. Handle the error at the call as well as branching
on the feature, and see
[the common API](common-api.md#feature-and-clientsupports) for the rest.

## The provider pages

- [Upbit](providers/upbit.md) ([한국어](providers/upbit.ko.md))
- [Bithumb](providers/bithumb.md) ([한국어](providers/bithumb.ko.md))
- [Binance](providers/binance.md) ([한국어](providers/binance.ko.md))
- [Hyperliquid](providers/hyperliquid.md) ([한국어](providers/hyperliquid.ko.md))
