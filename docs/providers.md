# Provider matrix

[English](providers.md) | [한국어](providers.ko.md)

All shipped adapters support public market data without credentials.

## Select a provider

| Requirement | Adapter |
| --- | --- |
| Korean won spot | `UpbitAdapter` or `BithumbAdapter` |
| Global spot | `BinanceAdapter::spot()` |
| USD-margined perpetuals | `BinanceAdapter::usd_m_futures()` or `HyperliquidAdapter` |
| Wallet signing | `HyperliquidAdapter` |
| Built-in testnet | `HyperliquidAdapter::testnet()` |

Binance testnet hosts are not exposed.

## Constructors and credentials

| Provider | Public constructor | Private configuration |
| --- | --- | --- |
| Upbit Korea | `UpbitAdapter::new()` | `.with_credentials(access_key, secret_key)` |
| Upbit Singapore, Indonesia, Thailand | `UpbitAdapter::with_region(region)` | `.with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `.with_credentials(access_key, secret_key)` |
| Binance Spot | `BinanceAdapter::spot()` | `.with_credentials(api_key, secret_key)` |
| Binance USD-M | `BinanceAdapter::usd_m_futures()` | `.with_credentials(api_key, secret_key)` |
| Hyperliquid mainnet | `HyperliquidAdapter::new()` | `.with_wallet(address, private_key)` |
| Hyperliquid testnet | `HyperliquidAdapter::testnet()` | `.with_wallet(address, private_key)` |

| State | Contract |
| --- | --- |
| Private operation mapped; credentials absent | `supports(feature) == false`; call returns `Error::Auth` |
| Structurally unsupported | `supports(feature) == false`; call returns `Error::Unsupported` |
| Operation mapped; credentials configured | `supports(feature) == true`; request validation and provider response determine the call result |

## Boundaries

| Provider | Markets | Key boundaries |
| --- | --- | --- |
| Upbit | `MarketKind::Spot` | Region-specific hosts, listings, books, and credentials |
| Bithumb | `MarketKind::Spot` | `supports(Feature::CandleStream) == false`; `open_orders()` returns one `/v1/orders` page |
| Binance Spot | `MarketKind::Spot` | Separate adapter and state from USD-M |
| Binance USD-M | `MarketKind::Perpetual` | One `Subscription` may merge multiple WebSockets |
| Hyperliquid | `Spot`, `Perpetual` | `positions_on(spot) == Ok(vec![])`; `Ticker::last_price = midPx.or(markPx)` |

Use common operations through `Client`. Access provider-only methods through
the concrete adapter returned by `Client::adapter()`.

## Provider references

- [Upbit](providers/upbit.md) ([한국어](providers/upbit.ko.md))
- [Bithumb](providers/bithumb.md) ([한국어](providers/bithumb.ko.md))
- [Binance](providers/binance.md) ([한국어](providers/binance.ko.md))
- [Hyperliquid](providers/hyperliquid.md) ([한국어](providers/hyperliquid.ko.md))
